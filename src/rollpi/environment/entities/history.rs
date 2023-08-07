use std::collections::{HashMap, HashSet};

use crossbeam::channel::{Receiver, Sender};

use crate::rollpi::{environment::types::MemoryPiece, syntax::{TagKey, ProcTag, TaggedProc, Process}};

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
    // same as join_links but in reverse for sending ressurect messages
    // For each ProcTag, keep the (sender, receiver) pair of tagged processes
    rev_join_links: HashMap<ProcTag, (TaggedProc, TaggedProc)>,
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
    pub dissapeared_tag: ProcTag,
    pub ress_tagged_proc: TaggedProc,
}

impl HistoryParticipant
{
    pub fn new(h_ctx: HistoryContext) -> Self
    {
        HistoryParticipant {
            ctx: h_ctx,
            tag_owner: HashMap::new(),
            join_links: HashMap::new(),
            rev_join_links: HashMap::new(),
            branch_links: HashMap::new(),
            frozen_tags: HashSet::new(),
        }
    }

    fn _generate_links(br_links: &mut HashMap<ProcTag, Vec<ProcTag>>, 
                       join_links: &mut HashMap<ProcTag, ProcTag>,  
                       rev_join_links: &mut HashMap<ProcTag, (TaggedProc, TaggedProc)>,
                       sender: TaggedProc,
                       receiver: TaggedProc, 
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

        update_branch_links(&sender.tag);
        update_branch_links(&receiver.tag);
        
        // Update join links data structure
        join_links.insert(receiver.tag.clone(), new_t.clone());
        join_links.insert(sender.tag.clone(), new_t.clone());

        // Update reverse join links data structure
        rev_join_links.insert(new_t.clone(), (sender, receiver));
    }

    // Tries to get a tag message from all the participants and process/respond
    fn run_tag_cycle(self: &mut Self)
    {
        let (hctx, br_links, join_links, rev_join_links) = (&self.ctx, &mut self.branch_links, &mut self.join_links, &mut self.rev_join_links);
        // Poll for receiving tagging messages
        for (_name, recv) in &hctx.hist_tag_recv {
            while let Ok(MemoryPiece{
                ids: (id_send, id_recv), 
                sender,
                receiver, 
                new_mem_tag, ..}) = recv.try_recv() 
            {
                let new_tag = ProcTag::PTKey(new_mem_tag.clone());
                
                // Update tag owner data structure
                self.tag_owner.insert(new_tag.clone(), id_recv.clone());
                self.tag_owner.insert(sender.tag.clone(), id_send.clone());
                self.tag_owner.insert(receiver.tag.clone(), id_recv.clone());

                // update causal dependency links
                HistoryParticipant::_generate_links(br_links, join_links, rev_join_links, 
                    sender, receiver, &ProcTag::PTKey(new_mem_tag.clone()),); 
                
                if let Some(x) = hctx.hist_not_send.get(&id_recv) {
                    // TODO: ? Check what to do in case of crash
                    let _ = x.send(new_mem_tag);
                }
            }
        }
    }

    fn _send_freeze_sgn_dfs(join_links: &HashMap<ProcTag, ProcTag>, branch_links: &HashMap<ProcTag, Vec<ProcTag>>, ctx: &HistoryContext, frozen_tags: &mut HashSet<ProcTag>, tag_owner: &HashMap<ProcTag, String>, p: &ProcTag) 
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

        // Try to go through join links
        if let Some(next_p) = join_links.get(p) {
            HistoryParticipant::_send_freeze_sgn_dfs(join_links, branch_links, ctx, frozen_tags, tag_owner, next_p);
        }

        // Try to go through branch links
        if let Some(next_ps) = branch_links.get(p) {
            for next_p in next_ps {
                HistoryParticipant::_send_freeze_sgn_dfs(join_links, branch_links, ctx, frozen_tags, tag_owner, next_p);
            }
        }
    }

    // TODO: Make DFS run through join links & branch links
    // Receives a rollback request from a participant and sends a freeze signal to all relevant participants
    fn run_rollback_cycle(self: &mut Self)
    {
        let (join_links, 
             branch_links, 
             frozen_tags, 
             tag_owner, 
             ctx) = (&self.join_links, &self.branch_links, &mut self.frozen_tags, &self.tag_owner, &self.ctx);
        
        for (_name, recv) in &ctx.roll_tag_recv {
            while let Ok(proc_tag) = recv.try_recv() {
                HistoryParticipant::_send_freeze_sgn_dfs(join_links, branch_links, ctx, frozen_tags, tag_owner, &proc_tag);
            }
        }
    }

    // TODO: Check data structures are correctly updated
    fn run_dissapear_cycle(self: &mut Self)
    {
        while let Ok(diss_tag) = self.ctx.diss_tag_recv.try_recv() {
            match self.rev_join_links.remove(&diss_tag)
            {
                Some((sender, receiver)) => {
                    // Check that the sender is of enum variant Send
                    assert!(matches!(sender.proc, Process::Send(..)));
                    assert!(matches!(receiver.proc, Process::Recv(..)));

                    let sender_id = self.tag_owner.get(&sender.tag).unwrap();
                    let receiver_id = self.tag_owner.get(&receiver.tag).unwrap();

                    let sender_ch = self.ctx.ress_tag_send.get(sender_id).unwrap();
                    let receiver_ch = self.ctx.ress_tag_send.get(receiver_id).unwrap();

                    // TODO: ? Check if can ignore the result
                    let _ = sender_ch.send(RessurectMsg {
                        dissapeared_tag: diss_tag.clone(),
                        ress_tagged_proc: receiver.clone(),
                    });

                    // TODO: ? Check if can ignore the result
                    let _ = receiver_ch.send(RessurectMsg {
                        dissapeared_tag: diss_tag.clone(),
                        ress_tagged_proc: sender.clone(),
                    });
                },
                None => {
                    println!("Received a dissapear tag that no longer is in the history's records")
                },
            }
            
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



