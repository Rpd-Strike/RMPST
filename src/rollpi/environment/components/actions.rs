use crate::rollpi::{syntax::*, environment::{entities::participant::PartyContext, types::PartyComm}};

pub enum PartyAction<'a>
{
    RunPrimary(&'a TaggedPrimProc),
    Crash,
    End,
}

pub trait ActionInterpreter : Send
{
    fn interpret_action(&self, action: &PartyAction, context: &PartyContext)
        -> Option<Process>;
}

impl Default for Box<dyn ActionInterpreter>
{
    fn default() -> Self
    {
        Box::new(SimpleInterpreter)
    }
}

struct SimpleInterpreter;

impl ActionInterpreter for SimpleInterpreter
{
    fn interpret_action(&self, action: &PartyAction, context: &PartyContext)
        -> Option<Process>
    {
        match action {
            PartyAction::RunPrimary(TaggedPrimProc{tag, proc}) => {
                println!("Running primary process: {:?}", proc);
                match proc {
                    PrimProcess::End => {
                        println!("Ending...");

                        return None
                    },
                    PrimProcess::RollK(_) => {
                        // TODO: Follow next steps
                        // 1. Send rollback notification to history

                        todo!()
                    },
                    PrimProcess::Send(ChName(ch_id), proc) => {
                        println!("Sending process: {:?}", proc);
                        let channel = context.chan_msg_ctx(ch_id).send_channel;
                        
                        // TODO: Send id somwhow
                        channel.send(PartyComm {
                            sender_id: "".to_owned(),
                            process: proc.clone(),
                            tag: tag.clone(),
                        }).unwrap();

                        return None
                    },
                    PrimProcess::Recv(
                        ChName(ch_id), 
                        ProcVar(p_var), 
                        TagVar(t_var), 
                        proc) => 
                    {
                        // TODO: follow next steps
                        // 1. receive message from channel
                        // 2. send new moery piece to history
                        // 3. Wait for confirmation from history
                        // 4. Evaluate new process variable received

                        // Should return Option<Continuation_proc>
                        todo!()
                    },
                }
            },
            PartyAction::Crash => {
                println!("Crashing...");
                todo!()
            },
            PartyAction::End => {
                println!("Ending...");
                todo!()
            },
        }
    }
}
