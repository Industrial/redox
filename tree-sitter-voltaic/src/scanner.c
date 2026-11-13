// The off-side rule, for tree-sitter.
//
// Plan 41 (`docs/36 - 41 - Tree-sitter Grammar.md`) leaf 2. `grammar.js` can
// express everything about Voltaic except where a block begins and ends, because
// that is decided by a column count and tree-sitter's own lexer has no notion
// of one. So three tokens come from here: NEWLINE, INDENT and DEDENT.
//
// This is the same algorithm `voltaic-lexer` runs, written a second time, and
// §1.2 is the argument for why that duplication is acceptable here and not for
// precedence: an indent stack is forty lines with one observable behaviour,
// where a precedence table is a hundred rows whose disagreements are silent.
//
// # The shape of it, which took two attempts
//
// One measurement per call, and **INDENT absorbs the newline**: the scanner
// skips the line break and the indentation together, then emits whichever of
// the three the measured width implies. The grammar therefore has no
// `_newline` before an `_indent` — a block's opening *is* the indent.
//
// Both earlier attempts tried to emit NEWLINE and then INDENT at the same
// place, and neither can work: `advance(lexer, true)` marks a character as
// skipped **and consumes it**, so whitespace measured for one token is gone
// before the next call, and `mark_end` does not hold it back. The symptom was
// a zero-width ERROR at the head of every indented block in the corpus —
// twenty-eight files, one cause.
//
// A run of DEDENTs needs state for the same reason: the width cannot be
// re-measured once consumed, so the count is computed once and `pending`
// pays it out one token per call. That is also where the inconsistent-dedent
// check belongs, because it is a question about the *whole* run.
//
// # The error case is a *refusal*, not a token
//
// Doc 03 §3.1's `RDX0002` is a dedent to a width matching no enclosing level.
// This scanner reports it by emitting **nothing**: tree-sitter then has no
// token where it needs one and produces an ERROR node, which is what
// criterion 4 asks for. Accepting the line and guessing a depth — which is
// what most indentation scanners do — would make criterion 3's no-ERROR sweep
// vacuous, because the grammar would never reject anything.

#include <tree_sitter/parser.h>

#include <stdlib.h>
#include <string.h>

enum TokenType {
  NEWLINE,
  INDENT,
  DEDENT,
  // `{- … -}`, which **nests**: `{- a {- b -} c -}` is one comment. The one
  // token here that is not about indentation, and it is here for the same
  // reason the others are — a regex cannot count.
  FENCED_COMMENT,
};

// The enclosing indent widths, outermost first. Index 0 is always 0 — column
// zero is a level, and an item at the top of a file sits on it.
//
// 64 is the depth cap, needed because `serialize` has a fixed buffer and a
// silently truncated stack would corrupt the depth after a re-parse. Doc 06
// caps a file's own nesting far below this, so a program reaching it is one no
// editor could usefully display.
#define MAX_DEPTH 64

typedef struct {
  uint16_t len;
  // Tokens the last measurement found but could not emit, because the grammar
  // wanted a different one first.
  //
  // Both exist for the same reason: measuring **consumes** the indentation, so
  // a width can be read once and never again. A statement ending at a dedent
  // needs its NEWLINE before the DEDENT, and a statement ending at an indent
  // needs its NEWLINE before the INDENT — in each case the token the grammar
  // is not ready for has to wait, and it cannot wait in the input.
  uint16_t pending_dedents;
  // Non-zero when an INDENT of that width is owed.
  uint16_t pending_indent;
  uint16_t widths[MAX_DEPTH];
} Scanner;

void *tree_sitter_voltaic_external_scanner_create(void) {
  Scanner *scanner = calloc(1, sizeof(Scanner));
  if (scanner != NULL) {
    scanner->len = 1;
    scanner->pending_dedents = 0;
    scanner->pending_indent = 0;
    scanner->widths[0] = 0;
  }
  return scanner;
}

void tree_sitter_voltaic_external_scanner_destroy(void *payload) { free(payload); }

// tree-sitter re-parses from a saved state, so the indent stack has to survive
// a round trip. A scanner that did not serialise would get the depth right on
// a full parse and wrong on every incremental one — the failure mode an editor
// sees and a corpus test does not.
unsigned tree_sitter_voltaic_external_scanner_serialize(void *payload, char *buffer) {
  Scanner *scanner = (Scanner *)payload;
  unsigned size = sizeof(uint16_t) * (3 + scanner->len);
  if (size > TREE_SITTER_SERIALIZATION_BUFFER_SIZE) {
    return 0;
  }
  memcpy(buffer, scanner, size);
  return size;
}

void tree_sitter_voltaic_external_scanner_deserialize(void *payload, const char *buffer,
                                                    unsigned length) {
  Scanner *scanner = (Scanner *)payload;
  scanner->len = 1;
  scanner->pending_dedents = 0;
  scanner->pending_indent = 0;
  scanner->widths[0] = 0;
  if (length > 0) {
    memcpy(scanner, buffer, length);
  }
}

static bool is_space(int32_t c) { return c == ' ' || c == '\t'; }

// Consume a `{- … -}`, counting nesting, with the opening `{` already the
// lookahead. Reports false if there was no `-` after it, having consumed only
// the brace.
//
// `skip` chooses whether the text belongs to the token: the external
// `_fenced_comment` wants it, and `skip_to_content` measuring an indentation
// does not.
static bool consume_fence(TSLexer *lexer, bool skip) {
  lexer->advance(lexer, skip);
  if (lexer->lookahead != '-') {
    return false;
  }
  lexer->advance(lexer, skip);
  unsigned depth = 1;
  int32_t previous = 0;
  while (!lexer->eof(lexer)) {
    if (previous == '{' && lexer->lookahead == '-') {
      depth++;
      previous = 0;
      lexer->advance(lexer, skip);
      continue;
    }
    if (previous == '-' && lexer->lookahead == '}') {
      lexer->advance(lexer, skip);
      depth--;
      if (depth == 0) {
        return true;
      }
      previous = 0;
      continue;
    }
    previous = lexer->lookahead;
    lexer->advance(lexer, skip);
  }
  // Unterminated at the end of the file: taken as a comment to that point,
  // which is what an editor wants while the author is still typing the
  // closing fence — the alternative colours the rest of the file as code.
  return true;
}

// Skip to the next line that carries depth, reporting its indentation and
// whether a line break was crossed to reach it.
//
// A blank line and a comment-only line carry **no depth** — `voltaic-lexer`'s
// `dedent_block` makes the same exclusion, and plan 43 §10.2 records why: a
// blank line's width would otherwise force every comparison to zero.
//
// Everything is skipped rather than consumed into the token, so all three
// tokens come out zero-width, which is what lets the grammar place them
// between constructs rather than inside them.
static bool skip_to_content(TSLexer *lexer, uint16_t *width, bool *crossed_a_line) {
  *width = 0;
  *crossed_a_line = false;
  while (true) {
    if (lexer->lookahead == '\n') {
      *crossed_a_line = true;
      *width = 0;
      lexer->advance(lexer, true);
      continue;
    }
    if (lexer->lookahead == '\r') {
      lexer->advance(lexer, true);
      continue;
    }
    if (is_space(lexer->lookahead)) {
      // A tab counts as one column, as Voltaic's own lexer counts it. The two
      // agreeing matters more than either being clever: a file mixing tabs
      // and spaces is already `RDX0002`'s business.
      (*width)++;
      lexer->advance(lexer, true);
      continue;
    }
    if (lexer->eof(lexer)) {
      return false;
    }
    if (lexer->lookahead == '-') {
      // Perhaps a comment. Only `--` is one; a lone `-` begins a continuation
      // line, and a statement cannot begin with `-` in Voltaic — the parser's
      // `starts_argument` excludes it precisely so `a - b` does not read as
      // `a (-b)`.
      lexer->advance(lexer, true);
      if (lexer->lookahead != '-') {
        return true;
      }
      while (lexer->lookahead != '\n' && !lexer->eof(lexer)) {
        lexer->advance(lexer, true);
      }
      continue;
    }
    if (lexer->lookahead == '{') {
      if (!consume_fence(lexer, true)) {
        // A bare `{`, which is a record's brace and real content.
        return true;
      }
      continue;
    }
    return true;
  }
}

bool tree_sitter_voltaic_external_scanner_scan(void *payload, TSLexer *lexer,
                                             const bool *valid_symbols) {
  Scanner *scanner = (Scanner *)payload;

  // Tokens owed from the previous measurement, before anything else: the
  // width they refer to has already been consumed and cannot be found again.
  if (scanner->pending_dedents > 0) {
    if (!valid_symbols[DEDENT]) {
      // A dedent the grammar cannot accept. Returning nothing lets
      // tree-sitter report it, which is right: the block structure and the
      // grammar disagree, and guessing would hide it.
      return false;
    }
    scanner->pending_dedents--;
    scanner->len--;
    lexer->result_symbol = DEDENT;
    return true;
  }
  if (scanner->pending_indent > 0) {
    if (valid_symbols[INDENT] && scanner->len < MAX_DEPTH) {
      scanner->widths[scanner->len++] = scanner->pending_indent;
      scanner->pending_indent = 0;
      lexer->result_symbol = INDENT;
      return true;
    }
    // The grammar did not want a block after all — a continuation line it
    // models some other way. Dropped rather than held, so it cannot surface
    // against an unrelated construct later.
    scanner->pending_indent = 0;
  }

  // A nesting fence, **after** the deferred tokens and before the
  // measurement. Before them it fired on the call that owed an INDENT, so a
  // block whose first line is `{- … -}` never opened — the fence consumed
  // the position the indent was waiting for. It is in `extras`, so it is
  // valid almost everywhere, and the `{` it opens with is also a record's
  // brace — which is why the **second** character decides, and why returning
  // false matters: tree-sitter restores the position when a scanner declines,
  // so the brace is left for the grammar untouched.
  if (valid_symbols[FENCED_COMMENT] && lexer->lookahead == '{') {
    if (!consume_fence(lexer, false)) {
      // Not a fence after all. Returning false restores the position, so the
      // brace is left for the grammar untouched — which is why the **second**
      // character is allowed to decide.
      return false;
    }
    lexer->mark_end(lexer);
    lexer->result_symbol = FENCED_COMMENT;
    return true;
  }

  // Nothing here runs unless the grammar is expecting one of the three. That
  // is what makes a bracketed continuation line work without this scanner
  // knowing about brackets: inside `(`…`)` the grammar expects none of them,
  // so the newline falls through to `extras` as ordinary whitespace.
  if (!valid_symbols[NEWLINE] && !valid_symbols[INDENT] && !valid_symbols[DEDENT]) {
    return false;
  }

  // At the end of the file every open block closes, one token per call.
  if (lexer->eof(lexer)) {
    if (scanner->len > 1 && valid_symbols[DEDENT]) {
      scanner->len--;
      lexer->result_symbol = DEDENT;
      return true;
    }
    return false;
  }

  uint16_t width = 0;
  bool crossed_a_line = false;
  bool found = skip_to_content(lexer, &width, &crossed_a_line);
  if (!crossed_a_line) {
    // A closing delimiter ends every block its bracket opened — plan 65
    // leaf 1, and the fourth reader of the surface change doc 04's fifth
    // ordering commitment names.
    //
    // `voltaic-lexer` emits those `Dedent`s because it carries a layout-context
    // mark per open delimiter. This scanner cannot: `(` is an ordinary
    // grammar token it is never asked to lex, so it has no way to see one
    // open. It gets the same answer from the grammar instead — the only
    // reason `DEDENT` is valid with a closer ahead is that a
    // `statement_block` inside the bracket is still open — and closes one
    // level per call, letting the next call decide whether another is wanted.
    //
    // `print (if c` with indented branches and a trailing `)` is the shape.
    // Without this the `)` arrived while `statement_block` still wanted its
    // `_dedent`, and the whole function was an `ERROR`: the measurement below
    // needs a line break to reach, and the closer sits on the body's own
    // line, so the break never comes.
    // A closer **ends the line** as far as layout is concerned, so it closes
    // a block the way a line break does: the NEWLINE that terminates the
    // block's last statement, then one DEDENT per level. That order is the
    // only one the grammar is built for — `expression_statement` is
    // `seq($._expression, $._newline)`, and emitting the DEDENT alone left a
    // `(MISSING _newline)` in the tree, which `tree-sitter test` reports and
    // `tree-sitter parse` does not print because `_newline` is hidden.
    if (scanner->len > 1 &&
        (lexer->lookahead == ')' || lexer->lookahead == ']' ||
         lexer->lookahead == '}')) {
      if (valid_symbols[NEWLINE]) {
        // Owed rather than emitted, so this cannot loop: the next call pops a
        // level whatever the grammar then asks for.
        scanner->pending_dedents = 1;
        lexer->result_symbol = NEWLINE;
        return true;
      }
      if (valid_symbols[DEDENT]) {
        scanner->len--;
        lexer->result_symbol = DEDENT;
        return true;
      }
    }
    // Mid-line whitespace, which is `extras`' business and not this
    // scanner's. Returning false here is what keeps a space between two
    // tokens from being mistaken for an indentation.
    return false;
  }
  lexer->mark_end(lexer);

  if (!found) {
    // Only blank lines and comments to the end of the file.
    if (scanner->len > 1 && valid_symbols[DEDENT]) {
      scanner->len--;
      lexer->result_symbol = DEDENT;
      return true;
    }
    if (valid_symbols[NEWLINE]) {
      lexer->result_symbol = NEWLINE;
      return true;
    }
    return false;
  }

  uint16_t current = scanner->widths[scanner->len - 1];

  if (width > current) {
    if (scanner->len >= MAX_DEPTH) {
      return false;
    }
    if (valid_symbols[INDENT]) {
      scanner->widths[scanner->len++] = width;
      lexer->result_symbol = INDENT;
      return true;
    }
    // The statement wants its NEWLINE first — `f =` then an indented body is
    // the shape where the grammar is not ready for the INDENT yet. Emit the
    // NEWLINE and **owe** the indent, because the width is now consumed.
    if (valid_symbols[NEWLINE]) {
      scanner->pending_indent = width;
      lexer->result_symbol = NEWLINE;
      return true;
    }
    return false;
  }

  if (width < current) {
    // How many levels close, counted here because the width is about to be
    // unavailable.
    uint16_t closing = 0;
    while (scanner->len - closing > 1 && scanner->widths[scanner->len - 1 - closing] > width) {
      closing++;
    }
    // The inconsistent-dedent check, and the whole of criterion 4. After
    // closing every level deeper than this line, the width must *be* a level
    // that is still open. If it sits between two there is no block to return
    // to and no token to emit — doc 03 §3.1's `RDX0002`, as an ERROR node.
    if (scanner->widths[scanner->len - 1 - closing] != width) {
      return false;
    }
    if (valid_symbols[DEDENT]) {
      scanner->len--;
      scanner->pending_dedents = closing - 1;
      lexer->result_symbol = DEDENT;
      return true;
    }
    // The statement wants its NEWLINE first, which is the common case at the
    // end of a block: `print x` then a line at column zero needs NEWLINE,
    // DEDENT. Owing the whole run is what makes that work — and getting this
    // wrong produced a zero-width ERROR at the first item after **every**
    // indented block in the corpus.
    if (valid_symbols[NEWLINE]) {
      scanner->pending_dedents = closing;
      lexer->result_symbol = NEWLINE;
      return true;
    }
    return false;
  }

  if (valid_symbols[NEWLINE]) {
    lexer->result_symbol = NEWLINE;
    return true;
  }
  return false;
}
