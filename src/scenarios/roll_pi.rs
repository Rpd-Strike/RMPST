use crate::rollpi::syntax::{ChName, Process, ProcVar, TagVar};

pub fn basic_roll_pi_test() -> Vec<(String, Process)>
{
    let ch_a = ChName("a".to_string());
    // using this should give an error
    let _cont_1 = Process::PVar(ProcVar("p".to_string()));
    // using this option should work ok
    let cont_2 = Process::Send(ChName("b".to_string()), Box::new(Process::End));

    let parties = vec![
        "A".to_string(),
        "B".to_string(),
        // "C".to_string(),
    ];

    let processes = vec![
        Process::Send(
            ch_a.clone(),
            Box::new(cont_2),
        ),

        Process::Recv(ch_a.clone(), ProcVar("pv".to_string()), TagVar("tv".to_string()), 
            Box::new(Process::PVar(ProcVar("pv".to_string()))),
        ),

        // Process::RollK(TagKey("roll_kk".to_string())),
    ];

    assert_eq!(parties.len(), processes.len());

    parties.into_iter().zip(processes.into_iter()).collect()
}

pub fn basic_roll_pi_rollback() -> Vec<(String, Process)>
{
    let ch_a = ChName("a".to_string());
    let ch_b = ChName("b".to_string());

    let x_var = ProcVar("x".to_string());
    let g_tag = TagVar("g".to_string());
    // let g_key = TagKey("g".to_string());

    let s_tag = TagVar("s".to_string());

    let parties = vec![
        "A".to_string(),
        "B".to_string(),
        // "C".to_string(),
    ];

    let processes = vec![
        Process::Send(
            ch_a.clone(),
            Box::new(Process::End),
        ),

        Process::Recv(ch_a.clone(), x_var.clone(), g_tag.clone(), Box::new(
            Process::Par(
                Box::new(Process::Send(ch_b.clone(), Box::new(
                    Process::End
                ))),
                Box::new(Process::Recv(ch_b.clone(), x_var.clone(), s_tag.clone(), Box::new(
                    Process::RollV(g_tag.clone()
                ))))
            )
        ))
    ];

    assert_eq!(parties.len(), processes.len());

    parties.into_iter().zip(processes.into_iter()).collect()
}