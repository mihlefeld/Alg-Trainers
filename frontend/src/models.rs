use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub authenticated: bool,
    pub email: Option<String>,
    pub wca_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrainerSummary {
    pub trainer_id: String,
    pub puzzle: String,
    pub method: String,
    pub name: String,
    pub order: i32,
    pub max_algs_per_row: i32,
    pub selection_key: String,
    pub times_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrainerCase {
    pub case_id: String,
    pub name: String,
    pub setup: Option<String>,
    pub image_svg: Option<String>,
    pub algs: Vec<TrainerAlg>,
    pub scrambles: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainerAlg {
    pub alg_id: String,
    pub alg: String,
    pub position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAlgState {
    pub alg_id: String,
    pub alg: String,
    pub position: i32,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayAlg {
    pub alg_id: String,
    pub alg: String,
    pub position: i32,
    pub user: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TrainerGroup {
    pub name: String,
    pub cases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrainerAlgset {
    pub name: String,
    pub groups: Vec<String>,
    pub selected: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrainerBundle {
    pub trainer_id: String,
    pub title: String,
    pub puzzle: String,
    pub method: String,
    pub cases: Vec<TrainerCase>,
    pub groups: Vec<TrainerGroup>,
    pub algsets: Vec<TrainerAlgset>,
    pub pre_rotations: Vec<String>,
    pub post_rotations: Vec<String>,
    pub pre_moves: Vec<String>,
    pub post_moves: Vec<String>,
    pub max_algs_per_row: i32,
    #[serde(default)]
    pub u_move_cycles: Vec<Vec<usize>>,
    pub selection_key: String,
    pub times_key: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct StoredResult {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub datetime: String,
    pub ms: i64,
    #[serde(deserialize_with = "deserialize_case_id")]
    pub case: String,
    #[serde(default)]
    pub time: String,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SelectionPreset {
    pub sel_cases: Vec<String>,
    #[serde(default)]
    pub selected_alg_sets: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SyncBundle {
    pub settings: Option<SettingsSync>,
    pub selections: Vec<SelectionSync>,
    pub user_algs: Vec<UserAlgSync>,
    pub results: Vec<ResultSync>,
    #[serde(default)]
    pub next_results: Option<ResultCursor>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SettingsSync {
    pub settings_json: serde_json::Value,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectionSync {
    pub trainer_id: String,
    pub selection_json: serde_json::Value,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserAlgSync {
    pub trainer_id: String,
    pub case_id: String,
    pub alg_id: String,
    pub alg: String,
    pub position: i32,
    pub hidden: bool,
    pub deleted: bool,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResultSync {
    pub id: String,
    pub datetime: String,
    pub result_ms: i64,
    pub trainer_id: String,
    pub case_id: String,
    pub updated_at: String,
    pub deleted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResultCursor {
    pub datetime: String,
    pub id: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Select,
    Practice,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dialog {
    None,
    Settings,
    Hotkeys,
    Credits,
    Stats,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlgsetClickAction {
    Disable,
    SelectAll,
    SelectNone,
}

fn deserialize_case_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(value) => Ok(value),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        _ => Err(serde::de::Error::custom("case must be a string or number")),
    }
}
