use actix_web::{
    HttpRequest,
    HttpResponse,
    http::header,
    web::{Data, Query},
};
use api_utils::context::WyrmContext;
use database::models::local_user::{LocalUser, LocalUserUpdateForm};
use serde::Deserialize;
use wyrm_auth::{Auth, oidc::CompletedLogin, session::Authenticated};
use wyrm_utils::{
    error::{AuthError, DatabaseError, WyrmError},
    result::WyrmResult,
};

#[derive(Deserialize)]
pub struct OidcLoginQuery {
    return_to: Option<String>,
    /// Link the identity to the signed-in user instead of signing in.
    #[serde(default)]
    link: bool,
}

#[derive(Deserialize)]
pub struct OidcCallbackQuery {
    state: Option<String>,
    code: Option<String>,
    error: Option<String>,
}

pub async fn login(query: Query<OidcLoginQuery>, auth: Data<Auth>) -> WyrmResult<HttpResponse> {
    let oidc = auth.oidc()?;
    let OidcLoginQuery { return_to, link } = query.into_inner();

    let (location, cookie) = match oidc.begin_login(return_to, link).await {
        Ok((url, cookie)) => (url.to_string(), Some(cookie)),
        Err(error) => {
            tracing::warn!(%error, "OIDC login could not start");
            ("/?auth_error=oidc_unavailable".to_owned(), None)
        }
    };

    let mut response = HttpResponse::SeeOther();
    response
        .insert_header((header::LOCATION, location))
        .insert_header((header::CACHE_CONTROL, "no-store"));

    if let Some(cookie) = cookie {
        response.cookie(cookie);
    }

    Ok(response.finish())
}

/// A GET that creates a session: the provider redirects the browser here.
pub async fn callback(
    req: HttpRequest,
    query: Query<OidcCallbackQuery>,
    ctx: Data<WyrmContext>,
    auth: Data<Auth>,
) -> WyrmResult<HttpResponse> {
    let oidc = auth.oidc()?;

    let mut response = HttpResponse::SeeOther();
    response
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .insert_header((header::REFERRER_POLICY, "no-referrer"))
        .cookie(oidc.clear_login_cookie());

    match sign_in(&req, query.into_inner(), &auth, &ctx).await {
        Ok((return_to, authenticated)) => {
            response.insert_header((header::LOCATION, return_to));

            authenticated.set_cookie(&mut response);
        }
        Err(error) => {
            tracing::warn!(%error, "OIDC login failed");
            let location = match error {
                WyrmError::Database(DatabaseError::UniqueViolation(_)) => {
                    "/?auth_error=oidc_already_linked"
                }
                WyrmError::Auth(AuthError::OidcSubjectNotLinked(_)) => {
                    "/?auth_error=oidc_not_linked"
                }
                _ => "/?auth_error=oidc",
            };
            response.insert_header((header::LOCATION, location));
        }
    }

    Ok(response.finish())
}

/// Verifies the callback with the provider, then links the identity to the signed-in user or
/// starts a session for the user it's linked to.
async fn sign_in(
    req: &HttpRequest,
    query: OidcCallbackQuery,
    auth: &Auth,
    ctx: &WyrmContext,
) -> WyrmResult<(String, Authenticated)> {
    if let Some(error) = query.error {
        return Err(AuthError::LoginFailed(format!("OIDC: provider returned {error}")).into());
    }

    let (Some(state), Some(code)) = (query.state, query.code) else {
        return Err(AuthError::LoginFailed("OIDC: missing state or code".into()).into());
    };

    let CompletedLogin {
        return_to,
        subject,
        link,
    } = auth.oidc()?.complete_login(req, &state, &code).await?;

    let sessions = auth.sessions()?;

    if link {
        // Keeps the current session, so the user stays signed in.
        let authenticated = sessions
            .authenticate(&ctx.db_pool, req)
            .await?
            .ok_or_else(|| AuthError::LoginFailed("OIDC: link requires a signed-in user".into()))?;

        let form = LocalUserUpdateForm {
            username: None,
            password_hash: None,
            oidc_subject: Some(subject),
        };

        let mut conn = ctx.db_pool.get().await?;
        LocalUser::update(&mut conn, authenticated.user_id, form).await?;

        return Ok((return_to, authenticated));
    }

    let user = LocalUser::get_by_oidc_subject(&ctx.db_pool, &subject)
        .await?
        .ok_or_else(|| AuthError::OidcSubjectNotLinked(subject.clone()))?;

    let authenticated = sessions.create(&ctx.db_pool, user.id).await?;

    Ok((return_to, authenticated))
}
