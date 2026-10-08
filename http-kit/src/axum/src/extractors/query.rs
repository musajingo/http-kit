use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use serde::de::DeserializeOwned;
use validator::Validate;

use super::error::Error;

/// Wrapper for validated query string parameters.
///
/// This struct is used to extract and validate query string data from incoming requests.
/// It ensures that the parameters conform to the validation rules defined in the
/// corresponding struct `T`.
///
/// # Type Parameters
/// - `T`: The type to deserialize the query string into, which must implement
///   `DeserializeOwned` and `Validate`.
///
/// # Example
/// ```rust,ignore
/// use serde::Deserialize;
/// use validator::Validate;
/// use validation::ValidatedQuery;
///
/// #[derive(Deserialize, Validate)]
/// struct ListParams {
///     #[validate(range(min = 1))]
///     page: Option<u32>,
///
///     #[validate(range(min = 1, max = 100))]
///     per_page: Option<u32>,
/// }
///
/// async fn handler(
///     ValidatedQuery(params): ValidatedQuery<ListParams>,
/// ) {
///     // `params` is guaranteed to be valid here (if provided and deserializable)
/// }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

impl<T> ValidatedQuery<T> {
    /// Extracts the inner value.
    ///
    /// This method can be used to retrieve the validated payload after extraction.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state).await?;
        value.validate()?;
        Ok(ValidatedQuery(value))
    }
}
