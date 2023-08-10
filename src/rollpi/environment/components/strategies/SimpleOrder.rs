use crate::rollpi::{environment::{components::{actions::ActionInterpreter, picker::{Strategy, PrimProcTransf}}, entities::participant::{PartyContext, ParticipantState}}, syntax::{TaggedPrimProc, PrimProcess, ChName, PrimeState}};

use super::SimpleDeterministic::ActionContext;

#[derive(Default)]
pub struct SimpleOrderStrat {
    interpreter: Box<dyn ActionInterpreter>,
}

// The idea for this ordering of evaluation is:
//  0. Run end of processes to clean up
//  1. First receive whatever is to receive
//  2. Then try to do rollbacks
//  3. Then try to send first stuff that is not recursive variables
//  4. When sending recursive variables, prioritise rec_norm
//  5. Then sending recursive variables, prioritise rec_comb
//  6. Important, when doing a send with whatever variable, put the resulting processes at the end of the list
impl Strategy for SimpleOrderStrat
{
    fn run_strategy<'a>(&'a self, pctx: &mut PartyContext, state: &'a ParticipantState) -> Option<PrimProcTransf>
    {
        let ParticipantState { pr_state, frozen_tags } = state;

        let non_frozen_states = || {
            pr_state
                .iter()
                .enumerate()
                .filter(|(_, x)| !frozen_tags.contains(&x.tag))
        };

        non_frozen_states().for_each(|(pos, x)| {
            let TaggedPrimProc { tag, proc } = x;
            let tip = match proc {
                PrimProcess::End => "End".to_string(),
                PrimProcess::RollK(_) => "Roll".to_string(),
                PrimProcess::Send(ChName(ch_name), _) => format!("Send {}", ch_name),
                PrimProcess::Recv(ChName(ch_name), _, _, _) => format!("Recv {}", ch_name),
            };

            print!(" | {}", tip);
        });
        println!(" || Part: {}", pctx.get_id());

        check_for_non_rec_comm(&state.pr_state, pctx.get_id());
        
        let pos = None;

        // Try End processes
        let pos = pos.or_else(|| {non_frozen_states().find_map(|(i, x)| match x {
            TaggedPrimProc{ proc: PrimProcess::End, .. } => {
                // println!("Chose end process");
                Some((i, ActionContext::End))
            },
            _ => None,
        })});  


        // Try recv processes
        let pos = pos.or_else(|| {non_frozen_states().find_map(|(i, x)| match x {
            TaggedPrimProc { proc: PrimProcess::Recv(ch_name, p_var, t_var, next_proc), tag } => {
                // println!("Trying to receive from channel {:?}", ch_name);
                self.interpreter.probe_recv_channel(&pctx, ch_name)
                    .map(|comm| {
                        // println!("Chose receiving process {:?} with tag {:?} from channel {:?}", comm.process, comm.tag, ch_name);
                        (i, ActionContext::RecvCont(comm, ch_name.clone(), p_var, t_var, next_proc, tag.clone()))
                    })
            },
            _ => None
        })}); 

        // Try rollbacks
        let pos = pos.or_else(|| non_frozen_states().find_map(|(i, x)| match x {
            TaggedPrimProc{ proc: PrimProcess::RollK(tag_key), .. } => {
                // println!("Chose rollk process with tag {:?}", tag_key);
                Some((i, ActionContext::RollK(tag_key)))
            },
            _ => None,
        }));

        // TODO: Make smarter ordering
        // Try sends -- Channel communication
        let pos = pos.or_else(|| {non_frozen_states().find_map(|(i, x)| match x {
            TaggedPrimProc{ proc: PrimProcess::Send(ch_name, send_proc), tag} => {
                if ch_name.0.starts_with("comm") {
                    Some((i, ActionContext::Send(ch_name, send_proc, tag)))
                }
                else {
                    None
                }
            },
            _ => None,
        })});

        // Try sends -- The Recv side of recursion variable
        let pos = pos.or_else(|| {non_frozen_states().find_map(|(i, x)| match x {
            TaggedPrimProc{ proc: PrimProcess::Send(ch_name, send_proc), tag} => {
                if ch_name.0.starts_with("rec_norm") {
                    Some((i, ActionContext::Send(ch_name, send_proc, tag)))
                }
                else {
                    None
                }
            },
            _ => None,
        })});

        // Try sends -- The rest
        let pos = pos.or_else(|| {non_frozen_states().find_map(|(i, x)| match x {
            TaggedPrimProc{ proc: PrimProcess::Send(ch_name, send_proc), tag} => {
                Some((i, ActionContext::Send(ch_name, send_proc, tag)))
            },
            _ => None,
        })});
        
                      

        // if let Some((i, ac)) = &pos {
            // println!("Pos to eliminate: {}", i);
            // println!("Content: {:?}", ac);
        // }

        pos.map(|(el_pos, ac)| {
            PrimProcTransf(el_pos, 
                           self.interpreter.interpret_action(pctx, ac))
        })
    }
}

fn check_for_non_rec_comm(state: &PrimeState, id: &String)
{
    let mut is_non_rec = false;

    for TaggedPrimProc { tag, proc } in state {
        match proc {
            PrimProcess::Send(ChName(ch_name), _) => {
                if !ch_name.starts_with("rec") {
                    is_non_rec = true;
                }
            },
            PrimProcess::Recv(ChName(ch_name), _, _, _) => {
                if !ch_name.starts_with("rec") {
                    is_non_rec = true;
                }
            },            
            _ => (),
        }
    }

    if !is_non_rec {
        println!("I am basically done  --- {}", id);
    }
}