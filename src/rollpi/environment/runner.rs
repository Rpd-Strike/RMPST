use std::thread;

use super::entities::{participant::{Participant, Runnable}, history::HistoryParticipant};

pub struct RunningContext
{
    pub parties: Vec<Participant>,
    pub hist: HistoryParticipant,
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

    pub fn run(self: Self)
    {
        // start the participants on different threads
        for p in self.context.parties {
            thread::spawn(move || {
                p.run();
            });
        };

        thread::spawn(move || {
            self.context.hist.run();
        });
    }
}