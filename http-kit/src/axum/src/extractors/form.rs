use axum::extract::{Form, FromRequest, Request, rejection::FormRejection};
use serde::de::DeserializeOwned;
use validator::Validate;

use super::error::Error;

/// Wrapper for validated form payloads.
///
/// This struct is used to extract and validate form data from incoming requests.
/// It ensures that the payload conforms to the validation rules defined in the
/// corresponding struct.
///
/// # Type Parameters
/// - `T`: The type of the form payload, which must implement `DeserializeOwned`
///   and `Validate`.
///
/// # Example
/// ```rust,ignore
/// use serde::Deserialize;
/// use validator::Validate;
/// use validation::ValidatedFormPayload;
///
/// #[derive(Deserialize, Validate)]
/// struct MyForm {
///     #[validate(length(min = 1))]
///     name: String,
/// }
///
/// async fn handler(
///     ValidatedFormPayload(payload): ValidatedFormPayload<MyForm>,
/// ) {
///     // `payload` should be valid at this point.
/// }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedFormPayload<T>(pub T);

impl<T> ValidatedFormPayload<T> {
    /// Extracts the inner value.
    ///
    /// This method can be used to retrieve the validated payload after extraction.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, S> FromRequest<S> for ValidatedFormPayload<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedFormPayload(value))
    }
}
