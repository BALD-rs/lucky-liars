use dotenvy::dotenv;
use dotenvy_macro::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClearRequest {
    pub game_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InterrogateRequest {
    pub name: String,
    pub game_id: String,
    pub message: String,
    pub our_roll: u8,
    pub sus_roll: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InterrogateResponse {
    pub response: String,
    pub confidence: u8,
}

#[derive(Deserialize)]
pub struct StartResponse {
    pub game_id: String,
    pub clyde: String,
    pub glinda: String,
    pub harry: String,
    pub killer: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct dossiers {
    clyde: String,
    glinda: String,
    harry: String,
}

pub fn interrogate(req: InterrogateRequest) -> InterrogateResponse {
    dotenv().ok();
    let res = reqwest::blocking::Client::new()
        .post(format!("{}{}", dotenv!("API_BASE_URL"), "interrogate"))
        .json(&req)
        .send()
        .unwrap();
    let json = res.json::<InterrogateResponse>().unwrap();
    json
}

pub fn clear(req: ClearRequest) -> String {
    let res = reqwest::blocking::Client::new()
        .post(format!("{}{}", dotenv!("API_BASE_URL"), "clear"))
        .json(&req)
        .send()
        .unwrap();
    let json = res.json::<InterrogateResponse>().unwrap();
    json.response
}

pub fn start() -> StartResponse {
    let res = reqwest::blocking::Client::new()
        .post(format!("{}{}", dotenv!("API_BASE_URL"), "start"))
        .send()
        .unwrap();
    let json = res.json::<StartResponse>().unwrap();
    json
}
