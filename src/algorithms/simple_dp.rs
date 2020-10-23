use super::base;
use crate::field;
use crate::simulator;

use field::{Field, Point, State};
use num_traits::pow;
use simulator::Act;
use std::collections::{HashMap, HashSet};

pub struct SimpleDp<'a> {
    field: &'a Field,
    side: bool,
    data: HashMap<Point, f64>,
}

const TURN: u8 = 7;
const PER: f64 = 0.3;

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

#[derive(Clone)]
struct DpState {
    score: f64,
    used: HashSet<Point>,
    prev_act: Option<Act>,
    prev_pos: Option<Point>,
    prev_turn: Option<usize>,
}

impl DpState {
    fn new(
        act: Act,
        score: f64,
        prev_pos: Point,
        used: &HashSet<Point>,
        prev_turn: usize,
    ) -> DpState {
        let mut nex_used = used.clone();
        let pos = match act {
            Act::StayAct => prev_pos,
            Act::MoveAct(p) | Act::RemoveAct(p) | Act::PutAct(p) => p,
        };
        nex_used.insert(pos);
        DpState {
            score,
            used: nex_used,
            prev_act: Some(act),
            prev_pos: Some(prev_pos),
            prev_turn: Some(prev_turn),
        }
    }
}

impl SimpleDp<'_> {
    fn calc_base(&self, now_state: &DpState, nex_pos: &Point, act: &Act) -> Option<f64> {
        match base::point(self.side, act.clone(), self.field) {
            Some(point) => Some(if now_state.used.contains(nex_pos) {
                0.0
            } else {
                point as f64
            }),
            None => None,
        }
    }

    fn calc_dp(&mut self) {
        let turn = TURN.min(self.field.final_turn() - self.field.now_turn()) as usize;
        let mut dp = vec![HashMap::new(); turn + 1];
        for i in 0..self.field.width() {
            for j in 0..self.field.height() {
                dp[0].insert(
                    Point::new(i as i8, j as i8),
                    DpState {
                        score: 0.0,
                        used: HashSet::new(),
                        prev_turn: None,
                        prev_act: None,
                        prev_pos: None,
                    },
                );
            }
        }
        for t in 0..turn {
            for (pos, now_state) in dp[t].clone() {
                let score = now_state.score;
                let neighbors = base::make_neighbors(pos, self.field);
                for nex in neighbors {
                    if now_state.used.contains(&nex) {}

                    let tile = self.field.tile(nex.usize());
                    if let Some((nex_state, nex_turn)) = match tile.state() {
                        State::Wall(side_) if self.side != side_ => {
                            let act = Act::RemoveAct(nex);
                            if let Some(point) = self.calc_base(&now_state, &nex, &act) {
                                if t == turn - 1 {
                                    let nex_score = score + point * pow(PER, t);
                                    Some((
                                        DpState::new(act, nex_score, pos, &now_state.used, t),
                                        t + 1,
                                    ))
                                } else {
                                    let nex_score = score + (point * (1.0 + PER)) * pow(PER, t);
                                    Some((
                                        DpState::new(act, nex_score, pos, &now_state.used, t),
                                        t + 2,
                                    ))
                                }
                            } else {
                                None
                            }
                        }
                        _ => {
                            let act = Act::MoveAct(nex);
                            if let Some(point) = self.calc_base(&now_state, &nex, &act) {
                                let nex_score = score + point * pow(PER, t);
                                Some((DpState::new(act, nex_score, pos, &now_state.used, t), t + 1))
                            } else {
                                None
                            }
                        }
                    } {
                        if !dp[nex_turn].contains_key(&nex) {
                            dp[nex_turn].insert(nex, nex_state);
                        } else {
                            let state_ = &dp[nex_turn][&nex];
                            if state_.score < nex_state.score {
                                dp[nex_turn].insert(nex, nex_state);
                            }
                        }
                    }
                }
            }
        }
        for t in (1..turn).rev() {
            for (pos, now_state) in dp[t].clone() {
                if now_state.prev_turn.unwrap() == 0 {
                    match self.data.get(&pos) {
                        None => {
                            self.data.insert(pos, now_state.score);
                        }
                        Some(_) => {
                            self.data.get_mut(&pos).map(|x| x.max(now_state.score));
                        }
                    };
                }
                let mut val = dp[now_state.prev_turn.unwrap()]
                    .get_mut(&now_state.prev_pos.unwrap())
                    .unwrap();
                val.score = val.score.max(now_state.score);
            }
        }
        for (k, v) in &dp[0] {
            println!("{}", v.score);
            self.data.insert(*k, v.score);
        }
    }
}

impl base::EachEvalSolver for SimpleDp<'_> {
    fn eval(&self, _id: usize, act: Act) -> Option<f64> {
        match act {
            Act::StayAct => Some(0.0),
            Act::PutAct(pos) | Act::MoveAct(pos) => {
                if self.field.tile(pos.usize()).state() == field::State::Wall(!self.side) {
                    None
                } else {
                    self.data.get(&pos).cloned()
                }
            }
            Act::RemoveAct(pos) => {
                if self.field.tile(pos.usize()).state() == field::State::Wall(!self.side) {
                    self.data.get(&pos).cloned()
                } else {
                    None
                }
            }
        }
    }
}
