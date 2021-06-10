//! Data path compilation and evaluation.
//!
//! A data path describes a location in memory using a C-like syntax. It allows
//! pointer dereferencing, array indexing, and struct/union field accesses.
//!
//! There are two types of data paths:
//! - Global: a data path starting from a global variable address
//! - Local: a data path starting from a type, such as a specific struct
//!
//! It is possible to concatenate paths, either global + local or local + local, if the end type
//! of the first path and the start type of the second path match.
//!
//! The syntax mostly follows C syntax, e.g. `globalVariable.arrayField[3].x` for a global
//! path.
//! Local paths are similar, but the global variable is replaced with a type namespace and name,
//! e.g. `struct Foo.x`, `typedef Foo.x`.
//!
//! There are a few differences:
//! - `p.x` automatically dereferences a pointer type `p`
//! - `*` is not used for pointer dereferencing. Instead you can use `[0]`, `->`, or `.`
//! - `?` denotes that a pointer may be null. If so, the entire expression returns `Value::None`.
//!   If `?` is not used, an error is thrown instead.

#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

pub use data_path_types::*;
pub use error::*;

mod compile;
mod data_path_types;
mod error;
