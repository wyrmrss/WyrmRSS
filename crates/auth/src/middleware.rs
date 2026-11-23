use crate::{Auth, desktop_key, session::AuthenticatedUser};
use actix_web::{
    Error,
    HttpMessage,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    http::Method,
    middleware::Next,
    web,
};
use api_utils::context::WyrmContext;
use database::{models::local_user::LocalUser, newtypes::UserId};

pub async fn middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let ctx = req
        .app_data::<web::Data<WyrmContext>>()
        .expect("context configured");

    let auth = req
        .app_data::<web::Data<Auth>>()
        .expect("auth data configured");

    // The desktop app's per-launch key. Desktop is single-user, so it acts as user 1.
    if desktop_key::valid(req.request(), auth) {
        let user = LocalUser::get(&ctx.db_pool, UserId(1)).await?;
        req.extensions_mut().insert(AuthenticatedUser::from(user));
        return next.call(req).await;
    }

    let sessions = auth.sessions()?;

    let Some(authenticated) = sessions.authenticate(&ctx.db_pool, req.request()).await? else {
        return Err(ErrorUnauthorized("Authentication required"));
    };

    if !safe_method(req.method()) {
        sessions.check_csrf(req.request())?;
    }

    req.extensions_mut().insert(authenticated.user);

    let mut response = next.call(req).await?;
    if let Some(cookie) = authenticated.cookie {
        response.response_mut().add_cookie(&cookie)?;
    }
    Ok(response)
}

/// Read-only HTTP methods. They skip CSRF since they're read-only and don't alter server state.
fn safe_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}
