//! Sort direction utilities for list query operations.

/// Direction for data lists.
///
/// Serializes to lowercase (`"asc"`, `"desc"`).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum OrderByDirection {
    /// Ascending order (A-Z, 0-9, oldest first). This is the default.
    #[default]
    Asc,
    /// Descending order (Z-A, 9-0, newest first).
    Desc,
}

/// Convert to `pgkit`'s `OrderByDirection`.
#[cfg(feature = "with-pgkit")]
impl From<OrderByDirection> for ::pgkit::ordering::OrderByDirection {
    fn from(d: OrderByDirection) -> Self {
        match d {
            OrderByDirection::Asc => Self::Asc,
            OrderByDirection::Desc => Self::Desc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_asc() {
        assert_eq!(OrderByDirection::default(), OrderByDirection::Asc);
    }
}
