use actix_web::{
    HttpRequest,
    HttpResponse,
    web::{Data, Json},
};
use api_utils::{context::WyrmContext, responses::instance::LocalInstanceResponse};
use database::{
    models::{
        feed::Feed,
        instance::{LocalInstance, LocalInstanceUpdateForm, RegistrationMode},
    },
    utils::runtime_settings::RuntimeSettings,
};
use serde::Deserialize;
use wyrm_auth::{
    Auth,
    authenticated_user::AuthenticatedUser,
    password::{hash_password, validate_password},
    username::normalize_username,
};
use wyrm_rss::{rsshub, worker::WorkerCommand};
use wyrm_utils::{
    error::{AuthError::InvalidSetupToken, DatabaseError::Conflict, WyrmError},
    result::WyrmResult,
};

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct InstanceSetup {
    /// The setup token retrieved from the server logs.
    token: String,
    /// The username for the admin/owner user.
    username: String,
    /// The password for the admin/owner user.
    password: String,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, optional_fields)]
pub struct UpdateLocalInstance {
    registration_mode: Option<RegistrationMode>,
    feed_poll_interval_secs: Option<i32>,
    http_timeout_secs: Option<i32>,
    http_connect_timeout_secs: Option<i32>,
    http_retries: Option<i32>,
    /// Omitted = keep, `null` = clear back to the default, value = set.
    #[serde(default, with = "serde_with::rust::double_option")]
    #[ts(optional, as = "Option<Option<String>>")]
    http_user_agent: Option<Option<String>>,
    /// Omitted = keep, `null` or blank = clear, value = set.
    #[serde(default, with = "serde_with::rust::double_option")]
    #[ts(optional, as = "Option<Option<String>>")]
    rsshub_instance: Option<Option<String>>,
}

/// Updates instance-level configuration. Admins only.
pub async fn update(
    user: AuthenticatedUser,
    Json(data): Json<UpdateLocalInstance>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<LocalInstanceResponse>> {
    ctx.deny_in_demo()?;

    user.require_admin()?;

    let form = LocalInstanceUpdateForm {
        registration_mode: data.registration_mode,
        feed_poll_interval_secs: data.feed_poll_interval_secs,
        http_timeout_secs: data.http_timeout_secs,
        http_connect_timeout_secs: data.http_connect_timeout_secs,
        http_retries: data.http_retries,
        http_user_agent: data.http_user_agent,
        rsshub_instance: data
            .rsshub_instance
            .map(|instance| match instance {
                Some(value) => rsshub::validate_instance(&value),
                None => Ok(None),
            })
            .transpose()?,
    };

    let previous_rsshub = LocalInstance::get(&ctx.db_pool).await?.rsshub_instance;

    let instance = LocalInstance::update(&ctx.db_pool, form).await?;

    if instance.rsshub_instance != previous_rsshub {
        Feed::clear_rsshub_validators(&ctx.db_pool).await?;
    }

    *ctx.runtime_settings.write()? = RuntimeSettings::from(&instance);

    // Interrupts the worker's sleep so it reconfigures now. Safe to drop, since
    // runtime_settings is already updated and the worker reads it next iteration.
    let _ = ctx.worker_tx.try_send(WorkerCommand::Reconfigure);

    Ok(Json(LocalInstanceResponse::new(instance, true)))
}

/// Sets up a new instance: checks the setup token from the server log, gives the
/// admin (user 1) a username and password, and signs them in with a session cookie.
pub async fn setup(
    req: HttpRequest,
    Json(data): Json<InstanceSetup>,
    auth: Data<Auth>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<HttpResponse> {
    let sessions = auth.sessions();

    sessions.check_csrf(&req)?;

    if LocalInstance::get(&ctx.db_pool).await?.is_setup {
        return Err(WyrmError::Database(Conflict(
            "instance already set up".into(),
        )));
    }

    if !auth.setup_token_matches(&data.token) {
        return Err(WyrmError::Auth(InvalidSetupToken));
    }

    let username = normalize_username(&data.username)?;

    validate_password(&data.password)?;

    let password_hash = hash_password(data.password).await?;

    let admin = LocalInstance::complete_setup(&ctx.db_pool, username, password_hash).await?;

    let authenticated = sessions.create(&ctx.db_pool, admin).await?;

    let mut response = HttpResponse::NoContent();
    authenticated.set_cookie(&mut response);

    Ok(response.finish())
}

#[cfg(test)]
mod tests {
    use super::UpdateLocalInstance;

    #[test]
    fn partial_instance_distinguishes_omitted_null_and_values() {
        let partial: UpdateLocalInstance = serde_json::from_str(r#"{"http_retries":5}"#).unwrap();
        assert_eq!(partial.http_user_agent, None);
        assert_eq!(partial.rsshub_instance, None);
        let cleared: UpdateLocalInstance =
            serde_json::from_str(r#"{"http_user_agent":null,"rsshub_instance":null}"#).unwrap();
        assert_eq!(cleared.http_user_agent, Some(None));
        assert_eq!(cleared.rsshub_instance, Some(None));
        let set: UpdateLocalInstance = serde_json::from_str(
            r#"{"http_user_agent":"custom","rsshub_instance":"https://example.com/rsshub"}"#,
        )
        .unwrap();
        assert_eq!(set.http_user_agent, Some(Some("custom".into())));
        assert_eq!(
            set.rsshub_instance,
            Some(Some("https://example.com/rsshub".into()))
        );
    }
}
