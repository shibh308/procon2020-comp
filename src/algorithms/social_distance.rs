use super::base;
use crate::field;
use crate::simulator;

use base::MinOrdFloat;
use field::{Field, Point, State};
use num_traits::pow;
use simulator::Act;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};

const DEPTH: usize = 5;
const WIDTH: usize = 10;
const PER: f64 = 0.7;
const DIST_WEIGHT: f64 = 10.0;

const LCP_PER: f64 = 2.0;
const LCP_POW: f64 = 2.0;
const SAME_TILE_PER: f64 = 1.0;
const SAME_TILE_POW: f64 = 2.0;

pub struct SocialDistance<'a> {
    field: &'a Field,
    side: bool,
}

impl<'a> base::Solver<'a> for SocialDistance<'a> {
    fn new(side: bool, field: &'a Field) -> SocialDistance<'a> {
        SocialDistance { field, side }
    }
    fn field(&self) -> &Field {
        self.field
    }
    fn side(&self) -> bool {
        self.side
    }
    fn solve(&mut self) -> Vec<Act> {
        let mut acts = vec![Act::StayAct; self.field.agent_count()];
        self.move_confirm(&mut acts);
        self.put_confirm(&mut acts);
        acts
    }
}

#[derive(Clone, PartialEq, Eq)]
struct DpState {
    score: MinOrdFloat,
    pos: Point,
    act: Act,
    used: HashSet<Point>,
    prev_turn: Option<usize>,
    prev_idx: Option<usize>,
}

impl Ord for DpState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}
impl PartialOrd for DpState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl DpState {
    fn from(
        &self,
        nex_pos: Point,
        nex_act: Act,
        add_score: f64,
        prev_turn: usize,
        prev_idx: usize,
    ) -> DpState {
        let mut nex_used = self.used.clone();
        nex_used.insert(nex_pos.clone());
        DpState {
            score: MinOrdFloat::new(self.score.raw() + add_score),
            pos: nex_pos,
            act: if self.act == Act::StayAct {
                nex_act
            } else {
                self.act.clone()
            },
            used: nex_used,
            prev_turn: Some(prev_turn),
            prev_idx: Some(prev_idx),
        }
    }
}

impl SocialDistance<'_> {
    fn distance_eval(&self, poses: &Vec<Point>) -> f64 {
        let f = |p1: &Point, p2: &Point| -> f64 {
            let xd = (p1.x - p2.x).abs() as f64;
            let yd = (p1.y - p2.y).abs() as f64;
            xd * xd + yd * yd
        };
        let mut score = 0.0;
        for (i, p1) in poses.iter().enumerate() {
            for p2 in &poses[i + 1..] {
                score += f(p1, p2);
            }
        }
        score
    }
    fn move_confirm(&self, acts: &mut Vec<Act>) {
        let idxes = (0..self.field.agent_count())
            .filter(|id| {
                if let Some(_) = self.field.agent(self.side, *id) {
                    true
                } else {
                    false
                }
            })
            .collect::<Vec<_>>();
        let poses = idxes
            .iter()
            .map(|id| self.field.agent(self.side, *id).unwrap())
            .collect::<Vec<_>>();

        for (i, pos) in poses.iter().enumerate() {
            let res = self.beam_search(pos.clone(), DEPTH);
            acts[idxes[i]] = res[0].1.clone();
        }
    }
    fn put_confirm(&self, acts: &mut Vec<Act>) {
        let cnt = acts
            .iter()
            .filter(|act| **act == Act::StayAct)
            .collect::<Vec<_>>()
            .len();
    }
    fn get_poses(&self, state: &DpState, turn: usize, table: &Vec<Vec<DpState>>) -> Vec<Point> {
        let mut v = vec![None; turn + 1];
        let mut top = state;
        v[turn] = Some(top.pos);
        while top.prev_idx.is_some() {
            let prev_turn = top.prev_turn.unwrap();
            top = &table[prev_turn][top.prev_idx.unwrap()];
            v[prev_turn] = Some(top.pos.clone());
        }
        let mut now = None;
        let mut res = Vec::new();
        for elm in v {
            if elm.is_some() {
                now = elm.clone();
            }
            res.push(now.unwrap());
        }
        res
    }
    fn reduce_cand(
        &self,
        dp_table: &Vec<Vec<DpState>>,
        t: usize,
        lcp_per: f64,
        same_tile_per: f64,
    ) -> (Vec<DpState>, f64) {
        let cand = &dp_table[t];
        let mut res = Vec::new();

        let mut h_map = BTreeSet::new();
        let mut pos_v: Vec<BTreeSet<Point>> = Vec::new();

        for state in cand {
            let mut score = state.score.raw();

            let mut top = state;
            let mut poses = vec![top.pos.clone()];
            poses = self.get_poses(top, t, dp_table);
            let mut poses_pref = Vec::new();
            let mut lcp = poses.len();
            for i in 0..poses.len() {
                poses_pref.push(poses[i]);
                if !h_map.contains(&poses_pref) {
                    lcp = i;
                    break;
                }
            }
            let btree_used = state.used.iter().cloned().collect::<BTreeSet<_>>();

            let mut hs_vec = pos_v
                .iter()
                .map(|hs| hs.intersection(&btree_used).collect::<Vec<_>>().len())
                .collect::<Vec<_>>();

            // lcpがuniqueだとボーナスが入る
            let lcp_bonus = lcp_per * ((DEPTH + 1 - lcp.max(1)) as f64).powf(LCP_POW);
            let average_tile_conf = if hs_vec.is_empty() {
                0.0
            } else {
                hs_vec
                    .iter()
                    .fold(0.0, |b, x| b + (*x as f64).powf(SAME_TILE_POW))
                    / hs_vec.len() as f64
            };
            let tile_pena = same_tile_per * average_tile_conf;
            score += lcp_bonus;
            score -= tile_pena;

            if poses.len() == lcp {
                continue;
            }
            poses_pref = Vec::new();
            for i in 0..poses.len() {
                poses_pref.push(poses[i].clone());
                h_map.insert(poses_pref.clone());
            }
            pos_v.push(btree_used);
            res.push((MinOrdFloat::new(score), state.clone(), lcp));

            /*
            if t == DEPTH {
                println!(
                    "{} => {}  [{}, -{}]",
                    state.score.raw(),
                    score,
                    lcp,
                    average_tile_conf,
                );
            }
             */
        }
        res.sort();
        let siz = res.len().min(WIDTH);
        let val = res.iter().take(siz).fold(0.0, |b, x| b + x.2 as f64) / (siz as f64);
        /*
        if t == DEPTH {
            for c in res.iter().take(siz) {
                println!("x.2: {}", c.2);
            }
        }
         */
        let mut selected = res
            .iter()
            .take(siz)
            .map(|x| x.1.clone())
            .collect::<Vec<_>>();
        selected.sort();
        if t == DEPTH {
            for c in &selected {
                println!("{}: {:?}", c.score.raw(), self.get_poses(c, t, dp_table));
            }
        }
        (selected, val)
    }

    fn beam_search(&self, start_pos: Point, max_depth: usize) -> Vec<(f64, Act, Vec<Point>)> {
        let mut cand = vec![Vec::new(); max_depth + 1];
        cand[0].push(DpState {
            score: MinOrdFloat::new(0.0),
            pos: start_pos,
            act: Act::StayAct,
            used: HashSet::new(),
            prev_turn: None,
            prev_idx: None,
        });
        cand[0][0].used.insert(start_pos);
        for t in 0..max_depth {
            cand[t].sort();
            let (bef, _) = self.reduce_cand(&cand, t, LCP_PER, SAME_TILE_PER);
            for (idx, now_state) in bef.iter().enumerate() {
                let neighbors = base::make_neighbors(now_state.pos, self.field);
                for nex in neighbors {
                    let tile = self.field.tile(nex.usize());
                    if let Some((nex_state, nex_turn)) = match tile.state() {
                        State::Wall(side_) if self.side != side_ => {
                            let act = Act::RemoveAct(nex);
                            if let Some(point) = self.calc_base(&now_state, &nex, &act) {
                                if t == max_depth - 1 {
                                    Some((
                                        now_state.from(
                                            now_state.pos,
                                            act,
                                            point * pow(PER, t),
                                            t,
                                            idx,
                                        ),
                                        t + 1,
                                    ))
                                } else {
                                    Some((
                                        now_state.from(
                                            now_state.pos,
                                            act,
                                            point * (1.0 + PER) * pow(PER, t),
                                            t,
                                            idx,
                                        ),
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
                                Some((now_state.from(nex, act, point * pow(PER, t), t, idx), t + 1))
                            } else {
                                None
                            }
                        }
                    } {
                        cand[nex_turn].push(nex_state);
                    }
                }
            }
        }
        let (final_res, average) = self.reduce_cand(&cand, max_depth, LCP_PER, SAME_TILE_PER);
        // println!("avl: {}", average);
        let mut res = Vec::new();
        for top_res in final_res {
            let top = top_res.clone();
            let score = top.score.raw();
            let act = top.act.clone();
            let v = self.get_poses(&top, max_depth, &cand);
            res.push((score, act, v));
        }
        res
    }
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
}
