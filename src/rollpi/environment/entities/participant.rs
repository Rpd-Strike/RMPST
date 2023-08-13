use core::time;
use std::collections::{HashMap, HashSet};
use crate::rollpi::{environment::{components::picker::{Strategy, PrimProcTransf}, types::{MemoryPiece, PartyComm}}, syntax::{TagKey, PrimeState, ProcTag, TaggedPrimProc, PrimProcess, TaggedProc, Process}, logger::file_log::FileLogger};


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
    
    party_context: PartyContext,
    
    strategy: Box<dyn Strategy>,
}

pub struct ParticipantState
{
    pub dead_state: PrimeState,
    pub live_state: PrimeState,

    pub frozen_tags: HashSet<ProcTag>,
}

pub struct PartyContext
{
    id: String,
    comm_ctx: PartyCommCtx,
    tag_ctx: TagCreator,
    logger: FileLogger,
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

    pub fn get_logger(&mut self) -> &mut FileLogger
    {
        &mut self.logger
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
    // TODO: This is not an arbitrary ProcTag, but rather a TagKey
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
                live_state: state,
                dead_state: vec![],
                frozen_tags: HashSet::default(), 
            },
            strategy,
            party_context: PartyContext {
                id: id.clone(),
                comm_ctx: comm_context,
                tag_ctx: TagCreator::new(id.clone()),
                logger: FileLogger::new(format!("{}", id)),
            },
        }
    }

    fn evolve_state(&mut self)
    {
        let (ctx, state, strat) = (&mut self.party_context, &mut self.state, &*self.strategy);
        let action = strat.run_strategy(ctx, state);
        let pr_state = &mut state.live_state;

        // Remove the ran process, append the new processes
        match action {
            Some(PrimProcTransf(pos, proc)) => {
                let x = pr_state.remove(pos);
                match x.proc {
                    PrimProcess::RollK(_) | PrimProcess::End => state.dead_state.push(x),
                    _ => ()
                };

                pr_state.extend(proc.into_iter());
            },
            None => {
                println!("I am done .... {}", ctx.get_id());
                std::thread::sleep(time::Duration::from_secs(1));
            }
        }
    }   

    fn _mark_live_proc_as_frozen(_f_tag: &ProcTag, _pr_state: &PrimeState)
    {
        // TODO: Do smarter things here later
        //       like remembering causal links
    }

    fn freeze_logic(self: &mut Self)
    {
        let (state, ctx) = (&mut self.state, &mut self.party_context);
        let (comm_ctx, logger) = (&mut ctx.comm_ctx, &mut ctx.logger);
        let ParticipantState{live_state, dead_state, frozen_tags} = state;

        while let Ok(tag) = comm_ctx.rollback_ctx.freeze_not_channel.try_recv() {
            Participant::_mark_live_proc_as_frozen(&tag, live_state);
            frozen_tags.insert(tag.clone());

            logger.log(format!("Freezing process with tag {:?}\n", tag));
        }
    }
    
    // For all live processes that are frozen, delete them and send dissapear notif
    // For all the split processes, roll back to the united one, and dissapear that one 
    //     (by dissapearing all the fragments and sending dissapear notif for the original one)
    fn dissapear_logic(self: &mut Self)
    {
        let (state, ctx) = (&mut self.state, &mut self.party_context);
        let (comm_ctx, logger) = (&ctx.comm_ctx, &mut ctx.logger);
        let ParticipantState{live_state, dead_state, frozen_tags} = state;
        let mut key_fragments = HashMap::new();
        
        // Part 1: Identify, send dissapear notif and eliminate from the frozen processes set the simple frozen tags

        // This function has 2 purposes
        // 1: Identify simple frozen processes that can be dissapeared, send diss notif and remove from frozen
        // 2: Identify split tags and count for each parent key how many fragments are missing from the whole split
        let mut do_retain_simple = |container: &mut PrimeState| {
            container.retain(|TaggedPrimProc { tag, .. }| {
                match tag {
                    ProcTag::PTKey(_) => 
                        if frozen_tags.contains(tag) {
                            let diss_ch = &comm_ctx.dissapear_ctx.diss_send_channel;
                            // TODO: ? Decide if ignore or not send error
                            let _ = diss_ch.send(tag.clone());
                            assert!(frozen_tags.remove(tag));
    
                            logger.log(format!("Dissapearing process with tag {:?}", tag));
    
                            false
                        } else {
                            true
                        },
                    ProcTag::PTSplit(_frag_t, total_cnt, og_t) => {
                        if frozen_tags.contains(&ProcTag::PTKey(og_t.clone())) {
                            let curr_missing = key_fragments.entry(og_t.clone())
                                .or_insert(*total_cnt);
                            // subtract one to signal adding the current process
                            *curr_missing -= 1;
                        };
                        true
                    },
            }});
        };

        // get rid of processes which aren't "parallel split"
        do_retain_simple(live_state);
        do_retain_simple(dead_state);

        // Part 2: Identify, send dissapear notif and eliminate from the frozen processes set the split frozen tags
        
        // Retain only the original processes that are wholly frozen
        key_fragments.retain(|_key, missing_cnt| {
            *missing_cnt == 0
        });
        
        // Eliminate the frozen fragments
        let do_retain_splits = |container: &mut PrimeState| {
            container.retain(|TaggedPrimProc { tag, .. }| {
                if let ProcTag::PTSplit(_fr_key, _total_cnt, orig_key) = tag {
                    !key_fragments.contains_key(orig_key)
                } else {
                    true
                }
            });
        };

        do_retain_splits(live_state);
        do_retain_splits(dead_state);

        // Send the dissapear notifications
        key_fragments.keys().for_each(|k| {
            let diss_ch = &comm_ctx.dissapear_ctx.diss_send_channel;
            // TODO: ? Decide if ignore or not send error
            let _ = diss_ch.send(ProcTag::PTKey(k.clone()));

            logger.log(format!("Dissapearing process with tag {:?}", k));
        });

        // Eliminate from the frozen processes set
        frozen_tags.retain(|tag| {
            if let ProcTag::PTSplit(_fr_key, _total_cnt, orig_key) = tag {
                !key_fragments.contains_key(orig_key)
            } else {
                true
            }
        });

    }

    fn ressurect_logic(self: &mut Self)
    {
        let (state, ctx) = (&mut self.state, &self.party_context);
        let ress_ch = &ctx.get_comm_ctx().ressurect_ctx.ress_recv_channel;

        while let Ok(RessurectMsg { dissapeared_tag: _, ress_tagged_proc }) = ress_ch.try_recv() {
            // TODO: Inspect if this should be here, normaly remoing from frozen happens in dissapear logic
            // state.frozen_tags.remove(&ress_tagged_proc.tag);
            // We know that what we get now is a Send or Recv so we just use that as a Singleton PrimeState with the given tag
            let TaggedProc { tag, proc } = ress_tagged_proc;
            
            // assert that proc is either Send or Recv
            let is_send_recv = match proc {
                Process::Send(_, _) | Process::Recv(_, _, _, _) => true,
                _ => false,
            };
            assert!(is_send_recv);

            state.live_state.append(&mut vec![TaggedPrimProc { tag, proc: proc.to_prime_process() }]);
        }
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
