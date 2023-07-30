// trait 

use super::syntax::{Process, ProcVar, TagVar, TagKey};

pub fn perform_alpha_conv_proc(next_proc: &Process, p_var: ProcVar, in_process: &Process, t_var: TagVar, new_tag: TagKey) -> Process
{
    match next_proc {
        Process::End => Process::End,
        Process::PVar(var) => {
            assert!(*var == p_var);

            *in_process
        },
        Process::Par(a, b) => {
            let conv_a = perform_alpha_conv_proc(a, p_var, in_process, t_var, new_tag);
            let conv_b = perform_alpha_conv_proc(b, p_var, in_process, t_var, new_tag);
            Process::Par(Box::new(conv_a), Box::new(conv_b))
        },
        Process::Send(ch_name, p) => {
            let conv_p = perform_alpha_conv_proc(&p, p_var, in_process, t_var, new_tag);
            Process::Send(*ch_name, Box::new(conv_p))
        },
        Process::Recv(_, _, _, _) => todo!(),
        Process::RollV(_) => todo!(),
        Process::RollK(_) => todo!(),
    }
}