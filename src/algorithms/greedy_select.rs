use super::base;
use crate::field;
use crate::simulator;

use field::Field;
use simulator::Act;

pub struct GreedySelect {}

impl base::Solver for GreedySelect {
    fn solve(side: bool, field: &Field) -> Vec<Act> {
        base::solve::<GreedySelect>(side, field, -17.0)
    }
}

impl base::EachEvalSolver for GreedySelect {
    fn eval(side: bool, _id: usize, act: Act, field: &Field) -> Option<f64> {
        base::point(side, act, field).map(|x| x as f64)
    }
}
