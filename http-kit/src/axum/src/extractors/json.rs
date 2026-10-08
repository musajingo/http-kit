use axum::extract::{FromRequest, Request, rejection::JsonRejection};
use serde::de::DeserializeOwned;
use validator::Validate;

use super::error::Error;

/// Wrapper for validated JSON payloads.
///
/// This struct is used to extract and validate JSON data from incoming requests.
/// It ensures that the payload conforms to the validation rules defined in the
/// corresponding struct.
///
/// # Type Parameters
/// - `T`: The type of the JSON payload, which must implement `DeserializeOwned`
///   and `Validate`.
///
/// # Example
/// ```rust,ignore
/// use serde::Deserialize;
/// use validator::Validate;
/// use validation::ValidatedJsonPayload;
///
/// #[derive(Deserialize, Validate)]
/// struct MyJson {
///     #[validate(email)]
///     email: String,
/// }
///
/// async fn handler(
///     ValidatedJsonPayload(payload): ValidatedJsonPayload<MyJson>,
/// ) {
///     // `payload` is guaranteed to be valid here
/// }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJsonPayload<T>(pub T);

impl<T> ValidatedJsonPayload<T> {
    /// Extracts the inner value.
    ///
    /// This method can be used to retrieve the validated payload after extraction.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, S> FromRequest<S> for ValidatedJsonPayload<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJsonPayload(value))
    }
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde::Deserialize;
    use serde_json::Value;
    use validator::Validate;

    use super::*;

    #[derive(Debug, Deserialize)]
    enum Kind {
        #[serde(rename = "good")]
        #[allow(dead_code)]
        Good,
    }

    #[derive(Debug, Deserialize, Validate)]
    struct Payload {
        #[allow(dead_code)]
        kind: Kind,
    }

    async fn extract(body: &str) -> Result<ValidatedJsonPayload<Payload>, Error> {
        let req = Request::builder()
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        ValidatedJsonPayload::from_request(req, &()).await
    }

    async fn response_json(err: Error) -> (StatusCode, Value) {
        let response = err.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    #[tokio::test]
    async fn invalid_field_value_returns_field_level_validation_error() {
        let err = extract(r#"{"kind":"bad"}"#).await.unwrap_err();
        let (status, json) = response_json(err).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["error"]["type"], "ValidationError");
        assert_eq!(json["error"]["details"]["kind"][0]["code"], "invalid");
    }

    #[tokio::test]
    async fn malformed_json_returns_bad_request() {
        let err = extract("{not json").await.unwrap_err();
        let (status, _) = response_json(err).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
