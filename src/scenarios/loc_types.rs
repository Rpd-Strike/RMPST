use crate::rollpi::{local_types::{PartLocalType, LocalType}, syntax::{check_initial_conf_list, Process}};

pub fn simple_rec_lt() -> Vec<(String, Process)>
{
    let party_a = PartLocalType::new("A".to_string(), 
        LocalType::RAbs("t".to_string(), Box::new(
            LocalType::Send("B".to_string(), vec![
                ("lb_2".to_string(), LocalType::End),
                ("lb_1".to_string(), LocalType::Recv("B".to_string(), 
                                        vec![("lb_3".to_string(), LocalType::RVar("t".to_string()))])),
            ])
        ))
    );

    let party_b = PartLocalType::new("B".to_string(), 
        LocalType::RAbs("t".to_string(), Box::new(
            LocalType::Recv("A".to_string(), vec![
                ("lb_2".to_string(), LocalType::End),
                ("lb_1".to_string(), LocalType::Send("A".to_string(), 
                                        vec![("lb_3".to_string(), LocalType::RVar("t".to_string()))])),
            ])
        ))
    );

    let party_localtypes = vec![
        party_a,
        party_b,
    ];

    println!("Party A: {:?}", party_localtypes.get(0));

    let party_names: Vec<_> = party_localtypes.iter().map(|p| p.get_name()).collect();
    let party_procs: Vec<_> = party_localtypes.into_iter().map(|p| p.to_process()).collect();

    println!("Party A: {:?}", party_procs.get(0));

    if check_initial_conf_list(&party_procs) == false {
        panic!("The processes do not respect the checks! (pvar, tvar uniques and closed and rolls bounded)")
    } else {
        party_names.into_iter().zip(party_procs.into_iter()).collect()
    }
}