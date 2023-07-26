use std::collections::{HashSet, HashMap};

use crossbeam::channel::unbounded;

use crate::rollpi::syntax::{PrimeState, all_chn_names_proc, prime_proc_to_process};

use super::{super::syntax::Process, components::{picker::ActionPicker, actions::ActionInterpreter}, entities::participant::{Participant, PartyContext, PartyChPool}, types::PartyComm};

#[derive(Default)]
struct Generator
{
    participants: HashMap<String, (Box<dyn ActionPicker>, Box<dyn ActionInterpreter>, PrimeState)>,
}

impl Generator
{   
    fn create_participant(
        self: &mut Self,
        id: String,
        proc: PrimeState, 
        picker: Option<Box<dyn ActionPicker>>,
        interpreter: Option<Box<dyn ActionInterpreter>>,
    )
    {
        self.participants.insert(id, (
            picker.unwrap_or_default(),
            interpreter.unwrap_or_default(),
            proc,
        ));
    }

    // Add a participant to be generated. 
    // Returns true if the given id is not already used, false if automatically generated
    pub fn take_participant_conf(
        self: &mut Self,
        proc: PrimeState,
        id: Option<String>,
        strategy: Option<Box<dyn ActionPicker>>,
        interpreter: Option<Box<dyn ActionInterpreter>>,
    ) -> bool
    {
        // try to add the id given from the argument if not in the hashset
        if let Some(id) = id {
            if !self.participants.contains_key(&id) {
                self.create_participant(id, proc, strategy, interpreter);
                return true;    
            }
        }

        // If it is already present or not given, try to find a default one, A, B, C, D, ...
        let mut i = 1;
        let id = loop {
            let new_id = format!("p_{}", i);
            if !self.participants.contains_key(&new_id) {
                break new_id;
            }
            i += 1;
        };

        self.create_participant(id, proc, strategy, interpreter);
        return false;
    }

    fn generate_participants(self: Self) -> Vec<Participant>
    {
        // TODO: Create channels and create copy for each of the participants

        let mut pool_recv = HashMap::new();
        let mut pool_send = HashMap::new();

        let channels = self.participants.iter()
            .map(|(id, (_, _, proc))| {
                proc.iter().map(|tag_proc| {
                    all_chn_names_proc(prime_proc_to_process(&tag_proc.proc))
                })
                .flatten().collect::<HashSet<_>>()
            })
            .flatten();
        
        for ch_name in channels {
            let (send, recv) = unbounded::<PartyComm>();
            pool_recv.insert(ch_name.to_owned(), recv);
            pool_send.insert(ch_name.to_owned(), send);
        }

        let partChPool = PartyChPool {
            receivers: HashMap::new(),
            senders: HashMap::new(),
        };

        self.participants
            .into_iter()
            .map(|(id, (picker, interpreter, proc)) | {
                Participant::new(
                    id,
                    proc,
                    picker,
                    interpreter,
                    PartyContext::new(),
                )
            }).collect()
    }
}