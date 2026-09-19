#!/usr/bin/env python3
"""Write DIARY_LINKED.md: DIARY.md with every line linked to the commit that wrote it.

Each link opens the GitHub tree at that commit, so a diary entry takes you to the
repo as it was when the entry was written. `git blame -C -C` follows lines copied
across files, which matters here: the diary started life in README.md and was
moved to DIARY.md in 9186e6d, and a plain blame would credit that move.

A line shows its *last* change, so an entry edited later links to the edit.

Lines that already hold a markdown link, or a code span with brackets in it,
cannot be wrapped in a link; they keep their text and get a trailing `↗ <sha>`.

Run:  python3 scripts/link_diary.py [SOURCE] [OUTPUT]   (defaults: DIARY.md DIARY_LINKED.md)
"""
from __future__ import annotations

import datetime
import re
import subprocess
import sys

REPO_TREE = "https://github.com/ImperialBower/pkcore/tree/"
LINE_START = re.compile(r"^(\s*(?:[*+-]|\d+\.|#+)\s+)?(.*)$")
SHA = re.compile(r"[0-9a-f]{40}")


def blame(source: str) -> list[tuple[str, str, str]]:
    """Return (sha, yyyy-mm-dd, text) for each line of `source`."""
    porcelain = subprocess.run(
        ["git", "blame", "-w", "-C", "-C", "--line-porcelain", source],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.splitlines()
    rows, sha, date = [], "", ""
    for line in porcelain:
        if line.startswith("\t"):
            rows.append((sha, date, line[1:]))
            continue
        field, _, value = line.partition(" ")
        if SHA.fullmatch(field):
            sha = field
        elif field == "author-time":
            stamp = datetime.datetime.fromtimestamp(int(value), datetime.timezone.utc)
            date = stamp.strftime("%Y-%m-%d")
    return rows


def link_line(sha: str, date: str, text: str) -> str:
    if not text.strip():
        return text
    target = f'{REPO_TREE}{sha} "{date} · {sha[:7]}"'
    prefix, body = LINE_START.match(text).groups()
    has_brackets = "[" in body or "]" in body
    if "](" in body or ("`" in body and has_brackets) or body.startswith(("|", "<")):
        return f"{text} [↗ {sha[:7]}]({target})"
    escaped = body.replace("[", "\\[").replace("]", "\\]")
    return f"{prefix or ''}[{escaped}]({target})"


def main() -> int:
    source = sys.argv[1] if len(sys.argv) > 1 else "DIARY.md"
    output = sys.argv[2] if len(sys.argv) > 2 else "DIARY_LINKED.md"
    rows = blame(source)
    lines = [
        f"<!-- Generated from {source} by scripts/link_diary.py (make diary-links). "
        "Each line links to the repo as it was when that line was written. -->",
        "",
    ]
    lines += [link_line(sha, date, text) for sha, date, text in rows]
    with open(output, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    print(f"{output}: {len(rows)} lines, {len({sha for sha, _, _ in rows})} commits")
    return 0


if __name__ == "__main__":
    sys.exit(main())
