//! Crate for creating terminal cascades similar to what we see in Matrix.

#![deny(missing_debug_implementations)]

mod matrix;
mod strain;
/// Widget module for easy integration with the [tui-rs](<https://github.com/fdehau/tui-rs>) crate.
#[cfg(feature = "tui")]
pub mod widget;

pub use matrix::Matrix;
