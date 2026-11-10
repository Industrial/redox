/// <reference types="tree-sitter-cli/dsl" />

// The Redox grammar for tree-sitter.
//
// Plan 41 (`docs/36 - 41 - Tree-sitter Grammar.md`). §1.2 is the decision that
// shapes every rule below: this grammar aims at **highlighting, folding and
// selection**, not at parity with `redox-parser`. The compiler is the
// authority on what parses, and a grammar claiming to be a second one would
// make every disagreement a bug report against the wrong component.
//
// So it is **precise about the off-side rule** — items, statements, blocks and
// the indent tokens the external scanner supplies — and **deliberately flat
// inside an expression**: an expression is a run of operands and operators,
// not a precedence cascade. That avoids a second copy of `BinaryOp::power`,
// which plan 34 §10.13 found is exactly the kind of table that drifts silently
// while both spellings still compile. A highlight query colours a token, not a
// tree shape, so the flatness costs nothing the queries wanted.
//
// What it does cost is stated in §1.2 and §6: this grammar cannot answer *is
// this program valid*, structural selection inside an expression is coarser
// than a full grammar's, and row 46's inventory must compare token kinds and
// block structure rather than productions.

module.exports = grammar({
	name: "redox",

	// The off-side rule, from `src/scanner.c`. Nothing else can produce these:
	// tree-sitter's lexer has no notion of a column.
	externals: ($) => [$._newline, $._indent, $._dedent, $._fenced_comment],

	// Horizontal space and comments are invisible everywhere. A **newline** is
	// not in this list, which is what makes the scanner's job possible: a
	// newline is significant where the grammar expects one of the three
	// externals, and whitespace where it does not — so a bracketed continuation
	// line works without the scanner knowing about brackets.
	extras: ($) => [/[ \t\r]/, $.comment],

	word: ($) => $.identifier,

	// Ambiguities that are **the language's**, not this grammar's, and which
	// tree-sitter's GLR resolves by exploring both readings.
	//
	// `f (x) = y` is an equation and `f (x)` is a call, and the two are
	// indistinguishable until the `=` — which is exactly how `redox-parser`
	// resolves it: `looks_like_a_binding` scans forward to an `=` at depth zero
	// because, in its own words, *"its `=` is an unknown distance away"*. A
	// conflict here is the same decision made by a different mechanism, and
	// declaring it is more honest than a precedence that would silently prefer
	// one reading.
	conflicts: ($) => [
		[$._expression, $.section],
		[$._expression_part, $.section],
		[$._item, $._statement],
		[$._pattern, $._expression_part],
		[$.binding, $._expression_part],
		[$.binding],
		[$.equation, $._expression_part],
		[$.equation, $._pattern, $._expression_part],
		[$.equation, $._pattern],
		[$.equation],
		[$.expression_statement],
		[$.label, $.lifetime],
		[$.parenthesized_pattern, $.parenthesized],
		[$.record_pattern, $.record_literal],
		[$.record_pattern_field, $.record_literal_field],
		[$.section, $.parenthesized],
		[$.signature, $._expression_part],
		[$.signature, $._pattern, $._expression_part],
		[$.slice_pattern, $.list],
		[$.type_expression],
	],

	rules: {
		source_file: ($) => repeat(choice($._item, $._newline)),

		// ---------------------------------------------------------------- items

		_item: ($) =>
			choice(
				$.import,
				$.module,
				$.data_declaration,
				$.type_alias,
				$.class_declaration,
				$.instance_declaration,
				$.errors_declaration,
				$.test_declaration,
				$.const_declaration,
				$.extern_block,
				$.inline_rust,
				$.signature,
				$.equation,
				$.attribute,
			),

		// `export`, `export(crate)`, `export(super)`, `export(in path)`. Redox
		// spells this where Rust says `pub`.
		//
		// A **token**, not a rule: as a rule its optional `(crate)` competed
		// with an equation's parenthesised parameter, and
		// `export(crate) type Depth = f64` came out as an ERROR on `export`
		// plus a pattern `(crate)`. Lexing it as one unit removes the ambiguity
		// instead of declaring a conflict for it, which is the right trade for a
		// form with no internal structure any query wants.
		visibility: (_) =>
			token(
				/export(\((crate|super|in [A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*)\))?/,
			),

		import: ($) =>
			seq(
				optional($.visibility),
				"import",
				field("path", $.import_path),
				// The alias is usually a **type**: `… as Sorted`. Wanting a lowercase
				// identifier made `Sorted` the error and read as a problem with `as`.
				optional(
					seq("as", field("alias", choice($.type_identifier, $.identifier))),
				),
				$._newline,
			),

		// Not `path`: an import may end in a brace list — `std::fmt::{self, Write}`
		// — which no expression path does.
		import_path: ($) =>
			token(/[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_*]*)*(::\{[^}]*\})?/),

		module: ($) =>
			seq(
				optional($.visibility),
				"module",
				field("name", $.identifier),
				choice($.block, $._newline),
			),

		// `data Name = A | B`, and the indented form. Both shapes are the
		// author's choice and the grammar keeps both, which is the same
		// conclusion plan 34 §10.30 reached from the other side.
		// Plan 45 leaf 1. The body is one string token and stays opaque: Redox
		// never parses it, so a grammar that looked inside would be claiming to
		// know Rust. Highlighting it as a string is honest and is what an editor
		// can act on.
		inline_rust: ($) => seq("rust", field("code", $.string), $._newline),

		// Plan 45 leaf 3. The declarations are signatures, which the grammar
		// already has a rule for — doc 01 §26's FFI requirement needs no shape of
		// its own beyond the block.
		extern_block: ($) =>
			seq(
				"extern",
				field("abi", $.string),
				$._newline,
				$._indent,
				repeat1(choice($.signature, $._newline)),
				$._dedent,
			),

		data_declaration: ($) =>
			seq(
				optional($.visibility),
				choice("data", "newtype", "union"),
				field("name", $.type_identifier),
				choice(
					seq(
						"=",
						$._constructors,
						choice(
							seq(optional($.deriving), $._newline),
							// On its own indented line, which is what the corpus writes:
							// `data Point = Point { … }` then `  deriving (Debug)`.
							// Terminated by the **dedent**: after `deriving (Debug)` the
							// next real line is at column zero, so the scanner emits
							// DEDENT and never a NEWLINE.
							seq($._indent, $.deriving, $._dedent),
						),
					),
					seq($._indent, $._alternatives, $._dedent),
				),
			),

		_constructors: ($) => sep1($.constructor, "|"),

		_alternatives: ($) =>
			seq(
				seq("=", $.constructor, $._newline),
				repeat(seq("|", $.constructor, $._newline)),
				optional(seq($.deriving, $._newline)),
			),

		constructor: ($) =>
			seq(
				field("name", $.type_identifier),
				optional(choice($.record_fields, repeat1($.type_expression))),
			),

		record_fields: ($) =>
			seq("{", optional(sep1($.field_declaration, ",")), "}"),

		// Plan 52: `:` declares the type and `=` supplies the value, which makes
		// `data Point = Point { x : f64, y : f64 }` Rust's struct syntax with the
		// token unchanged.
		field_declaration: ($) =>
			seq(
				optional($.visibility),
				field("name", $.identifier),
				":",
				field("type", $.type_expression),
			),

		deriving: ($) => seq("deriving", "(", sep1($.type_identifier, ","), ")"),

		type_alias: ($) =>
			seq(
				optional($.visibility),
				"type",
				field("name", $.type_identifier),
				"=",
				field("type", $.type_expression),
				$._newline,
			),

		class_declaration: ($) =>
			seq(
				optional($.visibility),
				"class",
				optional($.constraints),
				field("name", $.type_identifier),
				field("variable", $.identifier),
				choice($.block, $._newline),
			),

		instance_declaration: ($) =>
			seq(
				"instance",
				optional($.constraints),
				field("target", repeat1(choice($.type_identifier, $.identifier))),
				choice($.block, $._newline),
			),

		constraints: ($) =>
			prec(1, seq(sep1(seq($.type_identifier, $.identifier), ","), "=>")),

		errors_declaration: ($) =>
			seq(
				optional($.visibility),
				"errors",
				field("name", $.type_identifier),
				choice($._newline, seq($._indent, repeat1($.error_variant), $._dedent)),
			),

		error_variant: ($) =>
			seq(
				choice(
					seq("from", $._path_text, "as", field("name", $.type_identifier)),
					seq(field("name", $.type_identifier), repeat($.type_expression)),
				),
				field("message", $.string),
				$._newline,
			),

		test_declaration: ($) =>
			seq("test", field("description", $.string), choice($.block, $._newline)),

		const_declaration: ($) =>
			seq(
				optional($.visibility),
				choice("const", "static"),
				// SCREAMING_CASE, so it lexes as a `type_identifier`. A rule wanting a
				// lowercase name rejected every constant in the corpus.
				field("name", choice($.type_identifier, $.identifier)),
				":",
				field("type", $.type_expression),
				// The value is **optional**: a `class` member declares
				// `const SIDES : u32` and an `instance` supplies the `= 4`. rustc
				// decides which is allowed where, as AGENTS.md prescribes, and a rule
				// that required one errored on every class in the corpus.
				optional(seq("=", field("value", $._expression))),
				$._newline,
			),

		// A function's type signature line. Told apart from an equation by the
		// `:`, which is the same single token the parser uses — plan 52 made it
		// `:` in every type position, so this is also what a typed binding and an
		// ascription are spelled with. The two readings of `total : i32 = 42` are
		// a declared conflict rather than a precedence, for the reason the header
		// gives about `f (x) = y`: which one it is depends on a `=` an unknown
		// distance away.
		signature: ($) =>
			seq(
				optional($.visibility),
				field("name", $.identifier),
				":",
				field("type", $.signature_type),
				$._newline,
			),

		// Everything after the `:`, as one node. Not decomposed into parameter
		// and return types: a signature's arrows, constraints and markers are a
		// type-level language of their own, and §1.2's answer is that a highlight
		// query wants the tokens rather than the shape.
		signature_type: ($) =>
			repeat1(
				choice(
					$.type_identifier,
					$.identifier,
					$.lifetime,
					// An array's length is a literal or a constant's name, so a
					// signature holds numbers too — `zeros : [u8; 4]`.
					$.number,
					"->",
					"=>",
					"async",
					"const",
					// Plan 45 leaf 2's `unsafe fn` marker, in the signature for
					// `async`'s reason: it sits inside the type run.
					"unsafe",
					$._type_punctuation,
				),
			),

		_type_punctuation: (_) =>
			token(
				choice(
					"(",
					")",
					"[",
					"]",
					"<",
					">",
					",",
					"::",
					";",
					"&",
					"*",
					"'",
					"{",
					"}",
				),
			),

		// `name pats = body`, with an inline or indented body, an optional guard
		// ladder, and an optional `where`.
		equation: ($) =>
			seq(
				optional($.visibility),
				field("name", $.identifier),
				field("parameters", repeat($._pattern)),
				choice(
					seq(
						"=",
						field("body", $._expression),
						// A `where` **replaces** the newline: its `_indent` absorbs the
						// line break, so asking for a newline first consumed the
						// indentation it needed. `g = x` then `  where` is the shape.
						choice($._newline, $.where_clause),
					),
					// A block-bearing body ends with its own `_dedent`, so there is no
					// newline left for the equation to consume — `label age = case age of`
					// plus its arms. Spelled as a separate branch rather than by making
					// the newline optional: an optional significant token made this rule
					// ambiguous with itself.
					seq(
						"=",
						field("body", $._block_expression),
						optional($.where_clause),
					),
					seq(
						"=",
						$._indent,
						field("body", repeat1($._statement)),
						$._dedent,
						optional($.where_clause),
					),
					// The receiver on the `=` line and the pipeline's stages indented
					// beneath it — `examples/application.sdx`'s multi-line `|>` chain,
					// and the one layout it says parses.
					//
					// A **pre-existing** gap, found by this row's sweep and not caused
					// by it: the example arrived in plan 51, after the grammar's last
					// change, and the same file fails identically when its signatures
					// are written back as `::`. Without this branch the scanner's
					// INDENT (preferred over NEWLINE, `scanner.c` line 322) had only
					// `where_clause` to land in, so the body came out as an ERROR
					// reaching into the next line.
					seq(
						"=",
						field("body", $._expression),
						$._indent,
						repeat1($.continuation),
						$._dedent,
						optional($.where_clause),
					),
					seq($._indent, $.guard_ladder, $._dedent, optional($.where_clause)),
				),
			),

		// One indented continuation line of the expression above it. A run of
		// `_expression_part`s like any other — a stage opens with `|>`, which is
		// in the `operator` token — so it needs no shape beyond the line.
		continuation: ($) => seq($._expression, $._newline),

		guard_ladder: ($) => repeat1($.guard),

		guard: ($) =>
			seq(
				"|",
				field("condition", choice("otherwise", $._expression)),
				"=",
				field("body", $._expression),
				$._newline,
			),

		where_clause: ($) =>
			seq(
				$._indent,
				"where",
				$._indent,
				repeat1($._statement),
				$._dedent,
				$._dedent,
			),

		// --------------------------------------------------------- class members

		block: ($) =>
			seq($._indent, repeat1(choice($._item, $._statement)), $._dedent),

		// ----------------------------------------------------------- statements

		_statement: ($) =>
			choice(
				$.binding,
				$.assignment,
				$.if_statement,
				$.while_statement,
				$.for_statement,
				$.loop_statement,
				$.break_statement,
				$.continue_statement,
				$.return_statement,
				$.assert_statement,
				$.guard_statement,
				$.draw_statement,
				$.await_statement,
				$.expression_statement,
				$.attribute,
				$._newline,
			),

		binding: ($) =>
			seq(
				optional("mut"),
				// A **run**, so a destructuring binding can hold an `@` or an `or`.
				field("pattern", $.pattern_run),
				optional(seq(":", field("type", $.type_expression))),
				"=",
				field(
					"value",
					choice(seq($._expression, $._newline), $._block_expression),
				),
			),

		// Plan 54 made `:=` the assignment operator and left `=` as a binding,
		// so this rule now takes six operators rather than five. `:=` is in
		// `assignment_operator` beside the compound forms because Rust spells
		// all six with `=` and the compiler's own AST holds them in one node
		// with an optional `BinaryOp` — one rule here keeps the two parsers
		// agreeing on the shape.
		//
		// No precedence is needed against `:` or `::`: tree-sitter's lexer
		// takes the longest match, and `:=` is longer than `:` while neither
		// it nor `::` is a prefix of the other.
		assignment: ($) =>
			seq(
				field("target", $._expression),
				field("operator", $.assignment_operator),
				field("value", $._expression),
				$._newline,
			),

		// `??=` is listed and accepted by no rule in the compiler — doc 18 §7
		// retires it in favour of its expanded form, and the token survives
		// only so the grammar does not report an ERROR on source the lexer
		// still produces a token for.
		assignment_operator: (_) =>
			token(choice(":=", "+=", "-=", "*=", "/=", "%=", "??=")),

		// `prec.right` for the dangling `else`: once leaf 2 let an indented `if`
		// stand where a value is read, `x = if c then if d <block> else <block>`
		// became ambiguous about which `if` owns the `else`. Right-associative
		// binds it to the nearest one, which is what the Rust parser does by
		// greedy recursion — `if_statement` consumes its own `else` before
		// returning, so the outer `if_expression` then has none and says so.
		if_statement: ($) =>
			prec.right(
				seq(
					"if",
					field("condition", $.condition),
					field("consequence", $.statement_block),
					optional($.else_clause),
				),
			),

		else_clause: ($) =>
			seq(
				"else",
				choice(
					seq($.if_statement),
					seq(field("alternative", $.statement_block)),
				),
			),

		while_statement: ($) =>
			seq(
				optional($.label),
				choice("while", "until"),
				field("condition", $.condition),
				$.statement_block,
			),

		for_statement: ($) =>
			seq(
				optional($.label),
				"for",
				field("binder", $.identifier),
				"in",
				field("source", $._expression),
				$.statement_block,
			),

		loop_statement: ($) => seq(optional($.label), "loop", $.statement_block),

		statement_block: ($) => seq($._indent, repeat1($._statement), $._dedent),

		break_statement: ($) =>
			seq("break", optional($.label), optional($._expression), $._newline),
		continue_statement: ($) => seq("continue", optional($.label), $._newline),
		return_statement: ($) => seq("return", optional($._expression), $._newline),
		assert_statement: ($) => seq("assert", $._expression, $._newline),

		guard_statement: ($) =>
			seq(
				"guard",
				// A **run**: `guard Some v = o else` is two patterns juxtaposed.
				field("pattern", $.pattern_run),
				"=",
				field("source", $._expression),
				"else",
				$.statement_block,
			),

		draw_statement: ($) =>
			seq(
				"with",
				field("name", $.identifier),
				"=",
				field("value", $._expression),
				$._newline,
			),

		// `<-` is `await` and nothing else — doc 30.
		await_statement: ($) =>
			seq(
				optional(field("pattern", $._pattern)),
				"<-",
				field("value", $._expression),
				$._newline,
			),

		expression_statement: ($) =>
			choice(seq($._expression, $._newline), $._block_expression),

		// A condition is one or more operands joined by `&&`, where an operand
		// may be a pattern test — `Some v = o`. Kept as a rule rather than folded
		// into the flat expression because `locals.scm` wants the bindings a
		// pattern test introduces.
		condition: ($) => sep1(choice($.pattern_test, $._expression), "&&"),

		// The pattern is a **run**: `Some v` and `Poll::Ready v` are two
		// patterns juxtaposed, and a rule wanting one failed at the `=`.
		//
		// No `prec`, for the reason `section` lost its own: a static
		// resolution discards the reading the parser needs when the `=`
		// arrives. A real ambiguity is a declared conflict.
		pattern_test: ($) =>
			seq(field("pattern", $.pattern_run), "=", field("source", $._expression)),

		// ------------------------------------------------------------- patterns

		_pattern: ($) =>
			choice(
				$.wildcard,
				$.identifier,
				// `Poll::Ready v` — a constructor reached through a module is a path,
				// and `examples/async.sdx` writes one in a condition.
				$.path,
				$.type_identifier,
				$.number,
				$.string,
				$.char,
				$.rest_pattern,
				$.slice_pattern,
				$.record_pattern,
				$.parenthesized_pattern,
			),

		wildcard: (_) => "_",
		rest_pattern: (_) => "..",

		// The elements are **runs**: `[_head, rest @ ..]` has an `@` in one of
		// them, which a single pattern cannot hold.
		slice_pattern: ($) => seq("[", optional(sep1($.pattern_run, ",")), "]"),

		record_pattern: ($) =>
			prec(
				1,
				seq(
					field("path", $.type_identifier),
					"{",
					optional(sep1(choice($.record_pattern_field, $.rest_pattern), ",")),
					"}",
				),
			),

		record_pattern_field: ($) =>
			seq(
				field("name", $.identifier),
				optional(seq("=", field("pattern", $._pattern))),
			),

		// Every parenthesised pattern: a group, a tuple, an `or` alternation, an
		// `@` binding, a range. **One** rule, because the text does not tell a
		// one-element tuple from a group — plan 33 §10.3 settled that by the
		// **comma**, which is a distinction the AST needs and a highlighter does
		// not. Two rules made `(p)` ambiguous, and tree-sitter said so.
		parenthesized_pattern: ($) =>
			seq("(", optional(sep1($.pattern_run, ",")), optional(","), ")"),

		// A run of patterns and the operators that join them: what sits in a
		// constructor's argument list, a `case` arm's head, and each slot of the
		// rule above.
		// `prec.left` because a `repeat1` is ambiguous with itself: two adjacent
		// runs and one longer run are the same text. Left means the longest match
		// wins, which is the reading a reader has — `Some (Some v)` is one
		// pattern and not two sitting beside each other.
		pattern_run: ($) =>
			prec.left(repeat1(choice($._pattern, "or", "@", "..="))),

		// ---------------------------------------------------------- expressions

		// §1.2's flat rule: a run of operands and operators. Not a precedence
		// cascade, and not a mistake — see the file header.
		_expression: ($) => prec.left(repeat1($._expression_part)),

		_expression_part: ($) =>
			choice(
				$.conditional_expression,
				$.lambda,
				$.comprehension,
				$.section,
				$.parenthesized,
				$.list,
				$.record_literal,
				$.path,
				$.identifier,
				$.type_identifier,
				$.self_expression,
				$.number,
				$.string,
				$.raw_string,
				$.char,
				$.boolean,
				$.lifetime,
				$.ascription,
				$.operator,
				// Keywords, not part of the `operator` token: inside
				// `token(choice(…))` they form one anonymous token, which competed
				// with the literal `"as"` in the `import` rule and broke
				// `import … as Sorted`. A keyword belongs where tree-sitter's
				// extraction can see it as one.
				"catch",
				"as",
			),

		// A value that **is** a block: it ends with its own `_dedent` and so
		// terminates the statement holding it. Kept out of `_expression_part`
		// for that reason — a `case` in the middle of an operand run would have
		// no way to end, and it is not a shape the corpus has.
		_block_expression: ($) =>
			choice(
				$.case_expression,
				$.do_expression,
				$.async_block,
				$.unsafe_block,
				$.try_block,
			),

		// Block-bearing, so they keep real rules: folding wants them, and so does
		// anyone pressing expand-selection.
		case_expression: ($) =>
			seq(
				"case",
				field("scrutinee", $._expression),
				"of",
				$._indent,
				repeat1($.case_arm),
				$._dedent,
			),

		// Plan 55 leaf 3: an arm body is an expression, and an indented block is
		// one — so `->` takes either, with no marker in front of the block.
		//
		// The terminator moves inside the choice because the two shapes end
		// differently: an inline body ends on a `_newline`, and a
		// `statement_block` ends on its own `_dedent`, which has already ended
		// the line. That is the same pairing `expression_statement` makes, and
		// the same reason `Parser::just_dedented` exists on the Rust side.
		case_arm: ($) =>
			seq(
				field("pattern", $.pattern_run),
				optional(seq("|", field("guard", $.condition))),
				"->",
				choice(
					seq(field("body", $._expression), $._newline),
					// Criterion 3's other half: a compound assignment is a legal
					// arm body, and plan 54's `:=` with it. `assignment` is reused
					// rather than copied, and it carries the `_newline` that ends
					// the arm — `_expression` holds no `assignment_operator`, so
					// the two readings cannot compete.
					field("body", $.assignment),
					field("body", $.statement_block),
				),
			),

		do_expression: ($) => seq("do", optional($.label), $.statement_block),
		async_block: ($) => seq("async", $.statement_block),
		// Plan 45 leaf 2. The fourth construct of this shape.
		unsafe_block: ($) => seq("unsafe", $.statement_block),
		try_block: ($) => seq("try", $.statement_block),

		// Plan 55 leaf 2 made both `if` spellings usable as a value, so this
		// position takes either. The Rust parser tells them apart with a
		// depth-zero scan for `then` before the line break; GLR needs no such
		// test — it explores both and the `then` or the indent settles it.
		//
		// `if_statement` is reused rather than copied: since plan 55 leaf 1
		// the two spellings are one AST node, and giving the grammar two rules
		// for one node is how the highlighter and the compiler drift.
		conditional_expression: ($) =>
			choice(
				prec.left(
					seq(
						"if",
						field("condition", $.condition),
						"then",
						field("consequence", $._expression),
						"else",
						field("alternative", $._expression),
					),
				),
				// Negative, so the *statement* reading wins where both are
				// possible. That is the same preference the Rust parser has, and
				// it gets it for free by asking `control_flow` before it reads an
				// expression at all; GLR has to be told.
				prec(-1, $.if_statement),
			),

		lambda: ($) =>
			prec.left(
				seq(
					"\\",
					optional("move"),
					field(
						"parameters",
						repeat1(choice($.identifier, $.parenthesized_pattern, "mut")),
					),
					"->",
					field("body", $._expression),
				),
			),

		comprehension: ($) =>
			seq(
				"[",
				field("element", $._expression),
				"|",
				sep1(choice($.generator, $._expression), ","),
				"]",
			),

		generator: ($) =>
			seq(field("binder", $.identifier), "in", field("source", $._expression)),

		// `(> 0)`, `(0 <)`, `(.name)`, `(|> len)`. A real rule because
		// `highlights.scm` wants the operator coloured as one, and because the
		// parentheses are part of the form rather than grouping around it.
		section: ($) =>
			seq(
				"(",
				choice(
					seq(".", $.identifier),
					seq("|>", $.identifier),
					seq($.operator, $._expression_part),
					seq($._expression_part, $.operator),
				),
				")",
			),

		// A comma list, so `(a, b)` and `x.m (a, b)` both land here. The comma
		// used to ride in the `operator` token, which is how removing the
		// separators from that list took two more files down with it.
		// `: T` — doc 18 §6's ascription, and a **real rule** rather than a `:`
		// operator followed by whatever comes next. The type is a type:
		// `: Result<i32, core::num::ParseIntError>` has commas inside angle
		// brackets, and the flat expression run has no comma in it since the
		// separators left the `operator` token.
		ascription: ($) => seq(":", $.type_expression),

		parenthesized: ($) =>
			seq("(", optional(sep1($._expression, ",")), optional(","), ")"),
		list: ($) =>
			seq(
				"[",
				optional(sep1($._expression, ",")),
				optional(seq(";", $._expression)),
				"]",
			),

		record_literal: ($) =>
			prec(
				1,
				seq(
					// A plain variable too: `r = p { x = 99 }` is the functional-update
					// form over a binding, which `examples/record.sdx` writes.
					field("path", choice($.type_identifier, $.path, $.identifier)),
					"{",
					optional(sep1($.record_literal_field, ",")),
					"}",
				),
			),

		record_literal_field: ($) =>
			seq(
				field("name", $.identifier),
				optional(seq("=", field("value", $._expression))),
			),

		// A type, in the positions where a type is the only thing possible — a
		// record field, an alias, a `const`'s annotation, a constructor's
		// arguments.
		//
		// Unlike `signature_type`, this one has structure, and the difference is
		// the position rather than inconsistency. A signature is a type-level
		// language with arrows, constraints and markers, and §1.2's answer there
		// is that a query wants the tokens. Here a type is **one** type, and
		// `repeat1($.type_expression)` is how a constructor's juxtaposed
		// arguments are spelled — so the rule has to know where one ends.
		type_expression: ($) =>
			choice(
				$.lifetime,
				seq(
					choice($.type_identifier, $.path),
					"<",
					sep1($.type_expression, ","),
					">",
				),
				$.type_identifier,
				$.path,
				$.identifier,
				// `&'static str`, `&mut String`: the lifetime and the `mut` are both
				// optional and both precede the type.
				seq("&", optional($.lifetime), optional("mut"), $.type_expression),
				seq("*", $.type_expression),
				// A tuple type, and `()` — the unit type, and the commonest return
				// type in the corpus.
				seq("(", optional(sep1($.type_expression, ",")), ")"),
				// An array or a slice: `[u8; 4]`, `[i32]`.
				seq(
					"[",
					$.type_expression,
					optional(seq(";", choice($.number, $.identifier, $.type_identifier))),
					"]",
				),
			),

		// --------------------------------------------------------------- tokens

		operator: (_) =>
			token(
				choice(
					"+",
					"-",
					"*",
					"/",
					"%",
					"==",
					"!=",
					"<=",
					">=",
					"<",
					">",
					"&&",
					"||",
					"!",
					"&",
					"|",
					"^",
					"<<",
					">>",
					"~",
					"..",
					"..=",
					"...",
					"??",
					"|>",
					"$",
					"?",
					"?.",
					"@",
					"@@",
					".",
					"::",
					// `=`, `->`, `=>`, `,`, `;` and `:` are **separators**, not
					// operators, and a flat expression rule that accepted them ate the
					// construct around it: a guard's `| n % 2 == 0 = Some (n / 2)` had
					// its `=` taken by the condition, leaving the guard no body. Each
					// belongs to a rule that names it.
				),
			),

		// `r#type` is plan 45 leaf 4's raw identifier: a Rust keyword used as a
		// name. The `r#` alternative comes first so it is preferred over
		// reading the `r` as an ordinary identifier's first character.
		identifier: (_) => /r#[a-z_][A-Za-z0-9_]*|[a-z_][A-Za-z0-9_]*/,
		type_identifier: (_) => /[A-Z][A-Za-z0-9_]*/,
		path: (_) => token(/[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)+/),
		_path_text: (_) =>
			token(/[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*/),

		self_expression: (_) => token(choice("self", "Self")),
		label: (_) => token(/'[a-z_][A-Za-z0-9_]*/),
		lifetime: (_) => token(/'[a-z_][A-Za-z0-9_]*/),

		number: (_) =>
			token(/[0-9][0-9_]*(\.[0-9][0-9_]*)?([eE][-+]?[0-9]+)?[a-zA-Z0-9_]*/),
		boolean: (_) => token(choice("true", "false")),

		// **One regex each**, not a `token(seq(regex, regex, string))`: as a
		// sequence the raw form did not match at all and `r#"he said "yes""#`
		// lexed as the identifier `r`. A literal is the place where one regex
		// is both shorter and easier to check.
		//
		// A block string is tried first, or its opening quotes read as an empty
		// string followed by one more.
		string: (_) =>
			token(choice(/"""([^"]|"[^"]|""[^"])*"""/, /"([^"\\\n]|\\[^\n])*"/)),

		// `r"…"`, `r#"…"#`, and the `b` and `c` prefixes doc 03 §11.2 lists.
		raw_string: (_) =>
			token(
				choice(
					// `([^"]|"[^#])*"#` cannot match `r#"he said "yes""#`: the body's
					// last quote can only be taken by `"[^#]`, which then steals the
					// terminator's own. A greedy **run** of quotes ending the body
					// leaves one behind for it.
					/[bc]?r#"([^"]|"+[^#"])*"*"#/,
					/[bc]?r"[^"]*"/,
					/[bc]"([^"\\\n]|\\[^\n])*"/,
				),
			),

		char: (_) => token(seq(/[b]?'/, choice(/[^'\\]/, seq("\\", /./)), "'")),

		// Doc 18 §5.1's markers. `-- |` and `-- !` are doc comments and `{- -}` is
		// a fence; all four are one node, because a highlight query wants one
		// colour and the emitter is what cares about the difference.
		//
		// The fence is **external**, because it nests: `{- a {- b -} c -}` is one
		// comment and a regex cannot count. Getting it wrong miscolours everything
		// after a nested comment, which is the most visible kind of highlighting
		// bug there is — and `examples/lexical.sdx` has one.
		comment: ($) => choice($._line_comment, $._fenced_comment),

		_line_comment: (_) => token(seq("--", /[^\n]*/)),

		// `#[…]` and `#![…]`. Trivia to the compiler (plan 18 §1.3), a node here,
		// because an editor colours it and a fold may contain it.
		attribute: ($) =>
			seq(token(choice("#[", "#![")), /[^\]]*/, "]", $._newline),
	},
});

/// One or more `rule`, separated by `separator`.
function sep1(rule, separator) {
	return seq(rule, repeat(seq(separator, rule)));
}
