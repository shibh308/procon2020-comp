use crate::field;
use crate::simulator;
use field::{Field, Point, PointUsize};
use ordered_float::OrderedFloat;
use simulator::Act;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

pub trait Solver {
    fn solve(side: bool, field: &Field) -> Vec<Act>;
}

pub trait EachEvalSolver {
    fn eval(side: bool, id: usize, act: Act, field: &Field) -> Option<f64>;
}

pub fn solve<T: EachEvalSolver>(side: bool, field: &Field) -> Vec<Act> {
    let mut eval_scores = Vec::new();
    for id in 0..field.agent_count() {
        let mut ev = HashMap::new();
        let acts = make_acts(side, id, field);
        for act in acts {
            if let Some(score) = T::eval(side, id, act.clone(), field) {
                ev.insert(act.clone(), score);
            }
        }
        ev.insert(Act::StayAct, 0.0);
        eval_scores.push(ev);
    }
    primal_dual(eval_scores)
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

fn primal_dual(acts: Vec<HashMap<Act, f64>>) -> Vec<Act> {
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
    let poses: Vec<Act> = acts
        .iter()
        .fold(HashSet::new(), |hs, hm| {
            let keys: HashSet<Act> = hm.keys().cloned().collect();
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
            let tile_idx = *tile_idx_map.get(&act).expect("tile_idx not found error");
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

pub fn make_acts(side: bool, id: usize, field: &Field) -> Vec<Act> {
    match field.agent(side, id) {
        Some(pos) => {
            let mut cand = Vec::new();
            let moves: Vec<Point> = (-1..2)
                .fold(Vec::new(), |v, x| {
                    v.into_iter()
                        .chain((-1..2).map(|y| Point::new(x as i8, y as i8)))
                        .collect()
                })
                .iter()
                .map(|p| (pos + *p))
                .filter(|p| field.inside(*p))
                .collect();
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
                        match field.tile(PointUsize::new(x, y)).state() {
                            field::State::Wall(side_) if side == side_ => None,
                            _ if hm.contains(&Point::new(x as i8, y as i8)) => None,
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
