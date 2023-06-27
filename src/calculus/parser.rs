use super::errors::ParseError;
use super::syntax::{Process, AtomicAction};

// operator precedence (highest priority first)
// Paranthesis
// Atomic Action
// New
// Sum
// Parralel


type ParseState<'a> = (Process, &'a str);

// Parse a string into the corresponding pi calculus term
pub fn parse(input: &str) -> Result<Process, ParseError>
{
    // eliminate spaces
    let input = input.replace(" ", "");

    let (term, rest) = parse_par(&input)?;

    if rest.len() > 0 {
        return Err(ParseError::Msg(format!("Unexpected token(s) - expected empty end of input: {}", rest)))
    }

    return Ok(term)
}

fn parse_par(input: &str) -> Result<ParseState, ParseError>
{
    let (mut term, mut rest) = parse_sum(input)?;

    while rest.starts_with("|") {
        let (term2, rest2) = parse_sum(&rest[1..])?;
        term = Process::Par(Box::new(term), Box::new(term2));
        rest = rest2;
    }

    return Ok((term, rest))
}

fn parse_sum(input: &str) -> Result<ParseState, ParseError>
{
    let (mut term, mut rest) = parse_new(input)?;

    while rest.starts_with("+") {
        let (term2, rest2) = parse_new(&rest[1..])?;
        term = Process::Sum(Box::new(term), Box::new(term2));
        rest = rest2;
    }

    return Ok((term, rest))
}

fn parse_new(input: &str) -> Result<ParseState, ParseError>
{
    if input.starts_with("\\") {
        if let Some((name, rest)) = &input[1..].split_once('.') {
            if name.len() > 0 {
                let (term, rest) = parse_new(&rest)?;
                return Ok((Process::New(name.to_string(), Box::new(term)), rest))
            }
        }
        return Err(ParseError::Msg(format!("Unexpected token for new expression - expected \\x.P: {}", input)))
    }
    else {
        return parse_atomic_action(input)
    }
}

fn parse_action(input: &str) -> Result<(AtomicAction, &str), ParseError>
{
    let action_parser = |input: &str, start_ch: char, end_ch: char| -> Option<AtomicAction> {
        if let Some((name_ch, rest)) = input.split_once(start_ch) {
            if let Some((name_val, _)) = rest.split_once(end_ch) {
                if name_ch.len() > 0 && name_val.len() > 0 && name_val != name_ch {
                    if start_ch == '<' {
                        return Some(AtomicAction::Send(name_ch.to_string(), name_val.to_string()))
                    }
                    else if start_ch == '[' {
                        return Some(AtomicAction::Receive(name_ch.to_string(), name_val.to_string()))
                    }
                }
            }
        }

        return None
    };

    if let Some(action) = action_parser(input, '<', '>') {
        let suffix_pos = input.find('>').unwrap() + 1;
        return Ok((action, &input[suffix_pos..]))
    }
    else if let Some(action) = action_parser(input, '[', ']') {
        let suffix_pos = input.find(']').unwrap() + 1;
        return Ok((action, &input[suffix_pos..]))
    }

    return Err(ParseError::Msg(format!("Unexpected token(s) for action - expected a<b> or a[b]: {}", input)))
}

fn parse_atomic_action(input: &str) -> Result<ParseState, ParseError>
{
    // TODO: Make this into a while, make clear distinction between E3 and E4

    if input.starts_with(|c: char| c.is_alphabetic()) {
        let (action, rest) = parse_action(&input)?;
        if rest.len() == 0 {
            return Ok((Process::Prefix(action, Box::new(Process::End)), rest))
        } 
        else if rest.starts_with(".") {
            let (term, rest) = parse_atomic_action(&rest[1..])?;
            return Ok((Process::Prefix(action, Box::new(term)), rest))
        }
        else {
            return Err(ParseError::Msg(format!("Unexpected token for atomic action - expected '.' or end of input: {}", input)))
        }
    } else {
        return parse_primary(input)
    }
}

fn parse_primary(input: &str) -> Result<ParseState, ParseError>
{
    if input.len() == 0
    {
        return Err(ParseError::Msg("Empty input".to_string()))
    }

    if input.starts_with("(") {
        let (term, rest) = parse_par(&input[1..])?;
        if rest.starts_with(")") {
            return Ok((term, &rest[1..]))
        }
        return Err(ParseError::Msg(format!("Unexpected token for paranthesis expression - expected ')': {}", input)))
    }

    if input.starts_with("0") {
        return Ok((Process::End, &input[1..]))
    }

    Err(ParseError::Msg(format!("Unexpected token for primary expression - expected 0 or '(': {}", input)))
}

