use std::collections::HashMap;
use crate::rollpi::{environment::{components::{actions::ActionInterpreter, picker::ActionPicker}, types::{MemoryPiece, PartyComm}}, syntax::{Process, TagKey, PrimeState}};


trait Runnable
{
    fn run(&mut self);
}

// Holds necessary information to run a participant on a thread
pub struct Participant
{
    id: String,

    state: PrimeState,
    
    action_picker: Box<dyn ActionPicker>,
    
    action_interpreter: Box<dyn ActionInterpreter>,

    comm_context: PartyContext,
}

pub struct PartyContext
{
    channel_pool: PartyChPool,
    history_ctx: HistoryContext,
    rollback_ctx: RollbackContext,
}

struct ChMsgContext<'a>
{
    pub send_channel: &'a Sender<PartyComm>,
    pub recv_channel: &'a Receiver<PartyComm>,
}

struct HistoryContext
{
    hist_tag_channel: Sender<MemoryPiece>,
    hist_conf_channel: Receiver<TagKey>,
}

struct RollbackContext
{
    roll_tag_channel: Sender<TODO_S>,
    roll_not_channel: Receiver<TODO_S>,
}

struct TODO_S;


#[derive(Default)]
pub struct PartyChPool
{
    receivers: HashMap<String, Receiver<PartyComm>>,
    senders: HashMap<String, Sender<PartyComm>>,
}

impl PartyChPool
{
    pub fn get_recv(&self, id: &str) -> &Receiver<PartyComm>
    {
        self.receivers.get(id).unwrap()
    }

    pub fn get_send(&self, id: &str) -> &Sender<PartyComm>
    {
        self.senders.get(id).unwrap()
    }
}

impl PartyContext
{
    pub fn chan_msg_ctx(&self, ch: &str) -> &ChMsgContext
    {
        &ChMsgContext {
            send_channel: self.channel_pool.get_send(ch),
            recv_channel: self.channel_pool.get_recv(ch),
        }
    }
}

impl Participant 
{
    pub fn new(
        id: String,
        state: PrimeState,
        action_picker: Box<dyn ActionPicker>,
        action_interpreter: Box<dyn ActionInterpreter>,
        comm_context: PartyContext,
    ) -> Self
    {
        Self {
            id,
            state,
            action_picker,
            action_interpreter,
            comm_context,
        }
    }

    fn evolve_state(self: &mut Self)
    {
        todo!();
    }

    fn rollback_logic(self: &mut Self)
    {
        todo!();
    }

}

impl Runnable for Participant
{
    fn run(self: &mut Self)
    {
        // TODO: Maybe use evolve_state / rollback_logic

        loop {
            let action = self.action_picker.pick_action(&self.state);
            self.action_interpreter.interpret_action(action, &self.comm_context);
        }
    }
}

use crossbeam::channel::{Sender, Receiver};
