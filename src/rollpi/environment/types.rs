



use crate::rollpi::syntax::{Process, ProcTag, ChName, ProcVar, TagVar, TagKey, TaggedProc};

#[derive(Debug)]
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
    pub sender: TaggedProc,
    // Receiver process
    pub receiver: TaggedProc,
    // Tag of the new process
    pub new_mem_tag: TagKey,

    // This has the role of forcing to use the new function to create MemoryPiece
    _secret: (),
}

impl MemoryPiece
{
    pub fn new( ids: (String, String), 
                sender: (ProcTag, (ChName, Process)), 
                receiver: (ProcTag, (ChName, ProcVar, TagVar, Process)),
                new_mem_tag: TagKey) -> Self
    {
        MemoryPiece { 
            ids, 
            sender: TaggedProc { 
                tag: sender.0, 
                proc: Process::Send(sender.1.0, Box::new(sender.1.1)),
            },
            receiver: TaggedProc { 
                tag: receiver.0, 
                proc: Process::Recv(receiver.1.0, receiver.1.1, receiver.1.2, Box::new(receiver.1.3)),
            },
            new_mem_tag, 
            _secret: () 
        }
    }
}

