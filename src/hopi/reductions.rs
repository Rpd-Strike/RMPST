use std::collections::{HashSet};

use super::syntax::Process;

type ProcessComm = (String, (Process, Process), Process);

// ? Consider something like NormProcess = (Vec<String>, Vec<Process>)
//   Where the processes do not have the abstraction of channel names
// ? Same idea but for ClosedNormProcess = (Vec<String>, Process) Where there are no free names either for channels or Processes

// Given a process p in normal form
// Returns a list of possible commmunications that can happen in the form:
//   (Channel_name, (Sender_proc, Receiver_proc), Leftover_proc)
fn possible_communications(p: &Process) -> ProcessComm
{
    // TODO: not implemented
    return ("".to_string(), (Process::End, Process::End), Process::End);
}

// Given a process p in normal form, perform communication
// Note this happens in a synchronously
fn perform_communication(p: ProcessComm) -> Process
{
    // TODO: Not implented
    return Process::End;
}

// Takes in a process and returns a congruent process in its normal form
fn normalize_process(p: Process) -> Process
{
    match p {
        Process::End => p,
        Process::PVar(_) => p,
        Process::New(c_name, proc) => {
            Process::New(c_name, Box::new(normalize_process(*proc)))
        },
        Process::Par(A, B) => {
            let nA = normalize_process(*A);
            let nB = normalize_process(*B);
            // if any of nA or nB is Process::New, then bring that on top level
            if let Process::New(c_name, proc) = nA {
                let norm = normalize_process(Process::Par(proc, Box::new(nB)));
                let norm = change_chn_names(norm, &c_name);
                return Process::New(c_name, Box::new(norm));
            } else if let Process::New(c_name, proc) = nB {
                let norm = normalize_process(Process::Par(Box::new(nA), proc));
                let norm = change_chn_names(norm, &c_name);
                return Process::New(c_name, Box::new(norm));
            } else {
                return Process::Par(Box::new(nA), Box::new(nB));
            }
        },
        Process::Send(c_name, proc) => {
            let norm = normalize_process(*proc);
            let norm = change_chn_names(norm, &c_name);
            if let Process::New(first_name, subproc) = norm {
                let newSubProc = Process::Send(c_name, Box::new(*subproc));
                let newProc = Process::New(first_name, Box::new(newSubProc));
                return normalize_process(newProc)
            }
            return norm;
        },
        Process::Receive(c_name, p_name, proc) => {
            let norm = normalize_process(*proc);
            let norm = change_chn_names(norm, &c_name);
            if let Process::New(first_name, subproc) = norm {
                let newSubProc = Process::Receive(c_name, p_name, Box::new(*subproc));
                let newProc = Process::New(first_name, Box::new(newSubProc));
                return normalize_process(newProc)
            }
            return norm;
        },
    }
}

// Takes a process in normal form, a channel name c_name
// Returns an equivalent normalized process which has all restricted channel names different to c_name
fn change_chn_names(p: Process, ch_name: &String) -> Process 
{
    match p {
        Process::End => Process::End,
        Process::PVar(_) => todo!(),
        Process::New(_, _) => todo!(),
        Process::Par(_, _) => todo!(),
        Process::Send(_, _) => todo!(),
        Process::Receive(_, _, _) => todo!(),
    }
}

// Changes the channel name c_from and changes its name to c_to 
// This substitution happens for all bound variables named c_from
fn chn_alpha_conversion(p: Process, c_from: String, c_to: String) -> Result<Process, String>
{
    if chn_free_names(&p).contains(&c_to) {
        return Err(format!("Name {} is already a free name inside the given process", c_to));
    }

    return Ok(_rec_chn_alpha_conversion(p, 0, &c_from, c_to))
}

// Precondition: c_to NOT IN fn(p)
// the boolean flag tells us if the c_from channel has been captured exactly once (so no nested captures are modified)
fn _rec_chn_alpha_conversion(p: Process, captures: i32, c_from: &String, c_to: String) -> Process {
    match p {
        // Trivial case
        Process::End => Process::End,
        // A process variable name is not subject to this alpha conversion
        Process::PVar(_) => p,
        // If the new channel name is c_from, change the boolean flag
        Process::New(c_name, proc) => {
            if &c_name == c_from {
                Process::New(c_to.clone(), Box::new(_rec_chn_alpha_conversion(*proc, captures + 1, c_from, c_to)))
            } else {
                Process::New(c_name, Box::new(_rec_chn_alpha_conversion(*proc, captures, c_from, c_to)))
            }
        },
        Process::Par(A, B) => {
            let pA = _rec_chn_alpha_conversion(*A, captures, c_from, c_to.clone());
            let pB = _rec_chn_alpha_conversion(*B, captures, c_from, c_to);
            Process::Par(Box::new(pA), Box::new(pB))
        },
        Process::Send(c_name, proc) => {
            if captures == 1 && &c_name == c_from {
                Process::Send(c_to.clone(), Box::new(_rec_chn_alpha_conversion(*proc, captures, c_from, c_to)))
            }
            else {
                Process::Send(c_name, Box::new(_rec_chn_alpha_conversion(*proc, captures, c_from, c_to)))
            }
        },
        Process::Receive(_, _, _) => todo!(),
    }
}

// Returns a list of free process variable names
fn var_free_names(p: &Process) -> HashSet<String>
{
    return _var_free_names_env(p, &mut HashSet::new())
}

fn _var_free_names_env(p: &Process, ctx: &mut HashSet<String>) -> HashSet<String>
{
    match p {
        // Trivial
        Process::End => HashSet::new(),
        // Check if we have it in the context
        Process::PVar(p_name) => {
            let mut r = HashSet::new();
            if !ctx.contains(p_name) {
                r.insert(p_name.to_string());
            }
            r
        }
        // New channel name, just cal lrecursively
        Process::New(_, proc) => {
            _var_free_names_env(proc, ctx)
        },
        // Run both processes with a copy of the context
        Process::Par(A, B) => {
            let mut ctx2 = ctx.clone();
            
            let mut name_a = _var_free_names_env(A, ctx);
            let     name_b = _var_free_names_env(B, &mut ctx2);

            name_a.extend(name_b.into_iter());
            name_a
        },
        // Ignore channel name, just call recursively
        Process::Send(_, proc) => {
            _var_free_names_env(proc, ctx)
        },
        // This is when we create a new process variable
        // Add in context, then call
        Process::Receive(_, p_name, proc) => {
            ctx.insert(p_name.to_string());
            _var_free_names_env(proc, ctx)
        }
    }
}

// Returns a list of free channel names
fn chn_free_names(p: &Process) -> HashSet<String>
{
    return _chn_free_names_env(p, &mut HashSet::new())
}

// Return a list of free channel names w.r.t context
fn _chn_free_names_env(p: &Process, ctx: &mut HashSet<String>) -> HashSet<String>
{
    match p {
        // Trivial
        Process::End => HashSet::new(),
        // This is a process variable name so we do not care about it
        Process::PVar(_) => HashSet::new(),
        // We create a new channel name, place in context and call recursively
        Process::New(c_name, proc) => {
            ctx.insert(c_name.to_string());
            _chn_free_names_env(proc, ctx)
        },
        // Call recursively with a copy of the context, then combine
        Process::Par(A, B) => {
            let mut ctx2 = ctx.clone();
            
            let mut name_a = _chn_free_names_env(A, ctx);
            let     name_b = _chn_free_names_env(B, &mut ctx2);

            name_a.extend(name_b.into_iter());
            name_a
        },
        // First check if it's free name, then call recursively without placing in context
        Process::Send(c_name, proc) => {
            let has_name = ctx.contains(c_name);
            let mut r = _chn_free_names_env(proc, ctx);
            if !has_name {
                r.insert(c_name.to_string());
            } else { }
            r
        }
        // Same as Send case
        Process::Receive(c_name, p_var, proc) => {
            let has_name = ctx.contains(c_name);
            let mut r = _chn_free_names_env(proc, ctx);
            if !has_name {
                r.insert(c_name.to_string());
            } else { }
            r
        }
    }
}

// returns a list of free tag names 
// ? This should always be empty, at least if coming from an encoding of local types
fn tag_free_names(p: &Process) -> Vec<String>
{
    todo!()
}

