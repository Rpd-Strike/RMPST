use pi_calculus::rollpi::{syntax::*, environment::{generator, runner::{self, RunningContext}}};

fn run_processes_as_parties(procs: Vec<(String, Process)>) 
{
    let mut generator = generator::Generator::default();
    
    procs.iter().for_each(|(id, p)| {
        let state = process_to_prime_state(p, TagKey(id.clone()));
        generator.take_participant_conf(state, Some(id.clone()), None, None);
    });

    let (parties, hist) = generator.generate_participants();

    let runner = runner::Runner::new(RunningContext {
        parties,
        hist,
    });

    runner.run();
}

fn main() {
    let ch_a = ChName("a".to_string());
    let cont_1 = Process::PVar(ProcVar("p".to_string()));
    let cont_2 = Process::Send(ChName("b".to_string()), Box::new(Process::End));

    let orig_proc_parties = vec![
        ("A".to_string(),
        Process::Send(
            ch_a.clone(),
            Box::new(cont_2),
        )),

        ("B".to_string(),
        Process::Recv(ch_a.clone(), ProcVar("pv".to_string()), TagVar("tv".to_string()), 
            Box::new(Process::PVar(ProcVar("pv".to_string()))),
        )),
    ];

    let procs = orig_proc_parties.iter()
        .map(|(id, p)| 
            p
        ).collect::<Vec<&Process>>();

    if check_all_list(procs) == false {
        println!("The processes do not respect the checks! (pvar, tvar uniques and closed)")
    } else {
        run_processes_as_parties(orig_proc_parties)
    }

    // This shouldn't pass the test
}
