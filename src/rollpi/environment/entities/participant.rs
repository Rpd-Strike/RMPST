use core::time;
use std::collections::{HashMap, HashSet};
use crate::rollpi::{environment::{components::picker::{Strategy, PrimProcTransf}, types::{MemoryPiece, PartyComm}}, syntax::{TagKey, PrimeState, ProcTag, TaggedPrimProc}};


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
    state: ParticipantState,
    
    strategy: Box<dyn Strategy>,

    party_context: PartyContext,
}

struct ParticipantState
{
    pr_state: PrimeState,

    frozen_tags: HashSet<ProcTag>,
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
    pub history_ctx: HistTagContext,
    pub rollback_ctx: RollbackContext,
    pub dissapear_ctx: DissapearContext,
    pub ressurect_ctx: RessurectContext,
}

pub struct ChMsgContext<'a>
{
    pub send_channel: &'a Sender<PartyComm>,
    pub recv_channel: &'a Receiver<PartyComm>,
}

pub struct HistTagContext
{
    pub hist_tag_channel: Sender<MemoryPiece>,
    pub hist_conf_channel: Receiver<TagKey>,
}

pub struct RollbackContext
{
    pub roll_tag_channel: Sender<ProcTag>,
    pub freeze_not_channel: Receiver<ProcTag>,
}

pub struct DissapearContext
{
    pub diss_send_channel: Sender<ProcTag>,
}

pub struct RessurectContext
{
    pub ress_recv_channel: Receiver<RessurectMsg>,
}


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
            println!("Creating channel-pair for {}", id);
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
            state: ParticipantState {
                pr_state: state,
                frozen_tags: HashSet::default(), 
            },
            strategy,
            party_context: PartyContext {
                id,
                comm_ctx: comm_context,
                tag_ctx: TagCreator::default(),
            },
        }
    }

    fn evolve_state(&mut self)
    {
        let (ctx, state, strat) = self.borrow_data();
        let action = strat.run_strategy(ctx, &state.pr_state);
        let pr_state = &mut state.pr_state;

        // Remove the ran process, append the new processes
        if let Some(PrimProcTransf(pos, proc)) = action {
            pr_state.swap_remove(pos);
            pr_state.extend(proc.into_iter());
        } else {
            println!("I am done .... {}", ctx.get_id());
            std::thread::sleep(time::Duration::from_secs(1));
        }
    }   

    fn _mark_live_proc_as_frozen(_f_tag: &ProcTag, _pr_state: &PrimeState)
    {
        // TODO: Do smarter things here later
        //       like remembering causal links
    }

    fn freeze_logic(self: &mut Self)
    {
        let (state, ctx) = (&mut self.state, &self.party_context);
        let ParticipantState{pr_state, frozen_tags} = state;

        while let Ok(tag) = ctx.get_comm_ctx().rollback_ctx.freeze_not_channel.try_recv() {
            Participant::_mark_live_proc_as_frozen(&tag, pr_state);
            frozen_tags.insert(tag);
        }
    }
    
    // For all live processes that are frozen, delete them and send dissapear notif
    // For all the split processes, roll back to the united one, and dissapear that one 
    //     (by dissapearing all the fragments and sending dissapear notif for the original one)
    fn dissapear_logic(self: &mut Self)
    {
        let (state, ctx) = (&mut self.state, &self.party_context);
        let ParticipantState{pr_state, frozen_tags} = state;
        let mut key_fragments = HashMap::new();
        
        // get rid of processes which aren't "parallel split"
        pr_state.retain(|TaggedPrimProc { tag, .. }| {
            if frozen_tags.contains(tag) {
                match tag {
                    ProcTag::PTKey(_) => {
                        let diss_ch = &ctx.get_comm_ctx().dissapear_ctx.diss_send_channel;
                        // TODO: ? Decide if ignore or not send error
                        let _ = diss_ch.send(tag.clone());
                        assert!(frozen_tags.remove(tag));

                        false
                    },
                    ProcTag::PTSplit(_frag_t, total_cnt, og_t) => {
                        let curr_missing = key_fragments.entry(og_t.clone())
                            .or_insert(*total_cnt);
                        // subtract one to signal adding the current process
                        *curr_missing -= 1;

                        true
                    },
                }
            } else {
                true
            }
        });

        // Retain only the original processes that are wholly frozen
        key_fragments.retain(|_key, missing_cnt| {
            *missing_cnt == 0
        });
        
        // Eliminate the frozen fragments
        pr_state.retain(|TaggedPrimProc { tag, proc: _ }| {
            if let ProcTag::PTSplit(_fr_key, _total_cnt, orig_key) = tag {
                !key_fragments.contains_key(orig_key)
            } else {
                true
            }
        });

        // Send the dissapear notifications
        key_fragments.keys().for_each(|k| {
            let diss_ch = &ctx.get_comm_ctx().dissapear_ctx.diss_send_channel;
            // TODO: ? Decide if ignore or not send error
            let _ = diss_ch.send(ProcTag::PTKey(k.clone()));
        });
    }

    fn ressurect_logic(self: &mut Self)
    {
        let (state, ctx) = (&mut self.state, &self.party_context);
        let ress_ch = &ctx.get_comm_ctx().ressurect_ctx.ress_recv_channel;

        while let Ok(RessurectMsg { descendant_tag: _, tagged_proc }) = ress_ch.try_recv() {
            state.frozen_tags.remove(&tagged_proc.tag);
            state.pr_state.append(&mut tagged_proc.to_prime_state());
        }
    }

    fn borrow_data(&mut self) -> (&mut PartyContext, &mut ParticipantState, &dyn Strategy)
    {
        (&mut self.party_context, &mut self.state, &*self.strategy)
    }

}

impl Runnable for Participant
{
    fn run(mut self: Self)
    {
        loop {
            self.evolve_state();

            self.freeze_logic();

            self.dissapear_logic();

            self.ressurect_logic();
        }
    }
}

use crossbeam::channel::{Sender, Receiver, unbounded};

use super::{tag_creator::TagCreator, history::RessurectMsg};
