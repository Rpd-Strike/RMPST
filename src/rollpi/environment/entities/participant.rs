use std::collections::HashMap;
use crate::rollpi::{environment::{components::picker::{Strategy, PrimProcTransf}, types::{MemoryPiece, PartyComm}}, syntax::{TagKey, PrimeState}};


pub trait Runnable : Send
{
    fn run(self: Self);
}

pub trait ContextGetter 
{
    fn get_context(self: &Self) -> &PartyCommCtx;

    fn get_id(self: &Self) -> &String;
}

// Holds necessary information to run a participant on a thread
pub struct Participant
{
    state: PrimeState,
    
    strategy: Box<dyn Strategy>,

    party_context: PartyContext,
}

pub struct PartyContext
{
    id: String,
    comm_ctx: PartyCommCtx,
    tag_ctx: TagCreator,
}

impl PartyContext
{
    pub fn get_id(&self) -> &String
    {
        &self.id
    }

    pub fn get_comm_ctx(&self) -> &PartyCommCtx
    {
        &self.comm_ctx
    }

    pub fn get_tag_ctx(&mut self) -> &mut TagCreator
    {
        &mut self.tag_ctx
    }
}

pub struct PartyCommCtx
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

    pub fn get_recv(&self, id: &str) -> &Receiver<PartyComm>
    {
        self.receivers.get(id).unwrap()
    }

    pub fn get_send(&self, id: &str) -> &Sender<PartyComm>
    {
        self.senders.get(id).unwrap()
    }
}

impl PartyCommCtx
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
        strategy: Box<dyn Strategy>,
        comm_context: PartyCommCtx,
    ) -> Self
    {
        Self {
            state,
            strategy,
            party_context: PartyContext {
                id,
                comm_ctx: comm_context,
                tag_ctx: TagCreator::default(),
            }
        }
    }

    fn evolve_state(&mut self)
    {
        let (ctx, state, strat) = self.borrow_data();
        let action = strat.run_strategy(ctx, &state);
        
        // Remove the ran process, append the new processes
        if let Some(PrimProcTransf(pos, proc)) = action {
            state.swap_remove(pos);
            state.extend(proc.into_iter());
        }
    }   

    fn rollback_logic(self: &Self)
    {
        todo!();
    }

    fn borrow_data(&mut self) -> (&mut PartyContext, &mut PrimeState, &dyn Strategy)
    {
        (&mut self.party_context, &mut self.state, &*self.strategy)
    }

}

impl Runnable for Participant
{
    fn run(mut self: Self)
    {
        loop {
            self.evolve_state()


            // TODO: Implement rollback logic
            // self.rollback_logic();
        }
    }
}

use crossbeam::channel::{Sender, Receiver, unbounded};

use super::tag_creator::TagCreator;
