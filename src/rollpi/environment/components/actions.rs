use crate::rollpi::{syntax::*, environment::{entities::{participant::{PartyContext}}, types::{PartyComm, MemoryPiece}}};

use super::{strategies::SimpleDeterministic::{SimpleDetermStrat, ActionContext}, picker::PrimProcTransf};

pub trait ActionInterpreter : Send
{
    fn interpret_action(&self, context: &mut PartyContext, ctx: ActionContext)
        -> PrimeState;

    fn probe_recv_channel(&self, context: &PartyContext, ChName(id): &ChName)
        -> Option<PartyComm>
    {
        let recv_channel = context.get_comm_ctx().chan_msg_ctx(&id).recv_channel;
        let data = recv_channel.try_recv().ok();

        if data.is_some() {
            println!("Probe ok on channel {:?}", id);
        }

        data
    }
}

impl Default for Box<dyn ActionInterpreter>
{
    fn default() -> Self
    {
        Box::new(SimpleDetermStrat::default())
    }
}
