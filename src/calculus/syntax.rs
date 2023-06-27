// express the pi calculus terms
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Process
{
    // End process
    End,
    // Parallel composition
    Par(Box<Process>, Box<Process>),
    // Restriction
    New(String, Box<Process>),
    // Choice
    Sum(Box<Process>, Box<Process>),
    // Atomic action
    Prefix(AtomicAction, Box<Process>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtomicAction
{
    Send(String, String),
    Receive(String, String),
    // Tau,
}
