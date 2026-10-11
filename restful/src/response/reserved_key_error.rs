/// Error from [`ResponseMetadata::set`](crate::response::ResponseMetadata::set):
/// the key collides with a built-in field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservedKeyError;

impl std::fmt::Display for ReservedKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("metadata key 'pagination' is reserved")
    }
}

impl std::error::Error for ReservedKeyError {}
