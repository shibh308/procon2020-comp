use crate::field;
use crate::simulator;
use field::{Field, Point, PointUsize};
use ordered_float::OrderedFloat;
use rand::Rng;
use simulator::Act;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

const PUT_BORDER: i8 = 0;

pub trait Solver<'a> {
    fn new(side: bool, field: &'a Field) -> Self;
    fn field(&self) -> &Field;
    fn side(&self) -> bool;
    fn solve(&mut self) -> Vec<Act>;
}

pub trait EachEvalSolver {
    fn eval(&self, id: usize, act: Act) -> Option<f64>;
}

pub fn solve<'a, T: Solver<'a> + EachEvalSolver>(solver: &T) -> Vec<Act> {
    let field = solver.field();
    let mut eval_scores = Vec::new();
    for id in 0..field.agent_count() {
        let mut ev = HashMap::new();
        let acts = make_acts(solver.side(), id, field);
        for act in acts {
            if let Some(score) = solver.eval(id, act.clone()) {
                ev.insert(act.clone(), score);
            }
        }
        eval_scores.push(ev);
    }
    primal_dual(solver.side(), eval_scores, &field)
}

pub fn solve_regret_matching<'a, T: Solver<'a> + EachEvalSolver>(
    side_: bool,
    field: &'a Field,
    num_iter: usize,
) -> Vec<Act> {
    let solver = [false, true]
        .iter()
        .map(|side| {
            let mut sol = T::new(false, field);
            sol.solve();
            sol
        })
        .collect::<Vec<_>>();

    let mut eval_scores = vec![Vec::new(); 2];
    for side in vec![false, true] {
        for id in 0..field.agent_count() {
            let mut ev = HashMap::new();
            let acts = make_acts(side, id, field);
            for act in acts {
                if let Some(score) = solver[side as usize].eval(id, act.clone()) {
                    ev.insert(act.clone(), score);
                }
            }
            eval_scores[side as usize].push(ev);
        }
    }
    let prob = regret_matching(side_, eval_scores, &field, num_iter);
    primal_dual(side_, prob, &field)
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct MinOrdFloat(Reverse<OrderedFloat<f64>>);

impl MinOrdFloat {
    fn new(x: f64) -> MinOrdFloat {
        MinOrdFloat(Reverse(OrderedFloat::<f64>::from(x)))
    }
    fn raw(&self) -> f64 {
        (self.0).0.into()
    }
}

#[derive(Clone)]
struct Edge {
    to: usize,
    cap: usize,
    cost: f64,
    rev: usize,
    act: Option<Act>,
}

impl Edge {
    fn new(to: usize, cap: usize, cost: f64, rev: usize, act: Option<Act>) -> Edge {
        Edge {
            to,
            cap,
            cost,
            rev,
            act,
        }
    }
}

struct FlowGraph {
    n: usize,
    edges: Vec<Vec<Edge>>,
    edge_list: Vec<(usize, usize)>,
}

impl FlowGraph {
    fn new(n: usize) -> FlowGraph {
        FlowGraph {
            n,
            edges: vec![Vec::new(); n],
            edge_list: Vec::new(),
        }
    }
    fn add(&mut self, from: usize, to: usize, cap: usize, cost: f64, act: Option<Act>) {
        let from_idx = self.edges[from].len();
        let to_idx = self.edges[to].len();
        self.edges[from].push(Edge::new(to, cap, cost, to_idx, act));
        self.edges[to].push(Edge::new(from, 0, -cost, from_idx, None));
        self.edge_list.push((from, from_idx));
    }
    fn solve(&mut self, s: usize, t: usize, flow_: usize) {
        let mut flow = flow_;
        let mut heap: BinaryHeap<(MinOrdFloat, usize)> = BinaryHeap::new();
        let mut prev_v = vec![0; self.n];
        let mut prev_e = vec![0; self.n];
        while flow > 0 {
            let mut min_cost = vec![1e18; self.n];
            min_cost[s] = 0.0;
            heap.push((MinOrdFloat::new(0.0), s));
            while !heap.is_empty() {
                let (dist, pos) = heap.pop().unwrap();
                if min_cost[pos] != dist.raw() {
                    continue;
                }
                for (i, ed) in self.edges[pos].iter().enumerate() {
                    let nex = dist.raw() + ed.cost;
                    if ed.cap > 0 && min_cost[ed.to] > nex {
                        min_cost[ed.to] = nex;
                        prev_v[ed.to] = pos;
                        prev_e[ed.to] = i;
                        heap.push((MinOrdFloat::new(min_cost[ed.to]), ed.to));
                    }
                }
            }
            assert_ne!(min_cost[t], 1e18);
            let mut add_flow = usize::max_value();
            let mut x = t;
            while x != s {
                add_flow = add_flow.min(self.edges[prev_v[x]][prev_e[x]].cap);
                x = prev_v[x];
            }
            flow -= add_flow;
            x = t;
            while x != s {
                let ed_rev = {
                    let mut ed = &mut self.edges[prev_v[x]][prev_e[x]];
                    ed.cap -= add_flow;
                    ed.rev
                };
                self.edges[x][ed_rev].cap += add_flow;
                x = prev_v[x];
            }
        }
    }
}

fn tile_pos(side: bool, id: usize, act: &Act, field: &Field) -> Point {
    match act {
        Act::PutAct(pos) | Act::MoveAct(pos) | Act::RemoveAct(pos) => pos.clone(),
        Act::StayAct => field.agent(side, id).unwrap(),
    }
}

fn primal_dual(side: bool, acts: Vec<HashMap<Act, f64>>, field: &Field) -> Vec<Act> {
    let max_val = acts.iter().fold(0.0, |ma, item| {
        *vec![
            ma,
            *item
                .values()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .expect("acts is empty"),
        ]
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .expect("max_by error")
    });

    let poses: Vec<Point> = acts
        .iter()
        .enumerate()
        .fold(HashSet::new(), |hs, (id, hm)| {
            let keys: HashSet<Point> = hm
                .keys()
                .map(|act| tile_pos(side, id, act, field))
                .collect();
            hs.union(&keys).cloned().collect()
        })
        .into_iter()
        .collect();
    let num_nodes = poses.len() + acts.len() + 2;
    let agent_count = acts.len();
    let source_idx = poses.len() + acts.len();
    let sink_idx = poses.len() + acts.len() + 1;

    let mut tile_idx_map = HashMap::new();
    for (idx, pos) in poses.iter().enumerate() {
        tile_idx_map.insert(pos, idx + agent_count);
    }

    let mut graph = FlowGraph::new(num_nodes);
    for agent_idx in 0..agent_count {
        for (act, value) in &acts[agent_idx] {
            let pos = tile_pos(side, agent_idx, act, field);
            let tile_idx = *tile_idx_map.get(&pos).expect("tile_idx not found error");
            graph.add(agent_idx, tile_idx, 1, max_val - value, Some(act.clone()));
        }
        graph.add(source_idx, agent_idx, 1, 0.0, None);
    }
    for tile_idx_ in 0..poses.len() {
        let tile_idx = agent_count + tile_idx_;
        graph.add(tile_idx, sink_idx, 1, 0.0, None);
    }
    graph.solve(source_idx, sink_idx, agent_count);

    let mut acts = vec![Act::StayAct; agent_count];

    for (from, idx) in graph.edge_list {
        let ed = &graph.edges[from][idx];
        let to = ed.to;
        if from < agent_count && agent_count <= to && to < source_idx {
            if ed.cap != 1 {
                acts[from] = ed.act.clone().expect("act is None");
            }
        }
    }
    acts
}

fn regret_matching(
    side_: bool,
    act_scores: Vec<Vec<HashMap<Act, f64>>>,
    field: &Field,
    num_iter: usize,
) -> Vec<HashMap<Act, f64>> {
    let mut rng = rand::thread_rng();
    let agent_count = field.agent_count();

    let mut calc_prob = |regret: &Vec<Vec<HashMap<Act, f64>>>| -> Vec<Vec<HashMap<Act, f64>>> {
        let regret_sum: Vec<Vec<f64>> = regret
            .iter()
            .map(|v| {
                v.iter()
                    .map(|hm| hm.iter().fold(0.0, |sum, (_, val)| sum + val))
                    .collect()
            })
            .collect();
        regret
            .iter()
            .enumerate()
            .map(|(side, v)| {
                v.iter()
                    .enumerate()
                    .map(|(id, hm)| {
                        hm.iter().fold(HashMap::new(), |mut new_hm, (act, val)| {
                            new_hm.insert(
                                act.clone(),
                                if regret_sum[side][id] == 0.0 {
                                    1.0 / hm.len() as f64
                                } else {
                                    val / regret_sum[side][id]
                                },
                            );
                            new_hm
                        })
                    })
                    .collect()
            })
            .collect()
    };
    let mut calc_acts = |regret: &Vec<Vec<HashMap<Act, f64>>>| -> Vec<Vec<Act>> {
        let prob = calc_prob(regret);
        let mut acts = vec![vec![Act::StayAct; agent_count]; 2];
        for side in 0..2 {
            for (id, hm) in prob[side].iter().enumerate() {
                let per = rng.gen::<f64>();
                let mut prob_sum = 0.0;
                for (k, v) in hm {
                    prob_sum += v;
                    if per < prob_sum {
                        acts[side][id] = k.clone();
                        break;
                    }
                }
            }
        }
        acts
    };

    let mut regret = act_scores
        .iter()
        .map(|v| {
            v.iter()
                .map(|hm| {
                    hm.iter().fold(HashMap::new(), |mut new_hm, (act, _)| {
                        new_hm.insert(act.clone(), 0.0);
                        new_hm
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    for t in 0..num_iter {
        let acts = calc_acts(&regret).clone();
        let mut act_values = vec![vec![0.0; agent_count]; 2];
        let mut pos_agents: HashMap<Point, Vec<(usize, usize)>> = HashMap::new();
        for side in 0..2 {
            for (id, act) in acts[side].iter().enumerate() {
                let pos = tile_pos(side != 0, id, act, field);
                let score = act_scores[side][id].get(act).cloned().unwrap();
                act_values[side][id] = score;
                match pos_agents.get(&pos) {
                    Some(_) => {
                        pos_agents.get_mut(&pos).unwrap().push((side, id));
                    }
                    None => {
                        pos_agents.insert(pos, vec![(side, id)]);
                    }
                }
            }
        }
        let mut regret_sum = 0.0;
        for side in 0..2 {
            for (id, act) in acts[side].iter().enumerate() {
                let now_val = act_values[side][id].clone();
                let now_pos = tile_pos(side != 0, id, act, field);
                let hm = &act_scores[side][id];
                for (nex_act, nex_val) in hm {
                    let nex_pos = tile_pos(side != 0, id, nex_act, field);
                    let bef_cnt = pos_agents.get(&now_pos).map(|v| v.len()).unwrap_or(0);
                    let aft_cnt = pos_agents.get(&nex_pos).map(|v| v.len()).unwrap_or(0);

                    let reg = (if now_pos == nex_pos {
                        if bef_cnt == 1 {
                            nex_val - now_val
                        } else {
                            0.0
                        }
                    } else {
                        let mut now_diff = 0.0;
                        match bef_cnt {
                            1 => {
                                now_diff -= now_val;
                            }
                            2 => {
                                for (side_, id_) in pos_agents.get(&now_pos).unwrap() {
                                    if (side, id) != (*side_, *id_) {
                                        now_diff += (if side == *side_ { 1.0 } else { -1.0 })
                                            * act_values[*side_][*id_];
                                    }
                                }
                            }
                            _ => {}
                        }
                        match aft_cnt {
                            0 => {
                                now_diff += nex_val;
                            }
                            1 => {
                                let (side_, id_) = pos_agents.get(&now_pos).unwrap()[0];
                                now_diff -= (if side == side_ { 1.0 } else { -1.0 })
                                    * act_values[side_][id_];
                            }
                            _ => {}
                        }
                        now_diff
                    })
                    .max(0.0);
                    *regret[side][id].get_mut(nex_act).unwrap() += reg;
                    regret_sum += reg;
                }
            }
        }
    }
    calc_prob(&regret)[side_ as usize].clone()
}

pub fn make_neighbors(pos: Point, field: &Field) -> Vec<Point> {
    (-1..2)
        .fold(Vec::new(), |v, x| {
            v.into_iter()
                .chain((-1..2).map(|y| Point::new(x as i8, y as i8)))
                .collect()
        })
        .iter()
        .map(|p| (pos + *p))
        .filter(|p| field.inside(*p))
        .collect()
}

pub fn make_acts(side: bool, id: usize, field: &Field) -> Vec<Act> {
    match field.agent(side, id) {
        Some(pos) => {
            let mut cand = vec![Act::StayAct];
            let moves: Vec<Point> = make_neighbors(pos, field);
            for mov in moves {
                match field.tile(mov.usize()).state() {
                    field::State::Neutral | field::State::Position(_) => {
                        cand.push(Act::MoveAct(mov))
                    }
                    field::State::Wall(side_) => {
                        cand.push(Act::RemoveAct(mov));
                        if side == side_ {
                            cand.push(Act::MoveAct(mov));
                        }
                    }
                }
            }
            cand
        }
        None => {
            let mut hm = HashSet::new();
            for id in 0..field.agent_count() {
                if let Some(pos) = field.agent(side, id) {
                    hm.insert(pos);
                }
            }
            (0..field.width()).fold(Vec::new(), |v, x| {
                v.into_iter()
                    .chain((0..field.height()).filter_map(|y| {
                        let tile = field.tile(PointUsize::new(x, y));
                        match tile.state() {
                            field::State::Wall(side_) if side == side_ => None,
                            _ if hm.contains(&Point::new(x as i8, y as i8)) => None,
                            _ if tile.point() < PUT_BORDER => None,
                            _ => Some(Act::PutAct(Point::new(x as i8, y as i8))),
                        }
                    }))
                    .collect()
            })
        }
    }
}

pub fn point(side: bool, act: Act, field: &Field) -> Option<i8> {
    match act {
        Act::StayAct => None,
        Act::PutAct(pos) | Act::MoveAct(pos) => {
            let tile = field.tile(pos.usize());
            let state = tile.state();
            match state {
                field::State::Wall(side_) if side != side_ => None,
                field::State::Neutral => Some(tile.point()),
                field::State::Position(side_) if side != side_ => {
                    Some(tile.point() - tile.point().abs())
                }
                _ => Some(0),
            }
        }
        Act::RemoveAct(pos) => {
            let tile = field.tile(pos.usize());
            let state = tile.state();
            match state {
                field::State::Wall(tile_side) => {
                    Some((if side == tile_side { -1 } else { 1 }) * tile.point())
                }
                _ => None,
            }
        }
    }
}
