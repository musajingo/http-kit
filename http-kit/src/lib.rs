#[cfg(feature = "ordering")]
pub mod ordering;

#[cfg(feature = "patch-request")]
pub mod patch_request;

#[cfg(any(feature = "offset-pagination", feature = "cursor-pagination"))]
pub mod pagination;

#[cfg(feature = "response")]
pub mod response;

#[cfg(feature = "projection")]
pub mod projection;

/// Validated Axum request extractors.
#[cfg(feature = "axum")]
pub mod axum;
