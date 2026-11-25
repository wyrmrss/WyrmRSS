use actix_web::web::{Data, Json};
use api_utils::{context::WyrmContext, responses::settings::SettingsResponse};
use database::models::settings::{ReadMode, Settings, SettingsUpdateForm};
use serde::Deserialize;
use wyrm_auth::authenticated_user::AuthenticatedUser;
use wyrm_utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdateSettings {
    page_size: Option<i32>,
    read_mode: Option<ReadMode>,
    #[serde(default, with = "serde_with::rust::double_option")]
    #[ts(optional, as = "Option<Option<i32>>")]
    expire_read_after_days: Option<Option<i32>>,
    #[serde(default, with = "serde_with::rust::double_option")]
    #[ts(optional, as = "Option<Option<i32>>")]
    expire_unread_after_days: Option<Option<i32>>,
    read_delay_secs: Option<i32>,
}

pub async fn update(
    user: AuthenticatedUser,
    Json(data): Json<UpdateSettings>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<SettingsResponse>> {
    ctx.deny_in_demo()?;

    let settings = Settings::update(
        &ctx.db_pool,
        SettingsUpdateForm {
            user_id: user.id,
            page_size: data.page_size,
            read_mode: data.read_mode,
            expire_read_after_days: data.expire_read_after_days,
            expire_unread_after_days: data.expire_unread_after_days,
            read_delay_secs: data.read_delay_secs,
        },
    )
    .await?;

    Ok(Json(SettingsResponse {
        settings,
        run_as_demo: ctx.run_as_demo,
    }))
}

#[cfg(test)]
mod tests {
    use super::UpdateSettings;

    #[test]
    fn partial_settings_distinguish_omitted_null_and_values() {
        let partial: UpdateSettings = serde_json::from_str(r#"{"page_size":50}"#).unwrap();
        assert_eq!(partial.expire_read_after_days, None);
        assert_eq!(partial.expire_unread_after_days, None);
        let cleared: UpdateSettings = serde_json::from_str(
            r#"{"expire_read_after_days":null,"expire_unread_after_days":null}"#,
        )
        .unwrap();
        assert_eq!(cleared.expire_read_after_days, Some(None));
        assert_eq!(cleared.expire_unread_after_days, Some(None));
        let set: UpdateSettings =
            serde_json::from_str(r#"{"expire_read_after_days":30,"expire_unread_after_days":90}"#)
                .unwrap();
        assert_eq!(set.expire_read_after_days, Some(Some(30)));
        assert_eq!(set.expire_unread_after_days, Some(Some(90)));
    }
}
