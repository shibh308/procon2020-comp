use crate::field;
use druid::Data;

#[derive(Clone, PartialEq)]
pub enum Act {
    StayAct,
    PutAct(field::Point),
    MoveAct(field::Point),
    RemoveAct(field::Point),
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
            acts: vec![vec![Act::StayAct; field.agent_count()]; 2],
        }
    }
    pub fn get_field(&self) -> &field::Field {
        &self.field
    }
    pub fn change_turn(&mut self) {
        self.field.update_region();
        self.acts = vec![vec![Act::StayAct; self.field.agent_count()]; 2]
    }
}
