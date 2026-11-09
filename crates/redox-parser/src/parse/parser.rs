pub(crate) mod above_and_and;
pub(crate) mod application;
pub(crate) mod arguments;
pub(crate) mod atom;
pub(crate) mod binary_op;
pub(crate) mod blocks_in_the_same_closure;
pub(crate) mod bracketed;
pub(crate) mod byte_literal;
pub(crate) mod can_end_a_type;
pub(crate) mod capture_type;
pub(crate) mod case_arm;
pub(crate) mod case_of;
pub(crate) mod check_callback_tails;
pub(crate) mod class_body;
pub(crate) mod class_decl;
pub(crate) mod class_member;
pub(crate) mod compound_assign;
pub(crate) mod condition;
pub(crate) mod condition_operand;
pub(crate) mod const_decl;
pub(crate) mod constraints;
pub(crate) mod constructor;
pub(crate) mod constructor_fields;
pub(crate) mod constructor_path;
pub(crate) mod control_flow;
pub(crate) mod data_decl;
pub(crate) mod deriving;
pub(crate) mod do_block;
pub(crate) mod error_variant;
pub(crate) mod errors_decl;
pub(crate) mod expr_from_tokens;
pub(crate) mod field_decl;
pub(crate) mod field_init;
pub(crate) mod flush_word;
pub(crate) mod generalise;
pub(crate) mod holds_a_pattern_test;
pub(crate) mod if_expression;
pub(crate) mod import_decl;
pub(crate) mod import_path;
pub(crate) mod instance_decl;
pub(crate) mod interpolated_string;
pub(crate) mod is_word;
pub(crate) mod lambda;
pub(crate) mod method_context;
pub(crate) mod module_decl;
pub(crate) mod opens_a_section;
pub(crate) mod optional_chain;
pub(crate) mod parse_expr;
pub(crate) mod parse_spine;
pub(crate) mod path_or_ident;
pub(crate) mod pattern;
pub(crate) mod pattern_alt;
pub(crate) mod pattern_app;
pub(crate) mod pattern_series;
pub(crate) mod patterns;
pub(crate) mod postfix;
pub(crate) mod prefix;
pub(crate) mod primitive_type;
pub(crate) mod qualifiers;
pub(crate) mod range_or_literal;
pub(crate) mod receiver;
pub(crate) mod receiver_text;
pub(crate) mod record_literal;
pub(crate) mod record_pattern;
pub(crate) mod return_in_tail;
pub(crate) mod rewritten;
pub(crate) mod rust_generic;
pub(crate) mod rust_string_body;
pub(crate) mod scan_run;
pub(crate) mod section;
pub(crate) mod signature_from;
pub(crate) mod signature_only;
pub(crate) mod spaced_colon_colon;
pub(crate) mod starts_a_field_type;
pub(crate) mod starts_a_pattern_argument;
pub(crate) mod starts_argument;
pub(crate) mod substitute_var;
pub(crate) mod test_decl;
pub(crate) mod tight_colon_colon;
pub(crate) mod token_text;
pub(crate) mod type_alias;
pub(crate) mod type_app;
pub(crate) mod type_stop;
pub(crate) mod type_vars;
pub(crate) mod unary_op;
pub(crate) mod visibility;
pub(crate) mod walk_item_for_tails;
pub(crate) mod walk_member_for_tails;

use crate::ParseError;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::check_callback_tails::check_callback_tails;
use crate::parse::parser::class_decl::class_decl;
use crate::parse::parser::compound_assign::compound_assign;
use crate::parse::parser::condition::condition;
use crate::parse::parser::const_decl::const_decl;
use crate::parse::parser::constraints::constraints;
use crate::parse::parser::control_flow::control_flow;
use crate::parse::parser::data_decl::data_decl;
use crate::parse::parser::errors_decl::errors_decl;
use crate::parse::parser::generalise::generalise;
use crate::parse::parser::import_decl::import_decl;
use crate::parse::parser::instance_decl::instance_decl;
use crate::parse::parser::module_decl::module_decl;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::parse_spine::parse_spine;
use crate::parse::parser::pattern_alt::pattern_alt;
use crate::parse::parser::patterns::patterns;
use crate::parse::parser::signature_from::signature_from;
use crate::parse::parser::signature_only::signature_only;
use crate::parse::parser::test_decl::test_decl;
use crate::parse::parser::type_alias::type_alias;
use crate::parse::parser::type_stop::TypeStop;
use crate::parse::parser::visibility::visibility;
use redox_ast::Visibility;
use redox_ast::{Binding, Block, FnDecl, Ident, Item, Program, Stmt, StmtKind, TypeExpr};
use redox_diag::Code;
use redox_lexer::{Keyword, Span, Token, TokenKind, Trivia};

/// The recursive-descent parser.
///
/// Private: [`crate::parse`] and [`crate::parse_source`] are the only ways in.
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) grew it from plan 01's three
/// productions to the expression, signature and binding surface, with the
/// expression grammar a Pratt loop over `crate::parse::precedence`.
///
/// Errors recover at **two** levels: an item boundary, and a statement
/// boundary within a block. The second is what stops one bad statement from
/// discarding its siblings — which matters for a language whose bodies are
/// indented blocks, and which `docs/COMPILER.md` §1 records LALRPOP never
/// offering at all.
pub(crate) struct Parser<'a> {
  toks: &'a [Token],
  pos: usize,
  errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
  pub(crate) fn new(toks: &'a [Token]) -> Self {
    Self {
      toks,
      pos: 0,
      errors: Vec::new(),
    }
  }

  /// Parse the whole token stream, consuming the parser.
  pub(crate) fn run(mut self, src_len: usize) -> Result<Program, Vec<ParseError>> {
    let items = self.program();

    // Doc 03 §7.6's `RDX0044` — plan 30. A whole-program pass rather than a
    // check inside the block parser: `return_in_tail` recurses into the blocks
    // a statement owns, so running it per block would report a nested tail's
    // `return` twice. One entry point per body is the shape that cannot
    // double-report.
    //
    // Run only when parsing otherwise succeeded, so a file with a syntax error
    // reports the syntax error rather than a second diagnostic about a tail the
    // parser may have recovered into.
    if self.errors.is_empty() {
      let probe = Program {
        items: items.clone(),
        span: (0, src_len),
        trailing: Vec::new(),
      };
      self.errors.extend(check_callback_tails(&probe));
    }

    if self.errors.is_empty() {
      let trailing = self.eof_trivia();
      Ok(Program {
        items,
        span: (0, src_len),
        trailing,
      })
    } else {
      Err(self.errors)
    }
  }

  fn program(&mut self) -> Vec<Item> {
    let mut items = Vec::new();
    self.skip_separators();
    while !self.at_eof() {
      match self.item() {
        Ok(item) => items.push(item),
        Err(e) => {
          self.errors.push(e);
          self.recover_to_item();
        }
      }
      self.skip_separators();
    }
    items
  }

  /// The items of an indented block, to the matching `Dedent`.
  ///
  /// Plan 14's `module` body. The same item-level recovery `program` uses —
  /// a bad item is reported and skipped and its siblings are still parsed —
  /// because a module body is a list of items and wants exactly that.
  pub(crate) fn items_until_dedent(&mut self) -> Result<Vec<Item>, ParseError> {
    let mut items = Vec::new();
    loop {
      self.skip_separators();
      if self.check(&TokenKind::Dedent) || self.at_eof() {
        break;
      }
      match self.item() {
        Ok(item) => items.push(item),
        Err(e) => {
          self.errors.push(e);
          self.recover_to_item();
        }
      }
    }
    self.expect(&TokenKind::Dedent, "expected the module body to end")?;
    Ok(items)
  }

  /// ```text
  /// Item := Ident "::" Spine Newline
  ///         Ident Binder* "=" Newline Indent Stmt+ Dedent
  /// ```
  ///
  /// Doc 18 §§2.2–2.3 — the signature line and the equation. Two lines where
  /// doc 03 had one, and the trade doc 18 §10.1 item 2 flags as the thing to
  /// watch: a one-line private helper now costs two lines. What it buys is
  /// that the types are written once, in Haskell's order, and the equation is
  /// a row of binders rather than a repeated annotation.
  ///
  /// The two names must match. Catching that here rather than at the emitter
  /// is what makes a mistyped equation a diagnostic about the equation
  /// instead of an undefined-function error from rustc about a line the user
  /// did not write.
  fn item(&mut self) -> Result<Item, ParseError> {
    let leading = self.peek_leading();
    let start = self.peek_span().0;
    // Doc 03 §9.6's marker comes first, before the keyword that says which
    // declaration this is — so it is read before the dispatch rather than
    // once per branch.
    let visibility = visibility(self)?;

    // Doc 18 §2.4's declarations are told apart by their first token, which
    // is a reserved word — so one token of lookahead settles which item this
    // is, with no backtracking and no ambiguity against a function's
    // signature line (a function begins with a *name*).
    if self.check(&TokenKind::Kw(Keyword::Data)) || self.check(&TokenKind::Kw(Keyword::Newtype)) {
      let decl = data_decl(self, leading, start, visibility)?;
      self.newline_with_trailing("expected a line break after the declaration")?;
      return Ok(Item::Data(decl));
    }
    if self.check(&TokenKind::Kw(Keyword::Type)) {
      let alias = type_alias(self, leading, start, visibility)?;
      self.newline_with_trailing("expected a line break after the alias")?;
      return Ok(Item::Type(alias));
    }
    if self.check(&TokenKind::Kw(Keyword::Const)) || self.check(&TokenKind::Kw(Keyword::Static)) {
      let decl = const_decl(self, leading, start, visibility)?;
      self.newline_with_trailing("expected a line break after the binding")?;
      return Ok(Item::Const(decl));
    }
    if self.check(&TokenKind::Kw(Keyword::Import)) {
      let decl = import_decl(self, leading, start, visibility)?;
      self.newline_with_trailing("expected a line break after the import")?;
      return Ok(Item::Import(decl));
    }
    if self.check(&TokenKind::Kw(Keyword::Module)) {
      let decl = module_decl(self, leading, start, visibility)?;
      self.newline_with_trailing("expected a line break after the module")?;
      return Ok(Item::Module(decl));
    }
    if self.check(&TokenKind::Kw(Keyword::Test)) {
      let decl = test_decl(self, leading, start)?;
      self.newline_with_trailing("expected a line break after the test")?;
      return Ok(Item::Test(decl));
    }
    if self.check(&TokenKind::Kw(Keyword::Errors)) {
      let decl = errors_decl(self, leading, start, visibility)?;
      self.newline_with_trailing("expected a line break after the declaration")?;
      return Ok(Item::Errors(decl));
    }
    if self.check(&TokenKind::Kw(Keyword::Class)) {
      let decl = class_decl(self, leading, start, visibility)?;
      self.newline_with_trailing("expected a line break after the class")?;
      return Ok(Item::Class(decl));
    }
    // An `impl` has no visibility in Rust and a test is never public, so
    // neither takes the marker — and a marker before one is rejected by the
    // `=` that follows, which is the right error for writing it.
    if self.check(&TokenKind::Kw(Keyword::Instance)) {
      let decl = instance_decl(self, leading, start)?;
      self.newline_with_trailing("expected a line break after the instance")?;
      return Ok(Item::Instance(decl));
    }

    let decl = self.function(&[])?;
    Ok(Item::Fn(FnDecl {
      leading,
      visibility,
      span: (start, decl.span.1),
      ..decl
    }))
  }

  /// A signature line and its equations — doc 18 §§2.2–2.3.
  ///
  /// Lifted out of [`Parser::item`] when doc 18 §2.5's class and instance
  /// bodies arrived, because a method **is** a function: the same signature
  /// line, the same equations, the same guard ladders. Sharing the code is
  /// what gives a default method several clauses and a guard ladder without
  /// anything new being written for it.
  ///
  /// `reserved` names type variables belonging to the *enclosing*
  /// declaration rather than to this function — a class's variable, which
  /// must not become a generic parameter of each of its methods. A top-level
  /// function passes nothing.
  ///
  /// **Zero equations is legal here**, and that is the one difference from a
  /// top-level function. `area :: &t -> f64` with no body is a *required*
  /// method, which Rust writes `fn area(&self) -> f64;` — so the caller, not
  /// this, decides whether a bodiless signature is an error.
  pub(crate) fn function(&mut self, reserved: &[String]) -> Result<FnDecl, ParseError> {
    let leading = self.peek_leading();
    let start = self.peek_span().0;

    let name = self.ident("expected a name to begin a declaration")?;
    // Doc 01 §4.3's `PI = 3.14159`, which doc 03 §13.4 corrected out of
    // existence. The generic "expected `::`" is true and unhelpful: the
    // author followed a published specification, so the message names the
    // correction instead of the token.
    //
    // **The value has to be on the same line.** `main =` followed by a line
    // break is a *function* whose signature line is missing, and that wants
    // the `::` message — the first version of this check fired on it too and
    // `reports_a_missing_signature_marker` caught it.
    let binds_on_this_line = self.check(&TokenKind::Eq)
      && self
        .peek_at(1)
        .is_some_and(|t| t.kind != TokenKind::Newline);
    if binds_on_this_line {
      return Err(ParseError::with(
        Code::ModuleScopeBinding,
        format!(
          "a module-scope binding needs `const` or `static` and a type: \
           `const {} :: T = …`. Rust forbids `let` at module scope, and doc \
           03 §13.4 corrects doc 01 §4.3 accordingly",
          name.text
        ),
        name.span,
      ));
    }
    self.expect(
      &TokenKind::ColonColon,
      "expected `::` after the name, to begin the signature",
    )?;
    // Doc 18 §2.2's `=>` clause. Read speculatively, because
    // `f :: Ord t => &[t] -> &t` and `f :: &[t] -> &t` begin with the same
    // token — see `constraints`.
    let bounds = constraints(self)?;
    // Doc 18 §2.2's `const fn` marker, consumed **before** the spine: a type
    // is verbatim text, so leaving `const` in one would make the first
    // parameter's type `const u32`, which is not a Rust type.
    let is_const = self.eat_keyword(Keyword::Const);
    let spine = parse_spine(self)?;
    self.newline_with_trailing("expected a line break after the signature")?;

    // The equations. Each repeats the name — doc 18 §2.3.
    let mut clauses = Vec::new();
    let mut sig = None;
    loop {
      // An equation begins with a name that is *not* followed by `::` — a
      // name that is starts the next declaration's signature line. That one
      // test tells three things apart: another clause of this function, a
      // new declaration, and (doc 18 §2.5) a required method with no
      // equation at all.
      let is_equation = matches!(&self.peek().kind, TokenKind::Ident(_))
        && !self
          .peek_at(1)
          .is_some_and(|t| t.kind == TokenKind::ColonColon);
      if !is_equation {
        break;
      }

      let again = self.ident("expected the equation to repeat the name")?;
      // The two names must match, and catching it here is what makes a
      // mistyped equation a diagnostic *about the equation* rather than an
      // undefined-function error from rustc about a line the user did not
      // write. An equation whose name differs is reported rather than left
      // to be misread as a new declaration, which is what it looks like to
      // one token of lookahead.
      if again.text != name.text {
        return Err(ParseError::with(
          Code::ExpectedName,
          format!(
            "this equation defines `{}`, but the signature above it is for `{}`",
            again.text, name.text
          ),
          again.span,
        ));
      }

      let row = patterns(self)?;
      // Every clause is checked against the same spine, so a row of the wrong
      // length is reported per clause rather than once for the function.
      let built = signature_from(&spine, &row, (again.span.0, self.prev_span().1))?;
      if sig.is_none() {
        sig = Some(built);
      }

      clauses.push(self.clause(row)?);
      self.skip_separators();
    }

    // With no equation there is no pattern row, so the signature comes from
    // the spine alone and the parameters take generated names. Not
    // `signature_from` with an empty row: that reports an arity error, which
    // is the right answer for `area = …` under `area :: &t -> f64` and the
    // wrong one for a required method with no equation at all.
    let mut sig = sig.unwrap_or_else(|| signature_only(&spine, (name.span.0, self.prev_span().1)));
    // The generic list is synthesised from the types, not written — doc 18
    // §5.3. Done here rather than in the two signature builders because it
    // needs the constraint clause, which only this knows about.
    generalise(&mut sig, &bounds, reserved);
    sig.is_const = is_const;

    let end = self.prev_span().1;
    let trailing = clauses.last().and_then(|c| c.trailing.clone());
    Ok(FnDecl {
      name,
      sig,
      clauses,
      // A method's marker is read by `class_member`; a top-level function's
      // by `item`, which replaces this.
      visibility: Visibility::Private,
      span: (start, end),
      leading,
      trailing,
    })
  }

  /// One clause's right-hand side, after its pattern row.
  ///
  /// Either `= body` or doc 18 §2.3's guard ladder. A guard list arrives on
  /// the *next* line, indented, so the newline and `Indent` are consumed
  /// before the `|` can be seen — which is why the two forms are told apart
  /// after the layout tokens rather than before them.
  fn clause(&mut self, patterns: Vec<redox_ast::Pattern>) -> Result<redox_ast::Clause, ParseError> {
    let start = patterns.first().map_or(self.peek_span().0, |p| p.span().0);

    if self.opens_a_guard_list() {
      self.advance();
      self.advance();
      let guards = self.guards()?;
      self.expect(&TokenKind::Dedent, "expected the guard list to end")?;
      let wheres = self.where_locals()?;
      return Ok(redox_ast::Clause {
        patterns,
        body: redox_ast::ClauseBody::Guarded(guards),
        wheres,
        span: (start, self.prev_span().1),
        trailing: None,
      });
    }

    self.expect(
      &TokenKind::Eq,
      "expected `=` after the equation's parameters, or an indented `|` guard list",
    )?;

    // Doc 18 §2.3's own examples put short bodies on the `=` line — `fib 0 =
    // 0` — and reserve the indented block for bodies that need statements.
    // Both are the same construct, so both are read here rather than in two
    // places that could drift apart.
    if !self.check(&TokenKind::Newline) {
      let body = self.inline_body()?;
      let trailing = self.newline_with_trailing("expected a line break after the body")?;
      let wheres = self.where_locals()?;
      return Ok(redox_ast::Clause {
        patterns,
        body: redox_ast::ClauseBody::Plain(body),
        wheres,
        span: (start, self.prev_span().1),
        trailing,
      });
    }

    let trailing = self.newline_with_trailing("expected a line break after `=`")?;
    self.expect_with(
      Code::EmptyBody,
      &TokenKind::Indent,
      "expected an indented body",
    )?;

    let block = self.statements()?;

    let wheres = self.where_locals()?;
    Ok(redox_ast::Clause {
      patterns,
      body: redox_ast::ClauseBody::Plain(block.clone()),
      wheres,
      span: (start, self.prev_span().1),
      trailing,
    })
  }

  /// The statements of an indented block, from just inside the `Indent` to
  /// the matching `Dedent`.
  ///
  /// Lifted out of [`Parser::clause`] when doc 03 §15's `test` block arrived,
  /// because a test's body **is** a clause body: same statements, same
  /// recovery, same emptiness rule. Sharing it is what gives a test the draw
  /// arrows and `case` expressions that need layout of their own, with
  /// nothing written for them.
  ///
  /// Statement-level recovery lives here: a bad statement is reported and
  /// skipped, and its siblings are still parsed.
  fn statements(&mut self) -> Result<Block, ParseError> {
    let start = self.peek_span().0;
    let mut stmts = Vec::new();
    loop {
      self.skip_separators();
      if self.check(&TokenKind::Dedent) || self.at_eof() {
        break;
      }
      match self.stmt() {
        Ok(stmt) => stmts.push(stmt),
        Err(e) => {
          self.errors.push(e);
          self.recover_to_stmt();
        }
      }
    }
    let end = self.prev_span().1;
    self.expect(&TokenKind::Dedent, "expected the body to end")?;

    if stmts.is_empty() && self.errors.is_empty() {
      return Err(ParseError::with(
        Code::EmptyBody,
        "a declaration body must contain at least one statement",
        (start, start),
      ));
    }
    Ok(Block {
      stmts,
      span: (start, end),
    })
  }

  /// An `Indent`, its statements, and the matching `Dedent`.
  ///
  /// What a caller wants when the block is the whole construct — a `test`
  /// body — rather than one of several shapes a clause can take.
  pub(crate) fn indented_block(&mut self, code: Code, message: &str) -> Result<Block, ParseError> {
    self.expect_with(code, &TokenKind::Indent, message)?;
    self.statements()
  }

  /// An indented block of statements, for a construct whose body is one.
  ///
  /// `indented_block` with a line break in front of it, which every
  /// control-flow body needs and no other caller does: an `if`'s condition
  /// ends its line, and the block opens on the next.
  pub(crate) fn statement_block(&mut self, message: &str) -> Result<Block, ParseError> {
    self.newline_with_trailing("expected a line break before the body")?;
    self.indented_block(Code::EmptyBody, message)
  }

  /// Whether the statement ends here — a line break, a dedent, or the end of
  /// the file.
  ///
  /// What tells `break` from `break v`, and the same test `return` already
  /// made inline. Named once now that two statements ask it.
  pub(crate) fn at_statement_end(&self) -> bool {
    self.check(&TokenKind::Newline) || self.check(&TokenKind::Dedent) || self.at_eof()
  }

  /// `if c` with an indented block, and its `else` — doc 01 §8, plan 20.
  ///
  /// `else if` is read by recursion: the `else` branch is a block holding one
  /// `If` statement, which is how Rust reads it and what lets the emitter
  /// write `} else if … {` without a third node.
  pub(crate) fn if_statement(&mut self) -> Result<StmtKind, ParseError> {
    self.expect(&TokenKind::Kw(Keyword::If), "expected `if`")?;
    let condition = condition(self)?;
    let then_block = self.statement_block("expected an indented `if` body")?;

    self.skip_separators();
    if !self.check(&TokenKind::Kw(Keyword::Else)) {
      return Ok(StmtKind::If {
        condition,
        then_block,
        otherwise: None,
      });
    }
    self.advance();

    // `else if` — one statement in a block of its own, so the chain is a
    // chain and the emitter needs no special case.
    if self.check(&TokenKind::Kw(Keyword::If)) {
      let start = self.peek_span().0;
      let nested = self.if_statement()?;
      let end = self.prev_span().1;
      return Ok(StmtKind::If {
        condition,
        then_block,
        otherwise: Some(Block {
          stmts: vec![Stmt {
            kind: nested,
            span: (start, end),
            leading: Vec::new(),
            trailing: None,
          }],
          span: (start, end),
        }),
      });
    }

    let otherwise = self.statement_block("expected an indented `else` body")?;
    Ok(StmtKind::If {
      condition,
      then_block,
      otherwise: Some(otherwise),
    })
  }

  /// Doc 18 §2.2's `where` locals, if the clause has any.
  ///
  /// ```text
  /// largest xs = best
  ///   where
  ///     best = xs |> iter |> max |> unwrap
  /// ```
  ///
  /// An indented `where`, then its own indented block of bindings. Two levels
  /// because that is how Haskell writes it and how the doc's example is
  /// written; the inner block is an ordinary statement block, so a local may
  /// be anything a statement may be.
  ///
  /// Returns an empty vector when there is no clause, so a caller can ask
  /// unconditionally — there are three clause shapes and each has to.
  fn where_locals(&mut self) -> Result<Vec<Stmt>, ParseError> {
    let opens = self.check(&TokenKind::Indent)
      && self
        .peek_at(1)
        .is_some_and(|t| t.kind == TokenKind::Kw(Keyword::Where));
    if !opens {
      return Ok(Vec::new());
    }
    self.advance();
    self.advance();
    self.newline_with_trailing("expected a line break after `where`")?;
    self.expect_with(
      Code::EmptyBody,
      &TokenKind::Indent,
      "expected indented `where` bindings",
    )?;

    let mut locals = Vec::new();
    loop {
      self.skip_separators();
      if self.check(&TokenKind::Dedent) || self.at_eof() {
        break;
      }
      locals.push(self.stmt()?);
    }
    self.expect(&TokenKind::Dedent, "expected the `where` bindings to end")?;
    self.skip_separators();
    self.expect(&TokenKind::Dedent, "expected the `where` clause to end")?;
    Ok(locals)
  }

  /// One expression as a body, for a clause or guard written on one line.
  ///
  /// A block of one tail statement, so the emitter has one body shape to
  /// handle rather than two — the inline form is a spelling, not a second
  /// kind of body.
  fn inline_body(&mut self) -> Result<Block, ParseError> {
    let start = self.peek_span().0;
    let value = parse_expr(self, 0)?;
    let end = self.prev_span().1;
    Ok(Block {
      stmts: vec![Stmt {
        kind: StmtKind::Expr(value),
        span: (start, end),
        leading: Vec::new(),
        trailing: None,
      }],
      span: (start, end),
    })
  }

  /// Whether a guard list follows: a line break, an indent, then `|`.
  ///
  /// Three tokens of lookahead rather than one, because a line break after
  /// the pattern row is also what a *forgotten* `=` looks like. Deciding on
  /// the break alone made `add a b` with no `=` report "expected at least one
  /// `|` guard" and then cascade into two more errors as the body was read as
  /// fresh items — a worse diagnostic than the one it replaced, for the
  /// likelier mistake. Doc 05 §4's rule is that the first error names the
  /// actual defect.
  fn opens_a_guard_list(&self) -> bool {
    self.check(&TokenKind::Newline)
      && self.peek_at(1).is_some_and(|t| t.kind == TokenKind::Indent)
      && self.peek_at(2).is_some_and(|t| t.kind == TokenKind::Or)
  }

  /// A guard ladder: `| cond = body` rows, in order.
  ///
  /// `otherwise` is Haskell's name for the final catch-all and is recognised
  /// here as a condition of `None`, so the emitter can write a bare `else`.
  /// Lowering it to `if true` instead would leave rustc unable to see that the
  /// ladder is total, and every guarded function would need a return the
  /// author did not write.
  fn guards(&mut self) -> Result<Vec<redox_ast::Guard>, ParseError> {
    let mut rows = Vec::new();
    loop {
      self.skip_separators();
      if !self.check(&TokenKind::Or) {
        break;
      }
      let start = self.peek_span().0;
      self.advance();

      // `otherwise` is a reserved word (doc 18 §7), not an identifier the
      // parser compares against: `| otherwise = …` must not be readable as a
      // comparison against a variable someone happened to name `otherwise`.
      let condition = if self.check(&TokenKind::Kw(Keyword::Otherwise)) {
        self.advance();
        None
      } else {
        Some(parse_expr(self, 0)?)
      };

      self.expect(&TokenKind::Eq, "expected `=` after a guard's condition")?;
      let body = self.inline_body()?;
      self.newline_with_trailing("expected a line break after a guard's body")?;

      rows.push(redox_ast::Guard {
        condition,
        body,
        span: (start, self.prev_span().1),
      });
    }

    debug_assert!(
      !rows.is_empty(),
      "`opens_a_guard_list` guarantees a first `|`"
    );
    Ok(rows)
  }

  /// `Stmt := Binding | "return" Expr? | Expr`, each closed by a `Newline`.
  fn stmt(&mut self) -> Result<Stmt, ParseError> {
    let leading = self.peek_leading();
    let start = self.peek_span().0;

    // Plan 20's control flow comes first: every one of its forms begins with
    // a keyword or a label, none of which can begin an expression, so asking
    // here costs one comparison and no backtracking.
    if let Some(kind) = control_flow(self)? {
      let end = self.prev_span().1;
      let trailing = self.newline_with_trailing("expected a line break after the statement")?;
      return Ok(Stmt {
        kind,
        span: (start, end),
        leading,
        trailing,
      });
    }

    let kind = if self.check(&TokenKind::Kw(Keyword::Assert)) {
      self.advance();
      StmtKind::Assert(parse_expr(self, 0)?)
    } else if self.check(&TokenKind::Kw(Keyword::Return)) {
      self.advance();
      if self.check(&TokenKind::Newline) || self.check(&TokenKind::Dedent) || self.at_eof() {
        StmtKind::Return(None)
      } else {
        StmtKind::Return(Some(parse_expr(self, 0)?))
      }
    } else if let Some(draw) = self.try_draw()? {
      draw
    } else if let Some(binding) = self.try_binding()? {
      StmtKind::Binding(binding)
    } else {
      let value = parse_expr(self, 0)?;
      // Doc 01 §4.3's compound assignment, read after the place expression
      // because the five tokens can only follow one. A plain `=` here is a
      // *binding*, which `try_binding` already took.
      match compound_assign(self) {
        Some(op) => StmtKind::Assign {
          target: value,
          op,
          value: parse_expr(self, 0)?,
        },
        None => StmtKind::Expr(value),
      }
    };

    let end = self.prev_span().1;
    let trailing = self.newline_with_trailing("expected a line break after the statement")?;
    Ok(Stmt {
      kind,
      span: (start, end),
      leading,
      trailing,
    })
  }

  /// Doc 30 §3.3's callback bind — `with conn = pool.with_connection`.
  ///
  /// Doc 03 §7.6's lowering, unchanged: *"rewrites the remainder of the
  /// enclosing block into a closure passed as the final argument"*. Only the
  /// spelling has moved, from §7.6's `use x <- f` via doc 18 §3.1's `x <- f`
  /// to this — because `<-` now means `await` and nothing else.
  ///
  /// Recognised by the keyword, so no lookahead at all. Doc 18 §3.1's version
  /// needed one token of it and a hard-won lexer fix: `x <- f` once lexed as
  /// `<` then `-`, parsed as `x < -f`, and emitted valid Rust comparing `x`
  /// against negated `f` with no diagnostic. A keyword cannot do that.
  fn try_draw(&mut self) -> Result<Option<StmtKind>, ParseError> {
    if !self.check(&TokenKind::Kw(Keyword::With)) {
      return Ok(None);
    }
    self.advance();

    let name = self.ident("expected a name after `with`")?;
    self.expect(
      &TokenKind::Eq,
      "expected `=` after a `with` binder — doc 30 §3.3",
    )?;
    let value = parse_expr(self, 0)?;
    Ok(Some(StmtKind::Draw { name, value }))
  }

  /// A binding, if the cursor is on one.
  ///
  /// Recognised by lookahead rather than by backtracking: `mut` opens one
  /// outright, and otherwise an `Ident` followed by `=` or `:` is a binding
  /// while anything else is an expression statement.
  fn try_binding(&mut self) -> Result<Option<Binding>, ParseError> {
    let is_mut = self.check(&TokenKind::Kw(Keyword::Mut));
    if !is_mut && !self.looks_like_a_binding() {
      return Ok(None);
    }

    let start = self.peek_span().0;
    if is_mut {
      self.advance();
    }
    // Doc 03 §6.5's destructuring — plan 33. A plain name is a
    // `Pattern::Binding` and takes the path it always took.
    let pattern = pattern_alt(self)?;

    let ty = if self.check(&TokenKind::Colon) {
      self.advance();
      Some(self.type_expr("expected a type after `:`")?)
    } else {
      None
    };

    self.expect(&TokenKind::Eq, "expected `=` in a binding")?;
    let value = parse_expr(self, 0)?;
    Ok(Some(Binding {
      pattern,
      ty,
      value,
      is_mut,
      span: (start, self.prev_span().1),
    }))
  }

  /// Whether a binding starts here — plan 33.
  ///
  /// A plain binding is an identifier then `=` or `:`, which is one token of
  /// lookahead. A **destructuring** binding opens with `(`, `[` or an
  /// uppercase name followed by `{`, and its `=` is an unknown distance away —
  /// so those three openers are recognised by their first token and the `=` is
  /// found by scanning to it at depth zero.
  ///
  /// Scanning rather than backtracking, for the reason plan 22's condition
  /// parser gives: a lookahead that reads token kinds and advances nothing
  /// cannot leave the parser somewhere unexpected when it answers no.
  fn looks_like_a_binding(&self) -> bool {
    match &self.peek().kind {
      TokenKind::Ident(_) => {
        matches!(
          self.peek_at(1).map(|t| &t.kind),
          Some(TokenKind::Eq) | Some(TokenKind::Colon) | Some(TokenKind::LBrace)
        ) && self.reaches_a_binding_eq()
      }
      TokenKind::LParen | TokenKind::LBracket => self.reaches_a_binding_eq(),
      _ => false,
    }
  }

  /// Whether a top-level `=` appears before this statement ends.
  ///
  /// What tells `(a, b) = p` from the expression `(a, b)`, and `Point { x } = p`
  /// from the record *literal* `Point { x = 1 }` — the literal's `=` is inside
  /// its braces, so it is not at depth zero.
  fn reaches_a_binding_eq(&self) -> bool {
    let mut depth = 0i32;
    let mut i = 0;
    while let Some(tok) = self.peek_at(i) {
      match &tok.kind {
        TokenKind::Eq if depth == 0 => return true,
        TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
        TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
          depth -= 1;
          if depth < 0 {
            return false;
          }
        }
        TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => {
          return false;
        }
        _ => {}
      }
      i += 1;
    }
    false
  }

  /// Capture a type as verbatim source text (plan 03 §1.1), reading to its
  /// terminator.
  pub(crate) fn type_expr(&mut self, message: &str) -> Result<TypeExpr, ParseError> {
    capture_type(self, message, TypeStop::Greedy)
  }

  // ---- token helpers, shared with the grammar functions ----

  pub(crate) fn peek(&self) -> &'a Token {
    // `lex` always terminates the stream with `Eof`, so the last token is a
    // safe clamp rather than an out-of-bounds risk.
    self.toks.get(self.pos).unwrap_or_else(|| {
      self
        .toks
        .last()
        .expect("the token stream always contains at least Eof")
    })
  }

  /// The token `n` positions ahead — the bounded lookahead `try_binding`
  /// needs, and which plan 19's call grammar will need too.
  pub(crate) fn peek_at(&self, n: usize) -> Option<&'a Token> {
    self.toks.get(self.pos + n)
  }

  pub(crate) fn peek_span(&self) -> Span {
    self.peek().span
  }

  fn peek_leading(&self) -> Vec<Trivia> {
    self.peek().leading.clone()
  }

  pub(crate) fn prev_span(&self) -> Span {
    if self.pos == 0 {
      return (0, 0);
    }
    self.toks[self.pos - 1].span
  }

  pub(crate) fn at_eof(&self) -> bool {
    matches!(self.peek().kind, TokenKind::Eof)
  }

  pub(crate) fn check(&self, kind: &TokenKind) -> bool {
    &self.peek().kind == kind
  }

  pub(crate) fn advance(&mut self) {
    if !self.at_eof() {
      self.pos += 1;
    }
  }

  pub(crate) fn expect(&mut self, kind: &TokenKind, message: &str) -> Result<(), ParseError> {
    self.expect_with(Code::ExpectedToken, kind, message)
  }

  /// [`Parser::expect`] with a code other than [`Code::ExpectedToken`].
  ///
  /// A missing token is usually nothing more specific than "the grammar
  /// wanted something else here", but a few are worth their own code because
  /// a reader recognises them: a declaration with no body is `RDX0105`, not a
  /// generic missing `Indent`.
  pub(crate) fn expect_with(
    &mut self,
    code: Code,
    kind: &TokenKind,
    message: &str,
  ) -> Result<(), ParseError> {
    if self.check(kind) {
      self.advance();
      Ok(())
    } else {
      Err(ParseError::with(code, message, self.peek_span()))
    }
  }

  /// Consume `kw` if it is there, reporting whether it was.
  pub(crate) fn eat_keyword(&mut self, kw: Keyword) -> bool {
    if self.check(&TokenKind::Kw(kw)) {
      self.advance();
      return true;
    }
    false
  }

  /// The current position, for a speculative read that may have to rewind.
  ///
  /// Doc 18 §2.5's constraint clause needs it: `class Shape t` and
  /// `class Debug t => Shape t` begin identically, so the constraints are
  /// read and then abandoned if no `=>` arrives. Exposed as an opaque mark
  /// rather than as the index it is, so a caller cannot do arithmetic on it.
  pub(crate) fn position(&self) -> usize {
    self.pos
  }

  /// Return to a position taken by [`Parser::position`].
  pub(crate) fn rewind(&mut self, mark: usize) {
    self.pos = mark;
  }

  /// The identifier here, if there is one, without erroring if there is not.
  pub(crate) fn try_ident(&mut self) -> Option<Ident> {
    if let TokenKind::Ident(text) = &self.peek().kind {
      let ident = Ident {
        text: text.clone(),
        span: self.peek_span(),
      };
      self.advance();
      return Some(ident);
    }
    None
  }

  pub(crate) fn ident(&mut self, message: &str) -> Result<Ident, ParseError> {
    if let TokenKind::Ident(text) = &self.peek().kind {
      let ident = Ident {
        text: text.clone(),
        span: self.peek_span(),
      };
      self.advance();
      Ok(ident)
    } else {
      Err(ParseError::with(
        Code::ExpectedName,
        message,
        self.peek_span(),
      ))
    }
  }

  /// Consume a `Newline`, returning any comment that trailed its line.
  fn newline_with_trailing(&mut self, message: &str) -> Result<Option<Trivia>, ParseError> {
    if self.check(&TokenKind::Newline) {
      let trailing = self.peek().trailing.clone();
      self.advance();
      Ok(trailing)
    } else if self.at_eof() || self.check(&TokenKind::Dedent) || self.just_dedented() {
      // A file ending without a trailing newline, or a block whose last
      // statement is immediately followed by a dedent, is well-formed — and
      // so is a statement whose expression *was* a block.
      Ok(None)
    } else {
      Err(ParseError::expected(message, self.peek_span()))
    }
  }

  /// Whether the token just consumed closed an indented block.
  ///
  /// Doc 18 §2.3's `case … of` is the first expression with layout of its
  /// own: its arms are an indented block, and the `Dedent` that ends them has
  /// already ended the line. A caller that then demanded a `Newline` would
  /// reject `label age = case age of …` — which is exactly how this was
  /// found, by writing §2.3's example into an example file.
  ///
  /// Expressed as "the previous token was a `Dedent`" rather than "the body
  /// was a `case`", because the next construct with its own layout should not
  /// have to be added here too.
  pub(crate) fn just_dedented(&self) -> bool {
    self
      .pos
      .checked_sub(1)
      .and_then(|i| self.toks.get(i))
      .is_some_and(|t| t.kind == TokenKind::Dedent)
  }

  fn skip_separators(&mut self) {
    while self.check(&TokenKind::Newline) {
      self.advance();
    }
  }

  /// Trivia left over at `Eof` — a comment at the end of the file.
  fn eof_trivia(&self) -> Vec<Trivia> {
    self
      .toks
      .last()
      .filter(|t| matches!(t.kind, TokenKind::Eof))
      .map(|t| t.leading.clone())
      .unwrap_or_default()
  }

  /// Skip to the next item boundary: a top-level name with the indent stack
  /// unwound.
  fn recover_to_item(&mut self) {
    let mut depth = 0usize;
    loop {
      match &self.peek().kind {
        TokenKind::Eof => return,
        TokenKind::Indent => {
          depth += 1;
          self.advance();
        }
        TokenKind::Dedent => {
          depth = depth.saturating_sub(1);
          self.advance();
        }
        TokenKind::Ident(_) if depth == 0 => return,
        _ => self.advance(),
      }
    }
  }

  /// Skip to the end of the current statement, so its siblings still parse.
  fn recover_to_stmt(&mut self) {
    let mut depth = 0usize;
    loop {
      match &self.peek().kind {
        TokenKind::Eof => return,
        TokenKind::Newline if depth == 0 => {
          self.advance();
          return;
        }
        TokenKind::Dedent if depth == 0 => return,
        TokenKind::Indent => {
          depth += 1;
          self.advance();
        }
        TokenKind::Dedent => {
          depth = depth.saturating_sub(1);
          self.advance();
        }
        _ => self.advance(),
      }
    }
  }
}
