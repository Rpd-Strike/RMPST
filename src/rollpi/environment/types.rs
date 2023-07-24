use std::collections::HashMap;

use crossbeam::channel::{Receiver, Sender};

use crate::rollpi::syntax::{Process, ProcTag, ChName, ProcVar, TagVar, TagKey};

pub struct PartyComm
{
    sender_id: String,
    process: Process,
    tag: ProcTag,
}

pub struct MemoryPiece
{
    sender: (ProcTag, (ChName, Process)),
    receiver: (ProcTag, (ChName, ProcVar, TagVar, Process)),
    new_mem_tag: TagKey,
}

