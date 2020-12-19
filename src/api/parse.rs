use crate::field;

use druid::Data;
use field::Field;
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use try_from::TryFrom;

fn to_result<T: Clone>(opt: Option<T>, target: &str) -> Result<T, String> {
    match opt {
        Some(val) => Ok(val.clone()),
        None => Err(format!("couldn't parse {}", target)),
    }
}

#[derive(Clone, Debug, PartialEq, Data)]
pub struct TeamData {
    pub team_id: u32,
    #[data(same_fn = "PartialEq::eq")]
    pub agent_id: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Data)]
pub struct MatchData {
    pub match_id: u32,
    pub final_turn: u32,
    #[data(same_fn = "PartialEq::eq")]
    pub teams: Vec<TeamData>,
}

pub struct FieldData {
    pub field: Field,
    pub teams: Vec<TeamData>,
}

#[derive(Clone, Debug, Deserialize, Data)]
pub struct Config {
    pub id: usize,
    pub token: String,
    pub url: String,
    pub visualizer: bool,
}

macro_rules! err_ret {
    ($f: ident) => {{
        if let Err(tmp) = $f {
            return Err(tmp);
        }
        $f.unwrap()
    }};
}

macro_rules! err_ret_vec {
    ($f: ident) => {{
        for tmp in &$f {
            if let Err(tmp2) = tmp {
                return Err(tmp2.clone());
            }
        }
        $f.into_iter()
            .map(|x| x.clone().unwrap())
            .collect::<Vec<_>>()
    }};
}

fn parse_2d_list<T: Clone + TryFrom<i64>>(v: &Value, msg: &str) -> Result<Vec<Vec<T>>, String>
where
    <T as try_from::TryFrom<i64>>::Err: std::fmt::Debug,
{
    let res = (to_result(v.as_array(), msg)?)
        .iter()
        .map(|w| {
            let w = to_result(w.as_array(), msg).map(|x| x.clone());
            let w = err_ret!(w);
            let u = w
                .iter()
                .map(|x| {
                    to_result(
                        x.as_i64()
                            .map(|y| TryFrom::try_from(y).expect("cast error")),
                        msg,
                    )
                })
                .collect::<Vec<_>>();
            Ok(err_ret_vec!(u))
        })
        .collect::<Vec<_>>();
    let r: Vec<Vec<T>> = err_ret_vec!(res);
    let mut v: Vec<Vec<T>> = vec![vec![r[0][0].clone(); r.len()]; r[0].len()];
    for i in 0..r.len() {
        for j in 0..r[i].len() {
            v[j][i] = r[i][j].clone();
        }
    }
    Ok(v)
}

fn get_team_data(teams: &Vec<Value>) -> Result<Vec<TeamData>, String> {
    let team_data = (0..2)
        .map(|i| {
            let id = to_result(teams[i]["teamID"].as_u64(), &format!("teams[{}].teamID", i));
            let id = err_ret!(id);
            let agents = to_result(
                teams[i]["agents"].as_array(),
                &format!("teams[{}].agents", i),
            );
            let agent_id = if let Ok(agents) = agents {
                let agent_id = agents
                    .iter()
                    .map(|dat| to_result(dat["agentID"].as_u64(), "agentID"))
                    .collect::<Vec<_>>();
                err_ret_vec!(agent_id).iter().map(|x| *x as u32).collect()
            } else {
                Vec::new()
            };

            Ok(TeamData {
                team_id: id as u32,
                agent_id,
            })
        })
        .collect::<Vec<_>>();
    Ok(err_ret_vec!(team_data))
}

pub fn parse_matches_data(val: Value) -> Result<Vec<MatchData>, String> {
    let matches = to_result(val["matches"].as_array(), "matches")?;
    let res = matches
        .iter()
        .map(|dat| {
            let match_id = to_result(dat["matchID"].as_u64(), "matchID");
            let final_turn = to_result(dat["turns"].as_u64(), "turns");
            let teams = to_result(dat["teams"].as_array(), "teams");
            let team_data = get_team_data(err_ret!(teams));
            Ok(MatchData {
                match_id: err_ret!(match_id) as u32,
                final_turn: err_ret!(final_turn) as u32,
                teams: err_ret!(team_data),
            })
        })
        .collect::<Vec<_>>();
    Ok(err_ret_vec!(res))
}

pub fn parse_field_data(val: Value, final_turn: u8) -> Result<FieldData, String> {
    let width = to_result(val["width"].as_u64(), "width")? as usize;
    let height = to_result(val["height"].as_u64(), "height")? as usize;
    let now_turn = to_result(val["turn"].as_u64(), "turn")? as u8;
    let teams = to_result(val["teams"].as_array(), "teams")?;

    let agent_data = (0..2)
        .map(|i| {
            let agents = to_result(
                teams[i]["agents"].as_array(),
                &format!("teams[{}][agents]", i),
            )?;
            let agent_pos = agents
                .iter()
                .enumerate()
                .map(|(idx, dat)| {
                    let x = to_result(dat["x"].as_u64(), "x");
                    let x = err_ret!(x) as i8 - 1;
                    let y = to_result(dat["y"].as_u64(), "y");
                    let y = err_ret!(y) as i8 - 1;
                    if x == -1 {
                        Ok(None)
                    } else {
                        Ok(Some(field::Point::new(x, y)))
                    }
                })
                .collect::<Vec<_>>();
            Ok(err_ret_vec!(agent_pos))
        })
        .collect::<Vec<_>>();
    let agent_data = err_ret_vec!(agent_data);

    let team_data = get_team_data(teams)?;

    let walls = parse_2d_list::<u32>(&val["walls"], "walls")?;
    let points = parse_2d_list::<i8>(&val["points"], "points")?;
    let field = Field::from_data(
        width,
        height,
        team_data.clone(),
        walls,
        points,
        agent_data,
        now_turn,
        final_turn,
    );

    Ok(FieldData {
        field,
        teams: team_data,
    })
}

pub fn read_config_json(path: &str) -> Config {
    let fp = File::open(path).expect("file not found");
    let res = serde_json::from_reader(fp).expect("config parse error");
    res
}

#[derive(Clone, Deserialize)]
pub struct Params {
    pub PER: f64,
    pub FIRST_MOVE_BONUS: f64,

    pub AG_CONF_PER: f64,
    pub REGION_PER: f64,
    pub REGION_POW: f64,
    pub PUT_CONF_POW: f64,

    pub SA_LAST_PENA: f64,
    pub SA_LAST_POW: f64,
    pub SA_LAST_SUPER_PENA: f64,
    pub SA_LAST_SUPER_BORDER: f64,
    pub SA_CONF_PER: f64,
    pub SA_CONF_PENA: f64,
    pub SA_DIST_PENA: f64,
    pub SA_DIST_POW: f64,

    pub LCP_PER: f64,
    pub LCP_POW: f64,
    pub SAME_TILE_PER: f64,
    pub SAME_TILE_POW: f64,
}

impl Params {
    pub fn default() -> Params {
        Params {
            PER: 0.61,
            FIRST_MOVE_BONUS: 1.10,

            AG_CONF_PER: 0.58,
            REGION_PER: 0.50,
            REGION_POW: 0.85,
            PUT_CONF_POW: 0.75,

            SA_LAST_PENA: 0.5,
            SA_LAST_POW: 4.8,
            SA_LAST_SUPER_PENA: 2.2,
            SA_LAST_SUPER_BORDER: 0.45,
            SA_CONF_PER: 0.7,
            SA_CONF_PENA: 1.15,
            SA_DIST_PENA: 27.0,
            SA_DIST_POW: 0.6,

            LCP_PER: 2.0,
            LCP_POW: 2.0,
            SAME_TILE_PER: 1.0,
            SAME_TILE_POW: 2.0,
        }
        /*
        Params {
            PER: 0.6,
            FIRST_MOVE_BONUS: 1.0,

            AG_CONF_PER: 0.3,
            REGION_PER: 1.0,
            REGION_POW: 0.9,
            PUT_CONF_POW: 0.7,

            SA_LAST_PENA: 0.3,
            SA_LAST_POW: 3.5,
            SA_LAST_SUPER_PENA: 2.5,
            SA_LAST_SUPER_BORDER: 0.25,
            SA_CONF_PER: 0.6,
            SA_CONF_PENA: 3.5,
            SA_DIST_PENA: 30.0,
            SA_DIST_POW: 0.4,

            LCP_PER: 2.0,
            LCP_POW: 2.0,
            SAME_TILE_PER: 1.0,
            SAME_TILE_POW: 2.0,
        }
        */
    }
}

pub fn read_params(path: &str) -> Params {
    let fp = File::open(path).expect("file not found");
    let res = serde_json::from_reader(fp).expect("params parse error");
    res
}
