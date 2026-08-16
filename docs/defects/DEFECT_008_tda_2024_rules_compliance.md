# Defect: Six TDA 2024 Compliance Gaps in Betting, Button and Pot Distribution

**File:** `docs/defects/DEFECT_008_tda_2024_rules_compliance.md`
**Date:** 2026-08-16
**Severity:** Major (4 Major, 2 Minor — no incorrect *pot total*; all findings distribute, size or gate chips wrongly)
**Status:** Open — no fix applied. All six verified by source reading at `90d60e70` (`main`, 2026-08-16), pkcore `0.4.0`.
**Reported by:** Audit of pkcore against the parsed TDA 2024 ruleset in the sibling `tda_parsed` repo (`tda_2024_online.yaml`)
**Introduced in:** Not bisected. These are absences and long-standing behaviours, not regressions — no introducing commit was identified, and none of the six has ever had a passing test that later broke.
**Fixed in:** —

---

## Summary

pkcore's betting engine was audited rule-by-rule against the 2024 Poker Tournament
Directors Association ruleset. The evaluator, side-pot stratification and min-raise
*sizing* all hold up well — including the two cases the TDA Illustration Addendum
singles out as commonly botched. Six gaps did not.

Four are Major: odd chips are awarded by seat index rather than by button position
(**D8-1**); there is no re-open gate, so a player who has already acted may re-raise
when facing less than a full raise (**D8-2**); pot-limit uses the actual pot pre-flop,
so a dead or short blind shrinks the maximum legal bet (**D8-3**); and the dead button
is not implemented, so blinds skip to the next occupied seat and a dead small blind can
never occur (**D8-4**). Two are structural: no substantial-action predicate exists
anywhere in the crate (**D8-5**), which blocks correct implementation of five further
rules; and the fixed-limit raise cap cannot lift at event-heads-up (**D8-6**).

None of the six produces a wrong pot *total* — the collection side is sound. Every one
of them mis-distributes, mis-sizes, or fails to gate.

---

## How this was found

Not from a hand history. The 2024 TDA longform PDF was parsed to YAML
(`tda_parsed/tda_2024.yaml`, 71 rules), filtered to the 50 that bind an automated
server-authoritative engine (`tda_parsed/tda_2024_online.yaml`), and each rule was then
searched for in `pkcore/src` and recorded as `yes` / `partial` / `no` with `file:line`
evidence. The audit standing at 31 direct rules: 7 implemented, 16 partial, 8 absent.

This is worth noting because **five of the six findings are invisible to the existing
test suite by construction** — they are absences, or they only manifest in asymmetric
setups (a button that is not seat 0, a dead blind, a player acting twice in a street).
See [Coverage Gap](#coverage-gap).

---

## Findings at a glance

| # | TDA rule | Finding | Severity | Evidence |
|---|---|---|---|---|
| D8-1 | 20-A/B/C | Odd chip goes to the highest-numbered winning seat, not first-left-of-button | Major | `cashier/chips.rs:95`, `analysis/case_eval.rs:231` |
| D8-2 | 47-A | No re-open gate: a player who already acted may re-raise facing a sub-minimum increment | Major | `casino/table/actions.rs:337` |
| D8-3 | 54-B | Pot-limit pre-flop max uses the actual pot; a dead/short blind shrinks it | Major | `games/betting_structure.rs:170` |
| D8-4 | 32 | Dead button not implemented — blinds skip to the next occupied seat | Major | `casino/table.rs:479` |
| D8-5 | 36 | No substantial-action predicate exists; blocks rules 22, 34-A, 35-D, 52-A, 53-B | Major (structural) | absence across `src/` |
| D8-6 | 48 | Fixed-limit raise cap cannot lift when the event reaches two players | Minor | `games/betting_structure.rs:231` |

Two further deviations are recorded as [Accepted divergences](#accepted-divergences)
rather than defects.

---

## D8-1: Odd chips are awarded by seat index, not by button position

**Severity:** Major — a wrong payout, bounded at one chip per split pot per layer.

### The Poker Rule

TDA Rule 20 fixes *which* tied winner receives the indivisible remainder. It has three
cases, and the order of operations matters:

> First, odd chips will be broken into the smallest denomination in play.
> **A)** Board games with 2 or more high or low hands: the odd chip goes to the **first
> seat left of the button**. **B)** Stud, razz, and if 2 or more high or low hands in
> stud/8: the odd chip goes to the **high card by suit** in the player's 5-card winning
> hand. **C)** H/L split: the odd chip in the total pot goes to the **high side**.

The rule exists because "split it evenly" is undefined for an odd pot, and any
tiebreak that is not positional is exploitable across a session.

### Root Cause

`Stack::divvy_up` distributes the remainder to the **last** `remainder` indices of the
winners vector:

```rust
// src/casino/cashier/chips.rs:84
pub fn divvy_up(&self, by: usize) -> Vec<Stack> {
    let winnings = self.take();
    match by {
        0 | 1 => vec![winnings],
        _ => {
            let total = winnings.count();
            let share = total / by;
            let remainder = total % by;

            (0..by)
                .map(|i| {
                    let amount = if i >= by - remainder { share + 1 } else { share };  // :95
                    Stack::new(amount)
                })
                .collect()
        }
    }
}
```

The winners vector is produced by `CaseEval::winning_seats`, which iterates seat
indices in **ascending order**:

```rust
// src/analysis/case_eval.rs:231
pub fn winning_seats(&self) -> Vec<u8> {
    let flags = self.flags_win();
    (0..self.0.len() as u8).filter(|i| (flags & (1 << i)) != 0).collect()
}
```

Composing the two: **the odd chip always goes to the highest-numbered winning seat.**
`divvy_up` never sees the button, and the call sites do not pass it.

### Symptom

Six-handed, two tied winners at seats 2 and 5, pot 101.
`divvy_up` computes `share = 50`, `remainder = 1`, `by = 2`; index `1` (seat 5) gets 51.

| Button | TDA order left of button | TDA awards odd chip to | pkcore awards to | Correct? |
|---|---|---|---|---|
| seat 7 | 0, 1, **2**, 3, 4, 5, 6 | seat 2 | seat 5 | ✗ |
| seat 3 | 4, **5**, 6, 7, 0, 1, 2 | seat 5 | seat 5 | ✓ by coincidence |
| seat 1 | **2**, 3, 4, 5, 6, 7, 0 | seat 2 | seat 5 | ✗ |

The behaviour is deterministic and button-independent, which is precisely the bug: it
is correct only when the button happens to fall so that the highest-indexed winner is
also the first seat to its left.

Cases **B** (stud/razz high card by suit) and **C** (hi/lo odd chip to the high side)
have no implementation at all. pkcore ships Seven-Card Stud Hi and Razz
(`src/games/stud.rs`, `src/games/razz.rs`), so case B is reachable today.

The first clause — breaking the odd chip into the smallest denomination in play — is
not applicable: pkcore models chips as integers, so there is no denomination to break.

### Fix sketch

`divvy_up` is the wrong altitude to fix this — it has no domain context and should
stay a pure arithmetic split. Add the ordering at the call site: have the showdown
paths order the winners vector by TDA precedence *before* calling `divvy_up`, so the
existing "remainder to the last indices" arithmetic lands on the right seat. That
requires passing the button (board games) or the winning 5-card hand (stud/razz) into
the ordering step. A `fn tda_odd_chip_order(winners, button, game) -> Vec<u8>` keeps
the rule in one testable place.

---

## D8-2: No re-open gate — a player who already acted can re-raise a sub-minimum increment

**Severity:** Major — permits an illegal action that materially changes hand outcomes.

### The Poker Rule

TDA Rule 47-A:

> In no-limit and pot limit, an all-in wager (or cumulative multiple short all-ins)
> totaling less than a full bet or raise **will not reopen betting for players who have
> already acted and are not facing at least a full bet or raise** when the action
> returns to them.

Two separate obligations live in that sentence: a **sizing** rule (what the minimum
re-raise is) and a **rights** rule (whether the player may raise at all). pkcore
implements the first and not the second.

### What is already correct

Raise *sizing* is right, and this is worth stating plainly because it is the half that
the TDA Illustration Addendum devotes the most space to. `act_all_in` updates
`raise_increment` only when a shove is itself at least a full raise:

```rust
// src/casino/table/actions.rs:660
let raise_delta = self.bet.saturating_sub(old_bet);
if raise_delta >= self.min_raise() {
    self.raise_increment = raise_delta;
    self.raises_this_street = self.raises_this_street.saturating_add(1);
}
```

So `raise_increment` holds *the last full valid bet or raise of the round* — exactly
the quantity TDA 47-A names as the minimum when short all-ins do re-open. Both
Addendum worked examples trace correctly:

- **Addendum 47 Example 1** (NLHE 50/100, A bets 100, B all-in 125, C calls, D all-in
  200, E calls): deltas of 25 and 75 each fall short of `min_raise()` = 100, so
  `raise_increment` stays 100 and `min_raise_to()` = `200 + 100` = **300**. TDA agrees.
- **Addendum 43 Example 2** (blinds 50/100, A all-in 150): delta 50 < 100, increment
  stays 100, so B's minimum re-raise is to **250**. TDA agrees.

Regression coverage exists in both directions: `all_in_full_raise_reopens_min_raise`
(`src/casino/table.rs:3254`) and `sub_min_all_in_does_not_reopen_min_raise`
(`src/casino/table.rs:3288`).

### Root Cause

The gate does not exist. `raise_bounds` is the sole authority on whether a raise is
legal, and it consults only the raise cap and the stack:

```rust
// src/casino/table/actions.rs:337
pub fn raise_bounds(&self, seat_number: u8) -> Option<(usize, usize)> {
    let min = self.min_raise_to();
    // validate_raise(min) folds every reason a raise could be illegal (cap
    // reached, min above the structure ceiling because the stack is short)
    // into one check.
    if self.validate_raise(seat_number, min).is_err() {
        return None;
    }
    Some((min, self.max_raise_for(seat_number)))
}
```

That comment — "every reason a raise could be illegal" — is the defect in one line. It
enumerates cap and stack. It does not include "this seat already acted this street and
is not now facing a full raise", because no such state is tracked.
`PlayerState::YetToAct` exists (`src/casino/player.rs:35`) but drives betting-round
completion, not re-open rights: it is cleared on action and never consulted to deny a
raise.

Note that the existing regression test asserts only on `raise_increment` — the sizing
side. Neither test asserts that a player *may not raise*, which is why the gap survived.

### Symptom

NLHE 50/100, post-flop, three players with deep stacks behind:

| Step | Action | `bet` | `raise_increment` | TDA | pkcore |
|---|---|---|---|---|---|
| 1 | A bets 300 | 300 | 300 | — | — |
| 2 | B all-in 450 (delta 150) | 450 | 300 *(unchanged, 150 < 300)* | does not re-open | agrees |
| 3 | C calls 450 | 450 | 300 | C had not acted; legal | agrees |
| 4 | **action returns to A** | 450 | 300 | A already acted and faces 150, **not** a full raise → **call or fold only** | `raise_bounds(A)` returns `Some((750, stack))` → **A may raise to 750** |

At step 4 pkcore permits an action the rules forbid. Because the sizing is right, the
raise it offers is a *plausible* 750 rather than an obviously wrong number, so the
error is unlikely to be spotted in a hand history by eye.

### Fix sketch

Track two things per seat per street: whether the seat has voluntarily acted, and the
`bet` level it faced when it last acted. Then gate in `raise_bounds`:

```
if seat.has_acted_this_street
   && (self.bet - seat.bet_faced_when_last_acted) < self.min_raise() {
    return None;   // call or fold only
}
```

Both fields reset wherever `raise_increment` resets (`src/casino/table.rs:1370`,
`:1490`). This is deliberately the *cumulative* form — it compares against the level
faced when the seat last acted, not against the previous single all-in — which is what
makes multiple short all-ins accumulate correctly per 47-A without any extra machinery.

---

## D8-3: Pot-limit pre-flop maximum uses the actual pot, so a dead or short blind shrinks it

**Severity:** Major — caps a legal bet below its true maximum in PLO.

### The Poker Rule

TDA Rule 54-B:

> Pre-flop **a dead or short all-in blind will not affect pot calculation. All pre-flop
> pot and re-pot bets will assume full blinds were posted.** Ex 1: PLO, 100-200 blinds,
> dead SB, BB posts 200. Ex 2: SB posts 100, BB short posts 100. In both examples the
> pot-limit bet for first player to act is **700**.

Both examples deliberately land on the same answer to make the point: the pot-limit
maximum is computed from the blind *structure*, not from what physically reached the pot.

### Root Cause

`max_raise` takes the pot as a caller-supplied parameter and uses it as given:

```rust
// src/games/betting_structure.rs:165
pub fn max_raise(&self, pot: usize, current_bet: usize, my_committed: usize, stack: usize, tier: BetTier) -> usize {
    match self {
        BettingStructure::NoLimit => stack,
        BettingStructure::PotLimit => {
            let call_amount = current_bet.saturating_sub(my_committed);
            let pot_max = current_bet.saturating_add(pot).saturating_add(call_amount);  // :170
            pot_max.min(stack)
        }
        ...
```

The formula itself is correct and doctested (`:155` asserts 1200 for `pot=1000,
current_bet=100`). There is simply no pre-flop branch that substitutes notional full
blinds for the actual short or dead ones, and no caller supplies one.

### Symptom

Both TDA worked examples, PLO 100/200, first player to act:

| | Actual pot | `current_bet` | `call_amount` | pkcore `pot_max` | TDA | Shortfall |
|---|---|---|---|---|---|---|
| **Ex 1** — dead SB, BB posts 200 | 200 | 200 | 200 | `200+200+200` = **600** | 700 | −100 |
| **Ex 2** — SB 100, BB short 100 | 200 | 100 | 100 | `100+200+100` = **400** | 700 | −300 |

Computed the TDA way — assuming full blinds, so `pot = 100 + 200 = 300` and
`current_bet = 200` — both give `200 + 300 + 200 = 700`.

The consequence is one-directional and therefore quiet: the engine only ever offers a
maximum that is too *small*. No illegal bet is accepted, so nothing errors; a legal
bet is simply unavailable. In Ex 2 the ceiling is 43% of its true value.

### Fix sketch

Pre-flop, derive the pot-limit inputs from `ForcedBets` rather than from collected
chips: substitute the full small and big blind for whatever was actually posted, and
use the full big blind as `current_bet` when the posted big blind is short. Post-flop
is already correct per 54-C and must keep using the actual pot, so the substitution
must be gated on `self.phase.is_preflop()` (`src/casino/table.rs:744`).

---

## D8-4: Dead button is not implemented — blinds skip to the next occupied seat

**Severity:** Major — changes which players post, the pot size, and the action order.

### The Poker Rule

TDA Rule 32, in full:

> Tournament play will use a dead button.

One sentence, large consequence. Under a dead button the button advances by *position*
and may land on a seat vacated by elimination; a small blind position that is empty is
simply **not posted** (a "dead" small blind). The alternative — advancing to the next
live player — is the moving-button/live-blind convention used in cash games, and it
makes a player post a blind they do not owe.

### Root Cause

Button *movement* is dead-button compatible. `button_up` advances by raw seat index and
can land on an empty seat:

```rust
// src/casino/table.rs:1463
pub fn button_up(&mut self) {
    self.button = (self.button + 1) % self.seats.size().max(1);
    self.log(TableAction::MoveButton(self.button));
}
```

Blind *derivation* then undoes it. Both blinds resolve by walking to the next
**occupied** seat:

```rust
// src/casino/table.rs:479
pub fn determine_small_blind(&self) -> u8 {
    if self.count_occupied_seats() <= 2 {
        // Heads-up rule: the button/dealer is the small blind.
        self.occupied_seat_at_or_after(self.button)
    } else {
        self.next_occupied_seat_after(self.button, 1)   // ← skips the dead seat
    }
}
```

`determine_big_blind` (`:518`) does the same with an offset of 2. Because the search
skips empties, a dead small blind is unreachable: some live player always posts it.

The heads-up branch and `determine_utg` (`:533`) implement TDA 34-B correctly and are
doctested — this finding is narrowly about the full-ring blind walk.

### Symptom

Six seats, button at seat 0, seats 1 and 2 eliminated, seats 3/4/5 live. `button_up()`
sets `button = 1` (an empty seat — correct so far).

| | Small blind | Big blind | Blinds posted | Pre-flop pot |
|---|---|---|---|---|
| **TDA (dead button)** | seat 2 — empty → **dead, unposted** | seat 3 | BB only | 1 BB |
| **pkcore** | seat 3 | seat 4 | SB + BB | 1 SB + 1 BB |

Two divergences follow from one root: the pot is a small blind too large, and first
action pre-flop sits on a different seat (`determine_utg` is derived from the same
walk). Over a tournament this also changes *how often* each player posts, since the
dead-button rule is what guarantees nobody pays a blind out of rotation after an
elimination.

### Fix sketch

Separate button position from blind assignment. Keep `button_up` as-is, then resolve
the small and big blind by **position** rather than by occupancy: compute the seat
index that owes each blind, and if that seat is unoccupied, mark the blind dead rather
than searching onward. This needs a representation for "this blind is dead" in
`ForcedBets`/`act_forced_bets` (`src/casino/table/actions.rs:70`, `:213`) so that
`act_forced_bet_small_blind` can be a no-op for the hand.

Deliberately out of scope here: this changes pot sizes in existing fixtures and
recorded hand histories. Any fix needs a decision on whether to migrate stored
histories or version the behaviour — see [Prevention](#prevention).

---

## D8-5: No substantial-action predicate exists

**Severity:** Major (structural) — an absence that blocks five other rules.

### The Poker Rule

TDA Rule 36 defines the single most reused predicate in the ruleset:

> Substantial Action is either **A)** any 2 actions in turn, at least one of which puts
> chips in the pot (i.e. any 2 actions except 2 checks or 2 folds) or **B)** any
> combination of 3 actions in turn (check, bet, raise, call, fold). **Posted blinds do
> not count towards SA.**

SA is the commit point past which errors stop being correctable. It is not itself a
player-facing rule — it is the boundary condition that five other rules key off.

### Root Cause

It does not exist. Searching `substantial` across `pkcore/src` returns only unrelated
prose in `src/play/game.rs` and `src/analysis/store/db/sqlite.rs` comments and a match
inside `src/lookups/LICENSE`. There is no counter, no predicate, and no caller.

The nearest existing machinery is `raises_this_street`
(`src/casino/table/actions.rs:663`), which counts raises only and so cannot express
clause A (2 actions, at least one committing chips) or clause B (3 actions of any
kind), and does not exclude posted blinds.

### Symptom

No runtime misbehaviour of its own. The cost is downstream — these rules cannot be
implemented correctly without it, and all five are currently absent:

| Rule | What SA gates |
|---|---|
| 22 | The window in which a pot-accounting error may still be disputed |
| 34-A | Whether a mis-set button is corrected or allowed to stand |
| 35-D | The point past which a misdeal can no longer be declared |
| 52-A | The window for correcting an undersized bet or raise on the current street |
| 53-B | Whether an out-of-turn action becomes binding over a skipped player |

Rule 23 (a level change applying to the next hand) also references SA, but is satisfied
independently in pkdealer by deriving the level from a completed-hand count
(`pkdealer/crates/pkdealer_service/src/blind_schedule.rs:73`), so it is not blocked.

### Fix sketch

A per-street counter on `Table`, incremented in the shared action path and reset
wherever `raise_increment` resets (`src/casino/table.rs:1370`, `:1490`). Two fields
suffice: a count of in-turn actions, and a count of those that committed chips. Then

```
fn substantial_action(&self) -> bool {
    self.actions_this_street >= 3
        || (self.actions_this_street >= 2 && self.chip_actions_this_street >= 1)
}
```

Forced blind posting must not increment either counter. This is the highest-leverage
item in this report: it is small, self-contained, and unblocks five rules.

---

## D8-6: Fixed-limit raise cap cannot lift when the event reaches two players

**Severity:** Minor — affects fixed-limit only, at a specific and rare tournament stage.

### The Poker Rule

TDA Rule 48:

> There is no cap on the number of raises in no-limit and pot-limit. In limit play,
> there is a limit to raises **even when heads-up until the event is down to 2
> players**; the house limit applies.

The distinction is table-heads-up versus *event*-heads-up. A limit table that is heads-up
because the other players are sitting out still caps raises; the final two players in
the whole tournament do not.

### Root Cause

`cap_reached` receives only the per-street raise count and so has no way to express the
condition:

```rust
// src/games/betting_structure.rs:231
pub fn cap_reached(&self, raises_this_street: u8) -> bool {
    match self {
        BettingStructure::NoLimit | BettingStructure::PotLimit => false,
        BettingStructure::FixedLimit { raise_cap, .. } => raises_this_street >= *raise_cap,
    }
}
```

The per-street cap is correct and doctested (`:225`). The gap is that
`BettingStructure` is a table-local value with no visibility of tournament state, and
nothing above it supplies one.

### Symptom

Fixed-limit hold'em, final two players of an event, `raise_cap = 3`. After three raises
`cap_reached` returns `true` and `raise_bounds` returns `None`, so a fourth raise is
refused. Per TDA the cap should no longer apply and raising should be uncapped.

Unreachable today from pkdealer, which runs a single table with no event model — so
this cannot currently be hit in the arena. Recorded because it becomes reachable the
moment multi-table tournament support lands.

### Fix sketch

Add an explicit parameter rather than reaching for global state:
`cap_reached(&self, raises_this_street: u8, event_heads_up: bool)`, returning `false`
for `FixedLimit` when `event_heads_up`. The caller that knows the answer is whatever
owns tournament state, which does not exist yet — so the honest interim step is to
thread the flag as `false` and document it, rather than to infer it from table
occupancy, which would be wrong for a sit-out.

---

## Accepted divergences

Recorded so they are not re-reported as defects. Both are deliberate, both are
defensible for a server-authoritative engine, and both should be a *decision* rather
than an accident:

- **Rule 53 (action out of turn), Rule 55 (invalid declarations), Rule 52 (incorrect
  bets)** — TDA repairs; pkcore rejects. `act_check` facing a bet returns
  `InvalidTableAction` (`src/casino/table/seats.rs:375`) where TDA would allow call or
  fold; an undersized raise returns `InsufficientIncrement` (`src/lib.rs:524`) where
  TDA would correct the bet on the current street; out-of-turn actions are refused
  outright (`src/casino/table/actions.rs:489`) where TDA backs up and may bind them.
  Rejection is simpler and safe when a client cannot physically misspeak. **Consequence
  to note:** pre-action buttons (fold / check-fold / call-any) are a form of
  out-of-turn action, so adding them later means revisiting Rule 53 rather than
  building on top of the current refusal.

- **Rule 60 (count of opponent's stack)** — TDA grants a precise count only when facing
  an all-in and only on your turn; pkcore and pkdealer expose exact stacks to everyone
  continuously. Conventional online, and probably right, but it is strictly more
  permissive than the rule and is also an information channel worth remembering when
  reasoning about Rule 67 isolation.

---

## Coverage Gap

Five of the six findings cannot fail an existing test, which is the more important
result than any individual bug:

1. **Absences cannot fail a test.** D8-5 and D8-6 are missing predicates. Nothing
   asserts they exist, so the suite is green and silent.

2. **`divvy_up` is tested symmetrically.** `divvy_up()` (`src/casino/cashier/chips.rs:235`)
   asserts `1000 / 3 → [333, 333, 334]`. That pins the *arithmetic*, which is correct.
   No test asserts *which seat* receives the extra chip relative to the button, so
   D8-1 is invisible at this altitude — and `divvy_up` cannot see the button anyway,
   so the assertion could not be written there.

3. **The re-open tests assert sizing, not rights.** Both
   `all_in_full_raise_reopens_min_raise` (`:3254`) and
   `sub_min_all_in_does_not_reopen_min_raise` (`:3288`) assert on `raise_increment` and
   `min_raise_to()`. Neither asserts that a seat which already acted is *denied* a
   raise. D8-2 sits exactly in the space between the two tests.

4. **Fixtures use a symmetric table.** The doctests and most fixtures build tables with
   the button at seat 0 and every seat occupied. D8-1 needs a button that is not seat 0;
   D8-4 needs eliminated seats between the button and the blinds. Neither shape is
   commonly constructed.

5. **D8-3 fails safe.** The pot-limit ceiling is only ever too small, so no error is
   raised and no invariant trips. Only an assertion on the exact maximum against a
   known TDA figure would catch it.

This matches the Gold Standard framing in `docs/EPIC-00f_Coverage.md`: a real
behavioural change should make a previously-passing test fail. For all six of these, a
fix would make *no* existing test fail — which is the signal that the tests are pinned
at the wrong altitude, not that the fixes are safe.

---

## Prevention

- **Add a TDA conformance test module.** The parsed ruleset in `tda_parsed` carries the
  Illustration Addendum worked examples with their expected numbers — the 700 pot-limit
  bets of 54-B, the 300 minimum re-raise of 47 Example 1, the 250 of 43 Example 2.
  Those are ready-made table-driven assertions with a citable authority, and three of
  them already pass, which makes the module honest rather than aspirational from day one.

- **Cite the rule in the code.** Where a function encodes a TDA rule, name it in the
  doc comment as `TDA 2024 Rule NN`. `completion_raise_to`
  (`src/games/betting_structure.rs:209`) and `heads_up_is_symmetric`
  (`src/casino/table_celled/showdown.rs:82`) are already written this way in spirit;
  making the citation explicit turns the audit from a one-off into something greppable.

- **Build fixtures with an off-zero button and dead seats.** A shared
  `TestData::tda_asymmetric_table()` with the button away from seat 0 and at least one
  eliminated seat between button and blinds would expose D8-1 and D8-4 in any test that
  touches distribution or blind posting.

- **Decide the hand-history question before fixing D8-4.** Changing blind derivation
  changes pot sizes in recorded histories. Whether to migrate stored fixtures or to
  version the behaviour is a decision that should precede the fix, not follow it.

---

## Affected Code

| File | Role | Findings |
|---|---|---|
| `src/casino/cashier/chips.rs:84` | `Stack::divvy_up` — remainder to the last indices | D8-1 |
| `src/analysis/case_eval.rs:231` | `CaseEval::winning_seats` — ascending seat order | D8-1 |
| `src/casino/table_celled/showdown.rs:116` | `divvy_up` call site (heads-up path) | D8-1 |
| `src/casino/table/actions.rs:337` | `raise_bounds` — sole raise-legality authority | D8-2 |
| `src/casino/table/actions.rs:660` | `act_all_in` — increment update (correct; context for D8-2) | D8-2 |
| `src/games/betting_structure.rs:165` | `max_raise` — pot-limit ceiling | D8-3 |
| `src/casino/table.rs:479` | `determine_small_blind` — occupied-seat walk | D8-4 |
| `src/casino/table.rs:518` | `determine_big_blind` — same walk, offset 2 | D8-4 |
| `src/casino/table.rs:1463` | `button_up` — index advance (correct; context for D8-4) | D8-4 |
| `src/games/betting_structure.rs:231` | `cap_reached` — per-street cap only | D8-6 |
| — | no substantial-action predicate anywhere in `src/` | D8-5 |

---

## Out of scope — pkdealer findings

The same audit surfaced gaps outside this crate. Recorded here only as pointers; they
belong in `pkdealer/docs/defects`:

| TDA rule | Finding |
|---|---|
| 29 | No action timer of any kind. The only `timeout` calls are `tokio::time::timeout` inside tests (`pkdealer/crates/pkdealer_service/src/main.rs:3753`). Agents may block indefinitely. |
| 16 | All hands reveal at hand end (`src/casino/table/seats.rs:477`), not when betting closes with a player all-in. |
| 17 | No order of show — no last-aggressor tracking. |
| 10, 11 | Single-table service; no table breaking, balancing, or halt trigger. |
| 71 | No penalty state machine. Rule 69 detection is strong (`pkdealer_boss` SPRT, `detector.rs:131`) but nothing enforces. |
| 67 | Observer path is correctly redacted; seat-to-seat context isolation is not enforced. |

---

## Verification

No test suite was executed for this report — every finding is from source reading, and
`Status: Open` reflects that nothing has been fixed. Before acting on any finding,
confirm the citations still point where they claim:

```bash
cd /Users/christoph/src/github.com/ImperialBower/pkcore
git rev-parse --short HEAD                                   # expect 90d60e70 or later

sed -n '84,101p'  src/casino/cashier/chips.rs                # D8-1 divvy_up remainder
sed -n '231,234p' src/analysis/case_eval.rs                  # D8-1 ascending winners
sed -n '337,347p' src/casino/table/actions.rs                # D8-2 raise_bounds
sed -n '655,665p' src/casino/table/actions.rs                # D8-2 increment update
sed -n '165,172p' src/games/betting_structure.rs             # D8-3 pot-limit ceiling
sed -n '479,486p' src/casino/table.rs                        # D8-4 occupied-seat walk
sed -n '231,236p' src/games/betting_structure.rs             # D8-6 cap_reached

rg -c 'substantial_action' src/                              # D8-5: expect no matches

cargo test                                                    # expect green — see Coverage Gap
```

The last two lines are the point of this report: a clean `cargo test` alongside a
`substantial_action` count of zero is the whole finding in miniature.

---

## References

- `tda_parsed/tda_2024.yaml` — the full 2024 TDA ruleset, 71 rules, parsed from
  *2024 Poker TDA Rules PDF Longform Redlines Vers 1.0 FINAL*
- `tda_parsed/tda_2024_online.yaml` — the 50-rule automated-play subset carrying the
  per-rule `implemented` / `evidence` / `gap` audit this report is drawn from
- TDA 2024 Illustration Addendum — worked examples for Rules 43, 45, 46, 47, 51, 52-B,
  53; the source of the 700 / 300 / 250 figures used above
- `docs/defects/DEFECT_003_heads_up_side_pot.md` — prior pot-distribution defect; the
  side-pot stratification it added is what makes Rule 21 pass this audit
- `docs/defects/DEFECT_007_decider_subminimum_raise.md` — prior sub-minimum raise
  defect; adjacent to D8-2 but on the decider side rather than the table gate
- `docs/EPIC-00f_Coverage.md` — the Gold Standard framing used in [Coverage Gap](#coverage-gap)

*TDA rules quoted under permission of the Poker TDA, http://www.pokertda.com, all
rights reserved.*
