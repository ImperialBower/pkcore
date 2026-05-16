# Position Indicators in `interactive_play` — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Git constraint:** This user's global CLAUDE.md forbids the executor from running any state-changing git command. Every "Commit" step shows the command to surface to the user — the executor must NOT run it. Stop after each task and ask the user to run the commit, then wait for them to confirm before starting the next task.

**Goal:** Add Dealer (BTN), Small Blind (SB), and Big Blind (BB) role indicators to three display points in `examples/interactive_play.rs`.

**Architecture:** Pure renderer change. One private `position_tag` helper at the bottom of the file. Three call-site updates: `print_stacks`, the action-line print inside `run_street`, and the "Your turn" box inside `read_human_action`. No public API changes; no engine changes.

**Tech Stack:** Rust example binary; existing `TableNoCell` accessors (`button`, `determine_small_blind`, `determine_big_blind`); existing `seat_label` helper.

**Spec:** `docs/superpowers/specs/2026-05-13-interactive-play-position-indicators-design.md`

---

## File Structure

| File | Change |
|---|---|
| `examples/interactive_play.rs` | Add `position_tag` helper + `#[cfg(test)] mod tests` at bottom; modify `print_stacks` (line 623), the action print site inside `run_street` (line 397), and `read_human_action` (line 425). |

No other files touched.

---

## Task 1: Add `position_tag` helper with TDD

**Files:**
- Modify: `examples/interactive_play.rs` (add at bottom of file)

- [ ] **Step 1: Append failing test module to the file**

Append to the end of `examples/interactive_play.rs` (after the existing `print_stacks` function):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_tag_button_only() {
        assert_eq!(position_tag(3, 3, 4, 5), Some("BTN"));
    }

    #[test]
    fn position_tag_small_blind() {
        assert_eq!(position_tag(4, 3, 4, 5), Some("SB"));
    }

    #[test]
    fn position_tag_big_blind() {
        assert_eq!(position_tag(5, 3, 4, 5), Some("BB"));
    }

    #[test]
    fn position_tag_heads_up_collapses_btn_and_sb() {
        // Heads-up: button IS the small blind. Same seat, both flags set.
        assert_eq!(position_tag(0, 0, 0, 1), Some("BTN/SB"));
        assert_eq!(position_tag(1, 0, 0, 1), Some("BB"));
    }

    #[test]
    fn position_tag_middle_position_returns_none() {
        assert_eq!(position_tag(7, 3, 4, 5), None);
    }
}
```

- [ ] **Step 2: Run the test and verify it fails**

```
cargo test --example interactive_play
```

Expected: build failure with `error[E0425]: cannot find function 'position_tag' in this scope` (5 occurrences).

- [ ] **Step 3: Add the `position_tag` helper**

Append immediately above the `#[cfg(test)] mod tests` block (still after `print_stacks`):

```rust
/// Returns a position role tag for `seat`, or `None` if the seat is
/// neither the button nor a blind. Heads-up collapses BTN+SB onto a
/// single seat (the button is the small blind in HU).
fn position_tag(seat: u8, btn: u8, sb: u8, bb: u8) -> Option<&'static str> {
    match (seat == btn, seat == sb, seat == bb) {
        (true, true, _) => Some("BTN/SB"),
        (true, _, _)    => Some("BTN"),
        (_, true, _)    => Some("SB"),
        (_, _, true)    => Some("BB"),
        _               => None,
    }
}
```

- [ ] **Step 4: Run the test and verify it passes**

```
cargo test --example interactive_play
```

Expected: `test result: ok. 5 passed; 0 failed`.

- [ ] **Step 5: Surface commit command to user (do NOT run)**

```
git add examples/interactive_play.rs && git commit -m "feat(interactive_play): add position_tag helper with HU collapse"
```

Stop here. Ask the user to run the commit, then wait for confirmation before starting Task 2.

---

## Task 2: Show position tags on the `Stacks:` line

**Files:**
- Modify: `examples/interactive_play.rs` — function `print_stacks` (currently line 623)

- [ ] **Step 1: Replace the `print_stacks` body**

Find this function:

```rust
fn print_stacks(table: &TableNoCell, profiles: &[BotProfile]) {
    print!("  Stacks:");
    if let Some(seat) = table.seats.get_seat(HUMAN_SEAT).filter(|s| !s.is_empty()) {
        print!("  {}={}", HUMAN_NAME, seat.player.chips);
    }
    for (i, profile) in profiles.iter().enumerate() {
        if let Some(seat) = table.seats.get_seat(i as u8 + 1).filter(|s| !s.is_empty()) {
            print!("  {}={}", profile.name, seat.player.chips);
        }
    }
    println!();
}
```

Replace with:

```rust
fn print_stacks(table: &TableNoCell, profiles: &[BotProfile]) {
    let btn = table.button;
    let sb = table.determine_small_blind();
    let bb = table.determine_big_blind();
    let tag_for = |seat: u8| match position_tag(seat, btn, sb, bb) {
        Some(t) => format!(" ({t})"),
        None => String::new(),
    };

    print!("  Stacks:");
    if let Some(seat) = table.seats.get_seat(HUMAN_SEAT).filter(|s| !s.is_empty()) {
        print!("  {}={}{}", HUMAN_NAME, seat.player.chips, tag_for(HUMAN_SEAT));
    }
    for (i, profile) in profiles.iter().enumerate() {
        let seat_idx = i as u8 + 1;
        if let Some(seat) = table.seats.get_seat(seat_idx).filter(|s| !s.is_empty()) {
            print!("  {}={}{}", profile.name, seat.player.chips, tag_for(seat_idx));
        }
    }
    println!();
}
```

- [ ] **Step 2: Build the example**

```
cargo build --example interactive_play
```

Expected: clean build, no warnings.

- [ ] **Step 3: Manually verify the Stacks line**

```
cargo run --example interactive_play
```

Expected first Stacks line (before any hand starts) — the human (`You`) is at seat 0, and on hand 1 the button starts at seat 0:

```
Stacks:  You=10000 (BTN)  gto=10000 (SB)  tight_passive=10000 (BB)  loose_aggressive=10000  tight_aggressive=10000  loose_passive=10000  maniac=10000  abc=10000  short_stack_ninja=10000
```

Fold the human (`f`) immediately to end hand 1 quickly, then confirm the Stacks line printed after the hand still shows the tags (still on the same button until next hand).

Then quit (`q`) — that triggers a session save, no other hands needed for this verification.

- [ ] **Step 4: Surface commit command to user (do NOT run)**

```
git add examples/interactive_play.rs && git commit -m "feat(interactive_play): show position tags on Stacks line"
```

Stop here. Ask the user to run the commit, then wait for confirmation before starting Task 3.

---

## Task 3: Add position tag column to per-action lines

**Files:**
- Modify: `examples/interactive_play.rs` — function `run_street` (currently line 361)

- [ ] **Step 1: Add `btn`/`sb`/`bb` locals at the top of `run_street`**

Find this function header and the start of the loop:

```rust
fn run_street(
    table: &mut TableNoCell,
    profiles: &[BotProfile],
    rng: &mut impl Rng,
    editor: &mut Reedline,
    collection: &HandCollection,
) {
    let max_iterations = (profiles.len() + 1) * 8;

    for _ in 0..max_iterations {
```

Insert the three locals immediately before the `for` loop:

```rust
fn run_street(
    table: &mut TableNoCell,
    profiles: &[BotProfile],
    rng: &mut impl Rng,
    editor: &mut Reedline,
    collection: &HandCollection,
) {
    let max_iterations = (profiles.len() + 1) * 8;
    let btn = table.button;
    let sb = table.determine_small_blind();
    let bb = table.determine_big_blind();

    for _ in 0..max_iterations {
```

- [ ] **Step 2: Replace the action-line `println!`**

Find this block near the bottom of the loop body (currently around line 397):

```rust
        let pot_after = table.effective_pot();
        println!(
            "    {:>20}  [pot: {}] {} [pot: {}]",
            seat_label(seat, profiles),
            pot_before,
            desc,
            pot_after
        );
```

Replace with:

```rust
        let pot_after = table.effective_pot();
        let tag = position_tag(seat, btn, sb, bb)
            .map(|t| format!("[{t}]"))
            .unwrap_or_default();
        println!(
            "    {:>8} {:<20}  [pot: {}] {} [pot: {}]",
            tag,
            seat_label(seat, profiles),
            pot_before,
            desc,
            pot_after
        );
```

- [ ] **Step 3: Build the example**

```
cargo build --example interactive_play
```

Expected: clean build, no warnings.

- [ ] **Step 4: Manually verify action lines**

```
cargo run --example interactive_play
```

On hand 1 (button on seat 0), the bots act first because the human (BTN) acts last preflop. After bots act and you reach your turn, the action lines printed before your prompt should look like (only seats with roles get tags; in this 9-handed setup with button on seat 0, that means SB on seat 1 and BB on seat 2):

```
                 loose_aggressive    [pot: 150] folds [pot: 150]
                 tight_aggressive    [pot: 150] folds [pot: 150]
                    loose_passive    [pot: 150] folds [pot: 150]
                           maniac    [pot: 150] folds [pot: 150]
                              abc    [pot: 150] folds [pot: 150]
                short_stack_ninja    [pot: 150] folds [pot: 150]
```

(All middle/late position; tag column is 8 spaces.)

After your action, you should see SB and BB act with their tags:

```
            [SB] gto                 [pot: 250] folds [pot: 250]
            [BB] tight_passive       [pot: 250] checks [pot: 250]
```

Quit with `q` after verifying.

- [ ] **Step 5: Surface commit command to user (do NOT run)**

```
git add examples/interactive_play.rs && git commit -m "feat(interactive_play): add position tag column to action lines"
```

Stop here. Ask the user to run the commit, then wait for confirmation before starting Task 4.

---

## Task 4: Show Position field in the "Your turn" box

**Files:**
- Modify: `examples/interactive_play.rs` — function `read_human_action` (currently line 425)

- [ ] **Step 1: Compute `position_suffix` once near the top of `read_human_action`**

Find the function header and the existing setup before the loop:

```rust
fn read_human_action(
    table: &mut TableNoCell,
    seat: u8,
    to_call: usize,
    chips: usize,
    pot: usize,
    hole: &str,
    editor: &mut Reedline,
    collection: &HandCollection,
) -> String {
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("  └> ".to_string()),
        right_prompt: DefaultPromptSegment::Empty,
    };

    println!();
    loop {
```

Insert position computation between `prompt` and the `println!()`:

```rust
fn read_human_action(
    table: &mut TableNoCell,
    seat: u8,
    to_call: usize,
    chips: usize,
    pot: usize,
    hole: &str,
    editor: &mut Reedline,
    collection: &HandCollection,
) -> String {
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("  └> ".to_string()),
        right_prompt: DefaultPromptSegment::Empty,
    };

    let btn = table.button;
    let sb = table.determine_small_blind();
    let bb = table.determine_big_blind();
    let position_suffix = match position_tag(seat, btn, sb, bb) {
        Some(t) => format!("   Position: {t}"),
        None => String::new(),
    };

    println!();
    loop {
```

- [ ] **Step 2: Append `position_suffix` to the Cards/Chips/Pot line**

Find this line near the top of the loop body:

```rust
        println!("  │  Cards: {}   Chips: {}   Pot: {}", hole, chips, pot);
```

Replace with:

```rust
        println!("  │  Cards: {}   Chips: {}   Pot: {}{}", hole, chips, pot, position_suffix);
```

- [ ] **Step 3: Build the example**

```
cargo build --example interactive_play
```

Expected: clean build, no warnings.

- [ ] **Step 4: Manually verify the Position field**

```
cargo run --example interactive_play
```

On hand 1 the human (seat 0) is on the button. The "Your turn" box for hand 1 should include the field:

```
  │  Cards: T♥ 2♥   Chips: 10000   Pot: 150   Position: BTN
```

Take an action (e.g. `f` to fold), let the hand finish, and observe hand 2: button moves to seat 1 (`gto`), so seat 0 becomes the BB. The next time you reach the "Your turn" box, you should see:

```
  │  Cards: ...   Chips: ...   Pot: ...   Position: BB
```

Continue folding through hand 3 (button on seat 2 — `tight_passive` — so the human at seat 0 is now in middle position). Verify the Position field is **omitted entirely** on hand 3:

```
  │  Cards: ...   Chips: ...   Pot: ...
```

Quit with `q` after verifying all three states (BTN, BB, none).

- [ ] **Step 5: Surface commit command to user (do NOT run)**

```
git add examples/interactive_play.rs && git commit -m "feat(interactive_play): show Position field in Your turn box"
```

Stop here. Ask the user to run the commit, then wait for confirmation before starting Task 5.

---

## Task 5: Heads-up edge case verification

**Files:** none modified — verification only.

- [ ] **Step 1: Run the example and reach a heads-up state**

```
cargo run --example interactive_play
```

Strategy to force HU quickly: each hand, fold the human (`f`) on the human's first turn. The bots will play out the hand against each other; one will lose chips each hand. Continue across many hands until only the human and one bot remain.

Faster alternative: take aggressive actions (all-in `a`) on early hands to either bust the human or knock out bots. Document whichever path you choose; the goal is to reach a 2-handed table.

When at 2 players, on a hand where you are the button (and therefore SB), the Stacks line should show:

```
Stacks:  You=... (BTN/SB)  <bot>=... (BB)
```

The action lines should show:

```
        [BTN/SB] You                 [pot: ...] ... [pot: ...]
            [BB] <bot>               [pot: ...] ... [pot: ...]
```

The "Your turn" box should show:

```
  │  Cards: ...   Chips: ...   Pot: ...   Position: BTN/SB
```

- [ ] **Step 2: Run the full test suite**

```
cargo test --example interactive_play
```

Expected: `test result: ok. 5 passed; 0 failed`.

- [ ] **Step 3: Run cargo check to confirm no warnings on the example**

```
cargo build --example interactive_play 2>&1 | tee /tmp/build.log
grep -E "warning|error" /tmp/build.log || echo "clean"
```

Expected: `clean`.

- [ ] **Step 4: Final state — no commit needed**

This task only verifies; no source change. If everything passes, the feature is complete.
