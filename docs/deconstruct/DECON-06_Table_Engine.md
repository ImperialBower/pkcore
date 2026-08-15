# DECON-06: Table Engine

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

DECON-05 described the five variants as static data. This epic makes one of them
*happen*. A **table** is a ring of seats, a button, a deck, a pot, and an
append-only **event log**. Dealing a hand means walking the variant's street
table: post the **forced bets**, deal in the prescribed order with the prescribed
visibility, determine who acts first, offer each actor its **legal actions**,
apply one, decide whether the betting round is complete, advance the street, and
eventually trigger **showdown**.

Almost none of that is variant-specific. The parts that are — who is forced to
put money in first, and who speaks first once cards are visible — are exactly the
parts this epic isolates. Board games are ordered by **position**. Stud games are
ordered by what everyone can see: in Stud Hi the best exposed hand speaks first,
in Razz the worst, recomputed from scratch every street as upcards accumulate.

This epic owns the mechanics of playing a hand. It does not own who wins the
chips at the end — that is DECON-07 — nor how the hand is recorded for replay,
which is DECON-08 reading the event log this epic writes.

**Out of scope (SD-07, pack-level):** the source carries a second, duplicate
table engine built on shared mutable references. Only the ordinary-ownership
engine is normative. Nothing in this epic describes or requires the duplicate.

## Status

| Component | Status |
|---|---|
| Seat ring, occupancy, and the button | Planned |
| Blinds, including the heads-up inversion and short posts | Planned |
| Antes as dead money | Planned |
| Stud bring-in selection and posting | Planned |
| Dealing order and per-card visibility | Planned |
| Board dealing with burns | Planned |
| Action ordering by position | Planned |
| Action ordering by visible hand (Stud Hi and Razz) | Planned |
| Legal-action enumeration | Planned |
| Action application and the advisory/mutating agreement | Planned |
| Player states and their transitions | Planned |
| Betting-round completion and street advance | Planned |
| All-in run-out | Planned |
| Showdown trigger and mucking | Planned |
| Event log and narration | Planned |
| Golden vectors: walkthrough, legal actions, forced bets | Planned |

## Goals

- Deal and bet out a complete hand of any of the five variants from a known deck
  to a determined showdown, emitting an ordered **event log** sufficient to
  reconstruct everything that happened.
- Make the **legal-action set** an answerable question, not something discovered
  by attempting an action and observing the failure — and guarantee that an
  action reported legal is never rejected when applied.
- Isolate variant-specific ordering to two decisions — who posts the forced bet,
  and who acts first — so the rest of the engine is variant-blind.
- Keep every hand's chip count conserved and every hand's deck accounted for.

## Scope

**Seat ring.** A table has a fixed number of seat positions, indexed from zero
and arranged in a ring. A seat is either occupied or empty. "Clockwise" means
ascending index, wrapping at the end. Empty seats are skipped when finding
players; they still occupy an index.

**Button.** The button is a seat index. It advances by one seat index each hand,
wrapping. All role derivations start from it.

**Forced bets, board families.** The small blind is the first occupied seat
clockwise after the button; the big blind is the second. **Heads-up inverts
this:** with two or fewer occupied seats, the button itself is the small blind
and the other seat is the big blind. A player who cannot cover a blind posts what
it has and is all-in; the table's required bet level is still the full big blind,
so the short poster still owes the difference and later callers still owe the
full amount.

**Forced bets, stud families.** Every occupied seat with chips posts an **ante**
before any card is dealt. Antes are **dead money**: they go straight to the pot
rather than into any player's street contribution, so no ante earns credit
against a later bet and the bring-in posts its full amount rather than the
difference above the ante. After 3rd street is dealt, one seat posts the
**bring-in**, and the table's required bet level rises to it.

**Bring-in selection.** Only each seat's **first face-up card in dealing order**
— its 3rd-street upcard — is considered, even if later upcards are already
present (as they are during replay).

- *Stud Hi:* the **lowest** upcard brings in, with the ace ranked **high**. Ties
  are broken by the lower suit in the deck's canonical suit order.
- *Razz:* the **highest** upcard brings in, with the ace ranked **low**. Ties are
  broken by the higher suit. Because the ace is low, a king outranks an ace for
  this purpose — the king brings in.

Scan direction and ace ranking are independent inputs, not one flag.

**Dealing order.** Cards are dealt clockwise beginning at the first occupied seat
after the button, one card per seat per pass, repeating for as many passes as the
street's plan requires. Only seats still in the hand receive cards. Each dealt
card carries a visibility — face down or face up — taken from the street table.
Community cards are preceded by a **burn**: one card off the deck to the muck,
then three (flop) or one (turn, river).

**Action ordering.** On any street, a first actor is determined, then the engine
scans clockwise from there for the first seat that still owes action.

- *Board families:* the first actor before the flop is the third occupied seat
  after the button — the seat under the gun — except heads-up, where the button
  acts first. On every later street the first actor is the first occupied seat
  after the button.
- *Stud families, 3rd street:* the first actor is the occupied seat immediately
  clockwise of the bring-in seat. The bring-in has already acted by posting.
- *Stud families, 4th street onward:* the first actor is the seat showing the
  **best** visible hand in Stud Hi, or the **worst** in Razz. This is recomputed
  every street from the upcards visible *on that street* — one on 3rd, two on
  4th, three on 5th, four on 6th and still four on 7th, because 7th street is
  dealt face down.

**Visible-hand strength.** Comparing exposed cards uses a pair-aware ordering:
four of a kind beats three of a kind beats two pair beats one pair beats high
card; within a tier the highest ranks decide in descending order. Razz uses
ace-low ranks and selects the weakest hand by this same ordering.

**Legal actions.** At any point a seat's legal-action set is a subset of {fold,
check, call, bet, raise, all-in}, with bet and raise reported at their **minimum**
legal size — any larger amount up to the structure's ceiling is equally legal.

**Betting completion.** A round is complete when at most one seat is still
active in the hand, or when no active seat can still give action, or when every
active non-all-in seat has acted and has committed exactly the current required
level.

**Street advance.** Advancing sweeps every street contribution into the pot,
clears the required level, the last-raise size, and the raise count, resets
still-acting seats to owing action, and deals the next street.

**All-in run-out.** When betting completes with players all-in, every remaining
street is still dealt. The hand reaches its natural last street before showdown.

**Showdown.** The hand is over when at most one seat is active, or when the last
street's betting is complete. One survivor takes the pot without revealing.
Otherwise the surviving hands are compared. Folded cards go to the muck
immediately; at hand end every card in play returns to the deck and the deck
count is audited.

**Chip conservation.** The total chips at the table are snapshotted when forced
bets are posted and re-checked after distribution. A mismatch is an auditable
failure, not a silent correction.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| Seat ring | Fixed indexed positions; occupancy; clockwise traversal skipping empties | `hand-walkthrough.json` |
| Button | A seat index that advances one position per hand, wrapping | `hand-walkthrough.json` |
| Blinds | Small then big clockwise of the button; inverted heads-up; short posts go all-in without lowering the required level | `forced-bets.json` |
| Ante | Every funded seat posts; the chips are dead money in the pot | `forced-bets.json` |
| Bring-in | Chosen from 3rd-street upcards only; lowest ace-high for Stud Hi, highest ace-low for Razz; suit breaks ties in the scan direction | `forced-bets.json` |
| Dealing order | Clockwise from left of the button, one card per pass, in-hand seats only | `hand-walkthrough.json` |
| Visibility | Each dealt card is face up or face down per the street table | `hand-walkthrough.json` |
| Burn | One card to the muck before each community deal | `hand-walkthrough.json` |
| First actor | By position in board games; by bring-in then visible hand in stud games | `hand-walkthrough.json`, `legal-actions.json` |
| Legal actions | The exact set for a given table state, with minimum sizes | `legal-actions.json` |
| Action application | Applies one action, updates state, appends events | `hand-walkthrough.json` |
| Player state | The seat's standing in the current round and which state may follow which | `hand-walkthrough.json` |
| Betting completion | Deterministic verdict on whether the round is over | `hand-walkthrough.json` |
| Street advance | Sweep, reset, deal next | `hand-walkthrough.json` |
| Showdown trigger | When the hand ends, and whether cards are shown or mucked | `hand-walkthrough.json` |
| Event log | Ordered, append-only, reconstruction-complete, narratable | `hand-walkthrough.json` |

## Design

### Roles from the button

```
occupied_after(start, n) = the nth occupied seat clockwise strictly after start

if occupied_count <= 2:            # heads-up
    small_blind = the button seat itself (or the first occupied seat at or after it)
    big_blind   = occupied_after(small_blind, 1)
    first_preflop_actor = small_blind
else:
    small_blind = occupied_after(button, 1)
    big_blind   = occupied_after(button, 2)
    first_preflop_actor = occupied_after(button, 3)

first_postflop_actor = occupied_after(button, 1)
```

The button advances by one **seat index**, not one occupied seat. Role derivation
then skips empties. The two rules differ, and the difference is observable when
the ring has gaps: the button may sit on an empty chair for a hand while blinds
and action still fall on real players.

### Forced bets by family

| Family | Before dealing | After 3rd street | Required level after posting |
|---|---|---|---|
| Hold'em, Omaha | optional ante from every funded seat, then small blind, then big blind | — | the full big blind |
| Stud Hi, Razz | ante from every funded seat | bring-in from the extreme-upcard seat | at least the bring-in |

Antes never enter a player's street contribution. This is what makes the
bring-in a full post rather than a top-up, and it is why chip conservation must
account for ante chips travelling to the pot by a different route than bets.

### Bring-in selection

```
for each seat still in the hand:
    upcard = the seat's FIRST face-up card in dealing order
    rank_key = ace_low_rank(upcard) if the family ranks the ace low else rank(upcard)
    suit_key = the card's suit ordinal

choose the seat whose (rank_key, suit_key) is
    smallest, if the family's bring-in scans lowest   # Stud Hi
    largest,  if the family's bring-in scans highest  # Razz
```

Restricting the scan to the first face-up card is what makes live play and replay
agree: a replayed hand has all seven cards present from the start, and a scan
over all upcards would pick a different seat than the live dealer did.

The worked Razz case: a seat showing a king and a seat showing an ace. Under
ace-low ranking the ace is the *lowest* card, so the king is the highest upcard
and the king brings in. Under Stud Hi's ace-high ranking with the same two cards,
the ace is the highest and the king would be the lower — the king would bring in
there too, but for the opposite reason. Vectors must include a case where the two
families choose *different* seats.

### Action ordering

```
first_actor(street):
    if family is board:
        preflop -> first_preflop_actor
        else    -> first_postflop_actor
    if family is stud:
        3rd street -> occupied_after(bring_in_seat, 1)
        4th+       -> extreme_visible_hand_seat(best for Stud Hi, worst for Razz)
        anywhere else -> fall back to the positional actor
```

From the first actor, scan clockwise for the first seat that owes action,
skipping empty seats, seats not in the hand, and all-in seats. A seat owes action
if it is still to act, if it is sitting on an unraised forced blind, or if —
once everyone has acted at least once — its street contribution is below the
required level.

**Visible-hand strength** is computed over the upcards visible *on the current
street*, truncated to 1 / 2 / 3 / 4 / 4 for 3rd through 7th:

```
tier = 7 if four of a kind
       6 if three of a kind
       2 if two pair
       1 if one pair
       0 otherwise
score = tier, then the four highest ranks in descending order as tie-breakers
```

Stud Hi picks the highest score; Razz computes ranks with the ace low and picks
the lowest score. When no seat has an exposed card, the ordering falls back to
position rather than failing.

### Legal actions

Given a seat and the current table state, with `to_call` the chips the seat must
add to match the required level:

```
if the seat is unknown, folded, out, all-in, or has no chips:
    return the empty set          # no decision exists

if to_call == 0:
    offer CHECK
    if no bet has been made this street and the stack covers the minimum open:
        offer BET(minimum open)
    else if a voluntary raise is legal:
        offer RAISE(minimum raise-to)     # the big blind's option
else:
    offer FOLD
    offer CALL                    # a short stack's call becomes a partial all-in
    if a voluntary raise is legal:
        offer RAISE(minimum raise-to)

if the stack is non-empty:
    offer ALL-IN                  # always
```

"A voluntary raise is legal" is a single verdict combining three failures: the
minimum raise-to is below what the structure permits, the per-street raise cap is
reached, or the minimum raise-to exceeds the structure's ceiling for this seat.
DECON-05 supplies all three.

**The crown-jewel invariant: an action reported legal is never rejected when
applied.** This forces two consequences a rebuild cannot skip. First, the
advisory answer and the mutating check must come from one rule, not two rules
that agree today. Second, because all-in is always offered, a deep stack shoving
into a **capped** structure must degrade rather than error:

| Capped-structure shove | Result |
|---|---|
| A legal raise exists and the stack exceeds its ceiling | Raise to the ceiling; the seat is **not** all-in |
| The stack fits inside the ceiling | A genuine all-in |
| No raise is legal and the stack exceeds the call | A plain call; the seat is **not** all-in |
| No raise is legal and the stack cannot cover the call | A genuine all-in for less |

No-limit is unaffected: its ceiling is the stack, so every shove is a true
all-in.

**Applying an action.** Fold, check, bet, raise, and all-in map to themselves. A
call by a seat that already matches the required level degrades to a check. Every
application is refused unless the acting seat is the seat currently owing action;
a refusal must leave the table exactly as it was — a rejected raise must not have
moved chips, changed the seat's standing, or advanced the turn.

**Re-opening.** A bet or raise records its increment as the new minimum for
subsequent raises. An all-in that raises the required level by at least a full
minimum increment re-opens the betting and counts against the raise cap. An
all-in for **less** than a full increment does **not** re-open: players who have
already acted may only call the extra, not re-raise.

### Player states and transitions

A seat in a hand occupies exactly one standing:

| State | Meaning |
|---|---|
| Ready | Seated, awaiting the next hand |
| Yet to act | In the hand, has not acted this round |
| Blind | Holding a posted forced blind, not yet a voluntary action |
| Check | Passed without adding chips |
| Bet | Made the first voluntary wager this round |
| Call | Matched the required level |
| Raise / Re-raise | Increased the required level |
| All-in | Committed the entire stack |
| Showdown | Reached the comparison with an amount committed |
| Fold | Relinquished the hand |
| Out | Not participating in this hand |

Governing rules a rebuild must reproduce:

- A folded or out seat is **not active**. An all-in seat **is** still in the hand
  and still reaches showdown, but has no decision.
- Checking is illegal while an unmatched blind or bet stands.
- A seat that has acted may act again only by increasing its commitment, calling
  a higher level, folding, or going all-in — never by repeating the same amount.
- Advancing a street returns every in-hand, non-all-in seat to *yet to act*; when
  at most one seat can still act, standings are frozen instead, so a run-out does
  not manufacture new decisions.
- Emptying a seat's stack transitions it to all-in automatically, whatever action
  caused it.

### Round completion, street advance, and run-out

```
betting_complete:
    at most one seat active in the hand                -> complete
    no active seat that is not all-in                  -> complete
    any seat still to act or holding a live blind      -> not complete
    any active non-all-in seat below the required level -> not complete
    otherwise                                          -> complete

advance_street:
    sweep every street contribution into the pot
    clear the required level, the last-raise size, and the raise count
    reset standings (frozen if at most one seat can still act)
    deal the next street per the variant's street table
```

A driver alternates: ask whether the hand is over; if not and the round is
complete, advance one street; otherwise hand the current actor its legal set. A
single-step form of this loop must be able to report *street advanced* distinctly
from *player to act*, so that an all-in run-out reports one advance per remaining
street rather than skipping silently to the end.

### Showdown and mucking

The hand ends when at most one seat is active, or when the final street's betting
completes — the river for board families, 7th street for stud. With one survivor,
the pot is awarded with no cards revealed. With more, surviving hands are
compared (DECON-02, DECON-03) and the pot is settled (DECON-07). A folding seat's
cards go to the muck at the moment of folding; a seat reaching showdown may
decline to reveal and muck instead. On reset every card — hole cards, board, and
burns — returns to the deck, which is then audited back to 52 cards. The chip
total is audited against the snapshot taken at forced-bet time.

### The event vocabulary

Everything that happens is appended, in order, to a log that is never rewritten
or reordered. The log must be sufficient to reconstruct the hand — that is
DECON-08's contract with this epic — and each entry must be renderable as a
human-readable line. The vocabulary spans:

| Group | Entries |
|---|---|
| Table lifecycle | table opened; player seated; new hand; deck shuffled; button set; button moved; table reset |
| Forced bets | forced bets begin; a forced bet; small blind posted; big blind posted; ante posted; bring-in posted |
| Dealing | dealing N cards; a seat is dealt specific cards; flop dealt; turn dealt; river dealt; all players dealt; cards force-dealt (replay) |
| Turn order | action to a seat; a seat closes the action |
| Player actions | check; bet of an amount; call of an amount; raise to an amount; all-in for an amount; fold |
| Pot | bets collected; current pot size; main pot; side pot; split pot among N; hand closed out with a pot |
| Cards leaving play | cards mucked; a seat's cards mucked; a seat's cards taken; board cards taken |
| Resolution | showdown with N seats; a seat mucks at showdown; all folded to a seat; a seat wins/loses the main or a side pot; a seat wins an amount of a pot with a named hand; a seat loses an amount with a named hand; hand ended |
| Audits and faults | deck passes audit; not enough cards; too many cards; invalid seat; invalid action; invalid action by a seat in a given standing; chip audit failed with expected and actual |

Entries carry the seat and the amount where those apply, and a reader must be
able to extract either without knowing the entry's kind. Narration renders an
entry with a player's display name substituted for the seat where one is known —
"Alice raises to 300", "Flop is A♠ K♠ 7♦" — and narration must be side-effect
free.

> **Spec decision SD-04:** Is the seeded shuffle permutation pinned, or only the
> property that the same seed reproduces the same shuffle? **Options:** pin the
> permutation — a given seed must yield a specific card order / relax — only
> reproducibility is normative. **Chosen: RELAX** — the property is normative,
> the permutation is informative.

A shuffle algorithm and a pseudo-random generator are pure implementation
choices. Pinning the permutation would force every rebuild to adopt one
generator's exact bit stream and one specific shuffle traversal, which is a
transliteration requirement, not a poker requirement. What the domain actually
needs is that an experiment can be rerun: **the same seed, applied to the same
starting deck, must produce the same shuffled deck within one implementation, and
therefore the same hand.** The walkthrough vector accordingly fixes the deck
order *explicitly* rather than by seed, so conformance never depends on any
generator. A rebuild must also preserve the shuffle's other observable
properties: every card present exactly once, count unchanged, and every
permutation reachable.

> **Spec decision SD-09:** Must a rebuild reproduce the original's unbounded
> client access to table state — a directly reachable, writable deck and table?
> **Options:** yes, reproduce it / no, a rebuild should bound it. **Chosen: NO** —
> the unbounded access is an acknowledged soft spot, not a domain rule.

The pack's own perspective analysis rates the User/client boundary only
**Partial**, and says so explicitly: the read path is bounded — a per-identity
projection reveals only the requesting seat's own hole cards and carries no deck
at all — while the state path is not, because table state including the deck is
directly reachable and writable. That asymmetry is a description of an
unfinished boundary, not a behavior any correct implementation must have. No
vector depends on a client being able to rewrite the deck; no rule of poker does
either.

A rebuild **should** enforce that a client cannot see undealt cards and cannot
alter the deck. The redaction property that *is* normative — a viewer sees its own
seat's hole cards and no others, a viewer with no identity sees none, and no view
ever carries the undealt deck — must be preserved. Tightening the state path
beyond the original is explicitly permitted and is not a conformance failure.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | Change dealing order, forced-bet rules, action ordering, or what makes an action legal | Only the library decides how a hand is dealt and bet; a consumer drives a hand, it does not redefine one |
| **Administrative** | Seat and remove players, set stakes, move the button, start and settle hands, bust players out | Deal itself a card, choose which seat acts, or alter a hand in progress | An operator can seat, stake, start, and settle — without changing the rules of the game being dealt |
| **User/client** | See its own seat's hole cards, the board, the pot, the standings, and its own legal-action set | See any undealt card, see another seat's face-down cards, alter the deck, or act out of turn | A client sees only what its own seat is entitled to see, and can change the table only by taking a legal action at its own turn (see **SD-09**) |
| **Observer/operator** | Read the whole event log after the fact and narrate it | Alter or reorder what the log recorded | Anything that already happened can be reconstructed and re-examined without disturbing it |
| **Agent** | Receive a snapshot of the table from its own seat's point of view and choose from the offered actions | Learn the deck's remaining contents or any hidden card from that snapshot | An agent decides only what to do with the cards it holds; it can neither see nor change anything its seat would not know |
| **Trainer/researcher** | Replay a fixed deck through a scripted action sequence and obtain an identical event log | — | A hand is a deterministic function of its deck and its actions; nothing else influences the outcome |
| **Spectator** | See the board, the pot, the standings, and every event that is not a hidden card | See any face-down card | A spectator learns everything about a hand except what the players are holding |
| **Trustless/cryptographic peer** | N/A — recorded as a designed absence pack-wide. | | |

## Work Items

### Phase 0 — Vector harness

- [ ] **0a.** Stand up a runner over `vectors/table-engine/`, reporting the first
      mismatching case identifier and, for the walkthrough, the first divergent
      event index.
- [ ] **0b.** Assert all three vector files parse and are non-empty before any
      production work.

### Phase 1 — Ring, button, and roles

- [ ] **1a.** Write failing cases for small blind, big blind, first preflop actor,
      and first postflop actor across full-ring and heads-up, with and without
      empty seats; prove against `forced-bets.json`.
- [ ] **1b.** Implement clockwise traversal that skips empty seats, and button
      advance by one seat index.
- [ ] **1c.** Prove the heads-up inversion: the button is the small blind and acts
      first before the flop.

### Phase 2 — Forced bets

- [ ] **2a.** Write failing cases for blind posting including a short post that
      goes all-in without lowering the required level; prove against
      `forced-bets.json`.
- [ ] **2b.** Implement antes as dead money and prove that a bring-in following an
      ante posts its full amount.
- [ ] **2c.** Write failing cases for bring-in selection in both stud families,
      including a suit-broken tie and the king-outranks-ace Razz case; prove
      against `forced-bets.json`.
- [ ] **2d.** Prove that bring-in selection consults only the first face-up card,
      by running the same selection against a seat whose later upcards are already
      present.

### Phase 3 — Dealing

- [ ] **3a.** Write failing cases for dealing order and per-card visibility for
      each variant; prove against the deal events in `hand-walkthrough.json`.
- [ ] **3b.** Implement pass-based clockwise dealing from left of the button to
      in-hand seats only.
- [ ] **3c.** Implement 3rd street as two face-down passes then one face-up pass,
      and later stud streets as one pass with the street table's visibility.
- [ ] **3d.** Implement the burn before each community deal and prove the burned
      card leaves the deck and returns at reset.

### Phase 4 — Action ordering

- [ ] **4a.** Write failing cases for the first actor on every street of every
      variant; prove against `hand-walkthrough.json` and `legal-actions.json`.
- [ ] **4b.** Implement positional ordering for board families.
- [ ] **4c.** Implement 3rd-street ordering from left of the bring-in.
- [ ] **4d.** Implement visible-hand strength with the pair-aware tiers and prove
      Stud Hi picks the best and Razz the worst.
- [ ] **4e.** Prove ordering is recomputed each street by asserting that the first
      actor changes when a later upcard changes the best visible hand.
- [ ] **4f.** Prove the upcard window truncates to 1/2/3/4/4 so that a replayed
      hand with all cards present orders identically to live play.

### Phase 5 — Legal actions and application

- [ ] **5a.** Write failing cases for every legal-action set in
      `legal-actions.json`, including the empty set for folded, all-in, busted,
      and unknown seats.
- [ ] **5b.** Implement the legal-action set with bet and raise at minimum size.
- [ ] **5c.** Implement action application sharing **one** legality rule with the
      advisory set.
- [ ] **5d.** Prove the crown-jewel invariant by applying every advertised action
      from a fresh copy of the same state and asserting none is rejected — in
      no-limit and in a capped structure at the raise cap.
- [ ] **5e.** Prove a rejected action leaves the table byte-for-byte unchanged in
      every observable respect, including whose turn it is.
- [ ] **5f.** Implement the capped-structure shove degradation table and prove
      each of its four rows.
- [ ] **5g.** Prove the re-opening rule: a full-increment all-in re-opens and
      counts toward the cap; a short all-in does neither.

### Phase 6 — States, completion, and advance

- [ ] **6a.** Write failing cases for the player-state transitions, including
      illegal ones; prove refusal.
- [ ] **6b.** Implement round completion and prove each of its five verdicts.
- [ ] **6c.** Implement street advance with sweep, reset, and frozen standings.
- [ ] **6d.** Prove the all-in run-out deals every remaining street and reports one
      advance per street.

### Phase 7 — Showdown and audits

- [ ] **7a.** Implement the showdown trigger and the single-survivor award with no
      reveal.
- [ ] **7b.** Implement mucking on fold and at showdown.
- [ ] **7c.** Implement the deck audit at reset and the chip audit at hand end, and
      prove each fails loudly when violated.

### Phase 8 — Event log

- [ ] **8a.** Emit the full event vocabulary in order and prove
      `hand-walkthrough.json` matches event for event.
- [ ] **8b.** Prove the log is append-only across a whole hand — no entry is
      removed, edited, or reordered.
- [ ] **8c.** Implement narration and prove it is side-effect free by asserting the
      log is unchanged after narrating it twice.
- [ ] **8d.** Prove seat and amount are extractable from any entry that carries
      them, without knowing the entry's kind.

### Phase 9 — Closeout

- [ ] **9a.** Run the full vector suite green and record the case count.
- [ ] **9b.** Confirm no client-facing view exposes an undealt card (**SD-09**).

## Test Plan

**Given** a three-seat table with the button at seat 0, **when** roles are
derived, **then** seat 1 is the small blind, seat 2 the big blind, and seat 0 the
first preflop actor. *(`forced-bets.json`)*

**Given** a two-seat table, **when** roles are derived, **then** the button seat
is the small blind, the other is the big blind, and the button acts first before
the flop. *(`forced-bets.json`)*

**Given** a big blind whose stack is smaller than the blind, **when** it posts,
**then** it is all-in for its stack **and** the required level is still the full
big blind, so a caller owes the full amount. *(`forced-bets.json`)*

**Given** a stud table with an ante, **when** antes are posted, **then** every
funded seat's chips are in the pot, no seat has a street contribution, and the
subsequent bring-in posts its full amount. *(`forced-bets.json`)*

**Given** a Stud Hi 3rd street, **when** the bring-in is selected, **then** the
seat with the lowest upcard (ace high, ties to the lower suit) posts.
*(`forced-bets.json`)*

**Given** a Razz 3rd street where one seat shows a king and another an ace,
**when** the bring-in is selected, **then** the king posts. *(`forced-bets.json`)*

**Given** identical 3rd-street upcards dealt under Stud Hi and under Razz, **when**
each selects its bring-in, **then** the two families choose different seats.
*(`forced-bets.json`)*

**Given** a fixed deck and a table of known seats, **when** hole cards are dealt,
**then** the first card goes to the first occupied seat after the button and
subsequent cards follow clockwise, one per seat per pass.
*(`hand-walkthrough.json`)*

**Given** stud 3rd street, **when** dealing completes, **then** each in-hand seat
holds two face-down cards and one face-up card, and the face-up card is the third
card it received. *(`hand-walkthrough.json`)*

**Given** a board family before the flop, **when** the flop is dealt, **then**
exactly one card has been burned to the muck and exactly three are on the board.
*(`hand-walkthrough.json`)*

**Given** a Stud Hi 5th street where one seat shows a pair, **when** the first
actor is determined, **then** it is that seat; **given** the same upcards under
Razz, **then** it is not. *(`hand-walkthrough.json`)*

**Given** a replayed stud hand with all seven cards already present, **when** the
first actor is determined on 4th street, **then** it matches the live hand's
first actor, because only two upcards are considered.
*(`hand-walkthrough.json`)*

**Given** the seat under the gun facing the big blind, **when** its legal set is
requested, **then** it contains fold, call, raise at minimum, and all-in, and
contains neither check nor bet. *(`legal-actions.json`)*

**Given** a folded, all-in, busted, or unknown seat, **when** its legal set is
requested, **then** the set is empty. *(`legal-actions.json`)*

**Given** a stud completer facing only the bring-in, **when** its legal set is
requested, **then** the offered raise is the completion to one full small bet,
not the bring-in plus a small bet. *(`legal-actions.json`)*

**Given** any state in `legal-actions.json`, **when** each offered action is
applied to a fresh copy of that state, **then** none is rejected.
*(`legal-actions.json`)*

**Given** a fixed-limit street at the raise cap with a deep-stacked actor,
**when** its legal set is requested, **then** no raise is offered but all-in is;
**and when** all-in is applied, **then** it succeeds as a call and the seat is not
all-in. *(`legal-actions.json`)*

**Given** an under-minimum raise, **when** it is applied, **then** it is rejected
**and** the table is unchanged — the same seat still owes action and no chips
moved. *(`legal-actions.json`)*

**Given** an all-in that raises by less than a full increment, **when** a player
who has already acted responds, **then** it may call the extra but may not
re-raise; **given** an all-in that raises by a full increment, **then** it may
re-raise. *(`legal-actions.json`)*

**Given** all remaining players all-in before the flop, **when** the hand is
driven to completion, **then** the flop, turn, and river are each dealt and each
advance is reported separately. *(`hand-walkthrough.json`)*

**Given** every opponent folding, **when** the hand ends, **then** the survivor is
awarded the pot and no hole cards are revealed. *(`hand-walkthrough.json`)*

**Given** the scripted hand in `hand-walkthrough.json`, **when** it is replayed
from the recorded deck through the recorded actions, **then** the emitted event
sequence matches the recorded sequence entry for entry, in order.
*(`hand-walkthrough.json`)*

**Given** a completed hand, **when** the table resets, **then** the deck holds 52
cards and the chip total equals the total snapshotted at forced-bet time.
*(`hand-walkthrough.json`)*

## Not specified (implementer's choice)

- **Shuffle algorithm and random generator.** Any correct uniform shuffle. Only
  reproducibility under a fixed seed is required, and the walkthrough vector fixes
  the deck explicitly so conformance never touches the generator. See **SD-04**.
- **How the seat ring is stored** — a dense array with empty markers, a sparse
  map, a linked ring. Only clockwise traversal semantics bind.
- **How table state is guarded.** Ownership, immutability, message passing,
  locking, or transactions. A rebuild is free — and encouraged — to bound client
  access more tightly than the original. See **SD-09**.
- **Error and refusal representation.** Return values, errors, exceptions, or
  result objects, so long as a refusal changes nothing.
- **How the event log is stored and how its entries are named.** DECON-08 pins
  the recorded *information*; the entry names here are domain descriptions, not
  identifiers.
- **Narration wording.** Human-readable strings are cosmetic. Only the structured
  events bind.
- **Whether the engine drives itself or is driven.** A step function, a callback
  loop, a coroutine, or an external scheduler are all acceptable, provided the
  per-street advance remains individually observable.
- **Concurrency, multi-table operation, and telemetry.** The original has none in
  its core; a rebuild adding them changes nothing observable here.
- **Module organisation, memory layout, and naming throughout.**

## Spec decisions

> **Spec decision SD-04:** Is the seeded shuffle permutation pinned, or only the
> property that a seed reproduces a shuffle? **Options:** pin the permutation /
> relax to the property. **Chosen: RELAX** — the permutation is informative and
> the reproducibility property is normative, because pinning it would force a
> specific generator and traversal that carry no poker content.

> **Spec decision SD-09:** Must a rebuild reproduce the original's unbounded
> client access to table state (a publicly reachable, writable deck)?
> **Options:** yes / no. **Chosen: NO** — the original's own perspective analysis
> rates the client boundary only Partial and names this an acknowledged soft
> spot; a rebuild SHOULD enforce that a client can neither see undealt cards nor
> alter the deck, and doing so is not a conformance failure.

Both decisions are argued in full under Design, at the points where they bite.

## Verification

Any implementation must reproduce every file under `vectors/table-engine/`:

1. `hand-walkthrough.json` passes: replaying the recorded deck through the
   recorded action sequence emits the recorded event sequence entry for entry, in
   order, ending in the recorded showdown outcome.
2. `legal-actions.json` passes: every recorded table state yields exactly the
   recorded legal-action set, with bet and raise at the recorded minimum sizes.
3. `forced-bets.json` passes: every recorded blind, ante, and bring-in case yields
   the recorded posting seat and the recorded amount, across all five variants.
4. Every action reported legal is accepted when applied — demonstrated for every
   state in `legal-actions.json`, in an uncapped and in a capped structure.
5. A rejected action leaves every observable aspect of the table unchanged,
   including whose turn it is.
6. Bring-in selection consults only each seat's first face-up card, demonstrated
   against a state where later upcards are present.
7. Stud action ordering is recomputed every street from that street's visible
   window, demonstrated by a first-actor change caused by a newly dealt upcard.
8. An all-in run-out deals every remaining street and reports one advance per
   street.
9. Every hand conserves chips against the snapshot taken at forced-bet time, and
   every reset restores the deck to 52 cards.
10. The event log is append-only for the duration of a hand, and narrating it does
    not change it.
11. No client-facing view of the table exposes an undealt card or permits the deck
    to be altered (**SD-09**).

## Dependencies

**Builds on:** DECON-01 (cards, deck, ranks and suits), DECON-05 (street tables,
bet tiers, raise sizing, positions).
**Blocks:** DECON-07 (pot accounting settles what this epic collects), DECON-08
(the hand record is derived from this epic's event log), DECON-11 (agents occupy
the seats this epic drives).

## Provenance (non-normative)

- `src/casino/table.rs:83` — the table's constituents: seats, button, deck, board,
  muck, pot, required level, last-raise size, event log, chip snapshot, betting
  structure, per-street raise count.
- `src/casino/table.rs:144`, `:181`, `:222`, `:260`, `:305`, `:342` — the five
  variant constructors and the generic one.
- `src/casino/table.rs:408`, `:429` — clockwise traversal over occupied seats.
- `src/casino/table.rs:479`, `:518`, `:533` — small blind, big blind, and first
  preflop actor, including the heads-up inversion.
- `src/casino/table.rs:564`, `:594` — next actor and per-street first actor
  dispatch by family.
- `src/casino/table.rs:612`, `:635` — the Stud Hi and Razz first-actor resolvers.
- `src/casino/table.rs:656` — extreme visible-hand selection and the per-street
  upcard truncation window.
- `src/casino/table.rs:704` — the pair-aware visible-strength score.
- `src/casino/table.rs:779` — the hand-over verdict, including 7th street.
- `src/casino/table.rs:1088`, `:1103` — dealing one card with a visibility.
- `src/casino/table.rs:1124`, `:1176` — 3rd-street and later stud dealing.
- `src/casino/table.rs:1224` — pass-based hole-card dealing from left of the
  button.
- `src/casino/table.rs:1315`, `:1332`, `:1347` — flop, turn, and river, each
  burning first.
- `src/casino/table.rs:1364`, `:1386` — sweeping bets at a street boundary and at
  hand end, with the raise-count reset.
- `src/casino/table.rs:1399`, `:1422`, `:1432` — mucking a seat, all seats, and
  the board.
- `src/casino/table.rs:1463` — button advance by one seat index.
- `src/casino/table.rs:1470` — reset, card return, and the 52-card deck audit.
- `src/casino/table.rs:1748` — single-survivor award without reveal.
- `src/casino/table.rs:2040` — hand resolution and the chip-conservation audit.
- `src/casino/table.rs:53` — the two visible-hand selection modes.
- `src/casino/table/actions.rs:70` — forced-bet dispatch by family.
- `src/casino/table/actions.rs:105` — antes as dead money.
- `src/casino/table/actions.rs:138` — bring-in posting and the required-level
  raise.
- `src/casino/table/actions.rs:162` — 3rd-street extreme-upcard selection, the
  first-upcard restriction, and the independence of scan direction from ace
  ranking.
- `src/casino/table/actions.rs:213`, `:226` — small and big blind posting.
- `src/casino/table/actions.rs:261`, `:375`, `:426`, `:477`, `:527`, `:597` —
  fold, bet, call, check, raise, and all-in, including the out-of-turn refusal and
  the pre-validation that leaves state untouched.
- `src/casino/table/actions.rs:285`, `:314`, `:337` — the single raise-legality
  rule shared by the advisory and mutating surfaces.
- `src/casino/table/actions.rs:608` — the capped-structure shove degradation.
- `src/casino/table/actions.rs:645` — the re-opening rule for all-ins.
- `src/casino/table/transition.rs:63` — the legal-action set.
- `src/casino/table/transition.rs:147` — action application, including call
  degrading to check.
- `src/casino/table/transition.rs:249`, `:300`, `:320` — the source's own tests of
  the advertised-is-accepted invariant, in no-limit and at the fixed-limit cap.
- `src/casino/table/seats.rs:141`, `:176`, `:224` — dealt check, betting
  completion, and the scan for the seat owing action.
- `src/casino/table/seats.rs:279`, `:319` — sweeping contributions, with the
  frozen-standings case.
- `src/casino/table/seats.rs:411`, `:439`, `:457`, `:468`, `:477` — forced-bet
  posting, dead ante posting, standing resets, and showdown standings.
- `src/casino/state.rs:164` — the player standings.
- `src/casino/state.rs:210` — who may act after whom.
- `src/casino/state.rs:393`, `:465` — which standing may follow which, alone and
  against another seat.
- `src/casino/action.rs:41` — the player-decision vocabulary.
- `src/casino/action.rs:90` — the event vocabulary.
- `src/casino/action.rs:160`, `:214`, `:242` — narration, and seat and amount
  extraction from an entry.
- `src/casino/session.rs:326` — hand start: shuffle, forced bets, then
  family-dispatched dealing with the stud bring-in.
- `src/casino/session.rs:443`, `:540` — the driving loop and its single-step form
  distinguishing a street advance from a player to act.
- `src/casino/session.rs:645` — street advance dispatched by family.
- `src/casino/session.rs:710` — the per-identity view: own hole cards only, none
  for a viewer with no identity, and never the deck.
- `src/cards.rs:466`, `:476` — the shuffle and its seeded form.
- `src/games/street.rs` — the street tables consumed here (specified by DECON-05).
- `.okf/stud-rules.md`, `.okf/razz-rules.md` — the source's prose statement of the
  bring-in and visible-hand ordering rules.
