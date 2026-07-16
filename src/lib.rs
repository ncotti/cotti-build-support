#![warn(missing_docs)]

//! # Cotti build support
//!
//! This crate holds several functions focused on filesystem and OS management,
//! useful for installing and building dependencies.

pub mod common;
pub mod install;

pub use common::{find, find_dirs, find_files, rm_rf};
pub use install::{install, untar};
