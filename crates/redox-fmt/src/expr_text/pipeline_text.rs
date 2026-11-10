/// The widest pipeline written on one line.
pub(crate) mod pipeline_width;

use crate::expr_text::operand::operand;

use crate::expr_text::pipeline_text::pipeline_width::PIPELINE_WIDTH;

use crate::stmt_text::indent_text::indent_text;

use redox_ast::{BinaryOp, Expr};

/// Doc 03 §5.3's receiver pipeline, on one line or one stage per line.
///
/// ```text
/// chain = items |> map (+1) |> filter (> 0)
///
/// evens_doubled xs = xs.iter ()
///   |> filter (\n -> *n % 2 == 0)
///   |> map (\n -> n * 2)
///   |> count
/// ```
///
/// # Why the pipeline is the one binary operator with a layout
///
/// Because it is the one the lexer continues. `next_line_continues` looks
/// ahead for a line opening with `|>` and swallows it, indentation and all,
/// before the off-side rule can see it — and only `|>` qualifies, since a
/// leading `.` introduces a `with` block's statements. So the broken form
/// above is not a style choice among several: it is the only multi-line
/// spelling of a pipeline that parses, which is what
/// `examples/application.sdx` says about itself.
///
/// Every other operator joins and stays joined, which is why this takes
/// `BinaryOp::Pipeline` out of the general `Binary` arm rather than
/// generalising that arm.
///
/// # The chain is flattened before its width is measured
///
/// `a |> f |> g` is left-associative, so it arrives as nested `Binary` nodes
/// with the stages hanging off the left spine in reverse. Printing it by
/// recursion would ask each nested node to decide its own layout from a width
/// that is only true of the whole chain, so the spine is walked here and the
/// decision taken once. The joined text is identical to what the recursion
/// produced, because each part is still printed as the operand it was: the
/// receiver at the operator's own power, every stage one rung tighter.
pub(crate) fn pipeline_text(lhs: &Expr, rhs: &Expr, indent: usize, source: &str) -> String {
  let power = BinaryOp::Pipeline.power();
  let mut stages = vec![operand(rhs, power + 1, indent, source)];
  let mut head = lhs;
  // The crate's one `while let`. `guard … is … else` does not replace it:
  // this walks a left-leaning `Binary` chain to its receiver and has to keep
  // going, not leave.
  while let Expr::Binary {
    op: BinaryOp::Pipeline,
    lhs: left,
    rhs: right,
    ..
  } = head
  {
    stages.push(operand(right, power + 1, indent, source));
    head = left;
  }
  stages.reverse();
  let receiver = operand(head, power, indent, source);
  let one_line = format!("{} |> {}", receiver, stages.join(" |> "));
  // A stage that is already multi-line forces the break too, which is what
  // the emitter's `chain` does with a multi-line segment and for the same
  // reason: there is no joined form left to keep.
  if indent_text(indent).len() + one_line.len() <= PIPELINE_WIDTH && !one_line.contains('\n') {
    return one_line;
  }
  // One level in from the statement, which is the indentation
  // `examples/application.sdx` writes and doc 03 §5.3's own example before it.
  let pad = indent_text(indent + 1);
  let mut out = receiver;
  for stage in &stages {
    out.push('\n');
    out.push_str(&pad);
    out.push_str("|> ");
    out.push_str(stage);
  }
  out
}
