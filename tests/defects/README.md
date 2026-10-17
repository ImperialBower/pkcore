# Defect hands

Each `DEFECT_NNN_*.yaml` here is one poker hand that reproduces a fixed defect.
`tests/defect_hands.rs` replays every file on each `cargo test --features full`.

A file is a normal hand history (`HandHistory::from_yaml`), plus two blocks only
the runner reads:

- `expect:` — table state after a given action (`street`, `index` in that
  street's `actions` list). The keys are listed at the top of
  `tests/defect_hands.rs`.
- `final:` — final stacks, and whether the recorded results match.

To look at a defect, open its file: the comment says what went wrong, and the
`expect:` entry marks the action where it went wrong.

## Coverage

"Runs the fix" means the replay goes through the code the fix changed.
**Checked** means the old bug was put back by hand on 2026-10-17 and the file
failed at the expected step. The others are not yet checked that way.

| Defect | File | Runs the fix |
|---|---|---|
| 001 short big blind call target | `DEFECT_001_*` | yes |
| 003 heads-up side pot | `DEFECT_003_*` (2 hands) | yes |
| 007 bot raise below the minimum | `DEFECT_007_*` (2 hands) | table side only — the bug was in the bot |
| 009 substantial action | `DEFECT_009_*` | yes |
| 010 short all-in re-open gate | `DEFECT_010_*` | yes, **checked** |
| 011 odd chip order | `DEFECT_011_*` | yes, **checked** |
| 012 short blind, pot limit | `DEFECT_012_*` | yes |
| 013 dead button | `DEFECT_013_*` | yes |
| 014 replay table size | `DEFECT_014_*` | yes — the bug was in replay itself |
| 015 raise all-in for less | `DEFECT_015_*` | yes (same path as 027) |
| 017 Omaha two-card rule | `DEFECT_017_*` | no — the table never called the fixed function |
| 022 next to act after a re-raise | `DEFECT_022_*` | yes |
| 023 minimum raise never zero | `DEFECT_023_*` | no — the table never called the fixed function |
| 025 all-in run-out | `DEFECT_025_*` (2 hands) | no — replay deals streets itself, not through `Table::act` |
| 027 short all-in lowers the bet | `DEFECT_027_*` | yes, **checked** |

## No file, and why

| Defect | Why not |
|---|---|
| 002, 006 | Bot behaviour and random-seed flakes across many hands |
| 004 | A bot simulation over many seeds; no single hand is recorded |
| 005 | Memory allocation, not a hand |
| 008 | Umbrella for 009–013 |
| 016 | Solver cache key |
| 018 | Needs real stud dealing to 7th street |
| 019 | Session layer error handling |
| 020, 024 | Pluribus replay-driver error handling |
| 021 | Pluribus log parsing; hand-history amounts never go through it |
| 026 | Razz showdown: replay cannot take a stud or Razz hand past 3rd street |
