//! Compatibility wrapper for paired benchmarking statistics.
//!
//! The implementation now lives in `perfgate-domain`; this crate preserves the
//! existing `perfgate_paired` public import path for downstream users.

pub use perfgate_domain::paired::*;
