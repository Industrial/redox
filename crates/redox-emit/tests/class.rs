//! Doc 18 §2.5's `class` and `instance`.
//!
//! Rust's trait system **is** Haskell's class system, so §2.5 changes the
//! words and leaves the content alone: `class` means `trait`, `instance`
//! means `impl`, and `self` in a type marks the receiver — the one place
//! dictionary-passing and Rust's receiver differ in spelling but not in
//! content.
//!
//! Plan 67 is what made the marker a marker. Before it the receiver was the
//! *first* parameter whose type matched one of up to three spellings of the
//! implementing type, and a `class` spent its one head slot naming that type
//! rather than its own parameters.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

#[test]
fn a_class_becomes_a_trait() {
  let out = xrs("class Shape\n  area : &self -> f64\n");
  assert!(out.contains("trait Shape {"), "{out}");
  assert!(out.contains("    fn area(&self) -> f64;"), "{out}");
}

/// A superclass becomes a supertrait bound, and several join with `+`.
///
/// A trait name alone since plan 67 — a superclass constrains `Self`, and
/// there is no variable left for it to name.
#[test]
fn superclasses_become_supertrait_bounds() {
  let one = xrs("class Debug => Shape\n  area : &self -> f64\n");
  assert!(one.contains("trait Shape: Debug {"), "{one}");
  let two = xrs("class (Debug, Clone) => Shape\n  area : &self -> f64\n");
  assert!(two.contains("trait Shape: Debug + Clone {"), "{two}");
}

/// `class Shape` and `class Debug => Shape` differ only at the `=>`, which is
/// past the first name, so the clause is still read speculatively and
/// abandoned when no `=>` arrives. This is the test that the rewind actually
/// rewinds.
#[test]
fn a_class_without_constraints_keeps_its_name() {
  let out = xrs("class Shape\n  area : &self -> f64\n");
  assert!(out.contains("trait Shape {"), "{out}");
  assert!(!out.contains(": "), "no stray bound:\n{out}");
}

/// Plan 67 leaf 2: the names after the class's own are the **trait's**
/// parameters, which is what makes `trait From<T>` reachable — §1.4.
#[test]
fn a_classs_parameters_become_the_traits_generic_list() {
  let one = xrs("class Convert t\n  wrap : t -> Self\n");
  assert!(one.contains("trait Convert<T> {"), "{one}");
  assert!(one.contains("fn wrap(__rdx_a0: T) -> Self;"), "{one}");
  // Source order, because the emitter's output has to be byte-stable.
  let two = xrs("class Pair t u\n  fst : &self -> t\n  snd : &self -> u\n");
  assert!(two.contains("trait Pair<T, U> {"), "{two}");
}

/// A parameter list and a superclass at once, in Rust's order: the generic
/// list binds to the name and the bound follows the whole head.
#[test]
fn a_parameter_list_and_a_superclass_coexist() {
  let out = xrs("class Debug => Convert t\n  wrap : t -> Self\n");
  assert!(out.contains("trait Convert<T>: Debug {"), "{out}");
}

/// A method with no equation is **required** — the whole difference is the
/// `;` where a body would be.
#[test]
fn a_method_without_equations_is_required() {
  let out = xrs("class Shape\n  area : &self -> f64\n");
  assert!(out.contains("fn area(&self) -> f64;"), "{out}");
  assert!(!out.contains("fn area(&self) -> f64 {"), "{out}");
}

/// A default method is an ordinary equation, so everything a function body
/// can be, it can be — several clauses and guard ladders included.
#[test]
fn a_default_method_carries_its_body() {
  let out = xrs("class Shape\n  name : &self -> u32\n  name self = 1\n");
  assert!(out.contains("fn name(&self) -> u32 {"), "{out}");
  assert!(out.contains("        1\n"), "{out}");
}

#[test]
fn associated_types_and_constants_pass_through() {
  let out = xrs("class Shape\n  type Unit\n  const SIDES : u32\n");
  assert!(out.contains("    type Unit;"), "{out}");
  assert!(out.contains("    const SIDES: u32;"), "{out}");
}

/// A `class` declares the constant and an `instance` supplies it. Redox does
/// not enforce the pairing, because rustc does.
#[test]
fn an_associated_constant_may_carry_a_value() {
  let out = xrs("instance Shape Square\n  const SIDES : u32 = 4\n");
  assert!(out.contains("    const SIDES: u32 = 4;"), "{out}");
}

/// The borrow form comes from the prefix on the marker, so all three
/// receivers are expressible with one notation — plan 67 §9 decision 9 keeps
/// this half of doc 18 §2.5 exactly as it was.
#[test]
fn every_receiver_form_comes_from_the_type() {
  let shared = xrs("class C\n  f : &self -> u32\n");
  assert!(shared.contains("fn f(&self) -> u32;"), "{shared}");
  let unique = xrs("class C\n  f : &mut self -> u32\n");
  assert!(unique.contains("fn f(&mut self) -> u32;"), "{unique}");
  let owned = xrs("class C\n  f : self -> u32\n");
  assert!(owned.contains("fn f(self) -> u32;"), "{owned}");
  // Rust's fourth form, which plan 67's list of three missed and plan 74
  // found porting `redox-lexer`: a by-value receiver the method may mutate.
  // Before the fix this emitted `fn f(__rdx_a0: mut self)` — not Rust, and
  // reported by rustc rather than by Redox.
  let owned_mut = xrs("class C\n  f : mut self -> u32\n");
  assert!(owned_mut.contains("fn f(mut self) -> u32;"), "{owned_mut}");
}

/// No `self` in any of the types means no receiver, which is how an
/// associated function is written — Rust spells that one with no receiver at
/// all, and so does this. `Self` in **return** position is not a marker.
#[test]
fn a_member_with_no_marker_is_an_associated_function() {
  let out = xrs("class C\n  unit : f64 -> Self\n");
  assert!(out.contains("fn unit(__rdx_a0: f64) -> Self;"), "{out}");
  assert!(!out.contains("self"), "{out}");
}

/// Plan 67 §1.2, asserted rather than argued: **position carries no
/// meaning.** A two-argument method written receiver-first and receiver-last
/// emits byte-identical Rust, generated parameter name included — `receiver`
/// numbers `__rdx_aN` by its place in the Rust signature for exactly this.
#[test]
fn a_two_argument_method_emits_the_same_rust_in_either_order() {
  let first = xrs("class C\n  scaled : &self -> f64 -> Self\n");
  let last = xrs("class C\n  scaled : f64 -> &self -> Self\n");
  assert_eq!(first, last, "the two orders are the same declaration");
  assert!(
    first.contains("fn scaled(&self, __rdx_a1: f64) -> Self;"),
    "{first}"
  );
}

/// The same with an equation, where the parameter takes the binder's name and
/// the body is written against it either way.
#[test]
fn a_bound_two_argument_method_is_the_same_in_either_order() {
  let first = xrs("instance C\n  scaled : &self -> f64 -> f64\n  scaled self k = k\n");
  let last = xrs("instance C\n  scaled : f64 -> &self -> f64\n  scaled k self = k\n");
  assert_eq!(first, last, "the two orders are the same declaration");
  assert!(
    first.contains("fn scaled(&self, k: f64) -> f64 {"),
    "{first}"
  );
}

/// Doc 18 §5.3: a `class` parameter elsewhere in a member's signature is the
/// trait's own generic parameter, so it becomes its uppercase name. The
/// implementing type is `Self` and substitutes to itself, which is plan 67
/// §1.6's claim about plan 63's resolver.
#[test]
fn a_class_parameter_elsewhere_becomes_its_rust_name() {
  let out = xrs("class Combine t\n  combine : &self -> t -> t\n");
  assert!(
    out.contains("fn combine(&self, __rdx_a1: T) -> T;"),
    "{out}"
  );
}

#[test]
fn a_trait_instance_becomes_an_impl_for() {
  let out = xrs("instance Shape Square\n  area : &self -> f64\n  area self = 1.0\n");
  assert!(out.contains("impl Shape for Square {"), "{out}");
  assert!(out.contains("fn area(&self) -> f64 {"), "{out}");
}

/// Plan 67 leaf 3: **the last name in the head is the implementing type** and
/// the rest are the class's arguments, so the head reads in the same order as
/// the Rust it becomes.
#[test]
fn the_last_name_in_an_instance_head_is_the_implementing_type() {
  let out = xrs("instance Convert f64 Meters\n  wrap : f64 -> Self\n  wrap n = Meters n\n");
  assert!(out.contains("impl Convert<f64> for Meters {"), "{out}");
}

/// A primitive is lowercase and concrete, so it is **not** uppercased as a
/// type variable would be. Measured as `impl Convert<F64> for Meters` before
/// `TypeApp::is_var` grew `primitive_type`'s correction.
#[test]
fn a_primitive_class_argument_is_not_a_type_variable() {
  let out = xrs("instance Convert f64 Meters\n  wrap : f64 -> Self\n  wrap n = Meters n\n");
  assert!(!out.contains("F64"), "{out}");
  assert!(
    !out.contains("impl<"),
    "no generic list for a primitive:\n{out}"
  );
}

/// No class name, so no `for`.
#[test]
fn an_inherent_instance_has_no_for() {
  let out = xrs("instance Counter\n  get : &self -> u32\n  get self = 1\n");
  assert!(out.contains("impl Counter {"), "{out}");
  assert!(!out.contains(" for "), "{out}");
}

/// The generic list is synthesised from first appearance and the constraints
/// become its bounds — doc 18 §5.3.
#[test]
fn the_generic_list_is_synthesised() {
  let out =
    xrs("instance Display t => Describe (Wrapper t)\n  show : &self -> u32\n  show self = 1\n");
  assert!(
    out.contains("impl<T: Display> Describe for Wrapper<T> {"),
    "{out}"
  );
}

/// Several variables keep source order, which is what makes the output
/// byte-stable.
#[test]
fn several_variables_keep_source_order() {
  let out = xrs("instance Keyed (Pair k v)\n  key : &self -> u32\n  key self = 1\n");
  assert!(out.contains("impl<K, V> Keyed for Pair<K, V> {"), "{out}");
}

/// Parentheses on the *first* type make it the target, which is the inherent
/// impl on a generic type — the one case Haskell has no need for and Rust
/// does.
#[test]
fn a_parenthesised_first_type_is_the_target() {
  let out = xrs("instance (Wrapper t)\n  get : &self -> u32\n  get self = 1\n");
  assert!(out.contains("impl<T> Wrapper<T> {"), "{out}");
  assert!(!out.contains(" for "), "{out}");
}

/// `self` is an expression, because doc 18 §2.5 drops `@name` in favour of
/// `self.name`. It lexes as a reserved word, so it needs naming in both the
/// pattern and the expression grammar — and, since plan 67, in type position
/// as well.
#[test]
fn self_is_an_expression_and_a_binder() {
  let out = xrs("instance C\n  get : &self -> u32\n  get self = self.value\n");
  assert!(out.contains("self.value"), "{out}");
}

/// Plan 67 criterion 9: the marker synthesises **no generic parameter**.
///
/// Doc 18 §5.3 makes a lowercase name a type variable, so `&self` would
/// otherwise give `fn area<SELF>(…)`. The failure mode is a program that
/// still compiles in simple cases, which is why this is asserted directly
/// rather than left to the corpus.
#[test]
fn the_marker_synthesises_no_generic_parameter() {
  let out = xrs("class C\n  area : &self -> f64\n");
  assert!(out.contains("fn area(&self) -> f64;"), "{out}");
  assert!(!out.contains("fn area<"), "no generic list:\n{out}");
}
