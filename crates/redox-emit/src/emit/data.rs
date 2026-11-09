use crate::emit::fields::emit_fields;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::{ConstructorFields, DataDecl};

/// Emit doc 18 §2.4's `data` declaration as a `struct` or an `enum`.
///
/// **One constructor whose name matches the type is a `struct`; anything else
/// is an `enum`.** The rule is syntactic — decidable from the source text
/// alone, doc 03 §2's admission test 1 — which is why this needs no name
/// resolution and why doc 18 §2.4 calls it "the whole requirement".
///
/// ```text
/// data Point = Point { x :: f64 }   struct Point { x: f64 }
/// data Marker = Marker              struct Marker;
/// newtype Meters = Meters f64       struct Meters(f64);
/// data Event = Key char | Close     enum Event { Key(char), Close }
/// ```
///
/// A single constructor whose name *differs* from the type is an `enum` of one
/// variant, and that is correct rather than a gap: `data Shape = Circle f64`
/// introduces `Circle`, and a struct named `Point` would not.
pub(crate) fn emit_data(out: &mut Sink<'_>, d: &DataDecl, indent: usize) {
  out.mark(d.span);
  emit_trivia(out, &d.leading, indent);

  if !d.derives.is_empty() {
    let names: Vec<&str> = d.derives.iter().map(|i| i.text.as_str()).collect();
    pad(out, indent);
    out.push_str(&format!("#[derive({})]\n", names.join(", ")));
  }

  let as_struct = match d.constructors.as_slice() {
    [only] => only.name.text == d.name.text,
    // No constructors at all is `data Marker` with no `=`, which is the unit
    // struct. Doc 18 §2.4 writes it `data Marker = Marker`; both reach here.
    [] => true,
    _ => false,
  };

  if as_struct {
    let fields = d
      .constructors
      .first()
      .map_or(&ConstructorFields::Unit, |c| &c.fields);
    pad(out, indent);
    // Plan 45 leaf 4: `union` takes the `struct` slot. Everything after the
    // keyword — the fields, the generics, the derives — is a struct's, which
    // is the whole argument for it riding on `data` rather than being a
    // declaration of its own.
    out.push_str(&format!(
      "{}{} {}",
      d.visibility.rust_text(),
      if d.is_union { "union" } else { "struct" },
      d.name.text
    ));
    emit_fields(out, fields, indent, true);
    return;
  }

  pad(out, indent);
  out.push_str(&format!(
    "{}enum {} {{\n",
    d.visibility.rust_text(),
    d.name.text
  ));
  for c in &d.constructors {
    out.mark(c.span);
    pad(out, indent + RUST_INDENT);
    out.push_str(&c.name.text);
    emit_fields(out, &c.fields, indent + RUST_INDENT, false);
  }
  out.unmark();
  pad(out, indent);
  out.push_str("}\n");

  // **A constructor is in scope under its own name**, as it is in Haskell —
  // so `Close` and `Key 'k'` are written without the type, and a `case` arm
  // spelled `Close ->` *matches the variant*.
  //
  // This is a fourth new lowering, beyond the three doc 18 §6 discloses, and
  // it is not optional sugar. Without it `Close` is an ordinary name: as an
  // expression it fails to resolve, and as a `case` pattern it silently
  // becomes an irrefutable binding, so `describe` would return its first arm
  // for every input. That is doc 02 §5.2's worst failure mode exactly —
  // output that reads as Rust and means something else — reached by writing
  // doc 18 §2.4's own example.
  //
  // Two names colliding is rustc's error to give, which is the same division
  // of labour Haskell has: both languages reject the collision, neither
  // resolves it.
  pad(out, indent);
  out.push_str(&format!("use {}::*;\n", d.name.text));
}
