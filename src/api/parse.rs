use crate::field;

use field::Field;
use serde_json::Value;
use try_from::TryFrom;

fn to_result<T: Clone>(opt: Option<T>, target: &str) -> Result<T, String> {
    match opt {
        Some(val) => Ok(val.clone()),
        None => Err(format!("couldn't parse {}", target)),
    }
}

#[derive(Clone)]
pub struct TeamData {
    pub team_id: u32,
    pub agent_id: Vec<u32>,
}

#[derive(Clone)]
pub struct MatchData {
    pub match_id: u32,
    pub final_turn: u32,
    pub teams: Vec<TeamData>,
}

pub struct FieldData {
    pub field: Field,
    pub teams: Vec<TeamData>,
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
    Ok(err_ret_vec!(res))
}

fn get_team_data(teams: &Vec<Value>) -> Result<Vec<TeamData>, String> {
    let team_data = (0..2)
        .map(|i| {
            let id = to_result(teams[i]["teamID"].as_u64(), &format!("teams[{}].teamID", i));
            let id = err_ret!(id);
            let agents = to_result(
                teams[i]["agentID"].as_array(),
                &format!("teams[{}].agentID", i),
            );
            let agent_id = agents
                .iter()
                .enumerate()
                .map(|(idx, dat)| to_result(dat[idx]["agentID"].as_u64(), "agentID"))
                .collect::<Vec<_>>();
            let agent_id = err_ret_vec!(agent_id).iter().map(|x| *x as u32).collect();

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
                teams[i]["agentID"].as_array(),
                &format!("teams[{}].agentID", i),
            );
            let agent_pos = agents
                .iter()
                .enumerate()
                .map(|(idx, dat)| {
                    let x = to_result(dat[idx]["x"].as_u64(), "x");
                    let x = err_ret!(x) as i8 - 1;
                    let y = to_result(dat[idx]["y"].as_u64(), "y");
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