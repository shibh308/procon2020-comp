use crate::field;
use crate::simulator;

pub trait Solver {
    fn solve(field: &field::Field) -> Vec<simulator::Act>;
}
