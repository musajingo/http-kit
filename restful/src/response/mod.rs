//! API response envelope.
//!
//! [`ApiResponse`] wraps success data or an [`HttpErrorResponse`] with a
//! matching status code, and converts into an Axum response.
//! [`ResponseMetadata`] carries pagination and custom metadata fields.

mod api_response;
mod error;
mod error_type;
mod metadata;
mod reserved_key_error;
mod status;

pub use api_response::ApiResponse;
pub use error::HttpErrorResponse;
pub use error_type::HttpErrorResponseType;
pub use metadata::ResponseMetadata;
pub use reserved_key_error::ReservedKeyError;
pub use status::ApiResponseStatus;
