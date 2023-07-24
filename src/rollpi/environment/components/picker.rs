use crate::rollpi::syntax::PrimeState;

use super::actions::PartyAction;

// Trait that specifies how to pick next action for a participant
//   to take from one of its primitive processes
pub trait ActionPicker
{
    fn pick_action(&self, state: &PrimeState) -> &PartyAction;
}

struct DeterministPicker;
struct RandomPicker;

impl Default for Box<dyn ActionPicker>
{
    fn default() -> Self
    {
        Box::new(DeterministPicker)
    }
}

impl ActionPicker for DeterministPicker
{
    fn pick_action(&self, state: &PrimeState) -> &PartyAction
    {
        match state.get(0) {
            Some(proc) => &PartyAction::RunPrimary(proc),
            None => &PartyAction::End,
        }
    }
}

impl ActionPicker for RandomPicker
{
    fn pick_action(&self, state: &PrimeState) -> &PartyAction
    {
        todo!();
    }
}