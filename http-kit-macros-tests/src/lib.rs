//! Executable documentation examples. This private crate supplies the runtime
//! and integration dependencies needed by the macro examples.

#[doc = include_str!("../../http-kit/README.md")]
pub mod runtime_readme {}

#[doc = include_str!("../../http-kit-macros/README.md")]
pub mod macros_readme {}

#[doc = include_str!("../../http-kit-macros/src/docs/offset_pagination.md")]
pub mod offset_pagination {}

#[doc = include_str!("../../http-kit-macros/src/docs/cursor_pagination.md")]
pub mod cursor_pagination {}

#[doc = include_str!("../../http-kit-macros/src/docs/order_by.md")]
pub mod order_by {}

#[doc = include_str!("../../http-kit-macros/src/docs/projection.md")]
pub mod projection {}

#[doc = include_str!("../../http-kit-macros/src/docs/projection_field.md")]
pub mod projection_field {}

#[doc = include_str!("../../http-kit-macros/src/docs/partial_response.md")]
pub mod partial_response {}
