//! PATCH semantics for nullable fields.
//!
//! The type lives in the standalone [`delta`](https://github.com/musajingo/delta)
//! crate so non-HTTP consumers can use it without pulling in http-kit.
//! Re-exported here so HTTP-layer code can reach it as
//! `http_kit::patch_request::Delta`.

pub use ::delta::Delta;
