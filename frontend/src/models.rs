pub use alg_trainers_shared::{
    MeResponse, ResultSync, SelectionSync, SettingsSync, SyncBundle, TrainerBundle, TrainerCase,
    TrainerSummary, UserAlgSync,
};
#[cfg(test)]
pub use alg_trainers_shared::{TrainerAlg, TrainerAlgset, TrainerGroup};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    pub hidden: bool,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SelectionPreset {
    pub sel_cases: Vec<String>,
    #[serde(default)]
    pub selected_alg_sets: BTreeMap<String, bool>,
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
