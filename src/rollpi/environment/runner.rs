use std::thread;

use super::entities::{participant::{Participant, Runnable}, history::HistoryParticipant};

pub struct RunningContext
{
    pub parties: Vec<Participant>,
    pub memory: HistoryParticipant,
}

struct Runner
{
    context: RunningContext,
}

impl Runner
{
    pub fn new(context: RunningContext) -> Self
    {
        Runner {
            context,
        }
    }

    pub fn run(self: &'static mut Self)
    {
        // start the participants on different threads
        thread::spawn(move || {
            for p in &mut self.context.parties {
                p.run();
            }
        });
    }
}