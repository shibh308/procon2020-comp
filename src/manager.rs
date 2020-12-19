use crate::algorithms::{SimpleDp, SocialDistance, Solver};
use crate::api::parse::Params;
use crate::field;
use crate::simulator;

const CNT: usize = 6;

pub fn simulate(params: Params) {
    let mut diff_sum = 0;
    for i in 0..CNT {
        let mo = i % 2;
        let mut sim = simulator::Simulator::new(field::Field::new(None, None, None));

        while sim.get_field().now_turn() != sim.get_field().final_turn() {
            let mut solver_1 = SocialDistance::new(false, sim.get_field());
            let mut solver_2 = SocialDistance::new(true, sim.get_field());

            if mo == 1 {
                solver_2.set_params(params.clone());
            } else {
                solver_1.set_params(params.clone());
            }

            let act_1 = solver_1.solve();
            let act_2 = solver_2.solve();

            for (i, x) in act_1.iter().enumerate() {
                sim.set_act(false, i, x.clone());
            }

            for (i, x) in act_2.iter().enumerate() {
                sim.set_act(true, i, x.clone());
            }
            sim.change_turn();
        }
        println!(
            "{} - {}",
            sim.get_field().score(mo == 1).sum(),
            sim.get_field().score(mo == 0).sum()
        );
        diff_sum += sim.get_field().score(mo == 1).sum() - sim.get_field().score(mo == 0).sum();
    }
    println!("{}", diff_sum);
}
