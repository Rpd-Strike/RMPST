// trait 

use super::syntax::{Process, ProcVar, TagVar, TagKey};

impl Process
{
    pub fn substitution_on_trigger(self: Self, p_var: ProcVar, in_process: &Process, t_var: TagVar, new_tag: &TagKey) -> Process
    {
        self.__rec_subst(p_var, in_process, true, t_var, new_tag, true)
    }

    fn __rec_subst(self: Self, p_var: ProcVar, in_process: &Process, mut ok_proc: bool, t_var: TagVar, new_tag: &TagKey, mut ok_tag: bool) -> Process
    {
        if !ok_proc && !ok_tag {
            return self;
        }

        match self {
            Process::End => Process::End,

            Process::Par(a, b) => {
                let conv_a = a.__rec_subst(p_var.clone(), in_process, ok_proc, t_var.clone(), new_tag, ok_tag);
                let conv_b = b.__rec_subst(p_var, in_process, ok_proc, t_var, new_tag, ok_tag);
                Process::Par(Box::new(conv_a), Box::new(conv_b))
            },
            
            Process::Send(ch_name, p) => {
                let conv_p = p.__rec_subst(p_var, in_process, ok_proc, t_var, new_tag, ok_tag);
                Process::Send(ch_name, Box::new(conv_p))
            },
            // !! This works because Tvars and Pvars are unique
            Process::Recv(_ch_name, new_p_var, new_t_var, new_next_proc) => {
                if new_p_var == p_var {
                    ok_proc = false;
                }
                if new_t_var == t_var {
                    ok_tag = false;
                }
                let conv_next_proc = new_next_proc.__rec_subst(p_var, in_process, ok_proc, t_var, new_tag, ok_tag);
                Process::Recv(_ch_name, new_p_var, new_t_var, Box::new(conv_next_proc))
            },
            
            Process::PVar(var) => {
                if ok_proc && var == p_var {
                    in_process.clone()
                } else {
                    Process::PVar(var)
                }
            },
            
            Process::RollK(roll_key) => {
                Process::RollK(roll_key)
            },
            Process::RollV(roll_var) => {
                if ok_tag && roll_var == t_var {
                    Process::RollK(new_tag.clone())
                } else {
                    Process::RollV(roll_var)
                }
            },
        }
    }
}
