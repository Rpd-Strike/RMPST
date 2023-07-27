use std::collections::HashMap;
use crate::rollpi::{environment::{components::{actions::ActionInterpreter, picker::ActionPicker}, types::{MemoryPiece, PartyComm}}, syntax::{Process, TagKey, PrimeState}};


pub trait Runnable : Send
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
    pub channel_pool: PartyChPool,
    pub history_ctx: TagContext,
    pub rollback_ctx: RollbackContext,
}

pub struct ChMsgContext<'a>
{
    pub send_channel: &'a Sender<PartyComm>,
    pub recv_channel: &'a Receiver<PartyComm>,
}

pub struct TagContext
{
    pub hist_tag_channel: Sender<MemoryPiece>,
    pub hist_conf_channel: Receiver<TagKey>,
}

pub struct RollbackContext
{
    pub roll_tag_channel: Sender<TODO_S>,
    pub freeze_not_channel: Receiver<TODO_S>,
}

pub struct TODO_S;


#[derive(Default, Clone)]
pub struct PartyChPool
{
    senders: HashMap<String, Sender<PartyComm>>,
    receivers: HashMap<String, Receiver<PartyComm>>,
}

impl PartyChPool
{
    pub fn new(it: impl Iterator<Item = String>) 
        -> Self
    {
        let mut receivers = HashMap::new();
        let mut senders = HashMap::new();

        for id in it {
            let (tx, rx) = unbounded::<PartyComm>();
            receivers.insert(id.clone(), rx);
            senders.insert(id, tx);
        }

        Self {
            senders,
            receivers,
        }
    }
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
    pub fn chan_msg_ctx(&self, ch: &str) -> ChMsgContext
    {
        ChMsgContext {
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
            self.action_interpreter.interpret_action(&action, &self.comm_context);
        }
    }
}

use crossbeam::channel::{Sender, Receiver, unbounded};
