use axum::extract::{FromRequest, Request, rejection::JsonRejection};
use serde::de::DeserializeOwned;
use valid8r::Validate;

use super::rejection::ExtractorRejection;

/// Deserializes and validates a JSON body before the handler runs.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    /// Consumes the extractor and returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ExtractorRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde::Deserialize;
    use serde_json::Value;
    use valid8r::Validate;

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

    async fn extract(body: &str) -> Result<ValidatedJson<Payload>, ExtractorRejection> {
        let req = Request::builder()
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        ValidatedJson::from_request(req, &()).await
    }

    async fn response_json(err: ExtractorRejection) -> (StatusCode, Value) {
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

#[cfg(test)]
mod sensitive_tests {
    use axum::body::{Body, to_bytes};
    use axum::extract::{FromRequest, Request};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde::Deserialize;
    use valid8r::Validate;

    use super::ValidatedJson;

    #[derive(Debug, Deserialize, Validate)]
    struct Credentials {
        #[validate(sensitive)]
        #[allow(dead_code)]
        pin: String,
    }

    #[tokio::test]
    async fn type_mismatch_never_echoes_the_submitted_value() {
        let req = Request::builder()
            .header("content-type", "application/json")
            .body(Body::from(r#"{"pin":123456}"#))
            .unwrap();
        let err = ValidatedJson::<Credentials>::from_request(req, &())
            .await
            .unwrap_err();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(!body.contains("123456"), "value leaked: {body}");
        assert!(body.contains("expected a string"), "lost the hint: {body}");
    }
}
