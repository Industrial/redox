use lsp_types::SemanticTokenType;

/// The legend: the token types this server emits, in the order its indices
/// refer to.
///
/// The protocol encodes a token's type as an **index into this list**, so the
/// order is load-bearing and a reordering is a silent miscolouring. `token`'s
/// table is the only place that maps a `TokenKind` to one of these, and it
/// names them rather than numbering them for that reason.
pub(crate) fn token_types() -> Vec<SemanticTokenType> {
  vec![
    SemanticTokenType::KEYWORD,
    SemanticTokenType::TYPE,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::NUMBER,
    SemanticTokenType::STRING,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::COMMENT,
  ]
}
