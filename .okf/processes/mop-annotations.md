---
type: Process
title: Book annotations
description: Page-anchored reading notes on The Mathematics of Poker in docs/mop/, with the book's worked numbers reproduced by tests/mop.rs.
tags: [learning, documentation, testing]
timestamp: '2026-10-24T00:00:00Z'
---

# What it is

`docs/mop/` is a reading companion for Bill Chen and Jerrod Ankenman,
*The Mathematics of Poker* (ConJelCo, 2006) [1]. Page numbers are those of
the print edition. The index lists all 30 chapters with page ranges and a
status; each chapter gets its own file when it is read.

A note covers one page or page range. It states the idea in our own words,
links the pkcore code that does the same thing (or says it is not in pkcore
yet), and names the test that checks the page's number, if any.

# Rules

* **Own words only.** The book is copyrighted; no text from it is
  committed. A quote is at most one short line.
* **Checks live in `tests/mop.rs`**, one module per chapter
  (`mop__ch01_tests`), named `chNN_pPPP_<what>`. The file has no
  `required-features`, so it runs in the bare kernel build.
* **Exact ratios.** Checks compare whole-number counts by cross-multiplying,
  never floats within a tolerance.
* **A disagreement is never hidden.** If pkcore and the book differ, an
  independent count decides who is wrong, and the note records it.

# Citations

[1] [docs/mop/README.md](https://github.com/ImperialBower/pkcore/blob/main/docs/mop/README.md)
