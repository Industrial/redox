/// How deeply the input may nest brackets before the parser refuses it.
///
/// Plan 42's first find, and the reason this constant exists at all: a
/// recursive-descent parser with no limit **overflows its stack and aborts the
/// process** on deep input. Not a diagnostic and not a catchable panic — an
/// abort, which takes the whole program with it. Plan 40's language server
/// runs the parser on every keystroke, so it is a crash in front of someone
/// who typed one bracket too many.
///
/// # The number is measured, not chosen
///
/// 24 levels of nested parentheses parse and 32 abort, in a debug build. 16
/// sits well below the first failure, leaves headroom for a release build's
/// different frame sizes and for the forms that cost more stack per level than
/// a bare parenthesis, and is far above anything a human writes — the deepest
/// nesting anywhere in the corpus is a handful.
///
/// A limit chosen by taste would have been 64 or 256, either of which is past
/// the cliff. The reason to measure is that the cliff was two orders of
/// magnitude lower than it looks.
pub const MAX_NESTING: usize = 16;
