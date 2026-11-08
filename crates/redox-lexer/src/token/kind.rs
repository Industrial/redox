use crate::token::keyword::Keyword;
use crate::token::str_part::StrPart;

/// Every token Redox's lexical surface produces.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`). This enum decides *what
/// tokens exist*, not what they mean — a variant no parser yet accepts is
/// expected at this stage, not dead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
  // ---- names ----
  Ident(String),
  Kw(Keyword),
  /// `'name` — a lifetime or a block/loop label. Disambiguated from a char
  /// literal by lookahead, as Rust does (doc 03 §4.3).
  Label(String),

  // ---- literals ----
  /// A decimal, hex, octal or binary integer, carried verbatim including any
  /// `_` separators and type suffix so the emitter can pass it through.
  Int(String),
  Float(String),
  /// `"…"` or `"""…"""`.
  ///
  /// `parts` is the decoded structure — literal text and interpolations — and
  /// is what plan 23's emitter consumes. `raw` is the original source text
  /// including delimiters, which lets a consumer that does not yet handle
  /// interpolation pass a simple literal through verbatim. Both are carried
  /// because they serve different stages, and dropping `raw` would have made
  /// this plan a breaking change for the parser and emitter it has no business
  /// touching.
  Str {
    parts: Vec<StrPart>,
    raw: String,
  },
  /// `r"…"` / `r#"…"#` — no escapes, no interpolation. The stored text
  /// excludes the delimiters; `hashes` records how many `#` to re-emit.
  RawStr {
    text: String,
    hashes: usize,
  },
  /// `b"…"`, stored decoded.
  ByteStr(String),
  /// `c"…"`, stored decoded.
  CStr(String),
  /// `'x'`, stored decoded.
  Char(char),
  /// `b'x'`, stored decoded.
  Byte(u8),

  // ---- structural ----
  Newline,
  Indent,
  Dedent,
  Eof,

  // ---- delimiters ----
  LParen,
  RParen,
  LBracket,
  RBracket,
  LBrace,
  RBrace,
  /// `#{` — a set literal's opening delimiter (doc 03 §10.4). Closed by
  /// `RBrace`.
  HashBrace,
  /// `%w[` — a word-array literal's opening delimiter. Closed by `RBracket`.
  PercentW,
  /// `#[…]` — an attribute, carried verbatim for passthrough (doc 03 §13.2).
  /// The stored text excludes `#[` and `]`; `inner` distinguishes `#![…]`.
  Attr {
    text: String,
    inner: bool,
  },

  // ---- operators ----
  Eq,
  Arrow,
  FatArrow,
  /// `|>` — the receiver pipeline (doc 03 §5.3).
  Pipe,
  /// `??` and `??=` (doc 03 §12.1).
  Coalesce,
  CoalesceEq,
  /// `?.` — optional chaining (doc 03 §12.3).
  QuestionDot,
  /// `?` — error propagation.
  Question,
  /// `...` — spread, rest pattern, or argument forwarding (doc 03 §4.2).
  DotDotDot,
  /// `..` exclusive and `..=` inclusive, matching Rust exactly — doc 03 §4.2
  /// corrects doc 01 §21, which had them inverted.
  DotDot,
  DotDotEq,
  /// `.(` — the broadcast-call form (doc 03 §10.2).
  DotParen,
  /// `.+ .- .* ./ .%` — elementwise operators (doc 03 §10.2).
  DotPlus,
  DotMinus,
  DotStar,
  DotSlash,
  DotPercent,
  /// `&:name` — a method reference (doc 03 §8.5).
  AmpColon,
  /// `@field` → `self.field`, `@@name` → `Self::name` (doc 01 §16).
  At,
  AtAt,
  ColonColon,
  Colon,
  Comma,
  Dot,
  Semi,
  Plus,
  Minus,
  Star,
  Slash,
  Percent,
  PlusEq,
  MinusEq,
  StarEq,
  SlashEq,
  PercentEq,
  EqEq,
  Ne,
  Lt,
  Gt,
  Le,
  Ge,
  AndAnd,
  OrOr,
  Bang,
  Amp,
  Or,
  Caret,
  Tilde,
  Shl,
  Shr,
}

impl TokenKind {
  /// Whether this token is structural — emitted at a position the parser does
  /// not read trivia from.
  ///
  /// Structural tokens never claim trivia. Letting a `Dedent` absorb a pending
  /// comment loses it silently, which is how a comment between two top-level
  /// items, or at end of file, used to disappear. `Eof` counts as
  /// non-structural so a trailing comment has somewhere to land.
  pub(crate) fn is_structural(&self) -> bool {
    matches!(self, Self::Indent | Self::Dedent | Self::Newline)
  }

  /// Whether a line **ending** in this token continues onto the next.
  ///
  /// Plan 02 leaf 2: doc 01 §4.2 says "newlines terminate statements unless an
  /// expression is continued" without saying when. A line ending in a binary
  /// operator or a comma is continued.
  ///
  /// `=`, `->`, `=>` and `:` are deliberately **not** here, and that is not an
  /// oversight: each of them ends a header whose body follows on an *indented*
  /// line, so suppressing the newline would swallow the `Indent` the parser
  /// needs. `main = ->` is the case that proved it.
  pub(crate) fn continues_line(&self) -> bool {
    matches!(
      self,
      Self::Pipe
        | Self::Comma
        | Self::Plus
        | Self::Minus
        | Self::Star
        | Self::Slash
        | Self::Percent
        | Self::EqEq
        | Self::Ne
        | Self::Lt
        | Self::Gt
        | Self::Le
        | Self::Ge
        | Self::AndAnd
        | Self::OrOr
        | Self::Amp
        | Self::Or
        | Self::Caret
        | Self::Shl
        | Self::Shr
        | Self::Coalesce
        | Self::DotDot
        | Self::DotDotEq
    )
  }
}
