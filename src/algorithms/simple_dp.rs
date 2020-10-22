use super::base;
use crate::field;
use crate::simulator;

pub struct SimpleDP {}

impl base::Solver for SimpleDP {
    fn solve(field: &field::Field) -> Vec<simulator::Act> {
        vec![simulator::Act::PutAct(field::Point::new(0, 0)); field.agent_count()]
    }
}
