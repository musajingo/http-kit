//! Expansion logic for `#[derive(PartialResponse)]`.
//!
//! The only public surface of this module is the [`expand`] function, which is
//! called directly by the proc-macro entry point in `lib.rs`. Everything else
//! is private implementation detail.

mod derive;

pub(crate) use derive::expand;
