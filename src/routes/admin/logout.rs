use crate::session_state::TypedSession;
use axum::response::IntoResponse;
use axum_flash::Flash;

pub async fn logout<T>(
    flash: Flash,
    session: TypedSession<T>
) -> impl IntoResponse
where
    T: axum_session::DatabasePool + Clone + std::fmt::Debug + Sync + Send + 'static
{
    session.logout();
    let flash = flash.info("You have been successfully logged out.");
    return (flash, axum::response::Redirect::to("/login")).into_response();
}