use crate::api::parse::TeamData;
use druid::Data;
use rand;
use rand::Rng;
use std::collections::VecDeque;
use std::ops::Add;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i8,
    pub y: i8,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PointUsize {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: i8, y: i8) -> Point {
        Point { x, y }
    }
    pub fn usize(&self) -> PointUsize {
        PointUsize {
            x: self.x as usize,
            y: self.y as usize,
        }
    }
    pub fn neighbor(&self, other: Point) -> bool {
        (self.x - other.x).abs().max((self.y - other.y).abs()) <= 1
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl PointUsize {
    pub fn new(x: usize, y: usize) -> PointUsize {
        PointUsize { x, y }
    }
    pub fn normal(&self) -> Point {
        Point {
            x: self.x as i8,
            y: self.y as i8,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Score {
    tile: i16,
    region: i16,
}

impl Score {
    pub fn tile(&self) -> i16 {
        self.tile
    }
    pub fn region(&self) -> i16 {
        self.region
    }
    pub fn sum(&self) -> i16 {
        self.tile + self.region
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum State {
    Neutral,
    Position(bool),
    Wall(bool),
}

#[derive(Copy, Clone, PartialEq)]
pub struct Tile {
    state: State,
    point: i8,
}

impl State {
    pub fn is_wall(&self) -> bool {
        if let State::Wall(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_position(&self) -> bool {
        if let State::Position(_) = self {
            true
        } else {
            false
        }
    }
}

impl Tile {
    pub fn state(&self) -> State {
        self.state
    }
    pub fn point(&self) -> i8 {
        self.point
    }
}

#[derive(Clone, PartialEq)]
pub struct Field {
    now_turn: u8,
    final_turn: u8,
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
    pub fn new(
        width_: Option<usize>,
        height_: Option<usize>,
        agent_count_: Option<usize>,
    ) -> Field {
        let mut rng = rand::thread_rng();
        let width = if let Some(num) = width_ {
            num
        } else {
            rng.gen_range(12, 25)
        };
        let height = if let Some(num) = height_ {
            num
        } else {
            rng.gen_range(12, 25)
        };
        let agent_count = if let Some(num) = agent_count_ {
            num
        } else {
            rng.gen_range(6, 15)
        };
        let field = Field {
            now_turn: 0,
            final_turn: 50,
            tiles: (0..width)
                .map(|_x| {
                    (0..height)
                        .map(|_y| Tile {
                            state: State::Neutral,
                            point: if rng.gen::<f64>() < 0.25 {
                                rng.gen_range(-16, 0)
                            } else {
                                rng.gen_range(0, 17)
                            },
                        })
                        .collect()
                })
                .collect(),
            agents: vec![vec![None; agent_count]; 2],
            scores: vec![Score { tile: 0, region: 0 }; 2],
        };
        field
    }
    pub fn from_data(
        width: usize,
        height: usize,
        team_data: Vec<TeamData>,
        walls: Vec<Vec<u32>>,
        points: Vec<Vec<i8>>,
        agents: Vec<Vec<Option<Point>>>,
        now_turn: u8,
        final_turn: u8,
    ) -> Field {
        let mut field = Field {
            now_turn,
            final_turn,
            tiles: (0..width)
                .map(|x| {
                    (0..height)
                        .map(|y| Tile {
                            state: {
                                if walls[x][y] == team_data[0].team_id {
                                    State::Wall(false)
                                } else if walls[x][y] == team_data[1].team_id {
                                    State::Wall(true)
                                } else {
                                    State::Neutral
                                }
                            },
                            point: points[x][y] as i8,
                        })
                        .collect()
                })
                .collect(),
            agents: agents
                .iter()
                .map(|v| v.iter().map(|p| p.clone()).collect())
                .collect(),
            scores: vec![Score { tile: 0, region: 0 }; 2],
        };
        field.update_region();
        field.update_score();
        field
    }
    pub fn read_field(id: &str) -> Field {
        Field::new(None, None, None)
    }
    pub fn width(&self) -> usize {
        self.tiles.len()
    }
    pub fn height(&self) -> usize {
        self.tiles.get(0).unwrap().len()
    }
    pub fn agent_count(&self) -> usize {
        self.agents[0].len()
    }
    pub fn score(&self, side: bool) -> Score {
        self.scores[side as usize]
    }
    pub fn now_turn(&self) -> u8 {
        self.now_turn
    }
    pub fn final_turn(&self) -> u8 {
        self.final_turn
    }
    pub fn tile(&self, pos: PointUsize) -> Tile {
        self.tiles[pos.x][pos.y]
    }
    pub fn agent(&self, side: bool, id: usize) -> Option<Point> {
        self.agents[side as usize][id]
    }
    pub fn set_state(&mut self, pos: PointUsize, state: State) {
        self.tiles[pos.x][pos.y].state = state
    }
    pub fn set_agent(&mut self, side: bool, id: usize, pos: Option<Point>) {
        self.agents[side as usize][id] = pos
    }
    pub fn inside(&self, pos: Point) -> bool {
        let u_pos = pos.usize();
        0 <= pos.x.min(pos.y) && u_pos.x < self.width() && u_pos.y < self.height()
    }
    pub fn update_turn(&mut self) {
        assert_ne!(self.now_turn, self.final_turn);
        self.now_turn += 1;
    }
    pub fn update_region(&mut self) {
        let elm = vec![self.calc_region(false), self.calc_region(true)];
        for i in 0..self.width() {
            for j in 0..self.height() {
                if self.tile(PointUsize::new(i, j)).state.is_wall() {
                    continue;
                }
                if elm[0][i][j] < elm[1][i][j] {
                    self.set_state(PointUsize::new(i, j), State::Position(false));
                } else if elm[0][i][j] > elm[1][i][j] {
                    self.set_state(PointUsize::new(i, j), State::Position(true));
                }
            }
        }
    }
    pub fn update_score(&mut self) {
        let mut tile_point: [i16; 2] = [0, 0];
        let mut region_point: [i16; 2] = [0, 0];
        for i in 0..self.width() {
            for j in 0..self.height() {
                let tile = self.tile(PointUsize::new(i, j));
                match tile.state {
                    State::Wall(side) => tile_point[side as usize] += tile.point as i16,
                    State::Position(side) => region_point[side as usize] += tile.point.abs() as i16,
                    _ => {}
                }
            }
        }
        self.scores = vec![
            Score {
                tile: tile_point[0],
                region: region_point[0],
            },
            Score {
                tile: tile_point[1],
                region: region_point[1],
            },
        ];
    }
    fn calc_region(&self, side: bool) -> Vec<Vec<usize>> {
        let unk = self.width() * self.height();
        let mut elm = vec![vec![unk; self.height()]; self.width()];
        let mut cnt = 0;
        let mut siz = Vec::new();
        for i in 0..self.width() {
            for j in 0..self.height() {
                if let State::Wall(cmp_side) = self.tile(PointUsize::new(i, j)).state {
                    if side == cmp_side {
                        continue;
                    }
                }
                if elm[i][j] != unk {
                    continue;
                }
                elm[i][j] = cnt;
                let mut elm_cnt = 0_usize;
                let mut que = VecDeque::new();
                que.push_back(PointUsize::new(i, j));
                let mut out_flag = false;
                while !que.is_empty() {
                    elm_cnt += 1;
                    let top = que.front().unwrap().clone();
                    que.pop_front();
                    for diff in vec![
                        Point::new(0, 1),
                        Point::new(0, -1),
                        Point::new(1, 0),
                        Point::new(-1, 0),
                    ] {
                        if !self.inside(top.normal() + diff) {
                            out_flag = true;
                            continue;
                        }
                        let nex = (top.normal() + diff).usize();
                        if let State::Wall(cmp_side) = self.tile(nex).state {
                            if side == cmp_side {
                                continue;
                            }
                        }
                        if elm[nex.x][nex.y] != unk {
                            continue;
                        }
                        elm[nex.x][nex.y] = cnt;
                        que.push_back(nex);
                    }
                }
                siz.push(if out_flag { unk } else { elm_cnt });
                cnt += 1;
            }
        }
        elm.iter()
            .map(|v| {
                v.iter()
                    .map(|c| if *c == unk { unk } else { siz[*c] })
                    .collect()
            })
            .collect()
    }
}
