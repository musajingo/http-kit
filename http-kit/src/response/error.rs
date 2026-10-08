use super::error_type::HttpErrorResponseType;

/// Structured error details for failed API operations.
#[derive(Debug, serde::Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct HttpErrorResponse {
    /// Machine-readable error category.
    #[serde(rename = "type")]
    pub error_type: HttpErrorResponseType,
    /// Human-readable error description.
    pub message: String,
    /// Optional structured error details (e.g., field-level validation errors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl HttpErrorResponse {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::BadRequest,
            message: message.into(),
            details: None,
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::Unauthorized,
            message: message.into(),
            details: None,
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::Forbidden,
            message: message.into(),
            details: None,
        }
    }

    pub fn step_up_required(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::StepUpRequired,
            message: message.into(),
            details: None,
        }
    }

    pub fn method_not_allowed(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::MethodNotAllowed,
            message: message.into(),
            details: None,
        }
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::PayloadTooLarge,
            message: message.into(),
            details: None,
        }
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::ServiceUnavailable,
            message: message.into(),
            details: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::NotFound,
            message: message.into(),
            details: None,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::Conflict,
            message: message.into(),
            details: None,
        }
    }

    pub fn unsupported_media_type(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::UnsupportedMediaType,
            message: message.into(),
            details: None,
        }
    }

    pub fn validation_error(
        message: impl Into<String>,
        details: Option<serde_json::Value>,
    ) -> Self {
        Self {
            error_type: HttpErrorResponseType::ValidationError,
            message: message.into(),
            details,
        }
    }

    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::RateLimited,
            message: message.into(),
            details: None,
        }
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self {
            error_type: HttpErrorResponseType::InternalServerError,
            message: message.into(),
            details: None,
        }
    }
}
