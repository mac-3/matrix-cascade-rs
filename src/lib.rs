//! Crate for creating terminal cascades similar to what we see in Matrix.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

mod matrix;
mod strain;
///
#[cfg(feature = "tui")]
pub mod widget;

pub use matrix::Matrix;
