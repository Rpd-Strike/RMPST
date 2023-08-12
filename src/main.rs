use pi_calculus::rollpi::{syntax::*, environment::{generator, runner::{self, RunningContext}}, local_types::{PartLocalType, LocalType}};
use pi_calculus::scenarios;

fn run_processes_as_parties(conf: Vec<(String, Process)>) 
{
    let parties = conf.iter().map(|(id, _p)| id.clone()).collect::<Vec<_>>();
    let procs = conf.into_iter().map(|(_id, p)| p).collect::<Vec<_>>();

    if check_initial_conf_list(&procs) == false {
        panic!("The processes do not respect the checks! (pvar, tvar uniques and closed and rolls bounded)")
    }

    let conf = parties.into_iter().zip(procs.into_iter()).collect::<Vec<_>>();

    // Generate the contexts for running the parties and their associated processes
    let mut generator = generator::Generator::default();
    
    conf.iter().for_each(|(id, p)| {
        let new_tag = ProcTag::PTKey(TagKey(id.clone()));
        let state = p.clone().to_tagged_process(new_tag).to_prime_state();
        generator.take_participant_conf(state, Some(id.clone()), None);
    });

    let (parties, hist) = generator.generate_participants();

    let runner = runner::Runner::new(RunningContext {
        parties,
        hist,
    });

    runner.run();
}

// TODO: Create a module to transform from local types to processes and names


fn main() 
{
    let chosen_scenario = scenarios::loc_types::simple_rec_lt;
    // let chosen_scenario = scenarios::roll_pi::basic_roll_pi_test;

    run_processes_as_parties(chosen_scenario());
}
