//! Doc 18 §2.5's `class` and `instance`.
//!
//! Rust's trait system **is** Haskell's class system, so §2.5 changes the
//! words and leaves the content alone: `class` means `trait`, `instance`
//! means `impl`, and the first parameter typed by the class variable becomes
//! the receiver — the one place dictionary-passing and Rust's receiver differ
//! in spelling but not in content.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

#[test]
fn a_class_becomes_a_trait() {
  let out = xrs("class Shape t\n  area : &t -> f64\n");
  assert!(out.contains("trait Shape {"), "{out}");
  assert!(out.contains("    fn area(&self) -> f64;"), "{out}");
}

/// A superclass becomes a supertrait bound, and several join with `+`.
#[test]
fn superclasses_become_supertrait_bounds() {
  let one = xrs("class Debug t => Shape t\n  area : &t -> f64\n");
  assert!(one.contains("trait Shape: Debug {"), "{one}");
  let two = xrs("class (Debug t, Clone t) => Shape t\n  area : &t -> f64\n");
  assert!(two.contains("trait Shape: Debug + Clone {"), "{two}");
}

/// `class Shape t` and `class Debug t => Shape t` begin identically, so the
/// constraint clause is read speculatively and abandoned when no `=>`
/// arrives. This is the test that the rewind actually rewinds.
#[test]
fn a_class_without_constraints_keeps_its_name() {
  let out = xrs("class Shape t\n  area : &t -> f64\n");
  assert!(out.contains("trait Shape {"), "{out}");
  assert!(!out.contains(": "), "no stray bound:\n{out}");
}

/// A method with no equation is **required** — the whole difference is the
/// `;` where a body would be.
#[test]
fn a_method_without_equations_is_required() {
  let out = xrs("class Shape t\n  area : &t -> f64\n");
  assert!(out.contains("fn area(&self) -> f64;"), "{out}");
  assert!(!out.contains("fn area(&self) -> f64 {"), "{out}");
}

/// A default method is an ordinary equation, so everything a function body
/// can be, it can be — several clauses and guard ladders included.
#[test]
fn a_default_method_carries_its_body() {
  let out = xrs("class Shape t\n  name : &t -> u32\n  name self = 1\n");
  assert!(out.contains("fn name(&self) -> u32 {"), "{out}");
  assert!(out.contains("        1\n"), "{out}");
}

#[test]
fn associated_types_and_constants_pass_through() {
  let out = xrs("class Shape t\n  type Unit\n  const SIDES : u32\n");
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

/// The borrow form comes from the type, so all three receivers are
/// expressible with one notation.
#[test]
fn every_receiver_form_comes_from_the_type() {
  let shared = xrs("class C t\n  f : &t -> u32\n");
  assert!(shared.contains("fn f(&self) -> u32;"), "{shared}");
  let unique = xrs("class C t\n  f : &mut t -> u32\n");
  assert!(unique.contains("fn f(&mut self) -> u32;"), "{unique}");
  let owned = xrs("class C t\n  f : t -> u32\n");
  assert!(owned.contains("fn f(self) -> u32;"), "{owned}");
}

/// A first parameter *not* typed by the class variable keeps it, which is how
/// an associated function is written — Rust spells that one with no receiver
/// at all.
#[test]
fn a_non_subject_first_parameter_is_not_a_receiver() {
  let out = xrs("class C t\n  make : u32 -> t\n");
  assert!(out.contains("fn make(__rdx_a0: u32) -> Self;"), "{out}");
  assert!(!out.contains("self"), "{out}");
}

/// Doc 18 §5.3: `t` elsewhere in a class's signature is the implementing
/// type, so it becomes `Self`.
#[test]
fn the_class_variable_elsewhere_becomes_self() {
  let out = xrs("class Combine t\n  combine : &t -> t -> t\n");
  assert!(
    out.contains("fn combine(&self, __rdx_a1: Self) -> Self;"),
    "{out}"
  );
}

#[test]
fn a_trait_instance_becomes_an_impl_for() {
  let out = xrs("instance Shape Square\n  area : &Square -> f64\n  area self = 1.0\n");
  assert!(out.contains("impl Shape for Square {"), "{out}");
  assert!(out.contains("fn area(&self) -> f64 {"), "{out}");
}

/// `instance Shape Square` writes `area : &Square -> f64`, and that
/// `&Square` is as much the receiver as `&t` is — which is why receiver
/// detection takes a list of names rather than one.
#[test]
fn an_instances_own_target_names_the_receiver() {
  let out = xrs("instance Shape Square\n  area : &Square -> f64\n  area self = 1.0\n");
  assert!(!out.contains("self: &Square"), "{out}");
}

/// No class name, so no `for`.
#[test]
fn an_inherent_instance_has_no_for() {
  let out = xrs("instance Counter\n  get : &Self -> u32\n  get self = 1\n");
  assert!(out.contains("impl Counter {"), "{out}");
  assert!(!out.contains(" for "), "{out}");
}

/// The generic list is synthesised from first appearance and the constraints
/// become its bounds — doc 18 §5.3.
#[test]
fn the_generic_list_is_synthesised() {
  let out =
    xrs("instance Display t => Describe (Wrapper t)\n  show : &t -> u32\n  show self = 1\n");
  assert!(
    out.contains("impl<T: Display> Describe for Wrapper<T> {"),
    "{out}"
  );
}

/// Several variables keep source order, which is what makes the output
/// byte-stable.
#[test]
fn several_variables_keep_source_order() {
  let out = xrs("instance Keyed (Pair k v)\n  key : &k -> u32\n  key self = 1\n");
  assert!(out.contains("impl<K, V> Keyed for Pair<K, V> {"), "{out}");
}

/// Parentheses on the *first* type make it the target, which is the inherent
/// impl on a generic type — the one case Haskell has no need for and Rust
/// does.
#[test]
fn a_parenthesised_first_type_is_the_target() {
  let out = xrs("instance (Wrapper t)\n  get : &Self -> u32\n  get self = 1\n");
  assert!(out.contains("impl<T> Wrapper<T> {"), "{out}");
  assert!(!out.contains(" for "), "{out}");
}

/// `self` is an expression, because doc 18 §2.5 drops `@name` in favour of
/// `self.name`. It lexes as a reserved word, so it needs naming in both the
/// pattern and the expression grammar.
#[test]
fn self_is_an_expression_and_a_binder() {
  let out = xrs("instance C\n  get : &Self -> u32\n  get self = self.value\n");
  assert!(out.contains("self.value"), "{out}");
}
