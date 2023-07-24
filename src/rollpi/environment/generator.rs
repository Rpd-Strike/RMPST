use std::collections::HashSet;

use super::{super::syntax::Process, components::{picker::ActionPicker, actions::ActionInterpreter}, entities::participant::{Participant, PartyContext}};

#[derive(Default)]
struct Generator
{
    partial_participants: Vec<(String, Box<dyn ActionPicker>, Box<dyn ActionInterpreter>)>,
    ids: HashSet<String>,
}

impl Generator
{   
    fn create_participant(
        self: &mut Self,
        proc: Process, 
        id: String,
        picker: Option<Box<dyn ActionPicker>>,
        interpreter: Option<Box<dyn ActionInterpreter>>,
    )
    {
        self.ids.insert(id.clone());
        self.partial_participants.push((
            id,
            picker.unwrap_or_default(),
            interpreter.unwrap_or_default()
        ));
    }

    pub fn take_participant_conf(
        self: &mut Self,
        proc: Process, 
        id: Option<String>,
        strategy: Option<Box<dyn ActionPicker>>,
        interpreter: Option<Box<dyn ActionInterpreter>>,
    ) -> bool
    {
        // try to add the id given from the argument if not in the hashset. If it is already present, try to find a default one, A, B, C, D, ...
        if let Some(id) = id {
            if !self.ids.contains(&id) {
                self.ids.insert(id);
                self.create_participant(proc, id, strategy, interpreter);
                return true;    
            }
        }

        let mut i = 1;
        let id = loop {
            let new_id = format!("p_{}", i);
            if !self.ids.contains(&new_id) {
                break new_id;
            }
            i += 1;
        };

        self.create_participant(proc, id, strategy, interpreter);
        return false;
    }

    fn generate_participants(self: Self) -> Vec<Participant>
    {
        // TODO: Create channels and create copy for each of the participants

        self.partial_participants.into_iter().map(|(id, picker, interpreter)| {
            Participant::new(
                id,
                vec![],
                picker,
                interpreter,
                PartyContext::new(),
            )
        }).collect()
    }
}