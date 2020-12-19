use crate::algorithms::{SimpleDp, SocialDistance, Solver};
use crate::api::parse::Params;
use crate::field;
use crate::simulator;

const CNT: usize = 5;

pub fn simulate(params: Params) {
    let mut diff_sum = 0;
    for _ in 0..CNT {
        let mut sim = simulator::Simulator::new(field::Field::new(None, None, None));
        while sim.get_field().now_turn() != sim.get_field().final_turn() {
            let mut solver_1 = SocialDistance::new(false, sim.get_field());
            let act_1 = solver_1.solve();
            for (i, x) in act_1.iter().enumerate() {
                sim.set_act(false, i, x.clone());
            }

            let mut solver_2 = SimpleDp::new(true, sim.get_field());
            let act_2 = solver_2.solve();
            for (i, x) in act_2.iter().enumerate() {
                sim.set_act(true, i, x.clone());
            }
            sim.change_turn();
        }
        println!(
            "{} - {}",
            sim.get_field().score(false).sum(),
            sim.get_field().score(true).sum()
        );
        diff_sum += sim.get_field().score(false).sum() - sim.get_field().score(true).sum();
    }
    println!("{}", diff_sum);
}
