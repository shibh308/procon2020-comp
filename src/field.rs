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
    pub fn new(width: usize, height: usize) -> Field {
        let mut rng = rand::thread_rng();
        let field = Field {
            now_turn: 0,
            final_turn: 0,
            tiles: (0..width)
                .map(|x| {
                    (0..height)
                        .map(|y| Tile {
                            state: State::Neutral,
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
        Field::new(16, 16)
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
