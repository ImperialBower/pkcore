---
okf_version: '0.1'
---

# pkcore Knowledge Bundle

Curated knowledge about the pkcore poker library — start with
[Getting started](getting-started.md).

# Root Concepts

* [Getting started](getting-started.md) - How to navigate this bundle and where each kind of pkcore knowledge lives.
* [pkcore crate](crate.md) - Core poker library — cards, hand evaluation, equity, GTO analysis, bots, and full game simulation.
* [Cactus Kev lookup core](cactus-kev-lookup.md) - How pkcore packs cards into u32 values and evaluates 5-card hands via prime products and embedded lookup tables.
* [GTO combos](gto-combos.md) - How poker range notation strings parse into combo sets and how hero blockers prune villain combinations.
* [PLO rules](plo-rules.md) - PLO's exactly-2-hole / exactly-3-board evaluation rule, the 60-combination loop, and its zero-allocation constraints.
* [Stud rules](stud-rules.md) - Stud's five boardless streets, ante + bring-in forced bets, visible-hand action ordering, and its free showdown via the standard 7-card evaluator.
* [Razz rules](razz-rules.md) - Razz as Stud with three inversions — highest upcard brings in, worst visible hand acts first, A-5 lowball showdown via the CaliforniaHandRank lookup.

# Groups

* [Modules](modules/) - Concepts for the major src/ modules: cards, analysis, casino, games, bot.
* [Architecture](architecture/) - Ecosystem layering and recorded design decisions.
* [Data](data/) - The datasets in data/ and generated/: HUP equity databases, bot profiles, hand histories, Pluribus logs.
* [Pitfalls](pitfalls/) - Distilled defect knowledge: invariants broken before and how not to re-break them.
* [Processes](processes/) - EPIC workflow, testing conventions, and the release audit playbook.
* [Ecosystem](ecosystem/) - Downstream repositories that consume pkcore.
