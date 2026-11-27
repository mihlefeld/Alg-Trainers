use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub authenticated: bool,
    pub email: Option<String>,
    pub wca_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrainerCase {
    pub case_id: String,
    pub name: String,
    pub setup: Option<String>,
    pub image_svg: Option<String>,
    pub algs: Vec<TrainerAlg>,
    pub scrambles: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrainerAlg {
    pub alg_id: String,
    pub alg: String,
    pub position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrainerGroup {
    pub name: String,
    pub cases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrainerAlgset {
    pub name: String,
    pub groups: Vec<String>,
    pub selected: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SyncBundle<Id = String, Ts = String> {
    pub settings: Option<SettingsSync<Ts>>,
    pub selections: Vec<SelectionSync<Ts>>,
    pub user_algs: Vec<UserAlgSync<Ts>>,
    pub results: Vec<ResultSync<Id, Ts>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_results: Option<ResultCursor<Id, Ts>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_time: Option<Ts>,
}

impl<Id, Ts> Default for SyncBundle<Id, Ts> {
    fn default() -> Self {
        Self {
            settings: None,
            selections: Vec::new(),
            user_algs: Vec::new(),
            results: Vec::new(),
            next_results: None,
            server_time: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SettingsSync<Ts = String> {
    pub settings_json: serde_json::Value,
    pub updated_at: Ts,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SelectionSync<Ts = String> {
    pub trainer_id: String,
    #[serde(default)]
    pub selection_json: serde_json::Value,
    pub updated_at: Ts,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserAlgSync<Ts = String> {
    pub trainer_id: String,
    pub case_id: String,
    pub alg_id: String,
    pub alg: String,
    pub position: i32,
    pub hidden: bool,
    pub deleted: bool,
    pub updated_at: Ts,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResultSync<Id = String, Ts = String> {
    pub id: Id,
    pub datetime: Ts,
    pub result_ms: i64,
    pub trainer_id: String,
    pub case_id: String,
    pub updated_at: Ts,
    pub deleted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResultCursor<Id = String, Ts = String> {
    pub datetime: Ts,
    pub id: Id,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn me_response_uses_camel_case() {
        let value = serde_json::to_value(MeResponse {
            authenticated: true,
            email: Some("user@example.com".to_owned()),
            wca_id: Some("2020TEST01".to_owned()),
        })
        .expect("serialize MeResponse");
        assert_eq!(value["wcaId"], "2020TEST01");
        assert!(value.get("wca_id").is_none());
    }
}
