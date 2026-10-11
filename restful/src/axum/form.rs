use axum::extract::{Form, FromRequest, Request, rejection::FormRejection};
use serde::de::DeserializeOwned;
use valid8r::Validate;

use super::rejection::ExtractorRejection;

/// Deserializes and validates form data before the handler runs.
///
/// Like Axum's `Form`: GET requests read the URL query string; other methods
/// read an `application/x-www-form-urlencoded` body. Multipart forms are not
/// supported.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

impl<T> ValidatedForm<T> {
    /// Consumes the extractor and returns the inner value.
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

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::StatusCode,
        response::IntoResponse,
    };
    use serde::Deserialize;
    use serde_json::Value;

    use super::*;

    #[derive(Debug, Deserialize, Validate)]
    struct Payload {
        #[validate(range(min = 1))]
        count: u32,
    }

    async fn assert_error(
        content_type: Option<&str>,
        body: &str,
        expected_status: StatusCode,
        expected_type: &str,
    ) {
        let mut request = Request::builder().method("POST");
        if let Some(content_type) = content_type {
            request = request.header("content-type", content_type);
        }
        let request = request.body(Body::from(body.to_owned())).unwrap();
        let error = ValidatedForm::<Payload>::from_request(request, &())
            .await
            .unwrap_err();
        let response = error.into_response();
        assert_eq!(response.status(), expected_status);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["type"], expected_type);
    }

    #[tokio::test]
    async fn wrong_or_missing_content_type_returns_unsupported_media_type() {
        for content_type in [Some("application/json"), Some("text/plain"), None] {
            assert_error(
                content_type,
                "count=1",
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UnsupportedMediaType",
            )
            .await;
        }
    }

    #[tokio::test]
    async fn malformed_form_returns_bad_request() {
        assert_error(
            Some("application/x-www-form-urlencoded"),
            "count=invalid",
            StatusCode::BAD_REQUEST,
            "BadRequest",
        )
        .await;
    }
}
