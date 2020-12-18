use super::parse;
use crate::api::parse::MatchData;
use reqwest;

pub fn get_match_data(cfg: &parse::Config) -> Result<Vec<MatchData>, String> {
    let url = cfg.url.clone() + "/matches";
    let client = reqwest::blocking::Client::new();
    let result = client
        .get(&url)
        .header("x-api-token", cfg.token.clone())
        .send()
        .expect("get error")
        .text()
        .unwrap();
    Ok(parse::parse_matches_data(
        serde_json::from_str(&*result).unwrap(),
    )?)
}
