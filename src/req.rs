use dotenvy::dotenv;
use dotenvy_macro::dotenv;
use serde::{Deserialize, Serialize};

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
}

pub fn interrogate(req: InterrogateRequest) -> String {
    dotenv().ok();
    let res = reqwest::blocking::Client::new()
        .post(format!("{}{}", dotenv!("API_BASE_URL"), "interrogate"))
        .json(&req)
        .send()
        .unwrap();
    let json = res.json::<InterrogateResponse>().unwrap();
    json.response
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

pub fn start() {
    let res = reqwest::blocking::Client::new()
        .post(format!("{}{}", dotenv!("API_BASE_URL"), "start"))
        .send()
        .unwrap();
}