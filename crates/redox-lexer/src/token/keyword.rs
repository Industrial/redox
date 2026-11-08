/// A reserved word.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 8. Reserved words lex
/// as `Kw`, never `Ident`, so the parser matches a variant rather than a
/// string and `if = 5` fails in the lexer rather than as a confusing parse
/// error.
///
/// The set is doc 01's plus `docs/03 - Redox Surface Design.md` §4.5's. A
/// variant no parser yet accepts is expected at this stage, not dead.
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
  Mod,
  Errors,
  Derive,
  // Control flow
  If,
  Else,
  Unless,
  While,
  Until,
  For,
  In,
  Loop,
  Cond,
  Switch,
  When,
  Do,
  Break,
  Continue,
  Return,
  Guard,
  Defer,
  Try,
  Catch,
  Use,
  With,
  Where,
  Into,
  // Patterns. Doc 03 §4.4 fixes these as pattern combinators only; boolean
  // logic uses `&&`, `||` and `!`. The lexer reserves them; plan 21 enforces
  // that they appear only in pattern position.
  Is,
  And,
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
      "mod" => Self::Mod,
      "errors" => Self::Errors,
      "derive" => Self::Derive,
      "if" => Self::If,
      "else" => Self::Else,
      "unless" => Self::Unless,
      "while" => Self::While,
      "until" => Self::Until,
      "for" => Self::For,
      "in" => Self::In,
      "loop" => Self::Loop,
      "cond" => Self::Cond,
      "switch" => Self::Switch,
      "when" => Self::When,
      "do" => Self::Do,
      "break" => Self::Break,
      "continue" => Self::Continue,
      "return" => Self::Return,
      "guard" => Self::Guard,
      "defer" => Self::Defer,
      "try" => Self::Try,
      "catch" => Self::Catch,
      "use" => Self::Use,
      "with" => Self::With,
      "where" => Self::Where,
      "into" => Self::Into,
      "is" => Self::Is,
      "and" => Self::And,
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
      Self::Mod => "mod",
      Self::Errors => "errors",
      Self::Derive => "derive",
      Self::If => "if",
      Self::Else => "else",
      Self::Unless => "unless",
      Self::While => "while",
      Self::Until => "until",
      Self::For => "for",
      Self::In => "in",
      Self::Loop => "loop",
      Self::Cond => "cond",
      Self::Switch => "switch",
      Self::When => "when",
      Self::Do => "do",
      Self::Break => "break",
      Self::Continue => "continue",
      Self::Return => "return",
      Self::Guard => "guard",
      Self::Defer => "defer",
      Self::Try => "try",
      Self::Catch => "catch",
      Self::Use => "use",
      Self::With => "with",
      Self::Where => "where",
      Self::Into => "into",
      Self::Is => "is",
      Self::And => "and",
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
      "switch",
      "is",
      "and",
      "not",
      "guard",
      "defer",
      "catch",
      "loop",
      "cond",
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

  #[test]
  fn leaves_ordinary_identifiers_alone() {
    for text in [
      "main", "print", "iff", "switched", "island", "notable", "Selfish",
    ] {
      assert!(
        Keyword::from_text(text).is_none(),
        "{text} should not reserve"
      );
    }
  }
}
