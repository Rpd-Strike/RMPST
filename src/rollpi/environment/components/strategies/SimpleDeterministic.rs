use std::vec;

use crate::rollpi::{environment::{components::{picker::{Strategy, PrimProcTransf}, actions::{ActionInterpreter}}, entities::participant::{PartyCommCtx, PartyContext}, types::{PartyComm, MemoryPiece}}, syntax::{PrimeState, PrimProcess, TaggedPrimProc, ProcVar, TagVar, Process, TagKey, ChName, ProcTag}};

pub enum ActionContext<'a>
{
    End,
    RollK(&'a TagKey),
    Send(&'a ChName, &'a Process, &'a ProcTag),
    RecvCont(PartyComm, ChName, &'a ProcVar, &'a TagVar, &'a Process, ProcTag),
}


#[derive(Default)]
pub struct SimpleDetermStrat {}

impl ActionInterpreter for SimpleDetermStrat
{
    fn interpret_action(&self, ctx: &mut PartyContext, act_ctx: ActionContext)
        -> PrimeState
    {
        match act_ctx {
            ActionContext::RollK(tag_key) => {
                todo!()
            },
            ActionContext::Send(ChName(ch_name), proc, ptag) => {
                let send_channel = ctx.get_comm_ctx().chan_msg_ctx(&ch_name).send_channel;
                send_channel.send(PartyComm { 
                    sender_id: ctx.get_id().clone(), process: proc.clone(), tag: ptag.clone() 
                }).unwrap();
                
                vec![]
            },
            ActionContext::RecvCont(in_data, ch_name, p_var, t_var, next_proc, recv_tag) => {
                let new_tag = ctx.get_tag_ctx().create_new_tag();

                let send_ch = &ctx.get_comm_ctx().history_ctx.hist_tag_channel;
                send_ch.send(MemoryPiece {
                    ids: (in_data.sender_id, ctx.get_id().clone()),
                    sender: (in_data.tag, (ch_name.clone(), in_data.process)),
                    receiver: (recv_tag, (ch_name, p_var.clone(), t_var.clone(), next_proc.clone())),
                    new_mem_tag: new_tag,
                }).unwrap();

                let recv_ch = &ctx.get_comm_ctx().history_ctx.hist_conf_channel;
                match recv_ch.recv() {
                    Err(err) => todo!(),
                    Ok(x) => {
                        let new_proc = syntax::perform_alpha_conv_proc(next_proc, p_var, in_data.process, t_var, new_tag);
                        ctx.get_tag_ctx().
                    }
                }

                todo!()
            },
            ActionContext::End => 
                vec![]
        }
    }
}

impl Strategy for SimpleDetermStrat
{
    fn run_strategy<'a>(&self, pctx: &mut PartyContext, state: &'a PrimeState) -> Option<PrimProcTransf>
    {
        let mut ac = None;

        let pos = state.iter().position(|x| match x {
            TaggedPrimProc{ proc: PrimProcess::Send(ch_name, send_proc), tag} => {
                ac = Some(ActionContext::Send(ch_name, send_proc, tag));
                true
            },
            _ => false,
        });

        pos.or_else(|| state.iter().position(|x| match x {
            TaggedPrimProc{ proc: PrimProcess::RollK(tag_key), .. } => {
                ac = Some(ActionContext::RollK(tag_key));
                true
            },
            _ => false,
        }));

        pos.or_else(|| {
            state.iter().position(|x| match x {
                TaggedPrimProc { proc: PrimProcess::Recv(ch_name, p_var, t_var, next_proc), tag } => {
                    match self.probe_recv_channel(&pctx, ch_name) {
                        Some(comm) => {
                            ac = Some(ActionContext::RecvCont(comm, ch_name.clone(), p_var, t_var, next_proc, tag.clone()));
                            true
                        },
                        None => false
                    }
                },
                _ => false,
            })
        });

        pos.or_else(|| {
            state.iter().position(|x| match x {
                TaggedPrimProc{ proc: PrimProcess::End, .. } => {
                    ac = Some(ActionContext::End);
                    true
                },
                _ => false,
            })
        });
        
        // // Try to get a Send process
        // let s = state.iter().find_map(|x| match x {
        //     TaggedPrimProc{ proc: PrimProcess::Send(ch_name, p), tag } => {
        //         Some((x, ActionContext::Send(ch_name, p, tag)))
        //     },
        //     _ => None,
        // });
        
        // // Try to get a Rollback process
        // let s = s.or_else(|| state.iter().find_map(|x| match x {
        //     TaggedPrimProc{ proc: PrimProcess::RollK(tag_key), .. } => {
        //         Some((x, ActionContext::RollK(tag_key)))
        //     },
        //     _ => None,
        // }));

        // // Try to run wait for a receive process
        // let s = s.or_else(|| {
        //     state.iter().find_map(|x| match x {
        //         TaggedPrimProc{ proc: PrimProcess::Recv(ch_name, p_var, t_var, next_proc), .. } => {
        //             self.probe_recv_channel(pctx, ch_name)
        //             .map(|comm| {
        //                 (x, ActionContext::RecvCont(comm.process, p_var, t_var, next_proc))
        //             })
        //         },
        //         _ => None,
        //     })
        // });

        // Try to run an end process
        // let s = s.or_else(|| {
        //     state.iter().find_map(|x| match x {
        //         TaggedPrimProc{ proc: PrimProcess::End, .. } => {
        //             Some((x, ActionContext::End))
        //         },
        //         _ => None,
        //     })
        // });

        

        pos.map(|x| {
            PrimProcTransf(x, 
                           ac.map(|a| self.interpret_action(pctx, a))
                             .unwrap_or(vec![]))
        })

    }
}