use axum::{
    extract::rejection::{FormRejection, JsonRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::{Level, debug, span};
use valid8r::ValidationErrors;

use super::utils;

use crate::response::{ApiResponse, HttpErrorResponse};

/// An extraction or validation failure returned as an API error.
#[derive(Debug, thiserror::Error)]
pub enum ExtractorRejection {
    /// The form could not be extracted, such as an invalid content type or body.
    #[error("Form data rejection: {0}")]
    FormRejection(#[from] FormRejection),

    /// The JSON could not be extracted, such as an invalid content type or body.
    #[error("JSON data rejection: {0}")]
    JsonRejection(#[from] JsonRejection),

    /// The URL query parameters could not be deserialized into the target type.
    #[error("Query string rejection: {0}")]
    QueryRejection(#[from] QueryRejection),

    /// Deserialization succeeded, but the value failed its validation rules.
    #[error("Payload validation failed: {0}")]
    Validation(#[from] ValidationErrors),
}

/// Axum's rejection body text is `"<prefix>: <serde message>"`; the serde part
/// can echo submitted values, so sanitize it and keep the prefix.
fn sanitize_body_text(text: String) -> String {
    match text.split_once(": ") {
        Some((prefix, rest)) => format!("{prefix}: {}", utils::drop_submitted_value(rest)),
        None => text,
    }
}

impl IntoResponse for ExtractorRejection {
    fn into_response(self) -> Response {
        let error_details = match self {
            ExtractorRejection::FormRejection(err) => {
                let _span = span!(Level::DEBUG, "form_rejection_error").entered();
                debug!(message = "Form data rejection", status = %err.status());

                if matches!(err, FormRejection::InvalidFormContentType(_)) {
                    HttpErrorResponse::unsupported_media_type(err.body_text())
                } else if err.status() == StatusCode::PAYLOAD_TOO_LARGE {
                    HttpErrorResponse::payload_too_large("The request body is too large.")
                } else {
                    HttpErrorResponse::bad_request(sanitize_body_text(err.body_text()))
                }
            }
            ExtractorRejection::JsonRejection(err) => {
                let _span = span!(Level::DEBUG, "json_rejection_error").entered();
                debug!(message = "JSON data rejection", status = %err.status());

                // 415 for missing/wrong Content-Type, 413 when the body exceeded
                // the configured limit, 422 with field-level details when a field
                // failed to deserialize, 400 for other JSON errors
                if matches!(err, JsonRejection::MissingJsonContentType(_)) {
                    HttpErrorResponse::unsupported_media_type(err.body_text())
                } else if err.status() == StatusCode::PAYLOAD_TOO_LARGE {
                    HttpErrorResponse::payload_too_large("The request body is too large.")
                } else if let Some(details) = utils::json_data_error_details(&err) {
                    HttpErrorResponse::validation_error("Validation failed", Some(details))
                } else if let Some(message) = utils::json_data_error_message(&err) {
                    // Root-level data error: no field to attribute it to.
                    HttpErrorResponse::bad_request(message)
                } else {
                    HttpErrorResponse::bad_request(err.body_text())
                }
            }
            ExtractorRejection::QueryRejection(err) => {
                let _span = span!(Level::DEBUG, "query_rejection_error").entered();
                debug!(message = "Query string rejection", status = %err.status());

                HttpErrorResponse::bad_request(sanitize_body_text(err.body_text()))
            }
            ExtractorRejection::Validation(validation_errors) => {
                let _span = span!(Level::DEBUG, "payload_validation_error").entered();
                debug!(
                    message = "Validation failed",
                    rejected = ?utils::rejected_fields(&validation_errors)
                );

                HttpErrorResponse::validation_error(
                    "Validation failed",
                    Some(utils::validation_errors_to_json(&validation_errors)),
                )
            }
        };

        ApiResponse::from(error_details).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body;
    use serde::Deserialize;
    use serde_json::{Value, json};
    use valid8r::Validate;

    // Helper to deserialize response body to serde_json::Value
    async fn deserialize_response_to_value(response: Response) -> Value {
        let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        serde_json::from_slice(&body_bytes).expect("Failed to deserialize body to JSON Value")
    }

    #[derive(Debug, Validate, Deserialize)]
    struct TestPayload {
        #[validate(length(min = 3, message = "Name too short"))]
        name: String,
        #[validate(range(min = 18, message = "Must be an adult"))]
        age: u32,
    }

    #[tokio::test]
    async fn test_validation_error_into_response() {
        let invalid_payload = TestPayload {
            name: "Bo".to_string(), // Too short
            age: 17,                // Too young
        };

        let validation_errors = invalid_payload.validate().unwrap_err();
        let error = ExtractorRejection::from(validation_errors);

        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let api_response_json = deserialize_response_to_value(response).await;

        assert_eq!(api_response_json["error"]["type"], "ValidationError");
        assert_eq!(api_response_json["error"]["message"], "Validation failed");

        let details = &api_response_json["error"]["details"];
        assert_eq!(details["name"][0]["code"], "length");
        assert_eq!(details["name"][0]["message"], "Name too short");
        assert_eq!(details["age"][0]["code"], "range");
        assert_eq!(details["age"][0]["message"], "Must be an adult");
    }

    #[tokio::test]
    async fn test_multiple_validation_errors_into_response() {
        let invalid_payload = TestPayload {
            name: "".to_string(), // Too short
            age: 10,              // Too young
        };

        let validation_errors = invalid_payload.validate().unwrap_err();
        let error = ExtractorRejection::from(validation_errors);

        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let api_response_json = deserialize_response_to_value(response).await;

        assert_eq!(api_response_json["error"]["type"], "ValidationError");

        let details = &api_response_json["error"]["details"];
        assert_eq!(details["name"][0]["code"], "length");
        assert_eq!(details["name"][0]["message"], "Name too short");
        assert_eq!(details["age"][0]["code"], "range");
        assert_eq!(details["age"][0]["message"], "Must be an adult");
    }

    #[derive(Debug, Validate, Deserialize)]
    struct CredentialPayload {
        #[validate(length(min = 8, message = "password too short"))]
        #[validate(sensitive)]
        password: String,
    }

    #[tokio::test]
    async fn sensitive_field_value_never_reaches_the_response() {
        let payload = CredentialPayload {
            password: "hunter7".to_string(),
        };

        let validation_errors = payload.validate().unwrap_err();
        assert_eq!(
            utils::rejected_fields(&validation_errors),
            vec!["password:length"]
        );

        let response = ExtractorRejection::from(validation_errors).into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body = deserialize_response_to_value(response).await;
        let details = &body["error"]["details"]["password"][0];
        assert_eq!(details["code"], "length");
        assert_eq!(details["params"]["min"], 8);
        assert!(details["params"].get("value").is_none());
        assert!(!body.to_string().contains("hunter7"));
    }

    #[tokio::test]
    async fn test_empty_validation_errors_into_response() {
        let validation_errors = ValidationErrors::new();
        let error = ExtractorRejection::from(validation_errors);

        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let api_response_json = deserialize_response_to_value(response).await;

        assert_eq!(api_response_json["error"]["details"], json!({}));
    }
}

#[cfg(test)]
mod redaction_tests {
    use axum::body::{Body, to_bytes};
    use axum::extract::{FromRequest, FromRequestParts, Request};
    use serde::Deserialize;
    use valid8r::Validate;

    use super::*;
    use crate::axum::{ValidatedForm, ValidatedJson, ValidatedQuery};

    #[derive(Debug, Deserialize, Validate)]
    struct Payload {
        #[allow(dead_code)]
        kind: Kind,
    }

    #[derive(Debug, Deserialize)]
    enum Kind {
        #[serde(rename = "good")]
        #[allow(dead_code)]
        Good,
    }

    async fn body_of(response: Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn root_json_error_never_echoes_the_body() {
        let req = Request::builder()
            .header("content-type", "application/json")
            .body(Body::from("123456"))
            .unwrap();
        let err = ValidatedJson::<Payload>::from_request(req, &())
            .await
            .unwrap_err();
        let body = body_of(err.into_response()).await;
        assert!(!body.contains("123456"), "leaked: {body}");
        assert!(body.contains("expected"), "lost the hint: {body}");
    }

    #[tokio::test]
    async fn form_error_never_echoes_the_value() {
        let req = Request::builder()
            .method("POST")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("kind=secret-input"))
            .unwrap();
        let err = ValidatedForm::<Payload>::from_request(req, &())
            .await
            .unwrap_err();
        let body = body_of(err.into_response()).await;
        assert!(!body.contains("secret-input"), "leaked: {body}");
    }

    #[tokio::test]
    async fn query_error_never_echoes_the_value() {
        let req = Request::builder()
            .uri("/?kind=secret-input")
            .body(Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let err = ValidatedQuery::<Payload>::from_request_parts(&mut parts, &())
            .await
            .unwrap_err();
        let body = body_of(err.into_response()).await;
        assert!(!body.contains("secret-input"), "leaked: {body}");
    }
}
