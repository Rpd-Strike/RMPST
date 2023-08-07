use std::vec;

use crate::rollpi::{environment::{components::{picker::{Strategy, PrimProcTransf}, actions::ActionInterpreter}, entities::participant::{PartyContext, ParticipantState}, types::{PartyComm, MemoryPiece}}, syntax::{PrimeState, PrimProcess, TaggedPrimProc, ProcVar, TagVar, Process, TagKey, ChName, ProcTag}};

#[derive(Debug)]
pub enum ActionContext<'a>
{
    End,
    // Represents the prime process which reverts to the given TagKey   
    RollK(&'a TagKey),
    // Represents the prime process - Send on channel ChName, payload Process, and the whole process with given ProcTag 
    Send(&'a ChName, &'a Process, &'a ProcTag),
    // Represents the prime process - tagged with tag - 
    //    Receive on channel ChName, PVar and TVar replaced in the Process with what comes from PartyComm 
    RecvCont(PartyComm, ChName, &'a ProcVar, &'a TagVar, &'a Process, ProcTag),
}


#[derive(Default)]
pub struct SimpleDetermStrat {}

impl ActionInterpreter for SimpleDetermStrat
{
    fn interpret_action(&self, ctx: &mut PartyContext, act_ctx: ActionContext)
        -> PrimeState
    {
        println!("-- Interpreting action context: {:?}", act_ctx);

        match act_ctx {
            ActionContext::RollK(tag_key) => {
                let send_roll_ch = &ctx.get_comm_ctx().rollback_ctx.roll_tag_channel;
                // TODO: ? See for crash handling
                let _ = send_roll_ch.send(ProcTag::PTKey(tag_key.clone()));

                vec![]
            },
            ActionContext::Send(ChName(ch_name), proc, ptag) => {
                println!("INT: Sending process {:?} with tag {:?} to channel {:?}", proc, ptag, ch_name);

                let send_channel = ctx.get_comm_ctx().chan_msg_ctx(&ch_name).send_channel;
                send_channel.send(PartyComm { 
                    sender_id: ctx.get_id().clone(), process: proc.clone(), tag: ptag.clone() 
                }).unwrap();
                
                vec![]
            },
            ActionContext::RecvCont(in_data, ch_name, p_var, t_var, next_proc, recv_tag) => {
                println!("INT: Received process {:?} with tag {:?} from channel {:?}", in_data.process, in_data.tag, ch_name);
                
                let new_tag = ctx.get_tag_ctx().create_new_tag();

                let send_ch = &ctx.get_comm_ctx().history_ctx.hist_tag_channel;
                let rez = send_ch.send(MemoryPiece::new(
                    (in_data.sender_id, ctx.get_id().clone()),
                    (   in_data.tag,
                        (ch_name.clone(), in_data.process.clone())
                    ),
                    (   recv_tag,
                        (ch_name.clone(), p_var.clone(), t_var.clone(), next_proc.clone())
                    ),
                    new_tag.clone(),
                ));

                if let Err(e) = &rez {
                    println!("Error sending history tag: {:?}", e.to_string());
                    panic!("...");
                }

                let recv_ch = &ctx.get_comm_ctx().history_ctx.hist_conf_channel;
                match recv_ch.recv() {
                    Err(_err) => todo!(),
                    Ok(rec_tag_key) => {
                        assert_eq!(rec_tag_key, new_tag);
                        
                        next_proc.clone()
                            .substitution_on_trigger(p_var.clone(), &in_data.process, t_var.clone(), &new_tag)
                            .to_tagged_process(ProcTag::PTKey(new_tag))
                            .to_prime_state()
                    }
                }
            },
            ActionContext::End => {
                println!("INT: End of process");

                vec![]
            }
        }
    }
}

impl Strategy for SimpleDetermStrat
{
    fn run_strategy<'a>(&self, pctx: &mut PartyContext, state: &'a ParticipantState) -> Option<PrimProcTransf>
    {
        let ParticipantState { pr_state, frozen_tags } = state;

        let non_frozen_states = || {
            pr_state
                .iter()
                .enumerate()
                .filter(|(_, x)| !frozen_tags.contains(&x.tag))
        };

        let pos = non_frozen_states().find_map(|(i, x)| match x {
            TaggedPrimProc{ proc: PrimProcess::Send(ch_name, send_proc), tag} => {
                println!("Chose send process {:?} with tag {:?} to channel {:?}", send_proc, tag, ch_name);
                Some((i, ActionContext::Send(ch_name, send_proc, tag)))
            },
            _ => None,
        });

        let pos = pos.or_else(|| non_frozen_states().find_map(|(i, x)| match x {
            TaggedPrimProc{ proc: PrimProcess::RollK(tag_key), .. } => {
                println!("Chose rollk process with tag {:?}", tag_key);
                Some((i, ActionContext::RollK(tag_key)))
            },
            _ => None,
        }));

        let pos = pos.or_else(|| {
            non_frozen_states().find_map(|(i, x)| {
                match x {
                    TaggedPrimProc { proc: PrimProcess::Recv(ch_name, p_var, t_var, next_proc), tag } => {
                        // println!("Trying to receive from channel {:?}", ch_name);
                        self.probe_recv_channel(&pctx, ch_name)
                            .map(|comm| {
                                println!("Chose receiving process {:?} with tag {:?} from channel {:?}", comm.process, comm.tag, ch_name);
                                (i, ActionContext::RecvCont(comm, ch_name.clone(), p_var, t_var, next_proc, tag.clone()))
                            })
                    },
                    _ => None
                }
            })
        });
                        
        let pos = pos.or_else(|| {
            non_frozen_states().find_map(|(i, x)| match x {
                TaggedPrimProc{ proc: PrimProcess::End, .. } => {
                    println!("Chose end process");
                    Some((i, ActionContext::End))
                },
                _ => None,
            })
        });     

        if let Some((i, ac)) = &pos {
            println!("Pos to eliminate: {}", i);
            println!("Content: {:?}", ac);
        }

        pos.map(|(el_pos, ac)| {
            PrimProcTransf(el_pos, 
                           self.interpret_action(pctx, ac))
        })

    }
}