use super::base;
use crate::field;
use crate::simulator;

use field::Field;
use simulator::Act;

pub struct GreedySelect<'a> {
    field: &'a Field,
    side: bool,
}

impl<'a> base::Solver<'a> for GreedySelect<'a> {
    fn new(side: bool, field: &'a Field) -> GreedySelect<'a> {
        GreedySelect { field, side }
    }
    fn field(&self) -> &Field {
        self.field
    }
    fn side(&self) -> bool {
        self.side
    }
    fn solve(&mut self) -> Vec<Act> {
        base::solve(self)
    }
}

impl<'a> base::EachEvalSolver for GreedySelect<'a> {
    fn eval(&self, _id: usize, act: Act) -> Option<f64> {
        base::point(self.side, act, self.field).map(|x| x as f64)
    }
}
