use database::models::instance::{LocalInstance, RegistrationMode};
use serde::Serialize;

/// What the setup and sign-in screens need, readable without signing in.
#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct InstanceStatusResponse {
    pub is_setup: bool,
    pub registration_mode: RegistrationMode,
}

impl From<LocalInstance> for InstanceStatusResponse {
    fn from(instance: LocalInstance) -> Self {
        Self {
            is_setup: instance.is_setup,
            registration_mode: instance.registration_mode,
        }
    }
}

/// Instance-wide settings that admins change.
#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct InstanceConfigResponse {
    pub registration_mode: RegistrationMode,
}

impl From<LocalInstance> for InstanceConfigResponse {
    fn from(instance: LocalInstance) -> Self {
        Self {
            registration_mode: instance.registration_mode,
        }
    }
}
