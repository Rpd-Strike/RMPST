use std::{fmt::Display};

#[derive(Debug)]
pub struct ChName(pub String);

#[derive(Debug)]
pub struct ProcVar(pub String);

#[derive(Debug)]
pub struct TagVar(pub String);

#[derive(Debug, Clone)]
pub struct TagKey(String);

pub type PrimeState = Vec<TaggedPrimProc>;

#[derive(Debug, Clone)]
pub enum ProcTag 
{
    PTKey(TagKey),
    PTSplit(TagKey, i32, TagKey),
}

// A primitive process is the one that can appear as a top level process in the thread normal form
#[derive(Debug)]
pub enum PrimProcess
{
    End,
    RollK(TagKey),
    Send(ChName, Process),
    Recv(ChName, ProcVar, TagVar, Process),
}

// Our convention is that a process does not have free variables either for channels or process variables
#[derive(Debug)]
pub enum Process
{
    End,
    PVar(ProcVar),
    Par(Box<Process>, Box<Process>),
    Send(ChName, Box<Process>),
    Recv(ChName, ProcVar, TagVar, Box<Process>),
    RollV(TagVar),
    RollK(TagKey),
}

#[derive(Debug)]
pub struct TaggedPrimProc
{
    tag: ProcTag,
    proc: PrimProcess,
}