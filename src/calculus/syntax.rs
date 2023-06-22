// express the pi calculus terms
enum Process
{
    // Empty process
    Zero,
    // Parallel composition
    Par(Box<Process>, Box<Process>),
    // Restriction
    New(String, Box<Process>),
    // Choice
    Sum(Box<Process>, Box<Process>),
    // Atomic action
    Prefix(AtomicAction, Box<Process>),
}

enum AtomicAction
{
    Send(String, String),
    Receive(String, String),
    Tau,
}