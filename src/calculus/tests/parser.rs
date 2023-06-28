use crate::calculus::syntax::AtomicAction;

use super::super::syntax::Process;
use super::super::parser::parse;

// TODO: Add tests for "+"

fn run_tests(tests: Vec<(&str, &Process)>) 
{
    for (test, expected) in tests {
        let result = parse(test);
        match result {
            Ok(process) => assert_eq!(process, *expected),
            Err(err) => assert!(false, "Expected a process, but got {}", err),
        }
    }
}

#[test]
fn test_primary() 
{
    let expected = Process::End;

    // rewrite above code to use the auxiliary function
    let tests = vec![
        ("0", &expected),
        ("(0)", &expected),
        ("((0))", &expected),
    ];

    run_tests(tests)
}

#[test]
fn test_actions()
{
    let AsendB = AtomicAction::Send("a".to_string(), "b".to_string());
    let BrecvC = AtomicAction::Receive("b".to_string(), "c".to_string());

    let test1 = (
        "a<b>.0",
        &Process::Prefix(AsendB.clone(), Box::new(Process::End))
    );

    let test2 = (
        "a<b>.b[c].0",
        &Process::Prefix(AsendB, Box::new(Process::Prefix(BrecvC, Box::new(Process::End))))
    );

    run_tests(vec![test1, test2])
}

#[test]
fn test_restriction()
{
    let XsendB = AtomicAction::Send("x".to_string(), "b".to_string());
    let XrecvC = AtomicAction::Receive("x".to_string(), "c".to_string());
    let par_proc = Process::Par(
        Box::new(Process::Prefix(XsendB, Box::new(Process::End))),
        Box::new(Process::Prefix(XrecvC, Box::new(Process::End))),
    );
    let resA = Process::New("x".to_string(), Box::new(par_proc));


    run_tests(vec![
        ("\\x.(x<b>.0 | x[c].0)", &resA),
    ])
}

#[test]
fn test_restriction_2()
{
    let XsendB = AtomicAction::Send("x".to_string(), "b".to_string());
    let XrecvC = AtomicAction::Receive("x".to_string(), "c".to_string());
    let par_proc = Process::Par(
        Box::new(Process::Prefix(XsendB.clone(), Box::new(Process::End))),
        Box::new(Process::Prefix(XrecvC.clone(), Box::new(Process::End))),
    );
    let resA = Process::New("x".to_string(), Box::new(par_proc));

    let newY = Process::New("y".to_string(), Box::new(resA));

    run_tests(vec![
        ("\\y.\\x.(x<b>.0 | x[c].0)", &newY),
    ])
}

#[test]
fn test_restriction_par_prio()
{
    let XsendB = AtomicAction::Send("x".to_string(), "b".to_string());
    let XsendB_p = Process::Prefix(XsendB, Box::new(Process::End));

    let XrecvC = AtomicAction::Receive("x".to_string(), "c".to_string());
    let XrecvC_p = Process::Prefix(XrecvC, Box::new(Process::End));
    
    
    let abs = Process::New("y".to_string(), 
        Box::new(Process::New("x".to_string(), 
            Box::new(XsendB_p)
        ))
    );
    
    let res = Process::Par(
        Box::new(abs),
        Box::new(XrecvC_p),
    );

    run_tests(vec![
        ("\\y.\\x.x<b>.0 | x[c].0", &res),
    ])
}

