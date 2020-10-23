use super::base;
use crate::field;
use crate::simulator;

use field::{Field, Point, State};
use simulator::Act;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct SimpleDp<'a> {
    field: &'a Field,
    side: bool,
    data: HashMap<Point, f64>,
}

const TURN: u8 = 7;
const PER: f64 = 0.8;

impl<'a> base::Solver<'a> for SimpleDp<'a> {
    fn new(side: bool, field: &'a Field) -> SimpleDp<'a> {
        SimpleDp {
            field,
            side,
            data: HashMap::new(),
        }
    }
    fn field(&self) -> &Field {
        self.field
    }
    fn side(&self) -> bool {
        self.side
    }
    fn solve(&mut self) -> Vec<Act> {
        self.calc_dp();
        base::solve(self, 0.0)
    }
}

impl SimpleDp<'_> {
    fn calc_dp(&mut self) {
        /*
        let turn = TURN.min(self.field.final_turn() - self.field.now_turn());
        let mut dp = VecDeque::new();
        dp.push_back(HashSet::new());
        dp.push_back(HashSet::new());
        dp[0].insert()
        for _ in 0..turn {
            dp.push_back(HashSet::new());
            dp.pop_front();
        }
        base::point(side, act, field).map(|x| x as f64)
         */
    }
}

struct DpState {
    score: f64,
    used: HashSet<Point>,
}

impl base::EachEvalSolver for SimpleDp<'_> {
    fn eval(&self, _id: usize, act: Act) -> Option<f64> {
        /*
        match act {
            Act::StayAct => Some(0.0),
            Act::RemoveAct(pos) => {
                if self.field.tile(pos.usize()).state() == field::State::Wall(!self.side) {
                    self.data.get(&pos).cloned()
                } else {
                    None
                }
            }
        }
         */
        None
    }
}
