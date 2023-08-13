use crate::rollpi::{environment::{components::{picker::{Strategy, PrimProcTransf}, actions::ActionInterpreter}, entities::participant::{PartyContext, ParticipantState}, types::PartyComm}, syntax::{PrimProcess, TaggedPrimProc, ProcVar, TagVar, Process, TagKey, ChName, ProcTag}};

// #[derive(Debug)]
pub type TaggedActionContext<'a> = (&'a ProcTag, ActionContext<'a>);

#[derive(Debug)]
pub enum ActionContext<'a>
{
    End,
    // Represents the prime process which reverts to the given TagKey   
    RollK(&'a TagKey),
    // Represents the prime process - Send on channel ChName, payload Process, and the whole process with given ProcTag 
    Send(&'a ChName, &'a Process),
    // Represents the prime process - tagged with tag - 
    //    Receive on channel ChName, PVar and TVar replaced in the Process with what comes from PartyComm 
    RecvCont(PartyComm, ChName, &'a ProcVar, &'a TagVar, &'a Process),
}


#[derive(Default)]
pub struct SimpleDetermStrat {
    interpreter: Box<dyn ActionInterpreter>,
}

impl Strategy for SimpleDetermStrat
{
    fn run_strategy<'a>(&self, pctx: &mut PartyContext, state: &'a ParticipantState) -> Option<PrimProcTransf>
    {
        let ParticipantState { live_state, dead_state, frozen_tags } = state;

        let non_frozen_states = || {
            live_state
                .iter()
                .enumerate()
                .filter(|(_, x)| !frozen_tags.contains(&x.tag))
        };

        let pos = None;

        let pos = pos.or_else(|| non_frozen_states().find_map(|(i, x)| match x {
            TaggedPrimProc{ proc: PrimProcess::Send(ch_name, send_proc), tag} => {
                // println!("Chose send process {:?} with tag {:?} to channel {:?}", send_proc, tag, ch_name);
                Some((i, (tag, ActionContext::Send(ch_name, send_proc))))
            },
            _ => None,
        }));

        let pos = pos.or_else(|| non_frozen_states().find_map(|(i, x)| match x {
            TaggedPrimProc{ proc: PrimProcess::RollK(tag_key), tag } => {
                // println!("Chose rollk process with tag {:?}", tag_key);
                Some((i, (tag, ActionContext::RollK(tag_key))))
            },
            _ => None,
        }));

        let pos = pos.or_else(|| {
            non_frozen_states().find_map(|(i, x)| {
                match x {
                    TaggedPrimProc { proc: PrimProcess::Recv(ch_name, p_var, t_var, next_proc), tag } => {
                        // println!("Trying to receive from channel {:?}", ch_name);
                        self.interpreter.probe_recv_channel(&pctx, ch_name)
                            .map(|comm| {
                                // println!("Chose receiving process {:?} with tag {:?} from channel {:?}", comm.process, comm.tag, ch_name);
                                (i, (tag, ActionContext::RecvCont(comm, ch_name.clone(), p_var, t_var, next_proc)))
                            })
                    },
                    _ => None
                }
            })
        });
                        
        let pos = pos.or_else(|| {
            non_frozen_states().find_map(|(i, x)| match x {
                TaggedPrimProc{ proc: PrimProcess::End, tag } => {
                    // println!("Chose end process");
                    Some((i, (tag, ActionContext::End)))
                },
                _ => None,
            })
        });     

        pos.map(|(el_pos, ac)| {
            PrimProcTransf(el_pos, 
                           self.interpreter.interpret_action(pctx, ac))
        })

    }
}