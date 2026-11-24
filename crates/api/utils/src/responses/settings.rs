use database::models::settings::Settings;
use serde::Serialize;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SettingsResponse {
    #[serde(flatten)]
    #[ts(flatten)]
    pub settings: Settings,
    pub run_as_demo: bool,
}
