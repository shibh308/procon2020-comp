use crate::field;
use druid::Data;

#[derive(Clone, PartialEq)]
pub struct PutAct {}
#[derive(Clone, PartialEq)]
pub struct MoveAct {}
#[derive(Clone, PartialEq)]
pub struct RemoveAct {}

#[derive(Clone, PartialEq)]
pub enum Act {
    None,
    PutAct,
    MoveAct,
    RemoveAct,
}

#[derive(Clone, PartialEq)]
pub struct Simulator {
    field: field::Field,
    acts: Vec<Vec<Act>>,
}

impl Data for Simulator {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl Simulator {
    pub fn make(field: field::Field) -> Simulator {
        Simulator {
            field: field.clone(),
            acts: vec![vec![Act::None; field.get_agent_count()]; 2],
        }
    }
    pub fn get_field(&self) -> &field::Field {
        &self.field
    }
    pub fn change_turn(&mut self) {
        self.acts = vec![vec![Act::None; self.field.get_agent_count()]; 2]
    }
}
