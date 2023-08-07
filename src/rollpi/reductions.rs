// trait 

use super::syntax::{Process, ProcVar, TagVar, TagKey};

impl Process
{
    pub fn substitution_on_trigger(self: Self, p_var: ProcVar, in_process: &Process, t_var: TagVar, new_tag: &TagKey) -> Process
    {
        match self {
            Process::End => Process::End,
            Process::PVar(var) => {
                if var == p_var {
                    in_process.clone()
                } else {
                    Process::PVar(var)
                }
            },
            Process::Par(a, b) => {
                let conv_a = a.substitution_on_trigger(p_var.clone(), in_process, t_var.clone(), new_tag);
                let conv_b = b.substitution_on_trigger(p_var, in_process, t_var, new_tag);
                Process::Par(Box::new(conv_a), Box::new(conv_b))
            },
            Process::Send(ch_name, p) => {
                let conv_p = p.substitution_on_trigger(p_var, in_process, t_var, new_tag);
                Process::Send(ch_name, Box::new(conv_p))
            },
            // !! This works because Tvars and Pvars are unique
            Process::Recv(_ch_name, new_p_var, new_t_var, new_next_proc) => {
                let conv_next_proc = new_next_proc.substitution_on_trigger(p_var, in_process, t_var, new_tag);
                Process::Recv(_ch_name, new_p_var, new_t_var, Box::new(conv_next_proc))
            },
            Process::RollV(roll_var) => {
                if roll_var == t_var {
                    Process::RollK(new_tag.clone())
                } else {
                    Process::RollV(roll_var)
                }
            },
            Process::RollK(roll_key) => {
                Process::RollK(roll_key)
            },
        }
    }
}
