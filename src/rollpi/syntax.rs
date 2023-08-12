use core::panic;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct ChName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcVar(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagVar(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagKey(pub String);

pub type PrimeState = Vec<TaggedPrimProc>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProcTag 
{
    // A simple identifier, usually meaning the process is not a parallel composition
    PTKey(TagKey),
    // (paralel piece of new proc, nr_in_order, original_proc)
    PTSplit(TagKey, i32, TagKey),
}

// Our convention is that a process does not have free variables either for channels or process variables
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct TaggedProc
{
    pub tag: ProcTag,
    pub proc: Process,
}

// A primitive process is the one that can appear as a top level process in the thread normal form
#[derive(Debug, Clone)]
pub enum PrimProcess
{
    End,
    RollK(TagKey),
    Send(ChName, Process),
    Recv(ChName, ProcVar, TagVar, Process),
}

#[derive(Debug, Clone)]
pub struct TaggedPrimProc
{
    pub tag: ProcTag,
    pub proc: PrimProcess,
}

impl Process 
{
    pub fn to_prime_process(self: &Self) -> PrimProcess
    {
        match self {
            Process::PVar(_) => panic!("Process to prime process conversion: PVar not allowed"),
            Process::Par(_, _) => panic!("Process to prime process conversion: Par not allowed"),
            Process::End => 
                PrimProcess::End,
            Process::Send(chn, p) => 
                PrimProcess::Send(chn.clone(), *p.clone()),
            Process::Recv(chn, pv, tv, p) => 
                PrimProcess::Recv(chn.clone(), pv.clone(), tv.clone(), *p.clone()),
            Process::RollV(_) => panic!("Process to prime process conversion: RollV not allowed, only RollK(ey) allowed"),
            Process::RollK(roll_key) => 
                PrimProcess::RollK(roll_key.clone()),
        }
    }

    pub fn to_tagged_process(self: Self, arg_tag: ProcTag) -> TaggedProc
    {
        TaggedProc { tag: arg_tag, proc: self }
    }

    pub fn parallel_compose(mut procs: Vec<Process>) -> Process
    {
        let first_proc = procs.pop();
        let procs = first_proc.map(|first_p| {
            procs.into_iter().rfold(first_p, |acc, p| {
                Process::Par(Box::new(p), Box::new(acc))
            })
        });
        
        procs.unwrap_or(Process::End)
    }

    fn get_first_order_par_processes(self: Self) -> Vec<Process>
    {
        if let Process::Par(a, b) = self {
            let mut vec_a = a.get_first_order_par_processes();
            let mut vec_b = b.get_first_order_par_processes();
            vec_a.append(&mut vec_b);
            vec_a
        } else {
            vec![self]
        }
    }
}

impl PrimProcess 
{
    pub fn to_process(self: Self) -> Process
    {
        match self {
            PrimProcess::End => 
                Process::End,
            PrimProcess::RollK(tag) => 
                Process::RollK(tag),
            PrimProcess::Send(ch_name, proc) => 
                Process::Send(ch_name, Box::new(proc)),
            PrimProcess::Recv(ch_name, p_var, t_var, proc) => 
                Process::Recv(ch_name, p_var, t_var, Box::new(proc)),
        }
    }
}

impl TaggedProc
{
    pub fn to_prime_state(self: Self) -> PrimeState
    {
        let Self { tag, proc } = self; 
        let og_key = match tag {
            ProcTag::PTKey(t_key) => t_key,
            ProcTag::PTSplit(t_key, _cnt, _og_t) => t_key,
        };

        let vec_procs = proc.get_first_order_par_processes();
        let total_cnt_procs = vec_procs.len();

        vec_procs.iter().enumerate().map(|(i, p)| {
            let frag_tag = ProcTag::PTSplit(TagKey(format!("sp_{}_{}", i, og_key.0)), total_cnt_procs as i32, og_key.clone());
            TaggedPrimProc {
                tag: frag_tag,
                proc: p.to_prime_process(),
            }
        }).collect()
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
        all_chn_names_proc(&proc.clone().to_process())
    }).flatten().collect()
}

pub fn check_all_pvar_closed(proc: &Process) -> bool
{
    _rec_check_pvar_closed(proc, &mut HashSet::new())
}

fn _rec_check_pvar_closed(proc: &Process, env: &mut HashSet<ProcVar>) -> bool
{
    match proc {
        Process::End => true,
        Process::PVar(pvar) => 
            env.contains(pvar),
        Process::Par(a, b) => {
            let mut env_copy = env.clone();
            _rec_check_pvar_closed(a, &mut env_copy) && _rec_check_pvar_closed(b, env)
        },
        Process::Send(_, p) => 
            _rec_check_pvar_closed(p, env),
        Process::Recv(_, pvar, _, p) => {
            env.insert(pvar.clone());
            _rec_check_pvar_closed(p, env)
        },
        Process::RollV(_) => true,
        Process::RollK(_) => true,
    }
}

pub fn check_all_tvar_closed(proc: &Process) -> bool
{
    _rec_check_tvar_closed(proc, &mut HashSet::new())
}

fn _rec_check_tvar_closed(proc: &Process, env: &mut HashSet<TagVar>) -> bool
{
    match proc {
        Process::End => true,
        Process::PVar(_) => true,
        Process::Par(a, b) => {
            let mut env_copy = env.clone();
            _rec_check_tvar_closed(a, &mut env_copy) && _rec_check_tvar_closed(b, env)
        },
        Process::Send(_, p) => 
            _rec_check_tvar_closed(p, env),
        Process::Recv(_, _, tvar, p) => {
            env.insert(tvar.clone());
            _rec_check_tvar_closed(p, env)
        },
        Process::RollV(tag_var) => 
            env.contains(tag_var),
        Process::RollK(_) => false,
    }
}

pub fn check_unique_pvar_tvar(proc: &Process) -> bool
{
    _rec_check_unique_pvar_tvar(proc, &mut HashSet::new(), &mut HashSet::new())
}

fn _rec_check_unique_pvar_tvar(proc: &Process, p_env: &mut HashSet<ProcVar>, t_env: &mut HashSet<TagVar>) -> bool
{
    match proc {
        Process::End => true,
        Process::PVar(_) => true,
        Process::RollV(_) => true,
        Process::RollK(_) => true,
        Process::Par(a, b) => {
            let mut p_env_copy = p_env.clone();
            let mut t_env_copy = t_env.clone();
            _rec_check_unique_pvar_tvar(a, &mut p_env_copy, &mut t_env_copy) && _rec_check_unique_pvar_tvar(b, p_env, t_env)
        },
        Process::Send(_, p) => 
            _rec_check_unique_pvar_tvar(p, p_env, t_env),
        Process::Recv(_, pvar, tvar, p) => {
            if p_env.contains(pvar) || t_env.contains(tvar) {
                return false;
            }
            p_env.insert(pvar.clone());
            t_env.insert(tvar.clone());
            _rec_check_unique_pvar_tvar(p, p_env, t_env)
        },
    }
}

pub fn check_initial_conf_list(procs: &Vec<Process>) -> bool
{
    procs.iter().all(|proc| check_initial_conf(proc))
}

pub fn check_initial_conf(proc: &Process) -> bool
{
    check_all_pvar_closed(proc) && 
    check_all_tvar_closed(proc) && 
    // check_unique_pvar_tvar(proc)

    return true
}
