use crate::rollpi::{syntax::PrimeState, environment::entities::participant::{PartyContext, ParticipantState}};

use super::strategies::SimpleOrder::SimpleOrderStrat;

pub struct PrimProcTransf(pub usize, pub PrimeState);

// Trait that specifies how to pick next action for a participant
//   to take from one of its primitive processes
pub trait Strategy : Send
{
    fn run_strategy<'a>(&'a self, pctx: &mut PartyContext, state: &'a ParticipantState) -> Option<PrimProcTransf>;
}

impl Default for Box<dyn Strategy>
{
    fn default() -> Self
    {
        Box::new(SimpleOrderStrat::default())
    }
}
