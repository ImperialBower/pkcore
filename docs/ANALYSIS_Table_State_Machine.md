# Analysis: The `Table` Hand State Machine — An Abstract Definition of Its State Validations

**Date:** August 2026
**Files:** `src/casino/table.rs`, `src/casino/table/actions.rs`,
`src/casino/table/seats.rs`, `src/casino/table/player.rs`,
`src/casino/state.rs`, `src/games/mod.rs`
**Companion docs:** [`ANALYSIS_TableCelled_vs_Table.md`](./ANALYSIS_TableCelled_vs_Table.md),
[`MURATORI_AUDIT.md`](./MURATORI_AUDIT.md)

This document gives a pure, implementation-independent definition of the state
machine that `casino::table::Table` implements for a poker hand: its state
space, its transition guards, and the invariants that make a hand "valid."
Everything below is derived from the shipping code; method names are cited so
each abstract rule can be traced back to its enforcement point.

---

## 1. The state space

A table is the tuple:

```
T = ⟨ φ, S, β, ι, ρ, P, D, B, M, btn, K, L ⟩
```

| Symbol | Meaning | Field |
|---|---|---|
| `φ` | game-phase label | `phase: GamePhase` |
| `S` | vector of seat states | `seats: Seats` |
| `β` | highest wager this street | `bet: usize` |
| `ι` | last full-raise delta (min re-raise measure) | `raise_increment` |
| `ρ` | raises this street (fixed-limit cap counter) | `raises_this_street` |
| `P` | swept pot | `pot` |
| `D, B, M` | deck, board, muck | `deck, board, muck` |
| `btn` | dealer button | `button` |
| `K` | chip-total snapshot for the hand | `hand_chip_total` |
| `L` | append-only event journal | `event_log: Vec<TableAction>` |

Each seat `sᵢ` carries its own tuple `⟨σᵢ, cᵢ, bᵢ, pᵢ⟩`:

- `σᵢ` — a **player state** from the twelve-valued alphabet defined in
  `casino::state::PlayerState`:

  ```
  σ ∈ { Out, Ready, YetToAct, Check, Blind(n), Bet(n), Call(n),
        Raise(n), ReRaise(n), AllIn(n), Showdown(n), Fold }
  ```

- `cᵢ` — stack (`player.chips`)
- `bᵢ` — street commitment (`player.bet`)
- `pᵢ` — hand commitment (`player.chips_in_play`)

### The phase label is narration, not authority

The single most important structural fact: **`φ` is descriptive, not
prescriptive.** Every load-bearing predicate — `next_to_act()`,
`is_betting_complete()`, `is_game_over()` — is a pure function of the
seat-state vector `S` (plus `β`), *not* of `φ`. The dealing methods
(`deal_flop()`, `deal_turn()`, `deal_stud_street()`) *assign* `φ` after acting
rather than checking it as a precondition. The seat vector is the machine;
`GamePhase` is its narration, consumed by observers (session loop, hand
histories, bet-tier selection in stud) rather than by the validators.

---

## 2. The hand lifecycle (macro machine)

A hand is the following cycle. The `act()` regulator
(`table/actions.rs`) drives it for hold'em-family games; the session layer
sequences the same steps for stud/razz.

```
        ┌──────────────────────────────────────────────────────────────┐
        │                                                              │
        ▼                                                              │
     NewHand                                                           │
        │  act_forced_bets()      K := table_chip_count()              │
        │                         antes (dead) ▸ SB ▸ BB               │
        ▼                                                              │
    ForcedBets                                                         │
        │  deal_cards_to_seats()  clockwise from btn+1,                │
        │                         in-hand seats only                   │
        ▼                                                              │
   DealHoleCards                                                       │
        │                                                              │
        ▼                                                              │
  ┌─ Betting(street k) ◀────────────────────────────┐                  │
  │     │  act_fold / act_check / act_call /        │                  │
  │     │  act_bet / act_raise / act_all_in         │                  │
  │     │  (four-layer guard stack, §3)             │                  │
  │     │                                           │                  │
  │     │  quiescent?  is_betting_complete(S)       │                  │
  │     ▼                                           │                  │
  │  bring_it_in()   guard: ¬game_over ∧ quiescent  │                  │
  │     │            P += Σbᵢ; β,ι,ρ := 0           │                  │
  │     │            σ := YetToAct (unless frozen)  │                  │
  │     ▼                                           │                  │
  │  Deal(street k+1)  burn + board / stud upcard ──┘                  │
  │                                                                    │
  └─ terminal?  is_game_over(T) ── yes ─▶ end_hand()                   │
                                            │  settle pots (1-way /    │
                                            │  heads-up / multiway)    │
                                            │  reset()                 │
                                            │  chip audit: total = K ? │
                                            └──────────────────────────┘
```

Two derived predicates gate the macro transitions:

### Quiescence — `Seats::is_betting_complete()`

True iff no one is owed a turn and every live, non-all-in player has matched
the highest commitment:

```
|active(S)| ≤ 1
∨ |can_still_act(S)| < 1
∨ ∀ s ∈ S:  ¬(σₛ ∈ {YetToAct, Blind(_)})
          ∧ ( active(s) ∧ ¬all_in(s)  →  bₛ = max_bet(S) )
```

A street may only be closed when quiescent: both `Seats::bring_it_in()` and
`Seats::close_it_out()` return `PKError::ActionIsntFinished` otherwise.

### Termination — `Table::is_game_over()`

```
|active(S)| ≤ 1  ∨  (φ ∈ last_street ∧ is_betting_complete(S))
```

where `last_street` is river for board games and 7th street for stud/razz.
`end_hand()` refuses with `ActionIsntFinished` unless this holds.

---

## 3. Per-action validation: the four-layer guard stack

Every voluntary action `α(seat, amount)` passes **four ordered strata of
validation, all before any mutation**. Each stratum lives at a different level
of the aggregate:

```
Layer 1 — TURN ORDER       (Table)     seat = next_to_act() ?
Layer 2 — AMOUNT LEGALITY  (Structure) min ≤ amount ≤ max, cap not reached ?
Layer 3 — LOCAL TRANSITION (Player)    σ-transition and chip delta possible ?
Layer 4 — CONSERVATION     (Ledger)    stack → bet → pot, audited at hand end
```

### Layer 1 — Turn order (`Table::next_to_act`)

The actor is never stored; it is **derived** on every query:

```
next_to_act() = first seat i, scanning clockwise from
                first_to_act_this_street(), such that:
                    in_hand(i) ∧ ¬all_in(i)
                  ∧ ( σᵢ = Blind(_)
                    ∨ σᵢ = YetToAct
                    ∨ (everyone_has_bet ∧ bᵢ < max_bet(S)) )
```

`first_to_act_this_street()` dispatches on `GameFamily`:

| Family | Street | First to act |
|---|---|---|
| Hold'em / Omaha | all | UTG relative to button (`determine_utg`) |
| Stud Hi | 3rd | left of bring-in (bring-in has already "acted") |
| Stud Hi | 4th–7th | best visible hand (`best_visible_hand_seat(HighStud)`) |
| Razz | 3rd | left of bring-in (highest upcard posted it) |
| Razz | 4th–7th | lowest visible hand (`best_visible_hand_seat(LowRazz)`) |

Any action from a seat `≠ next_to_act()` fails with
`PKError::TableActionOutOfOrder`, is journaled as
`TableAction::InvalidPlayerAction`, and leaves state untouched.

### Layer 2 — Amount legality (`Table::validate_raise`, single source of truth)

For a non-all-in bet or raise-to `a`:

```
a ≥ min_raise_to()                    else InsufficientIncrement
¬ betting.cap_reached(ρ)              else RaiseCapReached      (fixed-limit)
a ≤ max_raise_for(seat)               else ExceedsBettingCap    (pot-limit /
                                                                fixed-limit ceiling)
```

One universal escape hatch applies: **the all-in bypass**. If
`a ≥ total_chip_count(seat)`, Layer 2 is skipped — a short stack may always
shove for less. In capped structures (`act_all_in`), a deep-stack "all-in" is
*degraded* to the largest legal action (raise to the ceiling, or a plain call)
so the machine never enters an illegal state and the `AllIn` that
`legal_actions()` advertises is always accepted.

An opening bet is validated as a raise-from-zero through the same function
(`act_bet` pre-validates via `validate_raise`), so bet and raise legality
cannot diverge.

### Layer 3 — Local transition (`Player::act_bet_internal` and kin)

```
amount > 0                            else InvalidAction
amount ≤ total_chip_count             else InsufficientChips
σ is active                           else InvalidTableAction
delta := amount − b;  delta > 0       else InsufficientChips
chips ≥ delta                         else InsufficientChips
check requires b ≥ max_bet(S)         else InvalidTableAction  (Seats::act_check)
```

Effect on success:

```
chips −= delta;   b += delta;   p += delta;   σ := α
with the coercion:   chips = 0  ⇒  σ := AllIn(b)     (regardless of request)
```

Alongside these procedural guards, `casino::state` defines a **pure transition
algebra** on `PlayerState` — `can_given(σ, σ′)` (legal succession for one
player), `can_given_against(σ, σ′, σ_other)` (legality relative to an
opponent), and `can_act_after(σ, σ_other)` (relative ordering) — a declarative
statement of the same rules. The `&mut self` `Table` enforces legality through
the four layers above; the algebra is the checked-transition backbone of the
`PlayerStateCell` / `TableCelled` variant (see
[`ANALYSIS_TableCelled_vs_Table.md`](./ANALYSIS_TableCelled_vs_Table.md)).

### Layer 4 — Conservation

Chips only ever move along the pipeline `stack → street bet → pot →
winner stacks`, and the endpoints are audited (§5, I1/I2).

### Two properties of the stack, by construction

- **Fail-atomicity** (audit P9d): all validation precedes all mutation. A
  rejected raise leaves the seat still next-to-act; the doc test on
  `act_raise` asserts exactly this.
- **No advisory drift** (audit P9b / P9j.1): the advertising surface
  (`legal_actions`, `raise_bounds`) and the mutating surface (`act_bet`,
  `act_raise`) call the **same** `validate_raise`. What the table says you can
  do and what it will accept cannot disagree.

---

## 4. The betting micro-machine (within one street)

Table-level betting state evolves under these rules:

- **Monotonicity.** `β` never decreases within a street. `Bet(a)` sets
  `β := a`. `Raise(a)` requires `a ≥ β + ι` and sets `β := a`, `ρ += 1`,
  `ι := a − β_old`.
- **Re-opening rule.** An all-in shove re-opens the action (updates `ι`,
  increments `ρ`) **iff** its delta over `β` is at least a full raise
  (`raise_delta ≥ min_raise()`). A sub-minimum all-in leaves `ι` untouched, so
  players who already acted may only call the difference — the classic
  "incomplete raise does not re-open the betting" rule, encoded as a guard on
  the assignment to `ι` (`act_all_in`, audit P9f).
- **Call semantics.** A call targets `β` in total and pays the delta
  `β − bᵢ`. If the stack cannot cover it, the call converts to
  all-in-for-less (side pots reconcile at showdown — see
  [`BUGFIX_short_blind_call_target.md`](./BUGFIX_short_blind_call_target.md)).
  A call when `β − bᵢ = 0` is recorded as a check.
- **Blinds.** Blinds are live partial commitments: `σ = Blind(n)` keeps the
  seat "owed a turn" (the `is_yet_to_act_or_blind` disjunct in both
  quiescence and turn order), which is what buys the big blind its option.
- **Antes are dead money.** `post_dead` routes antes through `pᵢ` without
  touching `bᵢ`: an ante never credits a call and never shrinks the bring-in,
  yet still triggers the all-in coercion if it takes the last chip
  (audit P9a).
- **Street boundary** (`bring_it_in`). Guard: `¬is_game_over ∧ quiescent`.
  Effect: `P += Σbᵢ; bᵢ := 0; β := ι := ρ := 0`, and each surviving seat
  resets `σ := YetToAct` — **unless frozen**: when at most one player still
  has action to give (everyone else folded or all-in), states are left as-is,
  because no further betting is meaningful and a reset would strand the
  quiescence predicate.

---

## 5. Global invariants (the conservation laws)

These hold across every transition; they are what "validated" ultimately
means for a hand.

| # | Invariant | Statement | Enforcement point |
|---|---|---|---|
| I1 | **Chip conservation** | `Σ stacks + Σ street-bets + pot = K` for the whole hand | `K` snapshotted in `act_forced_bets`; audited in `end_hand` → `PKError::ChipAuditFailed { expected, actual }` |
| I2 | **Pot identity** | at showdown, `pot = Σ chips_in_play` | all money — voluntary via `bᵢ`, dead antes directly — routes through `pᵢ` |
| I3 | **Card conservation** | `deck ⊎ board ⊎ muck ⊎ hands` is a permutation of the game's full deck | audited on `reset()` → `DeckPassesAudit` / `NotEnoughCards` / `TooManyCards` journaled |
| I4 | **Dealing exclusivity** | cards go only to in-hand seats, clockwise from `btn + 1`; a draw fails only on resource exhaustion | `deal_card_to_seat*`, `deal_cards_to_seats`, `deal_stud_*` → `PKError::NotEnoughCards` |
| I5 | **Dead money stays dead** | antes enter `pᵢ` but never `bᵢ` | `Player::post_dead`, `Seats::post_dead_ante` |
| I6 | **Journal completeness** | every accepted *and rejected* action appends to `L` (rejections as `InvalidPlayerAction`) | every `act_*` method |

### Settlement (`end_hand`)

```
guard   is_game_over(T)                      else ActionIsntFinished
        |active(S)| ≥ 1                      else Fubar

effect  winnings := match |active(S)|:
            1 → showdown_single_seat()       (fold-win: uncalled bet returned)
            2 → showdown_headsup()
            _ → showdown_multiway()          (side pots from pᵢ strata)
        reset()                              (muck → deck, sort, audit I3,
                                              φ := NewHand, β = ι = ρ = P = 0)
        assert table_chip_count() = K        else ChipAuditFailed   (I1)
```

`reset()` deliberately runs *before* the chip audit so the table is left in a
clean, reusable state even when the audit fails and the caller chooses to
continue.

---

## 6. The validation philosophy, in one paragraph

`Table` implements a hand as a **two-level machine**: an outer cycle
`post → deal → (bet* → sweep → deal)* → settle → reset` whose transitions are
gated by two derived predicates — *quiescence* (no one is owed action and all
live non-all-in commitments are equal) and *termination* (≤ 1 player remains,
or the last street is quiescent) — and an inner per-action protocol in which
every action must pass a four-layer guard stack: derived turn order,
structure-parameterized amount bounds with an all-in bypass, local
player-ledger feasibility, and chip movement through a conserved ledger. All
validation strictly precedes all mutation; legality is defined in exactly one
place shared by the advisory and mutating APIs; every event, including
rejections, is journaled; and the whole hand is bracketed by conservation
audits over chips and cards that convert any residual sequencing error into an
explicit failure at settlement. Per-action legality is enforced eagerly, but
macro-step *sequencing* is cooperative — `deal_flop()` does not verify the
phase; the `act()` regulator and the quiescence guards provide ordering, and
the I1/I3 audits are the safety net beneath them. This is the same split the
two engines embody: `TableCelled` validates transitions per-cell as they
happen; `Table` validates outcomes per-hand and makes each entry point
fail-atomic.
