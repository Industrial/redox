"""Print a chapter with every `{{#include}}` replaced by the file it names.

Identity and documentation site
(`.cursor/plans/voltaic-identity-and-site.plan.md`) wave 8.

`llms-full.txt` is built from `src/` rather than from the rendered HTML, so it
carries Markdown rather than stripped tags. That means mdBook has not run, and
every `{{#include}}` would otherwise appear in the output as the literal
directive — which is the one thing that would make the file useless to the
readers it exists for.

A missing target is reported on stderr and marked in place rather than
crashing: the caller is building a 24-chapter file, and losing all of it
because one sample moved would be the wrong trade. `www/machine.sh` and the
Pages workflow both assert the finished file contains no surviving directive,
so a silent gap cannot pass unnoticed either.
"""

import io
import os
import re
import sys

DIRECTIVE = re.compile(r"\{\{#include ([^}]+)\}\}")


def main() -> int:
    if len(sys.argv) != 2:
        sys.stderr.write("usage: expand_includes.py <chapter.md>\n")
        return 2

    page = sys.argv[1]
    base = os.path.dirname(page)
    body = io.open(page, encoding="utf-8").read()
    missing = []

    def resolve(match: "re.Match[str]") -> str:
        # `{{#include file:anchor}}` is a partial include; the file is the part
        # before the colon.
        target = match.group(1).strip().split(":")[0]
        path = os.path.normpath(os.path.join(base, target))
        try:
            return io.open(path, encoding="utf-8").read().rstrip("\n")
        except OSError:
            missing.append(target)
            return "(sample unavailable: %s)" % target

    sys.stdout.write(DIRECTIVE.sub(resolve, body))

    for target in missing:
        sys.stderr.write("expand_includes: %s: cannot read %s\n" % (page, target))
    return 0


if __name__ == "__main__":
    sys.exit(main())
