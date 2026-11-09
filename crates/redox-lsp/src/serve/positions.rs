//! Byte offsets as LSP positions.
//!
//! A grouping module (doc 06 §1.2): the two conversions are peers with no
//! natural head.
//!
//! # The protocol counts UTF-16 code units, and this is where that is handled
//!
//! LSP positions are `(line, character)` where `character` is an offset in
//! **UTF-16 code units** by default. Redox's spans are byte offsets. For ASCII
//! the three agree and for anything else they do not, so a server that
//! subtracted byte offsets would mis-place every diagnostic on a line
//! containing an accent — and place it correctly on every line a hand-written
//! test would think to write, which is what makes this worth doing right the
//! first time rather than after a bug report.
//!
//! `examples/strings.sdx` has non-ASCII content, so the corpus does reach it.

pub(crate) mod position_of;
pub(crate) mod range_of;
