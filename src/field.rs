use druid::{Data, WidgetExt};
use rand;
use rand::Rng;
use std::borrow::BorrowMut;

#[derive(Copy, Clone, PartialEq)]
pub struct Point {
    x: usize,
    y: usize,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Score {
    tile: i16,
    region: i16,
}

impl Score {
    fn sum(&self) -> i16 {
        self.tile + self.region
    }
}

#[derive(Copy, Clone, PartialEq)]
struct Tile {
    state: u8,
    point: i8,
}

#[derive(Clone, PartialEq)]
pub struct Field {
    now_turn: i8,
    final_turn: i8,
    tiles: Vec<Vec<Tile>>,
    agents: Vec<Vec<Option<Point>>>,
    scores: Vec<Score>,
}

impl Data for Field {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl Field {
    pub fn make(width: usize, height: usize) -> Field {
        let mut rng = rand::thread_rng();

        let field = Field {
            now_turn: 0,
            final_turn: 0,
            tiles: (0..width)
                .map(|x| {
                    (0..height)
                        .map(|y| Tile {
                            state: 0,
                            point: rng.gen_range(-16, 17),
                        })
                        .collect()
                })
                .collect(),
            agents: vec![vec![None; 2]; 2],
            scores: vec![Score { tile: 0, region: 0 }; 2],
        };
        field
    }
    fn read_field(id: &str) -> Field {
        Field::make(16, 16)
    }
    fn calc_score(&mut self) {}
    pub fn width(&self) -> usize {
        self.tiles.len()
    }
    pub fn height(&self) -> usize {
        self.tiles.get(0).unwrap().len()
    }
}
