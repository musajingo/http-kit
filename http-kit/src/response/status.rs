/// Status of an API response.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum ApiResponseStatus {
    Ok,
    Error,
}
