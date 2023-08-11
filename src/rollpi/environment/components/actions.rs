use crate::rollpi::{syntax::{PrimeState, ChName, ProcTag}, environment::{entities::participant::PartyContext, types::{PartyComm, MemoryPiece}}};

use super::{strategies::SimpleDeterministic::{SimpleDetermStrat, ActionContext}, picker::Strategy};

pub trait ActionInterpreter : Send
{
    fn interpret_action(&self, context: &mut PartyContext, ctx: ActionContext)
        -> PrimeState;

    fn probe_recv_channel(&self, context: &PartyContext, ChName(id): &ChName)
        -> Option<PartyComm>
    {
        let recv_channel = context.get_comm_ctx().chan_msg_ctx(&id).recv_channel;
        let data = recv_channel.try_recv().ok();

        // if data.is_some() {
        //     println!("Probe ok on channel {:?}", id);
        // }

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
    fn interpret_action(&self, ctx: &mut PartyContext, act_ctx: ActionContext)
        -> PrimeState
    {
        ctx.get_logger().log(format!("\n"));

        match act_ctx {
            ActionContext::RollK(tag_key) => {
                ctx.get_logger().log(format!("INT: ROLLK process - with tag {:?} \n", tag_key));

                let send_roll_ch = &ctx.get_comm_ctx().rollback_ctx.roll_tag_channel;
                // TODO: ? See for crash handling
                let _ = send_roll_ch.send(ProcTag::PTKey(tag_key.clone()));

                vec![]
            },
            ActionContext::Send(ChName(ch_name), proc, ptag) => {
                ctx.get_logger().log(format!("INT: SEND process - with tag {:?} to channel {:?} \n", ptag, ch_name));

                let send_channel = ctx.get_comm_ctx().chan_msg_ctx(&ch_name).send_channel;
                send_channel.send(PartyComm { 
                    sender_id: ctx.get_id().clone(), process: proc.clone(), tag: ptag.clone() 
                }).unwrap();
                
                vec![]
            },
            ActionContext::RecvCont(in_data, ch_name, p_var, t_var, next_proc, recv_tag) => {
                ctx.get_logger().log(format!("INT: RECV process - with tag {:?} from channel {:?} \n", in_data.tag, ch_name));
                
                let new_tag = ctx.get_tag_ctx().create_new_tag();

                let send_ch = &ctx.get_comm_ctx().history_ctx.hist_tag_channel;
                let rez = send_ch.send(MemoryPiece::new(
                    (in_data.sender_id, ctx.get_id().clone()),
                    (   in_data.tag,
                        (ch_name.clone(), in_data.process.clone())
                    ),
                    (   recv_tag,
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
            ActionContext::End => {
                ctx.get_logger().log(format!("INT: END process \n"));

                vec![]
            }
        }
    }
}
