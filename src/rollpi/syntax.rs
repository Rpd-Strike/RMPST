use std::{fmt::Display, collections::HashSet};

#[derive(Debug, Clone)]
pub struct ChName(pub String);

#[derive(Debug, Clone)]
pub struct ProcVar(pub String);

#[derive(Debug, Clone)]
pub struct TagVar(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagKey(String);

pub type PrimeState = Vec<TaggedPrimProc>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone)]
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
    pub tag: ProcTag,
    pub proc: PrimProcess,
}

pub fn prime_proc_to_process(prime: &PrimProcess) -> Process
{
    match prime {
        PrimProcess::End => 
            Process::End,
        PrimProcess::RollK(tag) => 
            Process::RollK(tag.clone()),
        PrimProcess::Send(ch_name, proc) => 
            Process::Send(ch_name.clone(), Box::new(proc.clone())),
        PrimProcess::Recv(ch_name, p_var, t_var, proc) => 
            Process::Recv(ch_name.clone(), p_var.clone(), t_var.clone(), Box::new(proc.clone())),
    }
}

pub fn all_chn_names_proc(proc: &Process) -> HashSet<String>
{
    match proc 
    {
        Process::End => HashSet::new(),
        Process::PVar(_) => HashSet::new(),
        Process::Par(p_left, p_right) => {
            let iter_left = all_chn_names_proc(p_left).into_iter();
            let iter_righ = all_chn_names_proc(p_right).into_iter();
            // combine iterators with chain
            iter_left.chain(iter_righ).collect()
        }
        Process::Send(ChName(ch_name), proc) => {
            let mut set = all_chn_names_proc(proc);
            set.insert(ch_name.clone());
            set
        },
        Process::Recv(ChName(ch_name), _, _, proc) => {
            let mut set = all_chn_names_proc(proc);
            set.insert(ch_name.clone());
            set
        },
        Process::RollV(_) => HashSet::new(),
        Process::RollK(_) => HashSet::new(),
    }
}

pub fn all_chn_names_state(proc: &PrimeState) -> HashSet<String>
{
    proc.iter().map(|TaggedPrimProc{proc, ..}| {
        all_chn_names_proc(&prime_proc_to_process(&proc))
    }).flatten().collect()
}