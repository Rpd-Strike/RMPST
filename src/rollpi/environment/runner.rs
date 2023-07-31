use std::thread;

use super::entities::{participant::{Participant, Runnable}, history::HistoryParticipant};

pub struct RunningContext
{
    pub parties: Vec<Participant>,
    pub hist: HistoryParticipant,
}

pub struct Runner
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
        let mut handles = vec![];

        // start the participants on different threads
        for p in self.context.parties {
            let h = thread::spawn(move || {
                p.run();
            });

            handles.push(h);
        };

        let hist_h = thread::spawn(move || {
            self.context.hist.run();
        });
        handles.push(hist_h);

        // wait for all threads to finish
        for h in handles {
            h.join().unwrap();
        }
    }
}