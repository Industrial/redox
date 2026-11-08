/// A half-open byte range into the source text.
///
/// Every token and every AST node carries one. Built in from the start rather
/// than added later: Emerald's plan 22 found a node type with no span at all
/// twenty-one plans after its diagnostics plan shipped, and the full fix was
/// by then judged too large (`docs/06 - Redox Code Architecture.md` §6).
pub type Span = (usize, usize);
