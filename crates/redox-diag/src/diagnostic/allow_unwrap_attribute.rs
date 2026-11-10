/// `RDX1001`'s suppression attribute, as the lexer stores it — plan 53 leaf 4.
///
/// The source spelling is `#[redox::allow(unwrap)]`; an attribute trivium
/// carries the text *between* the brackets, which is this.
///
/// # Why one string, in this crate
///
/// Two crates have to agree on it and neither can see the other. The driver
/// recognises it to **suppress** the warning; the emitter recognises it to
/// **drop** it, because plan 53 §11.6 measured what reaching rustc costs:
/// `error[E0433]: cannot find module or crate 'redox'` — a hard error, not an
/// unknown lint. Two spellings drifting apart therefore breaks every program
/// that uses the pragma, so there is one spelling and `redox-diag` holds it:
/// it is the only crate both the driver and the emitter depend on directly,
/// and the suppression spelling is part of what [`Code::UnwrapUsed`]
/// publishes.
///
/// # Exactly this text, after a trim and nothing else
///
/// `#[redox :: allow( unwrap )]` is **not** accepted. A spaced `::` is a
/// diagnostic everywhere else in the language — plan 52's
/// [`Code::SpacedPathSeparator`] — so there is no admitted spelling to be
/// generous towards, and a normaliser would be a second thing the emitter and
/// the driver must agree on rather than a first. An unrecognised spelling
/// fails **loudly**: it is not consumed, so it reaches rustc and `E0433`
/// names it on the author's own line.
///
/// [`Code::UnwrapUsed`]: crate::Code::UnwrapUsed
/// [`Code::SpacedPathSeparator`]: crate::Code::SpacedPathSeparator
pub const ALLOW_UNWRAP_ATTRIBUTE: &str = "redox::allow(unwrap)";
