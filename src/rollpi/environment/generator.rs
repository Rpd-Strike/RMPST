use std::collections::{HashSet, HashMap};

use crossbeam::channel::unbounded;

use crate::rollpi::syntax::{PrimeState, all_chn_names_proc, TagKey, ProcTag};

use super::{components::picker::Strategy, entities::{participant::{Participant, PartyCommCtx, PartyChPool, HistTagContext, RollbackContext, DissapearContext, RessurectContext}, history::{HistoryContext, HistoryParticipant, RessurectMsg}}, types::MemoryPiece};

#[derive(Default)]
pub struct Generator
{
    participants: HashMap<String, (Box<dyn Strategy>, PrimeState)>,
}

impl Generator
{   
    fn create_participant(
        self: &mut Self,
        id: String,
        proc: PrimeState, 
        strat: Option<Box<dyn Strategy>>,
    )
    {
        self.participants.insert(id, (
            strat.unwrap_or_default(),
            proc,
        ));
    }

    // Add a participant to be generated. 
    // Returns true if the given id is not already used, false if automatically generated
    pub fn take_participant_conf(
        self: &mut Self,
        proc: PrimeState,
        id: Option<String>,
        strategy: Option<Box<dyn Strategy>>,
    ) -> bool
    {
        // try to add the id given from the argument if not in the hashset
        if let Some(id) = id {
            if !self.participants.contains_key(&id) {
                self.create_participant(id, proc, strategy);
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

        self.create_participant(id, proc, strategy);
        return false;
    }

    pub fn generate_participants(self: Self) -> (Vec<Participant>, HistoryParticipant)
    {
        // Create channels and create copy for each of the participants
        let channels = self.participants.iter()
            .map(|(_id, (_, proc))| {
                proc.iter().map(|tag_proc| {
                    all_chn_names_proc(&tag_proc.proc.clone().to_process())
                })
                .flatten().collect::<HashSet<_>>()
            })
            .flatten().collect::<HashSet<_>>();

        let part_ch_pool = PartyChPool::new(channels.into_iter());
        
        let (diss_send, diss_recv) = unbounded::<ProcTag>();
        let mut memory_context = HistoryContext::new(diss_recv);
        

        let mut create_party_context = |id: &String| {
            let (h_tag_send, h_tag_recv) = unbounded::<MemoryPiece>();
            let (h_not_send, h_not_recv) = unbounded::<TagKey>();

            let (r_tag_send, r_tag_recv) = unbounded::<ProcTag>();
            let (r_frz_send, r_frz_recv) = unbounded::<ProcTag>();

            let (s_tag_send, s_tag_recv) = unbounded::<RessurectMsg>();

            memory_context.hist_tag_recv.insert(id.clone(), h_tag_recv);
            memory_context.hist_not_send.insert(id.clone(), h_not_send);
            memory_context.roll_tag_recv.insert(id.clone(), r_tag_recv);
            memory_context.roll_frz_send.insert(id.clone(), r_frz_send);
            memory_context.ress_tag_send.insert(id.clone(), s_tag_send);

            PartyCommCtx {
                channel_pool: part_ch_pool.clone(),
                history_ctx: HistTagContext {
                    hist_tag_channel: h_tag_send,
                    hist_conf_channel: h_not_recv,
                },
                rollback_ctx: RollbackContext {
                    roll_tag_channel: r_tag_send,
                    freeze_not_channel: r_frz_recv,
                },
                dissapear_ctx: DissapearContext {
                    diss_send_channel: diss_send.clone(),
                },
                ressurect_ctx: RessurectContext {
                    ress_recv_channel: s_tag_recv,
                },
            }
        };

        let parties = self.participants
            .into_iter()
            .map(|(id, (strat, proc)) | {
                let c_ctx = create_party_context(&id);
                Participant::new(
                    id,
                    proc,
                    strat,
                    c_ctx,
                )
            }).collect();
        
        let hist = HistoryParticipant::new(memory_context);

        return (parties, hist)
    }
}