use crate::rollpi::{syntax::{PrimeState, ChName, ProcTag}, environment::{entities::participant::PartyContext, types::{PartyComm, MemoryPiece}}};

use super::strategies::SimpleDeterministic::{TaggedActionContext, ActionContext};

pub trait ActionInterpreter : Send
{
    fn interpret_action(&self, context: &mut PartyContext, ctx: TaggedActionContext)
        -> PrimeState;

    fn probe_recv_channel(&self, context: &PartyContext, ChName(id): &ChName)
        -> Option<PartyComm>
    {
        let recv_channel = context.get_comm_ctx().chan_msg_ctx(&id).recv_channel;
        let data = recv_channel.try_recv().ok();

        data
    }
}

impl Default for Box<dyn ActionInterpreter>
{
    fn default() -> Self
    {
        Box::new(SimpleActionInterpreter::default())
    }
}

#[derive(Default)]
pub struct SimpleActionInterpreter {}

impl ActionInterpreter for SimpleActionInterpreter
{
    fn interpret_action(&self, ctx: &mut PartyContext, act_ctx: TaggedActionContext)
        -> PrimeState
    {
        ctx.get_logger().log(format!("\n"));

        match act_ctx {
            (proc_tag, ActionContext::RollK(target_roll_key)) => {
                ctx.get_logger().log(format!("INT: ROLLK {:?} with target {:?} \n", proc_tag, target_roll_key));

                let send_roll_ch = &ctx.get_comm_ctx().rollback_ctx.roll_tag_channel;
                // TODO: ? See for crash handling
                let _ = send_roll_ch.send(ProcTag::PTKey(target_roll_key.clone()));

                vec![]
            },
            (send_tag, ActionContext::Send(ChName(ch_name), proc)) => {
                ctx.get_logger().log(format!("INT: SEND {:?} to channel {:?} \n", send_tag, ch_name));

                let send_channel = ctx.get_comm_ctx().chan_msg_ctx(&ch_name).send_channel;
                send_channel.send(PartyComm { 
                    sender_id: ctx.get_id().clone(), process: proc.clone(), tag: send_tag.clone() 
                }).unwrap();
                
                vec![]
            },
            (recv_tag, ActionContext::RecvCont(in_data, ch_name, p_var, t_var, next_proc)) => {
                ctx.get_logger().log(format!("INT: RECV {:?} from channel {:?} \n", in_data.tag, ch_name));
                
                let new_tag = ctx.get_tag_ctx().create_new_tag();

                let send_ch = &ctx.get_comm_ctx().history_ctx.hist_tag_channel;
                let rez = send_ch.send(MemoryPiece::new(
                    (in_data.sender_id, ctx.get_id().clone()),
                    (   in_data.tag,
                        (ch_name.clone(), in_data.process.clone())
                    ),
                    (   recv_tag.clone(),
                        (ch_name.clone(), p_var.clone(), t_var.clone(), next_proc.clone())
                    ),
                    new_tag.clone(),
                ));

                if let Err(e) = &rez {
                    ctx.get_logger().log(format!("Error sending history tag: {:?} \n", e.to_string()));
                    panic!("...");
                }

                let recv_ch = &ctx.get_comm_ctx().history_ctx.hist_conf_channel;
                match recv_ch.recv() {
                    Err(_err) => todo!(),
                    Ok(rec_tag_key) => {
                        assert_eq!(rec_tag_key, new_tag);
                        
                        next_proc.clone()
                            .substitution_on_trigger(p_var.clone(), &in_data.process, t_var.clone(), &new_tag)
                            .to_tagged_process(ProcTag::PTKey(new_tag))
                            .to_prime_state()
                    }
                }
            },
            (proc_tag, ActionContext::End) => {
                ctx.get_logger().log(format!("INT: END {:?} \n", proc_tag));

                vec![]
            }
        }
    }
}
