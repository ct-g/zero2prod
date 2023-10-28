use axum::{
    extract::FromRequestParts,
    http::request::Parts, async_trait,
};
use axum_session::{Session, DatabasePool};
use uuid::Uuid;

use std::fmt::Debug;

#[derive(Debug)]
pub struct TypedSession<T: DatabasePool + Clone + Debug + Sync + Send + 'static>(Session<T>);

impl<T> TypedSession<T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static
{
    const USER_ID_KEY: &'static str = "user_id";

    pub fn renew(&self) {
        self.0.renew();
    }

    pub fn insert_user_id(&self, user_id: Uuid) {
        self.0.set(Self::USER_ID_KEY, user_id);
    }

    pub fn get_user_id(&self) -> Option<Uuid> {
        self.0.get(Self::USER_ID_KEY)
    }

    pub fn logout(&self) {
        self.0.clear()
    }
}

#[async_trait]
impl<S, T> FromRequestParts<S> for TypedSession<T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
    S: Send + Sync
{
    type Rejection = <Session<T> as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(
        parts: & mut Parts,
        state: &S
    ) ->  Result<Self, Self::Rejection>
    {
        let session: Session<T> = Session::<T>::from_request_parts(parts, state).await?;

        Ok(TypedSession(session))
    }
}
