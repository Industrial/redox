"""Insert a fragment before </head>, once.

Identity and documentation site
(`.cursor/plans/voltaic-identity-and-site.plan.md`) wave 8.

Called by `www/machine.sh` per built page. Idempotent: a page that already
carries a canonical link is left alone, so running the annotation step twice
over one build does not produce two of everything.

A separate file rather than a heredoc inside the shell script, because
`beautysh` cannot track indentation through an embedded heredoc and fails the
`:format` gate with *"indent/outdent mismatch"*.
"""

import io
import sys


def main() -> int:
    if len(sys.argv) != 3:
        sys.stderr.write("usage: inject_head.py <page.html> <fragment>\n")
        return 2

    page, fragment = sys.argv[1], sys.argv[2]
    body = io.open(page, encoding="utf-8").read()

    if 'rel="canonical"' in body:
        return 0

    at = body.lower().find("</head>")
    if at < 0:
        sys.stderr.write("inject_head: %s has no </head>\n" % page)
        return 1

    inject = io.open(fragment, encoding="utf-8").read()
    io.open(page, "w", encoding="utf-8").write(body[:at] + inject + body[at:])
    return 0


if __name__ == "__main__":
    sys.exit(main())
