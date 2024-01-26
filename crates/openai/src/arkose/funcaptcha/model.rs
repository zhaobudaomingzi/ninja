use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub(super) struct RequestChallenge<'a> {
    pub sid: &'a str,
    pub token: &'a str,
    pub analytics_tier: i32,
    pub render_type: &'a str,
    pub lang: &'a str,
    #[serde(rename = "isAudioGame")]
    pub is_audio_game: bool,
    #[serde(rename = "apiBreakerVersion")]
    pub api_breaker_version: &'a str,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub(super) struct Challenge {
    pub session_token: String,
    #[serde(rename = "challengeID")]
    pub challenge_id: String,
    pub game_data: GameData,
    pub string_table: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub(super) struct GameData {
    #[serde(rename = "gameType")]
    pub game_type: i32,
    pub game_variant: String,
    pub instruction_string: String,
    #[serde(rename = "customGUI")]
    pub custom_gui: CustomGUI,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub(super) struct CustomGUI {
    #[serde(rename = "_challenge_imgs")]
    pub challenge_imgs: Vec<String>,
    pub api_breaker: ApiBreaker,
    pub api_breaker_v2_enabled: isize,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct ApiBreaker {
    pub key: String,
    pub value: Vec<String>,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub(super) struct ConciseChallenge {
    pub game_type: &'static str,
    pub urls: Vec<String>,
    pub instructions: String,
    pub game_variant: String,
}

#[derive(Debug, Clone)]
pub struct FunCaptcha {
    pub image: String,
    pub instructions: String,
    pub game_variant: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct SubmitChallenge<'a> {
    pub session_token: &'a str,
    pub sid: &'a str,
    pub game_token: &'a str,
    pub guess: &'a str,
    pub render_type: &'static str,
    pub analytics_tier: i32,
    pub bio: &'static str,
}
