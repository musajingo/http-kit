#[cfg(feature = "response")]
use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
#[cfg(feature = "response")]
use serde_json::error::Error as SerdeError;

use super::{HttpErrorResponse, ResponseMetadata};

/// Standardized API response envelope.
///
/// Wraps all API responses in a consistent structure. Use the constructor methods
/// ([`ApiResponse::ok`], [`ApiResponse::created`], etc.) rather than building manually.
#[derive(Debug, serde::Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ApiResponse {
    #[serde(skip)]
    code: StatusCode,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<HttpErrorResponse>,

    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<ResponseMetadata>,
}

#[cfg(feature = "response")]
impl ApiResponse {
    pub fn ok<D: Serialize>(data: D, meta: Option<ResponseMetadata>) -> Result<Self, SerdeError> {
        let json_data = serde_json::to_value(data)?;
        Ok(Self {
            code: StatusCode::OK,
            data: Some(json_data),
            error: None,
            meta,
        })
    }

    pub fn created<D: Serialize>(data: D) -> Result<Self, SerdeError> {
        let json_data = serde_json::to_value(data)?;
        Ok(Self {
            code: StatusCode::CREATED,
            data: Some(json_data),
            error: None,
            meta: None,
        })
    }

    pub fn accepted<D: Serialize>(data: D) -> Result<Self, SerdeError> {
        let json_data = serde_json::to_value(data)?;
        Ok(Self {
            code: StatusCode::ACCEPTED,
            data: Some(json_data),
            error: None,
            meta: None,
        })
    }

    pub fn no_content() -> Self {
        Self {
            code: StatusCode::NO_CONTENT,
            data: None,
            error: None,
            meta: None,
        }
    }

    pub fn bad_request(message: String) -> Self {
        HttpErrorResponse::bad_request(message).into()
    }

    pub fn unauthorized(message: String) -> Self {
        HttpErrorResponse::unauthorized(message).into()
    }

    pub fn forbidden(message: String) -> Self {
        HttpErrorResponse::forbidden(message).into()
    }

    pub fn not_found(message: String) -> Self {
        HttpErrorResponse::not_found(message).into()
    }

    pub fn conflict(message: String) -> Self {
        HttpErrorResponse::conflict(message).into()
    }

    pub fn unprocessable_entity(message: String) -> Self {
        HttpErrorResponse::validation_error(message, None).into()
    }

    pub fn rate_limited(message: String) -> Self {
        HttpErrorResponse::rate_limited(message).into()
    }

    pub fn internal_server_error(message: String) -> Self {
        HttpErrorResponse::internal_server_error(message).into()
    }

    pub fn code(&self) -> StatusCode {
        self.code
    }
}

#[cfg(feature = "response")]
impl From<HttpErrorResponse> for ApiResponse {
    fn from(error: HttpErrorResponse) -> Self {
        Self {
            code: error.error_type.status_code(),
            error: Some(error),
            data: None,
            meta: None,
        }
    }
}

#[cfg(feature = "response")]
impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        let status_code = self.code();

        if status_code == StatusCode::NO_CONTENT {
            return StatusCode::NO_CONTENT.into_response();
        }

        let body = serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string());
        let headers = [(header::CONTENT_TYPE, "application/json")];
        (status_code, headers, body).into_response()
    }
}

#[cfg(all(test, feature = "response"))]
mod tests {
    use super::*;
    use crate::{
        pagination::cursor::CursorPaginationMetadata, pagination::offset::OffsetPaginationMetadata,
        response::HttpErrorResponseType,
    };
    use axum::body;
    use serde::Serialize;
    use serde_json::json;

    async fn deserialize_response_to_value(response: Response) -> serde_json::Value {
        let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        serde_json::from_slice(&body_bytes).expect("Failed to deserialize body to JSON Value")
    }

    #[test]
    fn test_api_response_ok() {
        #[derive(Serialize)]
        struct MyData {
            field: String,
        }
        let meta = ResponseMetadata::new(OffsetPaginationMetadata::new(10, 1, 5));
        let response = ApiResponse::ok(
            MyData {
                field: "test".to_string(),
            },
            Some(meta),
        )
        .unwrap();

        assert_eq!(response.code(), StatusCode::OK);
        assert!(response.error.is_none());
        assert_eq!(response.data, Some(json!({"field": "test"})));
        assert!(response.meta.is_some());
    }

    #[test]
    fn test_api_response_created() {
        #[derive(Serialize)]
        struct MyData {
            id: u32,
        }
        let response = ApiResponse::created(MyData { id: 1 }).unwrap();

        assert_eq!(response.code(), StatusCode::CREATED);
        assert!(response.error.is_none());
        assert_eq!(response.data, Some(json!({"id": 1})));
        assert!(response.meta.is_none());
    }

    #[test]
    fn test_api_response_no_content() {
        let response = ApiResponse::no_content();

        assert_eq!(response.code(), StatusCode::NO_CONTENT);
        assert!(response.error.is_none());
        assert!(response.data.is_none());
        assert!(response.meta.is_none());
    }

    #[test]
    fn test_api_response_bad_request() {
        let response = ApiResponse::bad_request("Invalid input".to_string());

        assert_eq!(response.code(), StatusCode::BAD_REQUEST);
        assert!(response.data.is_none());
        let err = response.error.unwrap();
        assert_eq!(err.error_type, HttpErrorResponseType::BadRequest);
        assert_eq!(err.message, "Invalid input");
        assert!(err.details.is_none());
    }

    #[test]
    fn test_api_response_not_found() {
        let response = ApiResponse::not_found("Resource ID 123 not found".to_string());

        assert_eq!(response.code(), StatusCode::NOT_FOUND);
        assert!(response.data.is_none());
        let err = response.error.unwrap();
        assert_eq!(err.error_type, HttpErrorResponseType::NotFound);
    }

    #[tokio::test]
    async fn test_into_response_success() {
        #[derive(Serialize)]
        struct MyData {
            value: String,
        }
        let api_resp = ApiResponse::ok(
            MyData {
                value: "hello".to_string(),
            },
            None,
        )
        .unwrap();
        let http_response = api_resp.into_response();

        assert_eq!(http_response.status(), StatusCode::OK);
        assert_eq!(
            http_response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body_json = deserialize_response_to_value(http_response).await;
        assert_eq!(body_json["data"]["value"], "hello");
        assert!(body_json.get("code").is_none() || body_json["code"].is_null());
    }

    #[tokio::test]
    async fn test_into_response_error() {
        let api_resp = ApiResponse::not_found("Item missing".to_string());
        let http_response = api_resp.into_response();

        assert_eq!(http_response.status(), StatusCode::NOT_FOUND);
        let body_json = deserialize_response_to_value(http_response).await;
        assert_eq!(body_json["error"]["type"], "NotFound");
        assert_eq!(body_json["error"]["message"], "Item missing");
        assert!(body_json["data"].is_null());
    }

    #[tokio::test]
    async fn test_no_content_has_empty_body() {
        let http_response = ApiResponse::no_content().into_response();

        assert_eq!(http_response.status(), StatusCode::NO_CONTENT);
        assert!(http_response.headers().get(header::CONTENT_TYPE).is_none());

        let body_bytes = body::to_bytes(http_response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        assert!(body_bytes.is_empty());
    }

    #[tokio::test]
    async fn test_meta_serializes_nested_pagination() {
        let cursor = CursorPaginationMetadata {
            has_more: true,
            next_cursor: Some("abc123".to_string()),
            limit: 20,
        };
        let meta = ResponseMetadata::new(cursor);
        let api_resp = ApiResponse::ok(vec!["item1", "item2"], Some(meta)).unwrap();
        let body_json = deserialize_response_to_value(api_resp.into_response()).await;

        assert_eq!(body_json["meta"]["pagination"]["hasMore"], true);
        assert_eq!(body_json["meta"]["pagination"]["nextCursor"], "abc123");
        assert_eq!(body_json["meta"]["pagination"]["limit"], 20);
    }
}
