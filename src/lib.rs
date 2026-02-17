#![deny(unused_must_use)]
#![deny(unused_variables)]
#![deny(dead_code)]

//! rust-cef-runtime
//!
//! Minimal Rust runtime for building Chromium-based desktop apps using CEF.
//! Provides a bootstrap API while exposing CEF underneath.

mod runtime;
mod app;
mod browser;
mod window;
mod client;
mod scheme;
mod error;
mod fs_pool;

#[cfg(target_os = "macos")]
mod platform;

pub use runtime::Runtime;
pub use error::RuntimeError;
