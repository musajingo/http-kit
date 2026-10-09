use axum::{
    extract::rejection::{FormRejection, JsonRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::{Level, debug, span};
use validator::ValidationErrors;

use super::utils;

use crate::response::{ApiResponse, HttpErrorResponse};

/// An extraction or validation failure returned as an API error.
#[derive(Debug)]
pub enum ExtractorRejection {
    FormRejection(FormRejection),

    JsonRejection(JsonRejection),

    QueryRejection(QueryRejection),

    Validation(ValidationErrors),
}

impl std::error::Error for ExtractorRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExtractorRejection::FormRejection(e) => Some(e),
            ExtractorRejection::JsonRejection(e) => Some(e),
            ExtractorRejection::QueryRejection(e) => Some(e),
            ExtractorRejection::Validation(e) => Some(e),
        }
    }
}

impl std::fmt::Display for ExtractorRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractorRejection::FormRejection(err) => write!(f, "Form data rejection: {}", err),
            ExtractorRejection::JsonRejection(err) => write!(f, "JSON data rejection: {}", err),
            ExtractorRejection::QueryRejection(err) => write!(f, "Query string rejection: {}", err),
            ExtractorRejection::Validation(err) => write!(f, "Payload validation failed: {}", err),
        }
    }
}

impl From<ValidationErrors> for ExtractorRejection {
    fn from(e: ValidationErrors) -> Self {
        ExtractorRejection::Validation(e)
    }
}

impl From<FormRejection> for ExtractorRejection {
    fn from(e: FormRejection) -> Self {
        ExtractorRejection::FormRejection(e)
    }
}

impl From<JsonRejection> for ExtractorRejection {
    fn from(e: JsonRejection) -> Self {
        ExtractorRejection::JsonRejection(e)
    }
}

impl From<QueryRejection> for ExtractorRejection {
    fn from(e: QueryRejection) -> Self {
        ExtractorRejection::QueryRejection(e)
    }
}

impl IntoResponse for ExtractorRejection {
    fn into_response(self) -> Response {
        let error_details = match self {
            ExtractorRejection::FormRejection(err) => {
                let _span = span!(Level::DEBUG, "form_rejection_error").entered();
                debug!(message = "Form data rejection", error = %err);

                if matches!(err, FormRejection::InvalidFormContentType(_)) {
                    HttpErrorResponse::unsupported_media_type(err.body_text())
                } else if err.status() == StatusCode::PAYLOAD_TOO_LARGE {
                    HttpErrorResponse::payload_too_large("The request body is too large.")
                } else {
                    HttpErrorResponse::bad_request(err.body_text())
                }
            }
            ExtractorRejection::JsonRejection(err) => {
                let _span = span!(Level::DEBUG, "json_rejection_error").entered();
                debug!(message = "JSON data rejection", error = %err);

                // 415 for missing/wrong Content-Type, 413 when the body exceeded
                // the configured limit, 422 with field-level details when a field
                // failed to deserialize, 400 for other JSON errors
                if matches!(err, JsonRejection::MissingJsonContentType(_)) {
                    HttpErrorResponse::unsupported_media_type(err.body_text())
                } else if err.status() == StatusCode::PAYLOAD_TOO_LARGE {
                    HttpErrorResponse::payload_too_large("The request body is too large.")
                } else if let Some(details) = utils::json_data_error_details(&err) {
                    HttpErrorResponse::validation_error("Validation failed", Some(details))
                } else {
                    HttpErrorResponse::bad_request(err.body_text())
                }
            }
            ExtractorRejection::QueryRejection(err) => {
                let _span = span!(Level::DEBUG, "query_rejection_error").entered();
                debug!(message = "Query string rejection", error = %err);

                HttpErrorResponse::bad_request(err.body_text())
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
    use validator::Validate;

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
