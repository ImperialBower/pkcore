#!/usr/bin/env python3
"""Write DIARY_LINKED.md: DIARY.md with every line linked to the commit that wrote it.

Each link opens the GitHub tree at that commit, so a diary entry takes you to the
repo as it was when the entry was written. `git blame -C -C` follows lines copied
across files, which matters here: the diary started life in README.md and was
moved to DIARY.md in 9186e6d, and a plain blame would credit that move.

A line shows its *last* change, so an entry edited later links to the edit.

Lines that already hold a markdown link, or a code span with brackets in it,
cannot be wrapped in a link; they keep their text and get a trailing `↗ <sha>`.

An entry whose subject is a type pkcore inherited from fudd — its predecessor,
written 2022-2023 — also gets a trailing `fudd` link to the file that holds the
same idea over there, so the rewrite can be read beside the original. fudd leans
on the `ckc-rs` crate for `Rank`, `Suit` and `HandRank`, so those point at the
fudd file that stands in for them rather than at a file fudd never had.

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

# fudd is finished, so its links are pinned to the tip of `main` rather than to a
# branch name that could move under them.
FUDD_BLOB = "https://github.com/ImperialBower/fudd/blob/9d20005ccf85a6108a718e6f71e29a6c8a6b5854/"
FUDD_PATHS = {
    "Rank": "src/types/playing_card.rs",
    "Suit": "src/types/playing_card.rs",
    "Card": "src/types/playing_card.rs",
    "PlayingCard": "src/types/playing_card.rs",
    "Cards": "src/types/playing_cards.rs",
    "PlayingCards": "src/types/playing_cards.rs",
    "CardSlot": "src/types/card_slot.rs",
    "Deck": "src/types/poker_deck.rs",
    "HandRank": "src/types/arrays/mod.rs",
    "HandRanker": "src/types/arrays/mod.rs",
    "Two": "src/types/arrays/two_card.rs",
    "Three": "src/types/arrays/three_card.rs",
    "Four": "src/types/arrays/four_card.rs",
    "Five": "src/types/arrays/five_card.rs",
    "Six": "src/types/arrays/six_card.rs",
    "Seven": "src/types/arrays/seven_card.rs",
    "HoleCards": "src/types/slots/hole_cards.rs",
    "Flop": "src/types/slots/flop.rs",
    "Omaha": "src/types/slots/omaha_hand.rs",
    "Chen": "src/types/ranges/chen_weighted.rs",
    "Board": "src/games/holdem/board.rs",
    "Deal": "src/games/holdem/deal.rs",
    "HeadsUp": "src/games/holdem/heads_up.rs",
    "Seat": "src/games/holdem/seat.rs",
    "Seats": "src/games/holdem/seats.rs",
    "Table": "src/games/holdem/table.rs",
    "Eval": "src/analysis/eval.rs",
    "Evals": "src/analysis/evals.rs",
    "Outs": "src/analysis/outs.rs",
    "Chances": "src/analysis/chances.rs",
}
# Longest name first, so `Cards` wins over `Card` and `Seats` over `Seat`.
FUDD_SUBJECT = re.compile(
    r"(?:{})\b".format("|".join(sorted(map(re.escape, FUDD_PATHS), key=len, reverse=True)))
)
# Diary shout-words that sit in front of the real subject of an entry.
DIARY_MARKER = re.compile(
    r"^(?:EPIC(?:\s+\w+)?|ASIDE|FEATURE|DEFECT|POSSIBLE DEFECT|REFACTOR(?:ING)?|"
    r"CLIPPY|HOUSECLEANING|INTRODUCING|TRAIT|NOTE)\b[:\s]+"
)


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


def fudd_suffix(body: str) -> str:
    """Return ` [fudd](…)` when this entry's subject is a type fudd also had."""
    subject = DIARY_MARKER.sub("", body.strip(), count=1)
    match = FUDD_SUBJECT.match(subject)
    if match is None:
        return ""
    path = FUDD_PATHS[match.group(0)]
    return f' [fudd]({FUDD_BLOB}{path} "fudd · {path}")'


def link_line(sha: str, date: str, text: str) -> str:
    if not text.strip():
        return text
    target = f'{REPO_TREE}{sha} "{date} · {sha[:7]}"'
    prefix, body = LINE_START.match(text).groups()
    fudd = fudd_suffix(body)
    has_brackets = "[" in body or "]" in body
    if "](" in body or ("`" in body and has_brackets) or body.startswith(("|", "<")):
        return f"{text} [↗ {sha[:7]}]({target}){fudd}"
    escaped = body.replace("[", "\\[").replace("]", "\\]")
    return f"{prefix or ''}[{escaped}]({target}){fudd}"


def main() -> int:
    source = sys.argv[1] if len(sys.argv) > 1 else "DIARY.md"
    output = sys.argv[2] if len(sys.argv) > 2 else "DIARY_LINKED.md"
    rows = blame(source)
    lines = [
        f"<!-- Generated from {source} by scripts/link_diary.py (make diary-links). "
        "Each line links to the repo as it was when that line was written; a trailing "
        "`fudd` link opens the same idea in pkcore's predecessor. -->",
        "",
    ]
    lines += [link_line(sha, date, text) for sha, date, text in rows]
    with open(output, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    print(f"{output}: {len(rows)} lines, {len({sha for sha, _, _ in rows})} commits")
    return 0


if __name__ == "__main__":
    sys.exit(main())
