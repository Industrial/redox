//! An ordinary Rust entry point over Redox-generated code.
//!
//! The `include!` is the whole mechanism. `build.rs` wrote
//! `$OUT_DIR/greeting.rs` from `src/greeting.sdx`, and this pulls it in — so
//! `greet` is an ordinary Rust function and the rest of the crate cannot tell
//! the difference.
//!
//! `include!` is what makes doc 02 §6.6 option (a) work rather than merely
//! compile: it keeps the path to the generated file, so rustc's spans point
//! into `$OUT_DIR/greeting.rs` and plan 06's map can carry them back to
//! `src/greeting.sdx`.
//!
//! **The module shape, which plan 09 could not have.** Plan 09 recorded that
//! this `include!` had to sit at the crate root: Redox emitted `fn greet(…)`
//! with no visibility, so inside `mod greeting { … }` every item was private
//! to that module and unreachable from here. Doc 03 §9.6 specified the
//! spelling that fixes it, **plan 24** implemented it — `export` lowers to
//! `pub` — and **plan 14** gave the module a declaration to be. The note plan
//! 09 left here named the blocking plan; this is what it looks like resolved.
//!
//! Plan 09 found it by building this crate, which is the argument for a worked
//! project over a unit test: nothing else would have noticed that Redox could
//! not produce a reachable item inside a module.
//!
//! A crate can hold both kinds of source. This file is plain Rust.

mod greeting {
  include!(concat!(env!("OUT_DIR"), "/greeting.rs"));
}

fn main() {
  println!("{}", greeting::greet("world"));
  println!("{}", greeting::twice(21));
}
