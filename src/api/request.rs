use super::parse;
use crate::api::parse::{FieldData, MatchData, TeamData};
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
    let mut url = cfg.url.clone() + &*format!("/matches/{}", match_data.match_id);
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
