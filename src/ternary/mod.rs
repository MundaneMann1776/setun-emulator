//! Balanced ternary number system primitives.
//!
//! This module provides the core types for working with balanced ternary:
//! - [`Trit`] - A single balanced ternary digit (-1, 0, +1)
//! - [`Tryte9`] - A 9-trit word (used for memory cells and instructions)
//! - [`Word18`] - An 18-trit word (used for the accumulator and computation)

mod trit;
mod word;
mod ops;
pub mod arith;

pub use trit::Trit;
pub use word::{Tryte9, Word18};
pub use ops::TritOps;
pub use arith::{add, subtract, multiply, negate};
