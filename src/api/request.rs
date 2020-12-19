use super::parse;
use crate::api::parse::{FieldData, MatchData, TeamData};
use crate::simulator::Act;
use reqwest;

pub fn get_match_data(cfg: &parse::Config) -> Result<Vec<MatchData>, String> {
    let url = cfg.url.clone() + &*format!("/teams/{}/matches", cfg.id);
    let client = reqwest::blocking::Client::new();
    let result = client
        .get(&url)
        .header("x-api-token", cfg.token.clone())
        .send()
        .map_err(|e| e.to_string())?;
    if result.status() != 200 {
        return Err(format!("invalid status {}", result.status()));
    }
    let result = result.text().map_err(|e| e.to_string())?;
    Ok(parse::parse_matches_data(
        serde_json::from_str(&*result).map_err(|e| e.to_string())?,
    )?)
}

pub fn get_field_data(
    match_data: &parse::MatchData,
    cfg: &parse::Config,
) -> Result<FieldData, String> {
    let url = cfg.url.clone() + &*format!("/matches/{}", match_data.match_id);
    let client = reqwest::blocking::Client::new();
    let result = client
        .get(&url)
        .header("x-api-token", cfg.token.clone())
        .body("{\"matchID:".to_string() + &*match_data.match_id.to_string() + "}")
        .send()
        .map_err(|e| e.to_string())?;
    if result.status() != 200 {
        return Err(format!("invalid status {}", result.status()));
    }
    let result = result.text().map_err(|e| e.to_string())?;
    println!("text: {}", result);
    Ok(parse::parse_field_data(
        serde_json::from_str(&*result).map_err(|e| e.to_string())?,
        match_data.final_turn as u8,
    )?)
}

pub fn send_act(
    acts: Vec<Act>,
    team_data: &parse::TeamData,
    match_data: &parse::MatchData,
    cfg: &parse::Config,
) {
    println!("{:?}", acts);
    let url = cfg.url.clone() + &*format!("/matches/{}/action", match_data.match_id);
    let client = reqwest::blocking::Client::new();

    let act_str = "{\"actions\":[".to_string()
        + &*acts
            .iter()
            .zip(&team_data.agent_id)
            .map(|(act, id)| {
                let (x, y, type_str) = match act {
                    Act::StayAct => (0, 0, "stay"),
                    Act::PutAct(p) => (p.x + 1, p.y + 1, "put"),
                    Act::MoveAct(p) => (p.x + 1, p.y + 1, "move"),
                    Act::RemoveAct(p) => (p.x + 1, p.y + 1, "remove"),
                };
                "{".to_string()
                    + &*format!(
                        "\"x\":{},\"y\":{},\"type\":\"{}\",\"agentID\":{}",
                        x, y, type_str, id
                    )
                    + "}"
            })
            .collect::<Vec<_>>()
            .join(",")
        + "]}";

    println!("str:{}", act_str);

    let result = client
        .post(&url)
        .header("x-api-token", cfg.token.clone())
        .header("Content-Type", "application/json")
        .body(act_str)
        .send()
        .map_err(|e| e.to_string());
    match result {
        Ok(result) => {
            if result.status().as_u16() >= 400 {
                println!("ERROR: invalid status {}", result.status());
            } else {
                println!("status: {}", result.status());
                println!("result: {}", result.text().unwrap());
            }
        }
        Err(ok) => println!("ERROR: {}", ok),
    }
}
