use std::collections::{HashMap, HashSet};

use crossbeam::channel::{Receiver, Sender};

use crate::rollpi::{environment::types::MemoryPiece, syntax::{TagKey, ProcTag, Process, TaggedProc}};

use super::participant::Runnable;


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
    branch_links: HashMap<ProcTag, Vec<ProcTag>>,

    // frozen tags
    frozen_tags: HashSet<ProcTag>,
}

pub struct HistoryContext
{
    // Channels for receiving tag creations
    pub hist_tag_recv: HashMap<String, Receiver<MemoryPiece>>,
    // Channels for sending notifications of tag creations
    pub hist_not_send: HashMap<String, Sender<TagKey>>,

    // TODO: This one can be just a mpsc channel
    // Channels for receiving rollback requests
    pub roll_tag_recv: HashMap<String, Receiver<ProcTag>>,
    // Channels for sending freeze signals
    pub roll_frz_send: HashMap<String, Sender<ProcTag>>,

    pub diss_tag_recv: Receiver<ProcTag>,

    pub ress_tag_send: HashMap<String, Sender<RessurectMsg>>,
}

impl HistoryContext
{
    pub fn new(arg_diss_tag_recv: Receiver<ProcTag>) -> Self
    {
        Self {
            hist_tag_recv: HashMap::default(),
            hist_not_send: HashMap::default(),

            roll_tag_recv: HashMap::default(),
            roll_frz_send: HashMap::default(),

            diss_tag_recv: arg_diss_tag_recv,

            ress_tag_send: HashMap::default(),
        }
    }
}

pub struct RessurectMsg
{
    pub descendant_tag: ProcTag,
    pub tagged_proc: TaggedProc,
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
            frozen_tags: HashSet::new(),
        }
    }

    fn _generate_links(br_links: &mut HashMap<ProcTag, Vec<ProcTag>>, 
                       join_links: &mut HashMap<ProcTag, ProcTag>,  
                       send_t: &ProcTag, 
                       recv_t: &ProcTag, 
                       new_t: &ProcTag)
    {
        let mut update_branch_links = |child_t: &ProcTag| {
            if let ProcTag::PTSplit(_piece, _nr, orig_t) = child_t {
                match br_links.get_mut(&ProcTag::PTKey(orig_t.clone())) {
                    Some(deps) => {
                        deps.push(child_t.clone());
                    },
                    None => {
                        br_links.insert(ProcTag::PTKey(orig_t.clone()), vec![child_t.clone()]);
                    },
                }
            }    
        };

        update_branch_links(send_t);
        update_branch_links(recv_t);
        
        // Update join links data structure
        join_links.insert(recv_t.clone(), new_t.clone());
        join_links.insert(send_t.clone(), new_t.clone());
    }

    // Tries to get a tag message from all the participants and process/respond
    fn run_tag_cycle(self: &mut Self)
    {
        let (hctx, br_links, join_links) = (&self.ctx, &mut self.branch_links, &mut self.join_links);
        // Poll for receiving tagging messages
        for (_name, recv) in &hctx.hist_tag_recv {
            while let Ok(MemoryPiece{
                ids: (id_send, id_recv), 
                sender: (sender_tag, _), 
                receiver: (recv_tag, _), 
                new_mem_tag}) = recv.try_recv() 
            {
                // Update tag owner data structure
                let new_tag = ProcTag::PTKey(new_mem_tag.clone());
                self.tag_owner.insert(new_tag.clone(), id_recv.clone());
                self.tag_owner.insert(sender_tag.clone(), id_send.clone());
                self.tag_owner.insert(recv_tag.clone(), id_recv.clone());

                // update causal dependency links
                HistoryParticipant::_generate_links(br_links, join_links, &sender_tag, &recv_tag, &ProcTag::PTKey(new_mem_tag.clone())); 
                
                if let Some(x) = hctx.hist_not_send.get(&id_recv) {
                    // TODO: ? Check what to do in case of crash
                    let _ = x.send(new_mem_tag);
                }
            }
        }
    }

    // TODO: Make DFS run through join links & branch links
    fn _send_freeze_sgn_dfs(ctx: &HistoryContext, frozen_tags: &mut HashSet<ProcTag>, tag_owner: &HashMap<ProcTag, String>, p: &ProcTag) 
    {
        if frozen_tags.contains(p) {
            return ();
        }

        // TODO: Make a context to avoid unwrap
        let owner = tag_owner.get(p).unwrap();
        let signal_ch = ctx.roll_frz_send.get(owner).unwrap();
        // TODO: ? Check what to do in case of crash
        let _ = signal_ch.send(p.clone());

        frozen_tags.insert(p.clone());
    }

    // TODO: Make DFS run through join links & branch links
    // Receives a rollback request from a participant and sends a freeze signal to all relevant participants
    fn run_rollback_cycle(self: &mut Self)
    {
        let (frozen_tags, tag_owner, ctx) = (&mut self.frozen_tags, &self.tag_owner, &self.ctx);
        for (_name, recv) in &ctx.roll_tag_recv {
            while let Ok(proc_tag) = recv.try_recv() {
                HistoryParticipant::_send_freeze_sgn_dfs(ctx, frozen_tags, tag_owner, &proc_tag);
            }
        }
    }

    // TODO: Get the sender and sender info with id, and send to correct 2 targets, the msgs to be recreated
    fn run_dissapear_cycle(self: &Self)
    {
        while let Ok(tag) = self.ctx.diss_tag_recv.try_recv() {
            // Find the owner of the send and receive branch of process produced out of the given Tag
            
        } 
    }

}

impl Runnable for HistoryParticipant
{
    fn run(mut self: Self)
    {
        loop {
            self.run_tag_cycle();
            self.run_rollback_cycle();
            self.run_dissapear_cycle();
        }
    }
}



