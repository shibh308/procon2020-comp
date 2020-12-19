use crate::field;
use druid::Data;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Act {
    StayAct,
    PutAct(field::Point),
    MoveAct(field::Point),
    RemoveAct(field::Point),
}

impl Act {
    fn move_remove(&self) -> bool {
        match self {
            Act::MoveAct(_) | Act::RemoveAct(_) => true,
            _ => false,
        }
    }
    fn pos(&self) -> Option<field::Point> {
        match self {
            Act::PutAct(x) | Act::MoveAct(x) | Act::RemoveAct(x) => Some(*x),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Simulator {
    field: field::Field,
    acts: Vec<Vec<Act>>,
    act_flag: [bool; 2],
}

impl Data for Simulator {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl Simulator {
    pub fn new(field: field::Field) -> Simulator {
        Simulator {
            field: field.clone(),
            acts: vec![vec![Act::StayAct; field.agent_count()]; 2],
            act_flag: [false, false],
        }
    }
    pub fn get_field(&self) -> &field::Field {
        &self.field
    }
    pub fn get_mut_field(&mut self) -> &mut field::Field {
        &mut self.field
    }
    pub fn set_field(&mut self, field: &field::Field) {
        self.field = field.clone();
    }
    pub fn get_act(&self, side: bool, id: usize) -> Act {
        self.acts[side as usize][id].clone()
    }
    pub fn get_acts(&self, side: usize) -> Vec<Act> {
        self.acts[side as usize].clone()
    }
    pub fn set_act(&mut self, side: bool, id: usize, act: Act) {
        self.acts[side as usize][id] = act;
        self.act_flag[side as usize] = true;
    }
    pub fn change_turn(&mut self) {
        if self.field.now_turn() == self.field.final_turn()
            || !self.act_flag[0]
            || !self.act_flag[1]
        {
            return;
        }

        let act_vec = act_list(&self.acts, &self.field);
        for (side, id, act) in act_vec {
            match act {
                Act::PutAct(nex_pos) => self.field.set_agent(side, id, Some(nex_pos)),
                Act::MoveAct(nex_pos) => {
                    self.field.set_agent(side, id, Some(nex_pos));
                    self.field
                        .set_state(nex_pos.usize(), field::State::Wall(side));
                }
                Act::RemoveAct(nex_pos) => {
                    self.field.set_state(nex_pos.usize(), field::State::Neutral);
                }
                _ => {}
            }
        }

        self.field.update_region();
        self.field.update_score();
        self.field.update_turn();
        self.acts = vec![vec![Act::StayAct; self.field.agent_count()]; 2];
        self.act_flag[0] = false;
        self.act_flag[1] = false;
    }
}

pub fn act_list(acts_: &Vec<Vec<Act>>, field: &field::Field) -> Vec<(bool, usize, Act)> {
    let mut acts = acts_.clone();
    let mut pos_map = HashSet::new();
    let mut act_map: HashMap<field::Point, Vec<(bool, usize)>> = HashMap::new();

    for side in vec![true, false] {
        for i in 0..field.agent_count() {
            let act = acts[side as usize][i].clone();
            acts[side as usize][i] = match field.agent(side, i) {
                Some(agent_pos) => {
                    if act.move_remove()
                        && field.inside(act.pos().unwrap())
                        && agent_pos.neighbor(act.pos().unwrap())
                    {
                        let state = field.tile(act.pos().unwrap().usize()).state();
                        match act {
                            Act::MoveAct(_) if state != field::State::Wall(!side) => act.clone(),
                            Act::RemoveAct(_) if state.is_wall() => act.clone(),
                            _ => Act::StayAct,
                        }
                    } else {
                        Act::StayAct
                    }
                }
                None => match act {
                    Act::PutAct(_) => {
                        let state = field.tile(act.pos().unwrap().usize()).state();
                        if state != field::State::Wall(!side) {
                            act.clone()
                        } else {
                            Act::StayAct
                        }
                    }
                    _ => Act::StayAct,
                },
            };
            if let Some(pos) = &act.pos() {
                match act_map.get_mut(&pos) {
                    Some(v) => v.push((side, i)),
                    None => {
                        act_map.insert(pos.clone(), vec![(side, i)]);
                    }
                }
            }
            if let Some(agent_pos) = &field.agent(side, i) {
                pos_map.insert(agent_pos.clone());
            }
        }
    }
    let mut que = VecDeque::new();
    for k in act_map.keys() {
        if !pos_map.contains(k) {
            que.push_back(k.clone());
        }
    }

    let mut act_vec = Vec::new();

    while !que.is_empty() {
        let k = que.front().unwrap().clone();
        que.pop_front();
        if let Some(out_moves) = act_map.get(&k) {
            if out_moves.len() >= 2 {
                continue;
            }
            for (side, idx) in out_moves {
                let before_pos = field.agent(*side, *idx).clone();
                let act = &acts[*side as usize][*idx];
                act_vec.push((*side, *idx, act.clone()));
                if let Some(bef_pos) = before_pos {
                    if let Act::MoveAct(_) = act {
                        if pos_map.contains(&bef_pos) {
                            pos_map.remove(&bef_pos);
                            que.push_back(bef_pos);
                        }
                    }
                }
            }
        }
    }
    act_vec
}
