



use crate::rollpi::syntax::{Process, ProcTag, ChName, ProcVar, TagVar, TagKey};

pub struct PartyComm
{
    pub sender_id: String,
    pub process: Process,
    pub tag: ProcTag,
}

pub struct MemoryPiece
{
    // ids of sender, receiver
    pub ids: (String, String),
    // Sender process
    pub sender: (ProcTag, (ChName, Process)),
    // Receiver process
    pub receiver: (ProcTag, (ChName, ProcVar, TagVar, Process)),
    // Tag of the new process
    pub new_mem_tag: TagKey,
}

