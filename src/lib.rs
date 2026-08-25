//! Audio translation core contracts.
//!
//! This crate intentionally contains the stable domain models and provider
//! interfaces only. Network acquisition, media decoding, persistence,
//! scheduling, playback, and model loading belong to adapters outside this
//! core module.

pub mod core;

pub use core::*;
