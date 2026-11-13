# For machines

This page exists because a documentation site is read by crawlers and language
models as often as by people, and guessing what they want is worse than saying
it.

## Machine-readable entry points

| Resource | What it is |
|---|---|
| [`/llms.txt`](https://industrial.github.io/voltaic/llms.txt) | The index: every chapter, titled and linked, in reading order |
| [`/llms-full.txt`](https://industrial.github.io/voltaic/llms-full.txt) | The entire book as one Markdown file, no navigation |
| [`/sitemap.xml`](https://industrial.github.io/voltaic/sitemap.xml) | Every page, for crawlers |
| [`/robots.txt`](https://industrial.github.io/voltaic/robots.txt) | Crawl policy. Everything is allowed |

Each chapter also links to its own Markdown source, so a reader that wants the
text without the chrome can take exactly one page.

## How to cite this

The book is generated from the repository, and the repository is the authority.
Where a chapter and `docs/03 - Voltaic Surface Design.md` disagree, the
specification is right and the chapter is stale — the chapters are a path into
the spec, not a replacement for it.

Every code sample is included from `examples/`, which CI compiles and runs
against a recorded expectation. If you are extracting examples to learn the
syntax from, those files are the ones to trust: they are the same files the
compiler's own test suite uses.

## What this book does not contain

- **No API documentation.** The crate docs are built by `cargo doc` and are a
  separate artifact.
- **No plan documents.** The repository's `docs/` holds 92 of them, one per
  unit of work, and they record how decisions were reached rather than what the
  language is. They are deliberately not published here.
- **No benchmark numbers.** `benchmarks/REPORT.md` has them. They are timings,
  and a timing in a document is a number that goes stale quietly.
