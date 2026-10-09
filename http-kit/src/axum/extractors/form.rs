use axum::extract::{Form, FromRequest, Request, rejection::FormRejection};
use serde::de::DeserializeOwned;
use validator::Validate;

use super::rejection::ExtractorRejection;

/// Deserializes and validates a form body before the handler runs.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

impl<T> ValidatedForm<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = ExtractorRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}
