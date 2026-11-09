/// Parse arbitrary bytes and, if that succeeds, emit.
///
/// The emitter cannot be fuzzed directly — it takes a `Program`, and
/// constructing an arbitrary one would need `arbitrary` derived over the whole
/// AST, which plan 42 §6 declines as most of a second parser. So the input is
/// bytes and the parser is the generator: whatever it accepts is a tree the
/// emitter must survive.
///
/// That is weaker than generating trees and it is not nothing: the parser
/// accepts a great deal more than the corpus contains, and every tree it
/// produces is one a real program could produce.
pub fn emit_program(data: &[u8]) {
  let text = String::from_utf8_lossy(data);
  let Ok(program) = redox_parser::parse_source(&text) else {
    return;
  };
  let _ = redox_emit::emit(&program);
}
