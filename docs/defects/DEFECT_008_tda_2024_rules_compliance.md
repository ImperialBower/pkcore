# Defect: Six TDA 2024 Compliance Gaps in Betting, Button and Pot Distribution

**File:** `docs/defects/DEFECT_008_tda_2024_rules_compliance.md`
**Date:** 2026-08-16
**Severity:** Major (4 Major, 2 Minor — no incorrect *pot total*; all findings distribute, size or gate chips wrongly)
**Status:** **Closed** (2026-08-21). D8-1 (`DEFECT_011`), D8-2 (`DEFECT_010`), D8-3 (`DEFECT_012`), D8-4 (`DEFECT_013`) and D8-5 (`DEFECT_009`) all fixed in `0.5.0`. D8-6 is closed as an [accepted divergence](#accepted-divergences): it needs a multi-table event model that does not exist, so there is nothing to fix yet. Reopen it as its own `DEFECT_0NN` when that model lands. All six verified by source reading at `90d60e70` (`main`, 2026-08-16), pkcore `0.4.0`.
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
| D8-1 | 20-A/B/C | Odd chip goes to the highest-numbered winning seat, not first-left-of-button | Major | `cashier/chips.rs:95`, `analysis/case_eval.rs:231` — promoted to `DEFECT_011`, **fixed in 0.5.0** |
| D8-2 | 47-A | No re-open gate: a player who already acted may re-raise facing a sub-minimum increment | Major | `casino/table/actions.rs:337` — promoted to `DEFECT_010`, **fixed in 0.5.0** |
| D8-3 | 54-B | Pot-limit pre-flop max uses the actual pot; a dead/short blind shrinks it | Major | `games/betting_structure.rs:170` — promoted to `DEFECT_012`, **fixed in 0.5.0** |
| D8-4 | 32 | Dead button not implemented — blinds skip to the next occupied seat | Major | `casino/table.rs:479` — promoted to `DEFECT_013`, **fixed in 0.5.0** |
| D8-5 | 36 | No substantial-action predicate exists; blocks rules 22, 34-A, 35-D, 52-A, 53-B | Major (structural) | absence across `src/` — promoted to `DEFECT_009`, **fixed in 0.5.0** |
| D8-6 | 48 | Fixed-limit raise cap cannot lift when the event reaches two players | Minor | `games/betting_structure.rs:231` |

Two further deviations are recorded as [Accepted divergences](#accepted-divergences)
rather than defects.

---

## D8-1: Odd chips are awarded by seat index, not by button position

> **Promoted to [`DEFECT_011_odd_chip_button_order.md`](DEFECT_011_odd_chip_button_order.md) on 2026-08-17,
> and fixed there the same day in pkcore `0.5.0`.**
> That document supersedes this section: it carries the fix design, the reason the
> rule lives in a new pure module rather than on either table, the stud
> interpretation, and the nine assertions that pin it. The analysis below is
> retained for the audit record.
>
> Rule 20 now has one implementation, `src/casino/tda.rs`, called by the three
> payout sites in `Table` **and** the three in `TableCelled` — the finding was
> reachable through both. Cases A and B are implemented; case C (hi/lo) is
> recorded as unreachable, because pkcore ships no hi/lo variant. Two findings
> remain open: D8-3 and D8-4.

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

> **Promoted to [`DEFECT_010_reopen_gate.md`](DEFECT_010_reopen_gate.md) on 2026-08-16,
> and fixed there the same day in pkcore `0.5.0`.**
> That document supersedes this section: it carries the full fix design (the
> has-acted predicate already exists as `is_yet_to_act_or_blind`, so only one new
> field is needed), the test plan, and the downstream impact on bot deciders.
> The analysis below is retained for the audit record.
>
> The rule now has one implementation, `Table::is_reopen_gated`, consulted by
> both `Table::raise_bounds` and `TableSnapshot`, and seven assertions in
> `tests/tda_conformance.rs` pin it. Three D8-N findings remain open: D8-1,
> D8-3, D8-4. The choke point this fix introduced,
> `record_voluntary_action`, is also where `DEFECT_009` counts substantial
> action.

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

> **Promoted to [`DEFECT_012_short_blind_pot_limit.md`](DEFECT_012_short_blind_pot_limit.md) on 2026-08-17,
> and fixed there the same day in pkcore `0.5.0`.**
> That document supersedes this section: it carries the fix design, why the
> shortfall is stored rather than derived, why the bots get a separate field
> instead of an adjusted `pot`, and the three assertions that pin it — including
> the over-correction guard for Rule 54-C. The analysis below is retained for
> the audit record.
>
> `Table::pot_limit_pot` is now the single source of the pot a pot-limit ceiling
> is sized against, backed by the new `Table::blind_shortfall` and carried to the
> bots as `TableSnapshot::pot_limit_pot`. D8-4 (dead button) was fixed next, as
> [`DEFECT_013`](DEFECT_013_dead_button.md).

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

**Half of 54-B is already satisfied.** Measured on a live table (`tests/tda_conformance.rs`,
Ex 2 setup — PLO 100/200 with a 100-chip big blind), `act_forced_bets` leaves
`table.bet = 200` even though the big blind could only post 100 and is all-in. So the
*bet-to-call* term already ignores the shortfall exactly as 54-B requires. Only the
**pot** term is wrong: it carries the 200 actually collected rather than the 300 that
full blinds would represent.

### Symptom

Both TDA worked examples, PLO 100/200, first player to act. Measured values, not
derived — `table.bet` and `effective_pot()` were read from a constructed table:

| | Actual pot | `current_bet` | `call_amount` | pkcore `pot_max` | TDA | Shortfall |
|---|---|---|---|---|---|---|
| **Ex 1** — dead SB, BB posts 200 | 200 | 200 | 200 | `200+200+200` = **600** | 700 | −100 |
| **Ex 2** — SB 100, BB short 100 | 200 | 200 | 200 | `200+200+200` = **600** | 700 | −100 |

Computed the TDA way — assuming full blinds, so `pot = 100 + 200 = 300` — both give
`200 + 300 + 200 = 700`.

Note the two examples converge on the same wrong answer for the same reason, and the
shortfall in each case is exactly the amount of blind that never reached the pot: 100.
That is the whole defect in one sentence — **the pot-limit maximum is short by the
unposted portion of the blinds.**

The consequence is one-directional and therefore quiet: the engine only ever offers a
maximum that is too *small*. No illegal bet is accepted, so nothing errors; a legal bet
is simply unavailable.

### Fix sketch

Narrower than it first appears, because `current_bet` is already right. Pre-flop, the
pot term handed to `max_raise` should be the pot *as if full blinds were posted* —
i.e. add back the difference between `ForcedBets`' small and big blind and what each
blind actually contributed. Post-flop
is already correct per 54-C and must keep using the actual pot, so the substitution
must be gated on `self.phase.is_preflop()` (`src/casino/table.rs:744`).

---

## D8-4: Dead button is not implemented — blinds skip to the next occupied seat

> **Promoted to [`DEFECT_013_dead_button.md`](DEFECT_013_dead_button.md) on 2026-08-17,
> and fixed there the same day in pkcore `0.5.0`.**
> That document supersedes this section: it carries the fix design, the
> dead-SB / live-BB interpretation and the evidence for it, the nine assertions
> that pin it across both table types, and the measurement of how many archived
> hands it changes. The analysis below is retained for the audit record.
>
> The small blind is now derived by **position** and goes unposted when that
> seat is vacant; the big blind walks from its position to the first live player
> and is never dead. `DEFECT_012`'s `blind_shortfall` absorbs a dead blind with
> no special case, which makes **TDA 54-B Example 1** reachable and green for
> the first time. The hand-history question this section flagged as needing a
> decision first was settled by archiving the recorded sessions to
> `data/hands/legacy/` rather than versioning the behaviour.

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

> **Promoted to [`DEFECT_009_substantial_action_predicate.md`](DEFECT_009_substantial_action_predicate.md) on 2026-08-16,
> and fixed there on 2026-08-17 in pkcore `0.5.0`.**
> That document supersedes this section: it carries the full predicate design, the
> increment and reset sites, the eleven-assertion test plan, and the open question
> about the stud bring-in. The analysis below is retained for the audit record.
>
> `Table::substantial_action` now exists, backed by two counters incremented at
> the same choke point `DEFECT_010` introduced, and eleven assertions in
> `tests/tda_conformance.rs` pin it. The five rules it unblocks — 22, 34-A, 35-D,
> 52-A, 53-B — remain unimplemented; each is its own change. Three D8-N findings
> remain open: D8-1, D8-3, D8-4.

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

> **Disposition (2026-08-21): closed as an accepted divergence, not fixed.** The
> fix sketch below needs an "event is heads-up" signal that nothing in pkcore or
> pkdealer can supply today. Reopen as a new `DEFECT_0NN` when a multi-table
> event model exists. Kept here in full so the reopening starts from the
> analysis, not from scratch.

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

Recorded so they are not re-reported as defects. The first two are deliberate,
defensible for a server-authoritative engine, and should be a *decision* rather
than an accident. The third is deferred, not decided:

- **Rule 48 (D8-6, limit raise cap at event-heads-up)** — closed 2026-08-21
  without a fix. `cap_reached` has no event-level signal and no caller can give
  it one until a multi-table event model exists. See [D8-6](#d8-6-fixed-limit-raise-cap-cannot-lift-when-the-event-reaches-two-players)
  for the fix sketch. **Consequence:** the final two players of a fixed-limit
  event would be capped; no such event can be run today.

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

This matches the Gold Standard framing in `../epics/EPIC-00f_Coverage.md`: a real
behavioural change should make a previously-passing test fail. For all six of these, a
fix would make *no* existing test fail — which is the signal that the tests are pinned
at the wrong altitude, not that the fixes are safe.

---

## Prevention

- **A TDA conformance test module — done, `tests/tda_conformance.rs`.** The Illustration
  Addendum publishes its worked examples with expected numbers — the 700 pot-limit bet
  of 54-B, the 300 minimum re-raise of 47 Example 1, the 250 of 43 Example 2 — so they
  are table-driven assertions backed by an external authority that cannot drift with
  pkcore. As landed: **5 conformant tests passed**, and **4 asserted the TDA answer
  for D8-1 through D8-4 and were `#[ignore]`d** with their finding id, so CI stayed
  green while the defects stayed recorded in executable form. Un-`ignore` each as it is
  fixed. **All four are now un-ignored and green** — D8-2 in `DEFECT_010`, D8-1 in
  `DEFECT_011`, D8-3 in `DEFECT_012`, D8-4 in `DEFECT_013`. The harness has no
  ignored tests left.
  D8-5 had no test — the predicate did not exist, so any assertion naming it would not
  compile. `DEFECT_009` closed that in `0.5.0`; eleven Rule 36 assertions are now in
  the conformant group, and D8-2's seven joined it in `0.5.0`.

  Writing it immediately paid for itself: the harness **corrected this report**. D8-3
  was originally written up as yielding 400 in TDA Example 2 on the reasoning that a
  short blind lowers `current_bet`. The live table says otherwise — `table.bet` is
  already 200 — so the real answer is 600 and the defect is half the size first
  claimed. That correction came from running the test, not from reading the code again.

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
  **Settled on 2026-08-17, before the fix:** the three recorded pkarena0 sessions were
  archived to `data/hands/legacy/` with a README, and the behaviour was *not* versioned —
  a permanent engine cost for a one-time archive. 40 of their 133 hands would post
  differently under the dead button; none is replayed by any test.

---

## Affected Code

| File | Role | Findings |
|---|---|---|
| `src/casino/tda.rs` | Rule 20 as pure functions (added in `0.5.0`) | D8-1 |
| `src/casino/cashier/chips.rs:84` | `Stack::divvy_up` — remainder to the last indices | D8-1 |
| `src/analysis/case_eval.rs:231` | `CaseEval::winning_seats` — ascending seat order | D8-1 |
| `src/casino/table_celled/showdown.rs:116` | `divvy_up` call site (heads-up path) | D8-1 |
| `src/casino/table/actions.rs:337` | `raise_bounds` — sole raise-legality authority | D8-2 |
| `src/casino/table/actions.rs:660` | `act_all_in` — increment update (correct; context for D8-2) | D8-2 |
| `src/games/betting_structure.rs:165` | `max_raise` — pot-limit ceiling (unchanged; the caller was fixed in `0.5.0`) | D8-3 |
| `src/casino/table.rs:479` | `determine_small_blind` — occupied-seat walk | D8-4 |
| `src/casino/table.rs:518` | `determine_big_blind` — same walk, offset 2 | D8-4 |
| `src/casino/table.rs:1463` | `button_up` — index advance (correct; context for D8-4) | D8-4 |
| `src/games/betting_structure.rs:231` | `cap_reached` — per-street cap only | D8-6 |
| — | no substantial-action predicate anywhere in `src/` (added in `0.5.0`) | D8-5 |

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

`Status: Open` — nothing has been fixed. What *has* landed is
`tests/tda_conformance.rs`, which turns D8-1 through D8-4 into executable assertions.

D8-1 through D8-4 are reproduced by that harness. D8-6 remains a source-reading
finding — it is unreachable until an event model exists. D8-5 was a source-reading
finding for the same reason an absence always is; `DEFECT_009` made it assertable and
then fixed it.

```bash
cd /Users/christoph/src/github.com/ImperialBower/pkcore
git rev-parse --short HEAD                                   # expect 90d60e70 or later

# The four reproducible findings. Expect 4 failures, one per finding.
cargo test --test tda_conformance -- --include-ignored

# Default run: the 5 conformant assertions pass, the 4 defects are skipped.
cargo test --test tda_conformance

# Citations for the findings the harness cannot hold.
sed -n '231,236p' src/games/betting_structure.rs             # D8-6 cap_reached
rg -c 'substantial_action' src/                              # D8-5: no matches at 90d60e70;
                                                             # non-zero from 0.5.0 onward

cargo test                                                    # expect green — see Coverage Gap
```

Observed at `90d60e70` on 2026-08-16:

```text
running 9 tests
test rule_20_a_odd_chip_goes_to_the_first_seat_left_of_the_button ... FAILED   # D8-1  left: 2  right: 5
test rule_32_dead_button_assigns_blinds_by_position_not_occupancy ... FAILED   # D8-4  left: 3  right: 4
test rule_47_a_player_who_already_acted_may_not_reraise_a_short_all_in ... FAILED   # D8-2 — fixed in 0.5.0, now passes
test rule_54_b_short_blind_must_not_shrink_the_preflop_pot_limit_maximum ... FAILED # D8-3  left: 700  right: 600
test rule_43_ex1_min_reraise_is_the_last_increment_not_the_total ... ok
test rule_43_ex2_short_all_in_does_not_raise_the_minimum ... ok
test rule_47_ex1_min_reraise_after_cumulative_short_all_ins ... ok
test rule_48_raise_cap_applies_only_to_fixed_limit ... ok
test rule_54_c_pot_limit_maximum_uses_the_actual_pot_postflop ... ok

test result: FAILED. 5 passed; 4 failed
```

The final `cargo test` line is still the point of this report: the full suite is green
while four TDA rules are demonstrably broken and `substantial_action` does not exist.
That gap between "green" and "correct" is what the harness exists to close.

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
- `../epics/EPIC-00f_Coverage.md` — the Gold Standard framing used in [Coverage Gap](#coverage-gap)

*TDA rules quoted under permission of the Poker TDA, http://www.pokertda.com, all
rights reserved.*
