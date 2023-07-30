use std::collections::HashMap;

use crossbeam::channel::{Receiver, Sender};

use crate::rollpi::{environment::types::MemoryPiece, syntax::{TagKey, ProcTag}};

use super::participant::{TODO_S, Runnable};


// Central place for holding process memories,
// Acts as the confirmation for generating labels on send/recv
// Receive incoming rellback requests 
// Send freeze signals to actual participants
pub struct HistoryParticipant
{
    ctx: HistoryContext,

    tag_owner: HashMap<ProcTag, String>,

    // links created by joining communication
    join_links: HashMap<ProcTag, ProcTag>,
    // links created by branching in parallel some process
    branch_links: HashMap<TagKey, Vec<ProcTag>>,
}

#[derive(Default)]
pub struct HistoryContext
{
    // Channels for receiving tag creations
    pub hist_tag_recv: HashMap<String, Receiver<MemoryPiece>>,
    // Channels for sending notifications of tag creations
    pub hist_not_send: HashMap<String, Sender<TagKey>>,

    // Channels for receiving rollback requests
    pub roll_tag_recv: HashMap<String, Receiver<TODO_S>>,
    // Channels for sending freeze signals
    pub roll_frz_send: HashMap<String, Sender<TODO_S>>,
}


impl HistoryParticipant
{
    pub fn new(h_ctx: HistoryContext) -> Self
    {
        HistoryParticipant {
            ctx: h_ctx,
            tag_owner: HashMap::new(),
            join_links: HashMap::new(),
            branch_links: HashMap::new(),
        }
    }

    // Tries to get a tag message from all the participants and process/respond
    fn run_tag_cycle(self: &mut Self)
    {
        for (_name, recv) in &self.ctx.hist_tag_recv {
            while let Ok(MemoryPiece{
                ids: (id_send, id_recv), 
                sender: (sender_tag, _), 
                receiver: (recv_tag, _), 
                new_mem_tag}) = recv.try_recv() 
            {
                let new_tag = ProcTag::PTKey(new_mem_tag.clone());
                self.tag_owner.insert(new_tag.clone(), id_recv.clone());
                self.tag_owner.insert(sender_tag.clone(), id_send.clone());
                self.tag_owner.insert(recv_tag.clone(), id_recv.clone());

                self.join_links.insert(sender_tag, new_tag.clone());
                self.join_links.insert(recv_tag, new_tag.clone());

                if let Some(x) = self.ctx.hist_not_send.get(&id_recv) {
                    x.send(new_mem_tag);
                }
            }
        }
    }

    // Receives a rollback request from a participant and sends a freeze signal to all relevant participants
    fn run_rollback_cycle(self: &mut Self)
    {
        for (_name, recv) in &self.ctx.roll_tag_recv {
            while let Ok(TODO_S) = recv.try_recv() {
                // TODO: ...
                todo!();
            }
        }
    }
}

impl Runnable for HistoryParticipant
{
    fn run(self: Self)
    {
        
    }
}



