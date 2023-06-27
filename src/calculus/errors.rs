use std::fmt::Display;

pub enum ParseError
{
    // The error message
    Msg(String),
}

// implement display trait
impl Display for ParseError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            ParseError::Msg(msg) => write!(f, "Parse Error: {}", msg),
        }
    }
}