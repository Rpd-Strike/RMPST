use crate::rollpi::{syntax::PrimeState, environment::entities::participant::PartyContext};

use super::strategies::SimpleDeterministic::SimpleDetermStrat;

pub struct PrimProcTransf(pub usize, pub PrimeState);

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
