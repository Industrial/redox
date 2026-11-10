use crate::TokenKind;

/// Every operator and punctuation token, **longest match first**.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 7. The order is the
/// whole correctness argument: `..=` must be tried before `..`, `??=` before
/// `??`, `...` before `..`, `>>>` before `>>`, `<<<` before `<<`, and `@@`
/// before `@`. A shorter token tried first
/// silently truncates the longer one, and
/// `scan_operator::tests::longer_operators_are_tried_before_their_prefixes`
/// checks every prefix pair mechanically rather than by eye.
pub(crate) const OPERATOR_TABLE: &[(&str, TokenKind)] = &[
  // Three characters.
  ("...", TokenKind::DotDotDot),
  ("..=", TokenKind::DotDotEq),
  ("??=", TokenKind::CoalesceEq),
  (">>>", TokenKind::Compose),
  ("<<<", TokenKind::ComposeBack),
  // Two characters.
  ("..", TokenKind::DotDot),
  (".(", TokenKind::DotParen),
  (".+", TokenKind::DotPlus),
  (".-", TokenKind::DotMinus),
  (".*", TokenKind::DotStar),
  ("./", TokenKind::DotSlash),
  (".%", TokenKind::DotPercent),
  ("??", TokenKind::Coalesce),
  ("|>", TokenKind::Pipe),
  ("&:", TokenKind::AmpColon),
  ("@@", TokenKind::AtAt),
  ("::", TokenKind::ColonColon),
  (":=", TokenKind::ColonEq),
  ("->", TokenKind::Arrow),
  ("<-", TokenKind::DrawArrow),
  ("=>", TokenKind::FatArrow),
  ("==", TokenKind::EqEq),
  ("!=", TokenKind::Ne),
  ("<=", TokenKind::Le),
  (">=", TokenKind::Ge),
  ("&&", TokenKind::AndAnd),
  ("||", TokenKind::OrOr),
  ("+=", TokenKind::PlusEq),
  ("-=", TokenKind::MinusEq),
  ("*=", TokenKind::StarEq),
  ("/=", TokenKind::SlashEq),
  ("%=", TokenKind::PercentEq),
  ("<<", TokenKind::Shl),
  (">>", TokenKind::Shr),
  // One character.
  ("(", TokenKind::LParen),
  (")", TokenKind::RParen),
  ("[", TokenKind::LBracket),
  ("]", TokenKind::RBracket),
  ("{", TokenKind::LBrace),
  ("}", TokenKind::RBrace),
  ("@", TokenKind::At),
  ("?", TokenKind::Question),
  (":", TokenKind::Colon),
  ("\\", TokenKind::Backslash),
  ("$", TokenKind::Dollar),
  (",", TokenKind::Comma),
  (".", TokenKind::Dot),
  (";", TokenKind::Semi),
  ("=", TokenKind::Eq),
  ("+", TokenKind::Plus),
  ("-", TokenKind::Minus),
  ("*", TokenKind::Star),
  ("/", TokenKind::Slash),
  ("%", TokenKind::Percent),
  ("<", TokenKind::Lt),
  (">", TokenKind::Gt),
  ("!", TokenKind::Bang),
  ("&", TokenKind::Amp),
  ("|", TokenKind::Or),
  ("^", TokenKind::Caret),
  ("~", TokenKind::Tilde),
];
