# Update Log

## 2026-07-22
* **Update**: Restored the full bundle after an accidental working-tree reset deleted everything except the data and pitfalls groups; all concepts rebuilt verbatim.
* **Creation**: Added the [data](/data/index.md) group — [HUP equity databases](/data/hup-databases.md) (schema + snapshot-family caveat), [bot profiles](/data/bot-profiles.md), [hand histories](/data/hand-histories.md), and [Pluribus logs](/data/pluribus-logs.md).
* **Creation**: Added the [pitfalls](/pitfalls/index.md) group distilling the docs/ defect corpus — [side-pot stratification](/pitfalls/side-pot-stratification.md), [short-blind call target](/pitfalls/short-blind-call-target.md), [bot raise escalation](/pitfalls/bot-raise-escalation.md), [showdown invariants](/pitfalls/showdown-invariants.md), and the open [betting-completion flake](/pitfalls/betting-completion-flake.md).
* **Update**: Added `description`/`timestamp` frontmatter to the externally-authored [PLO rules](/plo-rules.md) concept, corrected its `references` path to `src/analysis/omaha.rs`, and registered it in the root [index](/index.md).
* **Update**: Added `description`/`timestamp` frontmatter to the externally-authored [Cactus Kev lookup core](/cactus-kev-lookup.md) and [GTO combos](/gto-combos.md) concepts and registered them in the root [index](/index.md).
* **Update**: Corrected [cards](/modules/cards.md) — `Card` is a `u32` newtype (Cactus Kev bit packing), not `u8`; the stale `lib.rs` crate doc was fixed in source to match.
* **Creation**: Authored the initial concept set — [pkcore crate](/crate.md), five [module concepts](/modules/index.md), [ecosystem layers](/architecture/layers.md), the [Table vs TableCelled](/architecture/table-vs-tablecelled.md) decision record, three [process concepts](/processes/index.md), and [downstream repos](/ecosystem/downstream-repos.md).
* **Update**: Rewrote [getting started](/getting-started.md) and the root [index](/index.md) for progressive disclosure.
* **Creation**: Scaffolded the pkcore Knowledge Bundle bundle with `okf_init.py` — see [getting started](getting-started.md).
