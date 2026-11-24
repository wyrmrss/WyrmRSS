use database::models::instance::RegistrationMode;
use serde::Serialize;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct LocalInstanceResponse {
    pub is_setup: bool,
    pub registration_mode: RegistrationMode,
}
