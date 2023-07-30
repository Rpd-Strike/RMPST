use std::vec;

use crate::rollpi::{environment::{components::{picker::{Strategy, PrimProcTransf}, actions::{ActionInterpreter}}, entities::participant::{PartyCommCtx, PartyContext}, types::PartyComm}, syntax::{PrimeState, PrimProcess, TaggedPrimProc, ProcVar, TagVar, Process, TagKey, ChName, ProcTag}};

pub enum ActionContext<'a>
{
    End,
    RollK(&'a TagKey),
    Send(&'a ChName, &'a Process, &'a ProcTag),
    RecvCont(Process, &'a ProcVar, &'a TagVar, &'a Process),
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
            ActionContext::RecvCont(income_proc, p_var, t_var, nnext_proc) => {
                let new_tag = ctx.get_tag_ctx().create_new_tag();

                todo!()
            },
            ActionContext::End => 
                vec![]
        }
    }
}

impl Strategy for SimpleDetermStrat
{
    fn run_strategy<'a>(&self, pctx: &mut PartyContext, state: &'a PrimeState) -> Option<PrimProcTransf<'a>>
    {
        // Try to get a Send process
        let s = state.iter().find_map(|x| match x {
            TaggedPrimProc{ proc: PrimProcess::Send(ch_name, p), tag } => {
                Some((x, ActionContext::Send(ch_name, p, tag)))
            },
            _ => None,
        });
        
        // Try to get a Rollback process
        let s = s.or_else(|| state.iter().find_map(|x| match x {
            TaggedPrimProc{ proc: PrimProcess::RollK(tag_key), .. } => {
                Some((x, ActionContext::RollK(tag_key)))
            },
            _ => None,
        }));

        // Try to run wait for a receive process
        let s = s.or_else(|| {
            state.iter().find_map(|x| match x {
                TaggedPrimProc{ proc: PrimProcess::Recv(ch_name, p_var, t_var, next_proc), .. } => {
                    self.probe_recv_channel(pctx, ch_name)
                    .map(|comm| {
                        (x, ActionContext::RecvCont(comm.process, p_var, t_var, next_proc))
                    })
                },
                _ => None,
            })
        });

        // Try to run an end process
        let s = s.or_else(|| {
            state.iter().find_map(|x| match x {
                TaggedPrimProc{ proc: PrimProcess::End, .. } => {
                    Some((x, ActionContext::End))
                },
                _ => None,
            })
        });

        s.map(|(x, a_ctx)| {
            PrimProcTransf(x, self.interpret_action(pctx, a_ctx))
        })

    }
}