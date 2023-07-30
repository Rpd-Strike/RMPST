use std::task::Context;

use crate::rollpi::{syntax::{PrimeState, TaggedPrimProc, Process, ProcVar, TagVar}, environment::entities::participant::{PartyCommCtx, PartyContext}};

use super::{actions::ActionInterpreter, strategies::SimpleDeterministic::SimpleDetermStrat};

pub struct PrimProcTransf(pub usize, pub PrimeState);

pub enum ActionData
{
    None,
    // Received process / the p_var of the process received / tag of the trigger / continuation process
    RecvCont(Process, ProcVar, TagVar, Process),
}

// Trait that specifies how to pick next action for a participant
//   to take from one of its primitive processes
pub trait Strategy : Send
{
    fn run_strategy<'a>(&'a self, pctx: &mut PartyContext, state: &'a PrimeState) -> Option<PrimProcTransf>;
}

impl Default for Box<dyn Strategy>
{
    fn default() -> Self
    {
        Box::new(SimpleDetermStrat::default())
    }
}
