# Validate fixtures

These hand histories are wrong on purpose. Each file breaks the rules that
`HandHistory::validate` enforces, one file per defect, and the test in
`tests/hand_history_validate.rs` that loads it asserts which `Rule`s the
validator reports. They are excluded from the corpus tests (`data/hands/`,
`tests/defects/DEFECT_*.yaml`), which expect every file they read to be a true
record. `DEFECT_040_the_hand_before.yaml` is `data/hands/the_hand.yaml` as it
stood before EPIC-93 repaired it (see `docs/defects/DEFECT_040_the_hand_yaml.md`).
