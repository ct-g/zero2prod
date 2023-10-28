use crate::session_state::TypedSession;

use axum::{
    http::Request,
    middleware::Next,
    response::{Response, IntoResponse},
};
use axum_flash::Flash;
use axum_session::SessionRedisPool;
use uuid::Uuid;

use std::ops::Deref;

#[derive(Copy, Clone, Debug)]
pub struct UserId(Uuid);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for UserId {
    type Target = Uuid;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn reject_anonymous_users<B>(
    session: TypedSession<SessionRedisPool>,
    flash: Flash,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    match session.get_user_id() {
        Some(user_id) => {
            request.extensions_mut().insert(UserId(user_id));
            next.run(request).await
        },
        None => {
            let flash = flash.error("The user is not logged in.");
            (flash, axum::response::Redirect::to("/login")).into_response()
        }
    }
}