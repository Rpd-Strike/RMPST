// trait 

use super::syntax::{Process, ProcVar, TagVar, TagKey, PrimeState, TaggedPrimProc, PrimProcess, ProcTag};

pub fn perform_alpha_conv_proc(next_proc: &Process, p_var: ProcVar, in_process: Process, t_var: TagVar) -> Process
{
    match next_proc {
        Process::End => Process::End,
        Process::PVar(var) => {
            assert!(*var == p_var);

            in_process
        },
        Process::Par(a, b) => {
            let conv_a = perform_alpha_conv_proc(a, p_var.clone(), in_process.clone(), t_var.clone());
            let conv_b = perform_alpha_conv_proc(b, p_var, in_process, t_var);
            Process::Par(Box::new(conv_a), Box::new(conv_b))
        },
        Process::Send(ch_name, p) => {
            let conv_p = perform_alpha_conv_proc(&p, p_var, in_process, t_var);
            Process::Send(ch_name.clone(), Box::new(conv_p))
        },
        Process::Recv(_, _, _, _) => todo!(),
        Process::RollV(_) => todo!(),
        Process::RollK(_) => todo!(),
    }
}

pub fn transform_to_prime_state(proc: Process, original_tag: TagKey) -> PrimeState
{
    let ptag = ProcTag::PTKey(original_tag.clone());
    match proc {
        Process::End => 
            vec![TaggedPrimProc{ proc: PrimProcess::End, tag: ptag }],
        Process::PVar(var) => 
            panic!("PVar should not be represent a prime state (should be substituted)"),
        Process::RollV(t_var) =>
            panic!("Tvar should not be represent a prime state (should be substituted)"),
        Process::Par(a, b) => {
            let prime_a = transform_to_prime_state(*a, original_tag.clone());
            let prime_b = transform_to_prime_state(*b, original_tag);
            prime_a.into_iter().chain(prime_b.into_iter()).collect()
        },
        Process::Send(ch_name, p) => {
            vec![TaggedPrimProc{ proc: PrimProcess::Send(ch_name, *p), tag: ptag }]
        },
        Process::Recv(ch_name, p_var, t_var, p) => {
            vec![TaggedPrimProc{ proc: PrimProcess::Recv(ch_name, p_var, t_var, *p), tag: ptag }]
        },
        Process::RollK(roll_key) => {
            vec![TaggedPrimProc{ proc: PrimProcess::RollK(roll_key), tag: ptag }]
        },
    }
}