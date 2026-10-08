/// Errors that can occur during projection parsing and validation.
#[derive(Debug, serde::Serialize, thiserror::Error)]
pub enum ProjectionError {
    /// The input string was empty.
    #[error("field set must not be empty")]
    Empty,

    /// A token was not a recognized field name.
    #[error("unknown field '{field}'")]
    InvalidField {
        field: String,
        valid_fields: Vec<&'static str>,
    },

    /// Every available field of the named resource was excluded.
    #[error(
        "selection for '{}' excludes all {count} available fields",
        if namespace.is_empty() { "top-level" } else { namespace }
    )]
    AllExcluded { namespace: String, count: usize },

    /// The input was not clear about which fields to include or exclude.
    #[error(
        "selection for '{}' is ambiguous: unclear mix of inclusions and exclusions. \
         Use only inclusions, only exclusions, or '{}all' with exclusions.",
        if namespace.is_empty() { "top-level" } else { namespace },
        if namespace.is_empty() { String::new() } else { format!("{namespace}.") }
    )]
    AmbiguousSelection { namespace: String },

    /// The input combined the `all` keyword with explicit inclusions for the same resource.
    #[error(
        "selection for '{}' uses 'all' together with specific inclusions; \
         with 'all' you may only add exclusions",
        if namespace.is_empty() { "top-level" } else { namespace }
    )]
    AllWithInclusions { namespace: String },

    /// The input did not include any fields for the top-level resource.
    #[error("top-level resource selection is required")]
    TopLevelMissing,
}

// ––– ProjectionErrors –––

/// A collection of [`ProjectionError`]s.
#[derive(Debug, serde::Serialize)]
pub struct ProjectionErrors(pub Vec<ProjectionError>);

impl std::fmt::Display for ProjectionErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let messages: Vec<String> = self.0.iter().map(|err| err.to_string()).collect();
        write!(f, "{}", messages.join("; "))
    }
}

impl std::error::Error for ProjectionErrors {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_message() {
        assert_eq!(
            ProjectionError::Empty.to_string(),
            "field set must not be empty",
        );
    }

    #[test]
    fn invalid_field_message_uses_qualified_name() {
        let e = ProjectionError::InvalidField {
            field: "address.foo".to_string(),
            valid_fields: vec!["id", "name"],
        };
        assert_eq!(e.to_string(), "unknown field 'address.foo'");
    }

    #[test]
    fn all_excluded_top_level_message_uses_top_level_label() {
        let e = ProjectionError::AllExcluded {
            namespace: String::new(),
            count: 5,
        };
        let msg = e.to_string();
        assert!(msg.contains("'top-level'"), "got: {msg}");
        assert!(msg.contains("5"), "got: {msg}");
    }

    #[test]
    fn all_excluded_nested_message_uses_namespace() {
        let e = ProjectionError::AllExcluded {
            namespace: "country".into(),
            count: 3,
        };
        let msg = e.to_string();
        assert!(msg.contains("'country'"), "got: {msg}");
    }

    #[test]
    fn ambiguous_top_level_message_qualifies_all_unprefixed() {
        let e = ProjectionError::AmbiguousSelection {
            namespace: String::new(),
        };
        let msg = e.to_string();
        assert!(msg.contains("'top-level'"), "got: {msg}");
        assert!(msg.contains("inclusions"), "got: {msg}");
        assert!(msg.contains("exclusions"), "got: {msg}");
        // The hint should reference plain 'all' (no namespace prefix).
        assert!(
            msg.contains("''all''") || msg.contains("'all'"),
            "got: {msg}"
        );
    }

    #[test]
    fn ambiguous_nested_message_qualifies_all_with_namespace_prefix() {
        let e = ProjectionError::AmbiguousSelection {
            namespace: "country".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("'country'"), "got: {msg}");
        assert!(msg.contains("'country.all'"), "got: {msg}");
    }

    #[test]
    fn all_with_inclusions_message_mentions_all_and_namespace() {
        let e = ProjectionError::AllWithInclusions {
            namespace: "country".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("'country'"), "got: {msg}");
        assert!(msg.contains("'all'"), "got: {msg}");
        assert!(msg.contains("only add exclusions"), "got: {msg}");
    }

    #[test]
    fn all_with_inclusions_top_level_uses_top_level_label() {
        let e = ProjectionError::AllWithInclusions {
            namespace: String::new(),
        };
        assert!(e.to_string().contains("'top-level'"));
    }

    #[test]
    fn top_level_missing_message() {
        assert_eq!(
            ProjectionError::TopLevelMissing.to_string(),
            "top-level resource selection is required",
        );
    }

    #[test]
    fn projection_errors_collection_joins_with_semicolon() {
        let errs = ProjectionErrors(vec![
            ProjectionError::Empty,
            ProjectionError::TopLevelMissing,
        ]);
        let s = errs.to_string();
        assert!(s.contains("must not be empty"), "got: {s}");
        assert!(s.contains("top-level resource selection"), "got: {s}");
        assert!(s.contains("; "), "got: {s}");
    }
}
