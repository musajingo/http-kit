use axum::http::StatusCode;

/// HTTP error types for API responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, strum::Display, strum::AsRefStr)]
#[serde(rename_all = "PascalCase")]
#[strum(serialize_all = "PascalCase")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum HttpErrorResponseType {
    /// Generic bad request (400)
    BadRequest,
    /// Authentication required (401)
    Unauthorized,
    /// Access denied (403)
    Forbidden,
    /// Authenticated, but a fresh login is required for this operation (403)
    StepUpRequired,
    /// Resource not found (404)
    NotFound,
    /// Generic conflict (409)
    Conflict,
    /// The route exists but not for this method (405)
    MethodNotAllowed,
    /// Request body above the configured limit (413)
    PayloadTooLarge,
    /// Unsupported media type (415)
    UnsupportedMediaType,
    /// Validation failed (422)
    ValidationError,
    /// Rate limit exceeded (429)
    RateLimited,
    /// Internal server error (500)
    InternalServerError,
    /// The request could not be served in time or the server is shedding load (503)
    ServiceUnavailable,
}

impl HttpErrorResponseType {
    /// Returns the HTTP status code for this error type.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::StepUpRequired => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::ValidationError => StatusCode::UNPROCESSABLE_ENTITY,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}
