use redox_lexer::TokenKind;

/// Which semantic token type a lexical token is, as an index into
/// `capabilities::token_types`'s legend — or `None` for a token that is not
/// text.
///
/// The indices are positional, so this table and that list are one thing in
/// two places. They are kept together by the names in the comments rather than
/// by a constant each, which is a deliberate and stated trade: a reordering of
/// the legend silently miscolours every token, and the assertion that catches
/// it is `the_legend_covers_every_index` in the test suite rather than the type
/// system.
///
/// # What produces nothing
///
/// `Indent`, `Dedent`, `Newline` and `Eof` carry spans and are not text —
/// plan 40 §1.4. Nothing else is skipped: punctuation is `OPERATOR`, which is
/// how an editor's theme reaches it, and a token with no entry would simply be
/// uncoloured without saying so.
pub(crate) fn token_index(kind: &TokenKind) -> Option<u32> {
  Some(match kind {
    // 0: KEYWORD
    TokenKind::Kw(_) => 0,
    // 1: TYPE — a lifetime or loop label. Neither is a type, and `TYPE` is
    // the legend entry whose usual colour suits them: both are a short
    // `'name` that readers scan past rather than read.
    TokenKind::Label(_) => 1,
    // 3: VARIABLE — every identifier. Telling a function name from a
    // variable needs to know what the name *is*, which is name resolution,
    // which Redox does not have (plan 40 §6). So `FUNCTION`, index 2, is in
    // the legend and unused: it is what a future row with a symbol table
    // would reach for, and removing it would renumber everything after it.
    TokenKind::Ident(_) => 3,
    // 4: NUMBER
    TokenKind::Int(_) | TokenKind::Float(_) | TokenKind::Byte(_) => 4,
    // 5: STRING
    TokenKind::Str { .. }
    | TokenKind::RawStr { .. }
    | TokenKind::ByteStr(_)
    | TokenKind::CStr(_)
    | TokenKind::Char(_) => 5,
    // 7: COMMENT is in the legend and unreachable from here, because a
    // comment is **trivia** and never a token. Doc 02 §10.1 made that choice
    // and this is where it shows: an editor colours comments from its own
    // syntax file, and a server that wanted to own them would have to publish
    // trivia as tokens.
    TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => return None,
    // 6: OPERATOR — everything else, which is punctuation.
    _ => 6,
  })
}
