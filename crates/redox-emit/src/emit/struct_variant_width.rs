/// rustfmt's `struct_variant_width` default.
///
/// An enum's record variant stays on one line while its body is no wider than
/// this, and is expanded one field per line beyond it. The number is
/// rustfmt's, not Redox's, and it is here because plan 04's guarantee is that
/// the emitter never writes a line rustfmt would re-break — which requires
/// knowing where rustfmt breaks.
///
/// Pinned rather than discovered: reading it from a rustfmt invocation would
/// make the output depend on an external tool at *emission* time, which
/// `docs/COMPILER.md` §5 forbids for exactly the reasons that matter here —
/// determinism and the span map. If a future rustfmt changes the default,
/// `generated_rust_is_rustfmt_clean` fails and names this constant.
pub(crate) const STRUCT_VARIANT_WIDTH: usize = 35;
