use crate::parse::parser::rewritten::rewritten;
use crate::parse::parser::rust_generic::rust_generic;
use crate::parse::parser::type_vars::type_vars;
use redox_ast::{ConstructorFields, DataDecl};

/// Fill in a `data` declaration's generic list, and rewrite its field types.
///
/// ```text
/// data Cursor = Cursor { src : &'a str }    struct Cursor<'a> { src: &'a str }
/// data Holder = Holder { item : t }      →  struct Holder<T> { item: T }
/// data Point = Point { x : f64 }            struct Point { x: f64 }
/// ```
///
/// Doc 18 §5.3: **"type variables need no declaration … the generic list is
/// the order of their first appearance, lifetimes included"**. This is the
/// same rule [`super::generalise::generalise`] applies to a signature and
/// [`redox_ast::TypeApp::vars`] applies to an `instance` target — and plan 74
/// found it applied to neither a `data` nor a `newtype`, so a field type
/// mentioning `'a` emitted a struct with an undeclared lifetime. rustc catches
/// that, which is the only reason it was not a silent miscompile.
///
/// Two steps, in this order because the second needs the first:
///
/// 1. **the variables**, from every constructor's field types in source
///    order, with lifetimes before type parameters because rustc requires
///    that order and `type_vars` already guarantees it;
/// 2. **the rewrite**, replacing each variable with its Rust spelling
///    throughout those types — `t` becomes `T`, and a lifetime is left alone
///    because `rust_generic` says so.
///
/// A declaration with no variables is left exactly as it was, which is what
/// keeps `data Point = Point { x : f64 }` emitting the struct it always did.
///
/// There is no constraint list to turn into bounds. `data` has no `=>` clause
/// — doc 18 §2.4 gives it `deriving` and nothing else — so a bound on a
/// declaration's parameter is not expressible, and rustc is the right place
/// for that to be reported if a field needs one.
pub(crate) fn generalise_data(d: &mut DataDecl) {
  let mut texts: Vec<String> = Vec::new();
  for c in &d.constructors {
    match &c.fields {
      ConstructorFields::Unit => {}
      ConstructorFields::Tuple(ts) => texts.extend(ts.iter().map(|t| t.text.clone())),
      ConstructorFields::Record(fs) => texts.extend(fs.iter().map(|f| f.ty.text.clone())),
    }
  }

  let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
  let found = type_vars(&refs);
  if found.is_empty() {
    return;
  }

  d.generics = found.iter().map(|v| rust_generic(v)).collect();

  for c in &mut d.constructors {
    let types: Vec<&mut String> = match &mut c.fields {
      ConstructorFields::Unit => Vec::new(),
      ConstructorFields::Tuple(ts) => ts.iter_mut().map(|t| &mut t.text).collect(),
      ConstructorFields::Record(fs) => fs.iter_mut().map(|f| &mut f.ty.text).collect(),
    };
    for text in types {
      for v in &found {
        *text = rewritten(text, v, &rust_generic(v));
      }
    }
  }
}
