use super::base;
use crate::algorithms;
use crate::field;
use crate::simulator;

use field::Field;
use simulator::Act;

const NUM_ITER: usize = 100;

pub struct SimpleRegret<'a> {
    field: &'a Field,
    side: bool,
}

impl<'a> base::Solver<'a> for SimpleRegret<'a> {
    fn new(side: bool, field: &'a Field) -> SimpleRegret<'a> {
        SimpleRegret { field, side }
    }
    fn field(&self) -> &Field {
        self.field
    }
    fn side(&self) -> bool {
        self.side
    }
    fn solve(&mut self) -> Vec<Act> {
        // base::solve_regret_matching::<algorithms::GreedySelect>(self.side(), self.field, NUM_ITER)
        base::solve_regret_matching::<algorithms::SimpleDp>(self.side(), self.field, NUM_ITER)
    }
}
