use std::collections::HashSet;
use std::hash::Hash;

/// Per-namespace (resource) state during parsing and validation.
pub(crate) struct ParsingState<T: Hash + Eq> {
    /// `<namespace>.all` (or bare `all` for top-level) was specified.
    pub has_all: bool,
    /// Explicit includes for this namespace.
    pub include: HashSet<T>,
    /// Explicitly excluded with `-`.
    pub exclude: HashSet<String>,
}

impl<T: Hash + Eq> Default for ParsingState<T> {
    fn default() -> Self {
        Self {
            has_all: false,
            include: HashSet::new(),
            exclude: HashSet::new(),
        }
    }
}

/// Split a wire name into `(namespace, leaf)`. Top-level names return `("", name)`.
pub(crate) fn split_ns(name: &str) -> (&str, &str) {
    match name.split_once('.') {
        Some((ns, leaf)) => (ns, leaf),
        None => ("", name),
    }
}

/// Return `name` qualified with `resource` if it has no namespace prefix yet.
pub(crate) fn qualify(resource: &str, name: &str) -> String {
    if name.contains('.') {
        name.to_string()
    } else {
        format!("{resource}.{name}")
    }
}
