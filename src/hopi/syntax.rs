// express the pi calculus terms
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Process
{
    // End process
    End,
    // Process Variable
    PVar(String),
    // Restriction
    New(String, Box<Process>),
    // Parallel composition
    Par(Box<Process>, Box<Process>),
    // Send
    Send(String, Box<Process>),
    // Receive / Trigger  (channel name, proc_var, Process)
    Receive(String, String, Box<Process>),
}
