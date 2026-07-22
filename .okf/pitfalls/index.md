# Pitfalls

Distilled defect knowledge — invariants that have been broken before,
where the guard lives, and what not to re-break. Each concept cites the
full defect report in `docs/`.

* [Side-pot stratification](side-pot-stratification.md) - Pots must be layered by commitment cap; ties split per layer, never on the aggregate.
* [Short-blind call target](short-blind-call-target.md) - An all-in short BB does not lower the call target; the configured BB stands (TDA Rule 41).
* [Bot raise escalation](bot-raise-escalation.md) - Deterministic equity-threshold raise gates create infinite raise wars between bots.
* [Showdown invariants](showdown-invariants.md) - Three showdown-path invariants from the April 2026 RCA: full pot distribution, chip conservation, seat==index.
* [Betting-completion flake](betting-completion-flake.md) - OPEN: rare ActionIsntFinished panic in 1,000-hand self-play runs.
