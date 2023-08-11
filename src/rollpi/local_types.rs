use crate::rollpi::syntax::{ChName, ProcVar, TagVar};

use super::syntax::Process;

type VarName = String;
type Label = String;
type Party = String;

#[derive(Clone, Debug)]
pub enum LocalType
{
    End,
    Send(Party, Vec<(Label, LocalType)>),
    Recv(Party, Vec<(Label, LocalType)>),
    RAbs(VarName, Box<LocalType>),
    RVar(VarName),
}

#[derive(Clone, Debug)]
pub struct PartLocalType
{
    party: Party,
    local_type: LocalType,
}

impl PartLocalType
{
    pub fn new(party: Party, local_type: LocalType) -> Self
    {
        PartLocalType { party, local_type }
    }

    pub fn get_name(&self) -> Party
    {
        self.party.clone()
    }

    // TODO: Maybe make Tvar tags different from each other using a counter context 
    pub fn to_process(self: Self) -> Process
    {
        let PartLocalType { party, local_type } = self;
        match local_type
        {
            LocalType::End => Process::End,
            LocalType::Send(to_party, opts) => {
                let opt_ch = ChName(format!("comm_opt_{}_{}", party, to_party));
                let base_snd_ch = format!("comm_snd_{}_{}", party, to_party);
                // let ord_ch = ChName(format!("comm_ord_{}_{}", party, to_party));

                let choice_var = ProcVar("C".to_string());
                let receive_choice_branch = Process::Recv(opt_ch.clone(), choice_var.clone(), TagVar("_u_".to_string()), Box::new(Process::PVar(choice_var.clone())));

                let send_branch = opts.into_iter().map(|(label, lt)| {
                    let lt_enc = PartLocalType { party: party.clone(), local_type: lt };
                    let snd_ch = ChName(format!("{}_{}", base_snd_ch, label));
                    Process::Send(snd_ch.clone(), Box::new(lt_enc.to_process()))
                }).collect();

                let send_branch = Process::parallel_compose(send_branch);

                Process::Par(Box::new(send_branch), Box::new(receive_choice_branch))
            },
            LocalType::Recv(from_party, opts) => {
                let opt_ch = ChName(format!("comm_opt_{}_{}", from_party, party));
                let base_snd_ch = format!("comm_snd_{}_{}", from_party, party);
                let ord_ch = ChName(format!("comm_ord_{}_{}", from_party, party));

                let drain_var = ProcVar("D".to_string());
                let drain_branch = opts.iter().fold(Process::PVar(drain_var.clone()), |acc, _cnt| {
                    Process::Recv(ord_ch.clone(), drain_var.clone(), TagVar("_u_".to_string()), Box::new(acc))
                });

                let recv_branches = opts.into_iter().map(|(label, lt)| {
                    let lt_enc = PartLocalType { party: party.clone(), local_type: lt };
                    let snd_ch = ChName(format!("{}_{}", base_snd_ch, label));
                    let x_var = ProcVar(format!("X_{}", label));

                    let recv_cont = Process::Send(ord_ch.clone(), Box::new(Process::Par(
                        Box::new(lt_enc.to_process()),
                        Box::new(Process::Send(opt_ch.clone(), Box::new(Process::PVar(x_var.clone())))),

                        // Box::new(Process::PVar(x_var.clone())),
                        // Box::new(Process::Send(opt_ch.clone(), Box::new(lt_enc.to_process())))
                    )));
                    
                    Process::Recv(snd_ch.clone(), x_var.clone(), TagVar("_u_".to_string()), Box::new(recv_cont))
                }).collect();

                let recv_branches = Process::parallel_compose(recv_branches);

                Process::Par(
                    Box::new(recv_branches),
                    Box::new(drain_branch),
                )
            },
            LocalType::RAbs(r_label, t) => {
                let t_enc = PartLocalType { party: party.clone(), local_type: *t };
                let t_enc = t_enc.to_process();

                let norm_ch = ChName(format!("rec_norm_{}_{}", party, r_label));
                let comb_ch = ChName(format!("rec_comb_{}_{}", party, r_label));

                let xvar = ProcVar(format!("X_{}", r_label));

                let replica_proc = Process::parallel_compose(vec![
                    Process::PVar(xvar.clone()),
                    Process::Send(comb_ch.clone(), Box::new(Process::PVar(xvar.clone()))),
                    Process::Send(norm_ch.clone(), Box::new(t_enc.clone())),
                ]);

                let recv_proc = Process::Recv(comb_ch.clone(), xvar.clone(), TagVar("_u_".to_string()), Box::new(replica_proc.clone()));

                let send_proc = Process::Send(comb_ch.clone(), Box::new(recv_proc.clone()));

                Process::parallel_compose(vec![
                    recv_proc, 
                    send_proc,
                    t_enc.clone()
                ])
            },
            LocalType::RVar(r_label) => {
                let norm_ch = ChName(format!("rec_norm_{}_{}", party, r_label));
                let xvar = ProcVar(format!("X_r_{}", r_label));

                Process::Recv(norm_ch.clone(), xvar.clone(), TagVar("_u_".to_string()), Box::new(Process::PVar(xvar)))
            },
        }
    }
}