use super::base;
use crate::field;
use crate::simulator;

use crate::field::PointUsize;
use base::MinOrdFloat;
use field::{Field, Point, State};
use num_traits::pow;
use rand::Rng;
use simulator::Act;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::time::Instant;

const DEPTH: usize = 5;
const WIDTH: usize = 30;
const PUT_WIDTH: usize = 60;
const PER: f64 = 0.6;

const FIRST_MOVE_PER: f64 = 1.0;

const PUT_BORDER: f64 = 0.3;

const PUT_START_TEMP: f64 = 3.0;
const PUT_END_TEMP: f64 = 0.3;
const PUT_SA_SEC: f64 = 0.3;

const START_TEMP: f64 = 3.0;
const END_TEMP: f64 = 0.3;
const SA_SEC: f64 = 0.3;

const AG_CONF_PER: f64 = 0.3;

const REGION_PER: f64 = 1.0;
const REGION_POW: f64 = 0.9;

const PUT_CONF_POW: f64 = 0.7;

const SA_LAST_PENA: f64 = 0.3;
const SA_LAST_POW: f64 = 3.5;
const SA_LAST_SUPER_PENA: f64 = 2.5;
const SA_LAST_SUPER_BORDER: f64 = 0.25;
const SA_CONF_PER: f64 = 0.6;
const SA_CONF_PENA: f64 = 3.5;
const SA_DIST_PENA: f64 = 30.0;
const SA_DIST_POW: f64 = 0.4;

const LCP_PER: f64 = 2.0;
const LCP_POW: f64 = 2.0;
const SAME_TILE_PER: f64 = 1.0;
const SAME_TILE_POW: f64 = 2.0;

pub struct SocialDistance<'a> {
    field: &'a Field,
    agent_set: HashSet<Point>,
    side: bool,
}

impl<'a> base::Solver<'a> for SocialDistance<'a> {
    fn new(side: bool, field: &'a Field) -> SocialDistance<'a> {
        SocialDistance {
            field,
            side,
            agent_set: HashSet::new(),
        }
    }
    fn field(&self) -> &Field {
        self.field
    }
    fn side(&self) -> bool {
        self.side
    }
    fn solve(&mut self) -> Vec<Act> {
        let mut acts = vec![Act::StayAct; self.field.agent_count()];
        for id in 0..self.field.agent_count() {
            if let Some(pos) = self.field.agent(!self.side, id) {
                self.agent_set.insert(pos);
            }
        }
        self.move_confirm(&mut acts);
        acts
    }
}

#[derive(Clone, PartialEq, Eq)]
struct DpState {
    score: MinOrdFloat,
    pos: Point,
    act: Act,
    used: HashSet<Point>,
    poses: Vec<Point>,
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
    fn from(&self, nex_pos: Point, nex_act: Act, add_score: f64) -> DpState {
        assert_ne!(self.pos, nex_pos);
        let mut nex_used = self.used.clone();
        nex_used.insert(nex_pos.clone());
        let mut poses = self.poses.clone();
        poses.push(nex_pos.clone());
        if let Act::RemoveAct(_) = nex_act {
            poses.push(nex_pos.clone());
        }
        DpState {
            score: MinOrdFloat::new(self.score.raw() + add_score),
            pos: nex_pos,
            act: if self.act == Act::StayAct {
                nex_act
            } else {
                self.act.clone()
            },
            used: nex_used,
            poses,
        }
    }
}

impl SocialDistance<'_> {
    fn calc_score(
        &self,
        bs_data: &Vec<Vec<(f64, Act, Vec<Point>)>>,
        sel: &Vec<usize>,
        output: bool,
    ) -> f64 {
        let acts = sel
            .iter()
            .zip(bs_data)
            .map(|(idx, dat)| &dat[*idx])
            .collect::<Vec<_>>();
        let mut score = acts.iter().fold(0.0, |b, x| {
            b + x.0
                + self
                    .calc_base(&HashSet::new(), &x.2[1], &x.1)
                    .unwrap_or(-10000.0)
                    * FIRST_MOVE_PER
        });

        let pos_data = sel
            .iter()
            .enumerate()
            .map(|(i, x)| &bs_data[i][*x].2)
            .collect::<Vec<_>>();

        let mut field = self.field.clone();
        let init_region = field.score(self.side).region();

        let mut per_map = HashMap::new();
        let mut prev_per = vec![1.0; pos_data.len()];
        for j in 1..=DEPTH {
            let per_pow = SA_CONF_PER.powf((j - 1) as f64);
            let per_pow_dist = SA_DIST_POW.powf(j as f64);

            let poses = pos_data.iter().map(|x| x[j]).collect::<Vec<_>>();
            if j == 1 && poses.iter().collect::<HashSet<_>>().len() != poses.len() {
                score = -1e5;
                break;
            }
            let f = |p1: &Point, p2: &Point| -> f64 {
                let xd = (p1.x - p2.x).abs() as f64;
                let yd = (p1.y - p2.y).abs() as f64;
                xd * xd + yd * yd
            };
            for (idx1, p) in poses.iter().enumerate() {
                for (idx2, q) in poses.iter().take(idx1).enumerate() {
                    score -= SA_DIST_PENA * per_pow_dist * prev_per[idx1] * prev_per[idx2]
                        / f(p, q).max(0.5);
                }
            }

            for pos in poses {
                field.set_state(pos.usize(), State::Wall(self.side))
            }
            field.update_score();
            field.update_region();
            let region_diff = field.score(self.side).region() - init_region;
            let region_score = REGION_PER * REGION_POW.powf((j - 1) as f64) * region_diff as f64;
            score += region_score;
            if output {
                println!("region: {} {}", region_diff, region_score);
                println!("{}", score);
            }

            for (i, pd) in pos_data.iter().enumerate() {
                let pos = pd[j].clone();
                // 到達確率
                let per = if j != 0 && pd[j - 1] == pd[j] {
                    1.0
                } else {
                    // マスが踏まれている確率を更新していく
                    match per_map.get_mut(&pos) {
                        Some(val) => {
                            let prev_val = *val;
                            *val = prev_val + (1.0 - prev_val) * prev_per[i] * per_pow;
                            1.0 - prev_val
                        }
                        None => {
                            per_map.insert(pos, prev_per[i] * per_pow);
                            1.0
                        }
                    }
                } * prev_per[i];
                prev_per[i] = per;
                let tile = self.field.tile(pos.usize());
                let raw_score = match tile.state() {
                    State::Wall(s) if s == self.side => 0,
                    _ => tile.point(),
                };
                // 到達できない確率だけ減らしていく
                score -= SA_CONF_PENA * raw_score as f64 * per_pow * (1.0 - per);
            }
        }
        let last_pena = acts.iter().enumerate().fold(0.0, |b, (idx, x)| {
            b + if prev_per[idx] < SA_LAST_SUPER_BORDER {
                x.0 * (1.0 - prev_per[idx]) * SA_LAST_SUPER_PENA
            } else {
                x.0 * (1.0 - prev_per[idx]).powf(SA_LAST_POW)
            }
        }) * SA_LAST_PENA;
        score -= last_pena;

        if output {
            println!("poses:");
            for (i, p) in pos_data.iter().enumerate() {
                println!("{:?} => {:?}", acts[i].0, p);
            }
            println!("per: {:?}", prev_per);
        }
        score
    }

    fn simulated_annealing(&self, bs_res: &Vec<Vec<(f64, Act, Vec<Point>)>>) -> Vec<usize> {
        let mut sel = vec![0; bs_res.len()];
        let mut now_score = self.calc_score(&bs_res, &sel, false);
        let mut answer = (now_score, sel.clone());
        let mut stack = Vec::new();
        let start_time = Instant::now();
        let mut rng = rand::thread_rng();
        loop {
            let elapsed = start_time.elapsed().as_secs_f64();
            if elapsed >= SA_SEC {
                break;
            }

            if bs_res.len() <= 1 || rng.gen::<f32>() <= 0.8 {
                let idx = rng.gen_range(0, bs_res.len());
                let to = if bs_res[idx].len() == 1 {
                    0
                } else {
                    let p = rng.gen_range(0, bs_res[idx].len() - 1);
                    if p >= sel[idx] {
                        p + 1
                    } else {
                        p
                    }
                };
                stack.push((idx, sel[idx]));
                sel[idx] = to;
            } else {
                let idx1 = rng.gen_range(0, bs_res.len());
                let mut idx2 = idx1;
                while idx1 != idx2 {
                    idx2 = rng.gen_range(0, bs_res.len());
                }
                let to1 = if bs_res[idx1].len() == 1 {
                    0
                } else {
                    let p = rng.gen_range(0, bs_res[idx1].len() - 1);
                    if p >= sel[idx1] {
                        p + 1
                    } else {
                        p
                    }
                };
                let to2 = if bs_res[idx2].len() == 1 {
                    0
                } else {
                    let p = rng.gen_range(0, bs_res[idx2].len() - 1);
                    if p >= sel[idx2] {
                        p + 1
                    } else {
                        p
                    }
                };
                stack.push((idx1, sel[idx1]));
                stack.push((idx2, sel[idx2]));
                sel[idx1] = to1;
                sel[idx2] = to2;
            }

            let nex_score = self.calc_score(&bs_res, &sel, false);
            let temp = (END_TEMP - START_TEMP) * (start_time.elapsed().as_secs_f64() / SA_SEC)
                + START_TEMP;
            let prob = ((nex_score - now_score) / temp).exp();
            // println!("{} => {}  ({})", now_score, nex_score, prob);

            let updated = if prob >= rng.gen::<f64>() {
                now_score = nex_score;
                if now_score > answer.0 {
                    println!("updated: {} {} {} => {:?}", temp, elapsed, now_score, sel);
                    answer = (now_score, sel.clone());
                }
                true
            } else {
                false
            };
            while !stack.is_empty() {
                let (idx, sc) = stack.pop().unwrap();
                if !updated {
                    sel[idx] = sc;
                }
            }
        }
        self.calc_score(&bs_res, &answer.1, true);
        answer.1
    }
    fn move_confirm(&self, acts: &mut Vec<Act>) {
        let check_fn = |id: usize| {
            if let Some(_) = self.field.agent(self.side, id) {
                true
            } else {
                false
            }
        };
        let idxes = (0..self.field.agent_count())
            .filter(|id| check_fn(*id))
            .collect::<Vec<_>>();
        let put_idxes = (0..self.field.agent_count())
            .filter(|id| !check_fn(*id))
            .collect::<Vec<_>>();
        let mut move_pos_list = Vec::new();
        if !idxes.is_empty() {
            let poses = idxes
                .iter()
                .map(|id| self.field.agent(self.side, *id).unwrap())
                .collect::<Vec<_>>();

            let bs_res = poses
                .iter()
                .map(|x| self.beam_search(vec![x.clone()], DEPTH, WIDTH))
                .collect::<Vec<_>>();
            /*
            println!(
                "bs_res: {:?}",
                bs_res
                    .iter()
                    .map(|x| {
                        x.iter()
                            .fold(HashSet::new(), |mut b, x| {
                                b.insert(x.2[1].clone());
                                b
                            })
                            .len()
                    })
                    .collect::<Vec<_>>()
            );
             */
            let res = self.simulated_annealing(&bs_res);
            for (i, bs_v) in bs_res.iter().enumerate() {
                move_pos_list.push(bs_v[res[i]].2.iter().map(|x| x.usize()).collect::<Vec<_>>());
                acts[idxes[i]] = bs_v[res[i]].1.clone();
            }
        }
        // 多始点BFSでうまく管理する
        let st = Instant::now();

        let mut per = (0..self.field.width())
            .map(|_| vec![1.0; self.field.height()])
            .collect::<Vec<_>>();

        for j in 1..=DEPTH {
            let per_pow = PUT_CONF_POW.powf((j - 1) as f64);
            for moves in &move_pos_list {
                per[moves[j].x][moves[j].y] *= 1.0 - per_pow;
            }
        }

        let score_func = |x, y| {
            let tile = self.field.tile(PointUsize::new(x, y));
            match tile.state() {
                State::Wall(side) if side != self.side => -20.0,
                _ => tile.point() as f64 * per[x][y],
            }
        };

        let (cand_list, border) = {
            let mut res = (0..self.field.width()).fold(Vec::new(), |mut v, i| {
                let mut w = (0..self.field.height()).fold(Vec::new(), |mut u, j| {
                    let score = score_func(i, j);
                    u.push(score);
                    u
                });
                v.append(&mut w);
                v
            });
            let res = {
                let mut res_f = res.iter().map(|x| MinOrdFloat::new(*x)).collect::<Vec<_>>();
                res_f.sort();
                res_f.iter().map(|x| x.raw()).collect::<Vec<_>>()
            };
            println!("{:?}", res);
            let border = res[((res.len() as f64 * PUT_BORDER) as usize).min(res.len() - 1)];
            (
                (0..self.field.width()).fold(HashSet::new(), |v, i| {
                    let mut w = (0..self.field.height()).fold(HashSet::new(), |mut u, j| {
                        if score_func(i, j) >= border {
                            u.insert(PointUsize::new(i, j).normal());
                        }
                        u
                    });
                    v.union(&w).cloned().collect::<HashSet<_>>()
                }),
                border,
            )
        };
        let put_bs_res = self.beam_search(
            cand_list.iter().cloned().collect::<Vec<_>>(),
            DEPTH - 1,
            PUT_WIDTH,
        );
        let put_res = self.put_simulated_annealing(put_idxes.len(), &move_pos_list, &put_bs_res);
        for (i, idx) in put_res.iter().enumerate() {
            acts[put_idxes[i]] = put_bs_res[*idx].1.clone();
        }
        println!(
            "cand:{}, {} * {}",
            cand_list.len(),
            self.field.width(),
            self.field.height()
        );
        println!("border: {}", border);
        println!("time: {} [msec]", st.elapsed().as_millis());
    }
    fn put_simulated_annealing(
        &self,
        n: usize,
        move_pos_list: &Vec<Vec<PointUsize>>,
        bs_res: &Vec<(f64, Act, Vec<Point>)>,
    ) -> Vec<usize> {
        (0..n).collect::<Vec<_>>()
    }
    fn reduce_cand(
        &self,
        cand: &Vec<DpState>,
        lcp_per: f64,
        same_tile_per: f64,
        depth: usize,
        width: usize,
    ) -> Vec<DpState> {
        let mut res = Vec::new();

        let mut h_map = BTreeSet::new();
        let mut pos_v: Vec<BTreeSet<Point>> = Vec::new();

        for state in cand {
            let mut score = state.score.raw();

            let poses = &state.poses;
            let mut poses_pref = Vec::new();
            let mut lcp = state.poses.len();
            for i in 0..state.poses.len() {
                poses_pref.push(state.poses[i]);
                if !h_map.contains(&poses_pref) {
                    lcp = i;
                    break;
                }
            }
            let btree_used = state.used.iter().cloned().collect::<BTreeSet<_>>();

            let hs_vec = pos_v
                .iter()
                .map(|hs| hs.intersection(&btree_used).collect::<Vec<_>>().len())
                .collect::<Vec<_>>();

            // lcpがuniqueだとボーナスが入る
            let lcp_bonus = lcp_per * ((depth + 1 - lcp.max(1)) as f64).powf(LCP_POW);
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
            if t == depth {
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
        let siz = res.len().min(width);
        /*
        if t == depth {
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
        selected
        /*
        if t == depth {
            for c in &selected {
                println!("{}: {:?}", c.score.raw(), self.get_poses(c, t, dp_table));
            }
        }
         */
    }

    fn beam_search(
        &self,
        start_poses: Vec<Point>,
        max_depth: usize,
        width: usize,
    ) -> Vec<(f64, Act, Vec<Point>)> {
        let mut cand = vec![Vec::new(); max_depth + 1];
        for (i, start_pos) in start_poses.iter().enumerate() {
            cand[0].push(DpState {
                score: MinOrdFloat::new(0.0),
                pos: start_pos.clone(),
                act: Act::StayAct,
                used: HashSet::new(),
                poses: vec![start_pos.clone()],
            });
            cand[0][i].used.insert(start_pos.clone());
        }
        for t in 0..max_depth {
            cand[t].sort();
            let bef = self.reduce_cand(&cand[t], LCP_PER, SAME_TILE_PER, max_depth, width);
            cand[t] = bef.clone();
            for now_state in bef {
                let neighbors = base::make_neighbors(now_state.pos, self.field);
                for nex in neighbors {
                    let tile = self.field.tile(nex.usize());
                    if let Some((nex_state, nex_turn)) = match tile.state() {
                        State::Wall(side_) if self.side != side_ => {
                            let act = Act::RemoveAct(nex);
                            if let Some(point) = self.calc_base(&now_state.used, &nex, &act) {
                                if t == max_depth - 1 {
                                    Some((now_state.from(nex, act, point * pow(PER, t)), t + 1))
                                } else {
                                    Some((
                                        now_state.from(nex, act, point * (1.0 + PER) * pow(PER, t)),
                                        t + 2,
                                    ))
                                }
                            } else {
                                None
                            }
                        }
                        _ => {
                            let act = Act::MoveAct(nex);
                            if let Some(point) = self.calc_base(&now_state.used, &nex, &act) {
                                Some((now_state.from(nex, act, point * pow(PER, t)), t + 1))
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
        let final_res =
            self.reduce_cand(&cand[max_depth], LCP_PER, SAME_TILE_PER, max_depth, width);
        cand[max_depth] = final_res.clone();
        // println!("avl: {}", average);
        let mut res = Vec::new();
        for top_res in final_res {
            let top = top_res.clone();
            let score = top.score.raw();
            let act = top.act.clone();
            println!("poses: {:?}", top.poses);
            res.push((score, act, top.poses));
        }
        res
    }
    fn calc_base(&self, used: &HashSet<Point>, nex_pos: &Point, act: &Act) -> Option<f64> {
        match base::point(self.side, act.clone(), self.field) {
            Some(point) => Some(if used.contains(nex_pos) {
                0.0
            } else {
                point as f64
                    * if self.agent_set.contains(nex_pos) {
                        AG_CONF_PER
                    } else {
                        1.0
                    }
            }),
            None => None,
        }
    }
}
