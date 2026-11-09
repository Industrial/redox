//! Compiling to LLVM IR, and reading one function out of it.
//!
//! A grouping module (doc 06 §1.2): finding rustc and reading its output are
//! peers with no natural head.

pub(crate) mod function_ir;
pub(crate) mod rustc_path;
