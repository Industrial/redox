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
  // ---- operators ----
  Eq,
  Arrow,
  /// `<-`, doc 18 §3.1's draw arrow: *a name drawn out of something*.
  ///
  /// Four grammatically disjoint positions — a statement bind, a pattern test
  /// in a condition, a `for` generator, and a comprehension generator — which
  /// is what lets one token carry four of doc 03's constructs (`use`, `is`,
  /// `defer`, the comprehension's `for`) with no lookahead.
  ///
  /// **It must be one token.** Lexed as `<` then `-`, `x <- f` parses as
  /// `x < -f`, which is valid Rust, silently wrong, and emitted without a
  /// diagnostic until this token existed.
  DrawArrow,
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
  /// `\`, doc 18 §4's lambda binder: `\x -> x + 1`.
  Backslash,
  /// `$`, doc 18 §2.1's low-precedence grouping — it lowers to nothing at
  /// all, and is why a nested application rarely needs brackets.
  Dollar,
  /// `@field` → `self.field`, `@@name` → `Self::name` (doc 01 §16).
  At,
  AtAt,
  ColonColon,
  /// `:=` — assignment, doc 18 §2.5 and plan 54.
  ///
  /// Its own token because `=` is a **binding**: at statement level `x = v`
  /// introduces a name, and telling a rebinding from a binding by looking at
  /// it would need the scope tracking AGENTS.md forbids Redox growing. Plan
  /// 20 disclosed the consequence rather than designing around it — *"a loop
  /// can advance a numeric accumulator and cannot rebind anything else"* —
  /// and this token is what retires that limit.
  ///
  /// Two characters, so it is listed before `":"`; there is no ordering
  /// constraint against `"::"`, because neither string is a prefix of the
  /// other. `scan_operator`'s
  /// `longer_operators_are_tried_before_their_prefixes` enforces the one that
  /// matters.
  ColonEq,
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
  /// Plan 18 removed `Attr`. An attribute is **trivia**, not a token: it
  /// belongs to the declaration below it, which is what `Trivia` already means,
  /// and a token nothing produced would have been a variant every exhaustive
  /// match had to carry for nothing.
  ///
  /// Plan 02 leaf 2: doc 01 §4.2 says "newlines terminate statements unless an
  /// expression is continued" without saying when. A line ending in a binary
  /// operator or a comma is continued.
  ///
  /// `=`, `->`, `=>` and `:` are deliberately **not** here, and that is not an
  /// oversight: each of them ends a header whose body follows on an *indented*
  /// line, so suppressing the newline would swallow the `Indent` the parser
  /// needs. `main : ()` / `main =` is the case that proved it.
  ///
  /// `>` and `>>` are not here either, for a different reason: they close a
  /// generic argument list far more often than they dangle off a comparison.
  /// `wide = Some 1 : Option<u8>` swallowed the whole next line into the
  /// type's verbatim text, because `capture_type` ends a type at a newline and
  /// there was no newline to find. A comparison genuinely split after `>` is
  /// writable with parentheses; a type ending in `>` is not avoidable at all,
  /// and doc 18 §5.6 deleted the chained comparison that made the dangling
  /// form attractive in the first place.
  ///
  /// `*` left the set for the same reason, one plan later:
  /// `import crate::models::*` is doc 03 §13.1's glob, a `*` is the last
  /// token of every one of them, and the import path swallowed the following
  /// declaration whole. A multiplication split after `*` is writable with
  /// parentheses; a glob import is not avoidable at all. Twice is a pattern
  /// worth naming — **a token that ends a construct should not continue a
  /// line**, however operator-like it looks.
  pub(crate) fn continues_line(&self) -> bool {
    matches!(
      self,
      Self::Pipe
        | Self::Comma
        | Self::Plus
        | Self::Minus
        | Self::Slash
        | Self::Percent
        | Self::EqEq
        | Self::Ne
        | Self::Lt
        | Self::Le
        | Self::Ge
        | Self::AndAnd
        | Self::OrOr
        | Self::Amp
        | Self::Or
        | Self::Caret
        | Self::Shl
        | Self::Coalesce
        | Self::DotDot
        | Self::DotDotEq
    )
  }
}
