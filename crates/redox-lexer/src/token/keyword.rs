/// A reserved word.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 8. Reserved words lex
/// as `Kw`, never `Ident`, so the parser matches a variant rather than a
/// string and `if = 5` fails in the lexer rather than as a confusing parse
/// error.
///
/// The set is **doc 18 §7's**, which replaced doc 03 §4.5's: eleven words
/// leave — `is`, `and`, `use`, `with`, `defer`, `cond`, `into`, `derive`,
/// `mod`, `unless`, `switch` — and nine arrive — `case`, `of`, `then`,
/// `otherwise`, `data`, `newtype`, `instance`, `deriving`, `module`. The count
/// is flat, which doc 18 §7 says outright rather than claiming a saving: what
/// falls is the number of *constructs*, and a keyword that stops existing is
/// only evidence of that when a construct went with it.
///
/// A variant no parser yet accepts is expected at this stage, not dead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
  // Bindings and items
  Mut,
  Const,
  Static,
  Type,
  Struct,
  Enum,
  Trait,
  Impl,
  Class,
  Instance,
  Module,
  Errors,
  // Doc 18 §2.4's declaration form. `data` covers struct, enum, union and
  // tuple struct; `newtype` is the single-field case rustc wants named; and
  // `deriving` is a clause of the declaration rather than an attribute, which
  // is why `derive` as a standalone keyword is gone.
  Data,
  Newtype,
  Deriving,
  // Control flow
  If,
  Then,
  Else,
  While,
  Until,
  For,
  In,
  /// Doc 30 §3.3's callback bind — `with conn = pool.with_connection`.
  ///
  /// Doc 03 §7.6 names *"Koka's `with` statement as a second witness"* for
  /// the construct, and doc 18 §7 deleted doc 03 §7.5's unrelated `with`
  /// *blocks* — leaving the keyword unused for exactly this.
  With,
  Loop,
  // Doc 18 §2.3. `case … of` with `->` arms, which is what frees `=>` to take
  // up its Haskell job in constraints.
  Case,
  Of,
  // The final catch-all of a guard ladder. Reserved rather than contextual:
  // a guard row reading `| otherwise = …` must not be a comparison against a
  // variable someone happened to name `otherwise`.
  Otherwise,
  When,
  Do,
  Break,
  Continue,
  Return,
  Guard,
  Try,
  Catch,
  Where,
  // Patterns. Doc 03 §4.4 fixed `is`, `and`, `or` and `not` as pattern
  // combinators; doc 18 §2.3 deletes `is`, `and` and `not` in favour of
  // guards, and §5.4 keeps `or` **precisely so that `|` is unambiguously the
  // guard bar**. `not` returns to meaning boolean negation.
  Or,
  Not,
  // Types and traits
  As,
  Dyn,
  Impls,
  // Functions
  Fn,
  Move,
  Async,
  Await,
  Pure,
  // Modules
  Import,
  Export,
  // Values
  SelfValue,
  SelfType,
  True,
  False,
  // Testing
  Test,
  Assert,
  /// Doc 01 §23.1's inline Rust escape hatch — plan 45 leaf 1.
  ///
  /// Reserved rather than contextual. Before this, `rust """…"""` parsed as a
  /// **call** to a function named `rust` and emitted `rust("…")`, which no
  /// program defines; reserving the word is what makes the documented syntax
  /// mean what it says instead of silently meaning something else.
  Rust,
  /// `name :: unsafe T -> U` and the `unsafe` block — plan 45 leaf 2.
  ///
  /// Doc 01 §25.4 *"gestures at native Rust syntax or the inline Rust escape
  /// hatch without choosing"*; doc 03 §23 item 2 forbids the hatch for this,
  /// so the word is native syntax.
  Unsafe,
  /// `extern "C"` — plan 45 leaf 3, and doc 01 §26's FFI requirement.
  Extern,
  /// `union Reg = Reg { … }` — plan 45 leaf 4.
  ///
  /// A keyword of its own beside `data` and `newtype`, for `newtype`'s
  /// reason: doc 18 §2.4's rule reads a *shape* off the constructors — one
  /// constructor named for the type is a struct, anything else an enum — and
  /// a union is not a shape, it is the same shape with different storage.
  /// Nothing in the source text could select it, so it takes a word.
  Union,
}

impl Keyword {
  /// The reserved word `text` names, if it is one.
  pub fn from_text(text: &str) -> Option<Self> {
    Some(match text {
      "mut" => Self::Mut,
      "const" => Self::Const,
      "static" => Self::Static,
      "type" => Self::Type,
      "struct" => Self::Struct,
      "enum" => Self::Enum,
      "trait" => Self::Trait,
      "impl" => Self::Impl,
      "class" => Self::Class,
      "instance" => Self::Instance,
      "module" => Self::Module,
      "data" => Self::Data,
      "newtype" => Self::Newtype,
      "deriving" => Self::Deriving,
      "errors" => Self::Errors,
      "if" => Self::If,
      "then" => Self::Then,
      "else" => Self::Else,
      "while" => Self::While,
      "until" => Self::Until,
      "for" => Self::For,
      "in" => Self::In,
      "with" => Self::With,
      "loop" => Self::Loop,
      "case" => Self::Case,
      "of" => Self::Of,
      "otherwise" => Self::Otherwise,
      "when" => Self::When,
      "do" => Self::Do,
      "break" => Self::Break,
      "continue" => Self::Continue,
      "return" => Self::Return,
      "guard" => Self::Guard,
      "try" => Self::Try,
      "catch" => Self::Catch,
      "where" => Self::Where,
      "or" => Self::Or,
      "not" => Self::Not,
      "as" => Self::As,
      "dyn" => Self::Dyn,
      "implements" => Self::Impls,
      "fn" => Self::Fn,
      "move" => Self::Move,
      "async" => Self::Async,
      "await" => Self::Await,
      "pure" => Self::Pure,
      "import" => Self::Import,
      "export" => Self::Export,
      "self" => Self::SelfValue,
      "Self" => Self::SelfType,
      "true" => Self::True,
      "false" => Self::False,
      "test" => Self::Test,
      "assert" => Self::Assert,
      "rust" => Self::Rust,
      "unsafe" => Self::Unsafe,
      "extern" => Self::Extern,
      "union" => Self::Union,
      _ => return None,
    })
  }

  /// The source spelling, for diagnostics and for the formatter.
  pub fn text(self) -> &'static str {
    match self {
      Self::Mut => "mut",
      Self::Const => "const",
      Self::Static => "static",
      Self::Type => "type",
      Self::Struct => "struct",
      Self::Enum => "enum",
      Self::Trait => "trait",
      Self::Impl => "impl",
      Self::Class => "class",
      Self::Instance => "instance",
      Self::Module => "module",
      Self::Data => "data",
      Self::Newtype => "newtype",
      Self::Deriving => "deriving",
      Self::Errors => "errors",
      Self::If => "if",
      Self::Then => "then",
      Self::Else => "else",
      Self::While => "while",
      Self::Until => "until",
      Self::For => "for",
      Self::In => "in",
      Self::With => "with",
      Self::Loop => "loop",
      Self::Case => "case",
      Self::Of => "of",
      Self::Otherwise => "otherwise",
      Self::When => "when",
      Self::Do => "do",
      Self::Break => "break",
      Self::Continue => "continue",
      Self::Return => "return",
      Self::Guard => "guard",
      Self::Try => "try",
      Self::Catch => "catch",
      Self::Where => "where",
      Self::Or => "or",
      Self::Not => "not",
      Self::As => "as",
      Self::Dyn => "dyn",
      Self::Impls => "implements",
      Self::Fn => "fn",
      Self::Move => "move",
      Self::Async => "async",
      Self::Await => "await",
      Self::Pure => "pure",
      Self::Import => "import",
      Self::Export => "export",
      Self::SelfValue => "self",
      Self::SelfType => "Self",
      Self::True => "true",
      Self::False => "false",
      Self::Test => "test",
      Self::Assert => "assert",
      Self::Rust => "rust",
      Self::Unsafe => "unsafe",
      Self::Extern => "extern",
      Self::Union => "union",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Keyword;

  #[test]
  fn round_trips_every_spelling() {
    for text in [
      "mut",
      "if",
      "then",
      "case",
      "of",
      "otherwise",
      "data",
      "newtype",
      "deriving",
      "instance",
      "module",
      "not",
      "or",
      "guard",
      "catch",
      "loop",
      "Self",
      "self",
      "true",
      "errors",
      "implements",
    ] {
      let kw = Keyword::from_text(text).expect("should be reserved");
      assert_eq!(kw.text(), text);
    }
  }

  /// The words doc 18 §7 retires are ordinary identifiers again, and that is
  /// the point of checking them here: a retired keyword that stayed reserved
  /// would reject a program that is now legal, and nothing else in the suite
  /// would notice.
  #[test]
  fn leaves_ordinary_identifiers_alone() {
    for text in [
      "main", "print", "iff", "switched", "island", "notable", "Selfish", "is", "and", "use",
      "defer", "cond", "into", "derive", "mod", "unless", "switch", "withhold",
    ] {
      assert!(
        Keyword::from_text(text).is_none(),
        "{text} should not reserve"
      );
    }
  }
}
