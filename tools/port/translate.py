# -*- coding: utf-8 -*-
"""Rust to Redox, for the regular shapes `redox-ast` is made of.

**How the port was made, written down because it matters.** 38 of the crate's
53 files are pure data types — doc comments, derives, a `struct`, an `enum` or
a `type` — and those shapes are regular enough to translate mechanically. The
15 files carrying logic are done by hand.

The translator is not a general Rust front end and must not be mistaken for
one. It handles exactly what this crate contains, asserts rather than guesses
when it meets anything else, and every file it emits is read against its
original before it is kept.

Usage: `python3 tools/port/translate.py <relative/path.rs> …`
"""
import io
import os
import re
import sys

ROOT = '/home/tom/Code/rust/redox/'
_c = [a[6:] for a in sys.argv[1:] if a.startswith('crate=')]
CRATE = _c[0] if _c else 'redox-ast'
SRC = ROOT + 'crates/' + CRATE + '/src/'
OUT = ROOT + 'crates/' + CRATE + '/sdx/'

VIS = {
    'pub ': 'export ',
    'pub(crate) ': 'export(crate) ',
    'pub(super) ': 'export(super) ',
    '': '',
}


# **Keyword collisions get `r#`, rather than a parse error.** Owed since row 75
# and hit seven times by row 80: `data` (a fuzz parameter), `rust` (a `Pair`
# field, then a `Cached` field), `class` and `data` again (`redox-lsp`), and
# `with` (a `redox-driver` module). Each one arrived as `RDX0103` naming a line
# whose author chose the name freely in Rust, and the fix was always the same
# three characters.
#
# **The list is read out of the lexer, not retyped**, because a second copy of
# 57 keywords is a second copy free to drift.
#
# **Rust's own reserved words are in the union too.** Row 78 found `final`
# running the other way — reserved in *Rust* and not in Redox — so a binding
# called `final` parses and emits and then fails in rustc against the generated
# file. What a name must avoid is the union of both sets.
#
# `self`, `Self`, `crate` and `super` are excluded: `r#self` is not a legal raw
# identifier, and a Rust field or module cannot be called one of them anyway.
_KW_SDX = ROOT + 'crates/redox-lexer/sdx/token/keyword.sdx'
RUST_RESERVED = (
    'final', 'become', 'priv', 'typeof', 'unsized', 'virtual', 'abstract',
    'override', 'box', 'macro', 'yield', 'gen',
)
NEVER_RAW = ('self', 'Self', 'crate', 'super')
KEYWORDS = (
    set(re.findall(r'"([a-zA-Z_][a-zA-Z_0-9]*)" -> Some ',
                   io.open(_KW_SDX, encoding='utf-8').read()))
    | set(RUST_RESERVED)
) - set(NEVER_RAW)


def raw_ident(name):
    """`name`, prefixed with `r#` if it collides with either keyword set."""
    return 'r#' + name if name in KEYWORDS else name


def comment(line):
    """A Rust comment line as its Redox spelling, or `None`."""
    t = line.strip()
    for rust, redox in (('//!', '-- !'), ('///', '-- |'), ('//', '--')):
        if t == rust:
            return redox
        if t.startswith(rust + ' '):
            return redox + ' ' + t[len(rust) + 1:]
        if t.startswith(rust):
            # `///text` with no space, and `//!`-prefixed likewise.
            return redox + ' ' + t[len(rust):]
    return None


def vis_of(text):
    """Split a leading visibility off `text`, longest first."""
    for rust in ('pub(crate) ', 'pub(super) ', 'pub '):
        if text.startswith(rust):
            return VIS[rust], text[len(rust):]
    return '', text


class Fail(Exception):
    pass


def translate(path):
    rel = os.path.relpath(path, SRC)
    text = io.open(path, encoding='utf-8').read()
    lines = text.splitlines()

    # **A wrapped attribute is joined before anything reads the lines.**
    # `#[allow(\n  dead_code,\n  reason = "…"\n)]` is three lines, and the
    # top-level, field and variant scanners each match a single one — so
    # `reason = "…"` reached the variant scanner as a variant, `Fail` was
    # raised, and the file was skipped entirely. Plan 78 §2.1's gap one
    # construct over: `rustfmt` wraps an attribute past the width exactly as it
    # wraps a `use`, so the fix belongs ahead of every scanner rather than in
    # each of them.
    normalised = []
    at = 0
    while at < len(lines):
        line = lines[at]
        body = line.strip()
        if body.startswith('#['):
            pad = line[: len(line) - len(line.lstrip())]
            while body.count('[') != body.count(']') and at + 1 < len(lines):
                at += 1
                body += ' ' + lines[at].strip()
            body = re.sub(r'\s+', ' ', body)
            body = body.replace('( ', '(').replace(' )', ')').replace(', )', ')')
            normalised.append(pad + body)
            at += 1
            continue
        normalised.append(line)
        at += 1
    lines = normalised
    out = []
    pending = []          # comment lines waiting for the construct below
    derives = None
    i = 0
    n = len(lines)

    while i < n:
        raw = lines[i]
        line = raw.strip()

        if not line:
            out.extend(pending)
            pending = []
            out.append('')
            i += 1
            continue

        c = comment(raw)
        if c is not None:
            pending.append(c)
            i += 1
            continue

        # **An inner attribute passes through verbatim.** `#![allow(…)]` does
        # not start with `#[`, so without this branch it reached `PARTIAL`'s
        # TODO marker — which does not report a failure, it writes a *shorter
        # file*. `lib.sdx` lost its twenty `module` declarations and every
        # re-export to that, and the only symptom was a TODO count of 1.
        if line.startswith('#!['):
            out.extend(pending)
            pending = []
            out.append(line)
            i += 1
            continue

        m = re.match(r'#\[derive\((.*)\)\]$', line)
        if m:
            derives = m.group(1)
            i += 1
            continue

        if line.startswith('#['):
            # **`#[allow(unused_imports)]` is emitter output, never author
            # source.** Plan 68 made `data` emit it with the constructor glob,
            # so it can only reach a `.rs` file from the emitter — and it
            # reaches *this* translator because the Rust being read was itself
            # generated. Keeping it produces an orphan on whatever construct
            # follows, which is clippy's `empty_line_after_outer_attr`; and
            # because the orphan is then in the next round's input, pairing it
            # with the `use` below is not a fixpoint. Dropping it on sight is.
            if line == '#[allow(unused_imports)]':
                i += 1
                continue
            pending.append(line)
            i += 1
            continue

        # --- module declarations -------------------------------------------
        m = re.match(r'(pub(?:\(crate\)|\(super\))? )?mod ([a-z_0-9]+);$', line)
        if m:
            v, _ = vis_of(line)
            out.extend(pending)
            pending = []
            out.append('%smodule %s' % (v, raw_ident(m.group(2))))
            i += 1
            continue

        # --- imports --------------------------------------------------------
        # **A wrapped `use` is joined first.** `rustfmt` wraps one that exceeds
        # the width, and the match below is single-line — so a wrapped `use`
        # fell through to the generic token path, which lost the spaces around
        # `as` and sometimes dropped the statement entirely. Both are quiet:
        # the `.sdx` parses and rustc then reports a missing type in a file
        # whose author never touched the import.
        if re.match(r'(pub )?use [^;]*$', line):
            joined = line
            ahead = i + 1
            while ahead < n and not joined.rstrip().endswith(';'):
                joined += ' ' + lines[ahead].strip()
                ahead += 1
            # Whitespace runs collapse, and the trailing comma Rust allows
            # before `}` has no place in a Redox import path.
            joined = re.sub(r'\s+', ' ', joined)
            joined = joined.replace(', }', '}').replace(' }', '}')
            joined = joined.replace('{ ', '{')
            line = joined
            i = ahead - 1

        m = re.match(r'(pub )?use (.+);$', line)
        if m:
            # **`use Own::*;` is the emitter's output, not an import.** Plan 68
            # made `data` emit `#[allow(unused_imports)] use X::*;` itself, so
            # translating the pair back produces author source for something
            # the emitter will supply again: a duplicate glob the writer hoists
            # to the top, where no allow covers it, and an orphaned attribute
            # that lands on whatever construct follows. `unused_imports` and
            # `empty_line_after_outer_attr`, from one mechanism.
            #
            # Guarded on the type being **declared in this file** — a genuine
            # `use Foo::*;` naming someone else's enum is a real import and
            # still translates.
            g = re.match(r'^([A-Z][A-Za-z0-9_]*)::\*$', m.group(2))
            if g and re.search(r'^\s*(pub(\([a-z]+\))? )?enum %s[ <{]'
                               % re.escape(g.group(1)), text, re.M):
                out.extend(pending)
                pending = []
                i += 1
                continue
            v, _ = vis_of(line)
            out.extend(pending)
            pending = []
            out.append('%simport %s' % ('export ' if m.group(1) else '', m.group(2)))
            i += 1
            continue

        # --- type alias -----------------------------------------------------
        m = re.match(r'(pub(?:\(crate\))? )?type ([A-Za-z0-9_]+) = (.+);$', line)
        if m:
            v, _ = vis_of(line)
            out.extend(pending)
            pending = []
            out.append('%stype %s = %s' % (v, m.group(2), m.group(3)))
            i += 1
            continue

        # --- tuple struct ---------------------------------------------------
        m = re.match(r'(pub(?:\(crate\))? )?struct ([A-Za-z0-9_]+)\((.*)\);$', line)
        if m:
            v, _ = vis_of(line)
            inner = m.group(3)
            types = [vis_of(t.strip())[1] for t in split_top(inner)]
            out.extend(pending)
            pending = []
            out.append('%snewtype %s = %s %s' % (v, m.group(2), m.group(2), ' '.join(types)))
            if derives:
                out.append('  deriving (%s)' % derives)
                derives = None
            i += 1
            continue

        # --- braced struct --------------------------------------------------
        m = re.match(r'(pub(?:\(crate\))? )?struct ([A-Za-z0-9_]+) \{$', line)
        if m:
            v, _ = vis_of(line)
            name = m.group(2)
            out.extend(pending)
            pending = []
            out.append('%sdata %s = %s {' % (v, name, name))
            i += 1
            i = fields(lines, i, out)
            out.append('}')
            if derives:
                out.append('  deriving (%s)' % derives)
                derives = None
            continue

        # --- unit struct ----------------------------------------------------
        m = re.match(r'(pub(?:\(crate\))? )?struct ([A-Za-z0-9_]+);$', line)
        if m:
            v, _ = vis_of(line)
            out.extend(pending)
            pending = []
            out.append('%sdata %s = %s' % (v, m.group(2), m.group(2)))
            if derives:
                out.append('  deriving (%s)' % derives)
                derives = None
            i += 1
            continue

        # --- enum -----------------------------------------------------------
        m = re.match(r'(pub(?:\(crate\))? )?enum ([A-Za-z0-9_]+) \{$', line)
        if m:
            v, _ = vis_of(line)
            out.extend(pending)
            pending = []
            out.append('%sdata %s' % (v, m.group(2)))
            i += 1
            i = variants(lines, i, out)
            if derives:
                out.append('  deriving (%s)' % derives)
                derives = None
            continue

        if PARTIAL:
            out.extend(pending)
            out.append('')
            out.append('-- TODO plan 68: hand-port from line %d' % (i + 1))
            return '\n'.join(out).rstrip('\n') + '\n'
        raise Fail('%s:%d: no rule for %r' % (rel, i + 1, line))

    out.extend(pending)
    return '\n'.join(out).rstrip('\n') + '\n'


def split_top(text):
    """Split on top-level commas — generics and tuples hold their own."""
    parts, depth, start = [], 0, 0
    for j, ch in enumerate(text):
        if ch in '<([':
            depth += 1
        elif ch in '>)]':
            depth -= 1
        elif ch == ',' and depth == 0:
            parts.append(text[start:j])
            start = j + 1
    tail = text[start:].strip()
    if tail:
        parts.append(tail)
    return [p for p in (s.strip() for s in parts) if p]


def fields(lines, i, out):
    """A braced struct's fields, up to the closing `}`."""
    pending = []
    while i < len(lines):
        raw = lines[i]
        line = raw.strip()
        if line == '}':
            out.extend('  ' + p for p in pending)
            return i + 1
        if not line:
            out.extend('  ' + p for p in pending)
            pending = []
            out.append('')
            i += 1
            continue
        c = comment(raw)
        if c is not None:
            pending.append(c)
            i += 1
            continue
        if line.startswith('#['):
            pending.append(line)
            i += 1
            continue
        m = re.match(r'(pub(?:\(crate\))? )?([a-z_0-9]+): (.+),$', line)
        if not m:
            raise Fail('field %r' % line)
        v, _ = vis_of(line)
        out.extend('  ' + p for p in pending)
        pending = []
        out.append('  %s%s : %s,' % (v, raw_ident(m.group(2)), m.group(3)))
        i += 1
    raise Fail('unterminated struct')


def variants(lines, i, out):
    """An enum's variants, up to the closing `}`."""
    pending = []
    first = True
    while i < len(lines):
        raw = lines[i]
        line = raw.strip()
        if line == '}':
            out.extend('  ' + p for p in pending)
            return i + 1
        if not line:
            out.extend('  ' + p for p in pending)
            pending = []
            out.append('')
            i += 1
            continue
        c = comment(raw)
        if c is not None:
            pending.append(c)
            i += 1
            continue
        if line.startswith('#['):
            pending.append(line)
            i += 1
            continue

        bar = '=' if first else '|'
        first = False
        out.extend('  ' + p for p in pending)
        pending = []

        # `Name,` — a unit variant.
        m = re.match(r'([A-Za-z0-9_]+),$', line)
        if m:
            out.append('  %s %s' % (bar, m.group(1)))
            i += 1
            continue
        # `Name(T, U),` — a tuple variant.
        m = re.match(r'([A-Za-z0-9_]+)\((.*)\),$', line)
        if m:
            types = split_top(m.group(2))
            out.append('  %s %s %s' % (bar, m.group(1), ' '.join(types)))
            i += 1
            continue
        # `Name {` — a record variant, fields until `},`.
        m = re.match(r'([A-Za-z0-9_]+) \{$', line)
        if m:
            out.append('  %s %s {' % (bar, m.group(1)))
            i = record_variant(lines, i + 1, out)
            continue
        # `Name { a: T, b: U },` on one line.
        m = re.match(r'([A-Za-z0-9_]+) \{ (.*) \},$', line)
        if m:
            inner = []
            for f in split_top(m.group(2)):
                fm = re.match(r'(pub(?:\(crate\))? )?([a-z_0-9]+): (.+)$', f)
                if not fm:
                    raise Fail('inline variant field %r' % f)
                inner.append('%s : %s' % (fm.group(2), fm.group(3)))
            out.append('  %s %s { %s }' % (bar, m.group(1), ', '.join(inner)))
            i += 1
            continue
        raise Fail('variant %r' % line)
    raise Fail('unterminated enum')


def record_variant(lines, i, out):
    pending = []
    while i < len(lines):
        raw = lines[i]
        line = raw.strip()
        if line in ('},', '}'):
            out.extend('    ' + p for p in pending)
            out.append('  }')
            return i + 1
        c = comment(raw)
        if c is not None:
            pending.append(c)
            i += 1
            continue
        if not line:
            out.extend('    ' + p for p in pending)
            pending = []
            out.append('')
            i += 1
            continue
        m = re.match(r'(pub(?:\(crate\))? )?([a-z_0-9]+): (.+),$', line)
        if not m:
            raise Fail('record-variant field %r' % line)
        out.extend('    ' + p for p in pending)
        pending = []
        # **No visibility**: `pub` on an enum variant's field is `E0449`,
        # which rustc reports against the `.sdx` line. Dropped rather than
        # translated, because there is nothing to translate it to.
        out.append('    %s : %s,' % (raw_ident(m.group(2)), m.group(3)))
        i += 1
    raise Fail('unterminated record variant')


PARTIAL = False

if __name__ == '__main__':
    args = [a for a in sys.argv[1:] if a != '--partial' and not a.startswith('crate=')]
    PARTIAL = '--partial' in sys.argv
    done, failed = 0, []
    for rel in args:
        src = SRC + rel
        try:
            text = translate(src)
        except Fail as e:
            failed.append(str(e))
            continue
        dest = OUT + rel[:-3] + '.sdx'
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        io.open(dest, 'w', encoding='utf-8').write(text)
        done += 1
    print('%d translated' % done)
    for f in failed:
        print('  FAIL %s' % f)
