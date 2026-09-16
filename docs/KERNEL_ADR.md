# Decision record — `pkcore` boundary

*One per kernel. It records why the boundary sits here, and which wider and
narrower boundaries were rejected. Borrowed terms are explained once, in
brackets, where they first appear.*

| Field | Value |
|---|---|
| Kernel | `pkcore` |
| Status | Accepted |
| Date | 2026-09-16 |
| Contract | None language-neutral yet. The contract is the Rust public API at `0.15.0`; a WIT world is [`/domain-kernel` Mode C](KERNEL_PURITY_AUDIT.md) and has not been started |
| Closes | [`docs/KERNEL_PURITY_AUDIT.md`](KERNEL_PURITY_AUDIT.md) fix 4; answers invariants 7 and 8 |

---

## 1. Boundary chosen

**One hand, one table.**

`pkcore` owns everything that changes while a single hand is played at a single
table. `Table` owns seat chips and seat state, the pot, the current bet, the
deck, the board, the muck, the phase, the button, and the event log.
`PokerSession` owns hand numbering, the shuffled-deck record, and pending blind
changes (`SessionState`, `src/casino/session.rs:985-992`). Nothing else inside `pkcore` writes
table state.

Everything else belongs to a shell: storage, transport, presentation, player
accounts, and the scheduling of hands into sessions or tournaments. The kernel
never learns how it was called.

This is the smallest cluster of data that must stay consistent as a unit
(DDD: *aggregate*) for poker to be played correctly — and it is the reason
`pkcore` can be driven unchanged from a CLI, a gRPC service (`pkdealer`), a
browser (`pkcore.js`) and Python (`pkcore.py`).

## 2. Wider boundary rejected

**The multi-table session, or the tournament.**

Rejected because the rules that govern *several* tables are a different domain,
not a bigger version of this one. Blind schedules advance on a clock. Seat
balancing moves players between tables. Payout ladders depend on how many
entrants remain. All three require the kernel to know the time, or to know about
tables it is not playing — and invariant 1 forbids the first outright.

Folding them in would also make the all-or-nothing step enormous: a single
`apply` would have to be correct across every table at once. Tournament
operation belongs in a shell above `pkcore`, or in a second kernel of its own.

## 3. Narrower boundary rejected

**The seat.**

Rejected because a seat cannot decide anything by itself. `legal_actions` for one
seat depends on every other seat's bet and state: whether a raise is legal depends
on `raise_increment` and `raises_this_street`, which are table-wide; whether a
call is legal depends on the table's current `bet`. Side-pot resolution needs all
seats at once.

A seat-sized kernel would therefore push the betting rules out into every
consumer, which is exactly the duplication a kernel exists to prevent — and we
already have field evidence of what that looks like (see §7 below).

## 4. How special this logic is (DDD: *subdomain* type)

**Generic.**

The rules of poker are identical for everyone. `pkcore` implements no house
variation and no proprietary twist; a hand resolved here resolves the same way
anywhere. That is the strongest possible justification for extracting it: the
work is shared across every consumer and differentiates none of them, so the
cost of the kernel is paid once and recovered many times. It is also why
language bindings pay off — there is no reason to re-implement fixed public
rules in JavaScript and Python.

*Alternative considered:* label it **Core**, on the grounds that the engine is
the product. Rejected as a description of the *logic* — the engine's value is in
being correct and reusable, not in being unlike anyone else's. If `pkcore` ever
grows house-specific rules that are not public poker, revisit this.

The bot stack (`src/bot/`) is a different thing living in the same crate: it is
**an algorithm the domain uses** (DDD: *cohesive mechanism*), not rules that mean
something. It is pure and packaged like a kernel, but it is not a *domain*
kernel, and nothing in this record governs it.

## 5. What must be all-or-nothing

The kernel's single all-or-nothing step is **`Table::apply_action`**
(`src/casino/table/transition.rs:147`), and it is deliberately smaller than the
table.

One `apply_action` changes one seat's chips and state, `table.bet`,
`raise_increment`, `raises_this_street`, `actions_this_street`,
`chip_actions_this_street`, and two entries in `event_log`. It does **not** move
chips into the pot (that is `bring_it_in`), does not deal (that is `deal_flop` /
`deal_turn` / `deal_river` / `deal_stud_street`), and does not change `phase`.

**Intra-kernel decision.** One `apply` changes **one group, not the whole
state**. The betting group moves per action; the pot, board and phase catch up at
a street boundary, driven from outside. This was already in force and
undocumented. It is recorded here as deliberate.

**One operation must never half-happen and currently can.** "End the street and
deal the next card" is two calls composed in the kernel's own driver tier —
`PokerSession::advance_street` (`src/casino/session.rs:818`), which is
**private**, so no consumer can reach the halfway state on purpose:

```rust
self.table.bring_it_in()?;   // sweeps every seat's bet into the pot
self.table.deal_turn()?;     // can fail: NotEnoughCards, AlreadyDealt
```

If `deal_turn` fails after `bring_it_in` succeeded, the bets are in the pot and
the board is one card short, with no `GamePhase` naming that state and no
rollback — `Table` is mutated in place, so `?` is not a transaction. All four
arms have this shape: stud (`:824-825`), flop (`:830-831`), turn (`:834-835`) and
river (`:838-839`).

Being private limits the blast radius but does not fix it: `run_hand` and
`step` reach `advance_street` on every ordinary hand, so an exhausted deck
strands a real table, not just a misusing caller.

The remedy is not to split anything, because nothing crosses a kernel boundary —
there is only one kernel. It is to validate before sweeping, the same
pre-validation `act_bet` and `act_raise` already carry:
`Table::advance_street(&mut self) -> Result<(), PKError>`.
Tracked as fix 5 in the purity audit.

## 6. Who may write this state — and who currently does

State belongs to the kernel that changes it. Consumers get a projection, never a
write.

`pkcore` ships the projection: `PokerSession::view(Option<Principal>)`
(`src/casino/session.rs:883`) redacts hole cards by identity, and
`TableSnapshot::from_table` does the same for the wire form.

**`pkdealer_service` does not use it.** It writes `seat.player.chips` directly
(`crates/pkdealer_service/src/main.rs:1188`, `:4581`) and re-implements hole-card
redaction (`card_visibility_from_metadata:1052`, `hole_cards_string:1090`). It
calls `.view(` nowhere. That is a duplicated entitlement rule living outside the
kernel that owns it.

The decision recorded here, using *Software Architecture: The Hard Parts*' four
options: **one owner, others ask** (Hard Parts: *delegate*). The table owns chips
and card visibility. The service asks.

Explicitly rejected:

- **Merging the service into the kernel** (Hard Parts: *service consolidation*) —
  grows a god-kernel and breaks invariants 1 and 4.
- **A schema both sides write** (Hard Parts: *data domain*) — that is DDD's
  Shared Kernel in database form, with two writers and no owner.

The work that makes this record true rather than merely stated is purity audit
fixes 3 (adopt the projection in `pkdealer`), 6 (`Table::cap_stacks`, so the cap
is a kernel operation that logs a `TableAction` and stays inside the chip audit)
and 12 (seal the 22 public mutable fields behind getters, once no consumer writes
them). Until fix 12 lands, this section describes an agreement, not an
enforcement.

## 7. Contract change policy

None yet. `pkcore` has no `CONTRACT_POLICY.md` because it has no language-neutral
contract to govern — the public Rust API is versioned by Cargo semver and the
`CHANGELOG.md` rules in `CLAUDE.md`, and the wire form is versioned separately by
`SNAPSHOT_VERSION`.

Write one when Mode C lands a WIT world. Until then the honest statement is: the
contract is the Rust public API, and breaking it costs a major version.

## 8. Where this record is weakest

- The boundary is **described, not enforced**. 22 public mutable fields on
  `Table` mean any consumer can write state this record says it may only read.
- Invariant 7's answer (§5) is now written down but has **no test**. Nothing
  fails if a future change makes `apply_action` move the pot.
- `TableAction` is still both the kernel's internal event type and a declared
  wire enum (purity audit fix 10), so the kernel's vocabulary and the transport's
  vocabulary cannot drift apart without a break.
