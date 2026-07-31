# DECON-08: Hand History

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

A hand of poker happens once. Everything downstream — statistics, training,
review, dispute resolution — depends on there being a record complete enough
that the hand can be run again and come out the same.

That is the whole of this epic, and it is a single promise: **a recorded hand,
replayed, reproduces the recorded final stacks.** Not approximately, not for
the common case. If the record cannot do that, it is missing information, and
the missing information is a defect in the schema rather than in the replay.

The promise has teeth because it is checkable. Replay is not a rendering of
the record; it is a fresh hand driven through a real table engine by the
recorded actions, ending in a real settlement. When the record is lossless the
two settlements agree exactly. When something was dropped — a street's action
list, a betting structure, a bring-in — the engine rejects the sequence or
lands on different stacks, and the gap is visible immediately.

Around that promise sit the practical requirements: the record must survive
serialization and parsing unchanged; older records missing fields added later
must still parse and must not sprout those fields when written back; and a
collection of records must answer the questions analysts actually ask.

## Status

| Component | Status |
|---|---|
| Hand-record structure | Planned |
| Recorded pre-shuffle deck and consumption order | Planned |
| Per-street breakdown derived from the event log | Planned |
| Deterministic replay to recorded final stacks | Planned |
| Round-trip stability and structural equality | Planned |
| Backward compatibility with records lacking optional fields | Planned |
| Hand collections and their queries | Planned |
| Session accounting invariants | Planned |

## Goals

- Define a **lossless hand record**: everything needed to re-run the hand, and
  everything needed to interpret it afterwards.
- Guarantee **deterministic replay** — the recorded actions, driven through a
  live engine, settle to the recorded final stacks.
- Derive the **per-street breakdown** from the raw event log alone, so the
  structured record and the narrative log can never disagree.
- Make **round-tripping** total: serialize, parse, serialize is stable, and a
  reparsed record equals the original.
- Keep old records readable and **write them back unchanged**, without
  injecting fields they never had.
- Answer **collection-level questions** — by player, by table shape, by
  showdown — and hold a recorded session to its **accounting invariants**.

## Scope

### The record

A hand record carries the following **information**. Field names and
serialization syntax are the implementer's choice — see **SD-10**.

1. A **format version**: an integer identifying the schema generation, stamped
   on every record written, defaulted when absent so older records parse. It
   is carried for consumers to inspect; nothing in this slice rejects a record
   on version grounds.
2. A **producing-library version**, optional on an individual record. When a
   record is placed in a collection, the collection carries the single
   authoritative version and the record's own copy is cleared, because
   repeating it per record is noise.
3. **Hand metadata**: an identifier unique within its source, the variant
   played, and optional provenance — a timestamp, a source name, a free-text
   description. Provenance is what lets a record from a televised hand, a
   simulator run, and a live session coexist in one collection.
4. **Table information**: an optional table name, the number of seats, the
   button's seat, the **stakes** (small blind, big blind, and optional ante,
   straddle, and bring-in), and the **betting structure**. The betting
   structure defaults to no-limit when absent, and is always written.
5. **Player entries**, one per seat that took part: the seat, a display name,
   the **starting stack** as of before forced bets, an optional stable
   **identity** for the person or agent occupying the seat, optional hole
   cards (absent when never revealed), an optional per-card visibility list
   for stud-family variants, an optional record of which forced bet the seat
   posted, and an optional cumulative withdrawal figure for profit-and-loss
   accounting across a session.
6. The **board**, as dealt.
7. A **per-street breakdown**: for each street that occurred, the community
   cards it added, an ordered list of actions, and the pot size at the street's
   end. Streets that did not occur are absent, not empty. Each action carries
   the acting seat, the optional identity of the actor, the action taken, an
   amount where the action has one, whether it put the actor all-in, and
   optional analysis-only provenance about how an automated actor produced it.
8. **Per-seat results**: the seat, the best five-card hand it showed and that
   hand's rank, the outcome, the **net chip change** for the hand, the total
   amount won from the pot, and whether the hand was mucked. The best hand and
   its rank are recorded only when the board was complete enough to determine
   them.
9. Optional **analysis context**: ranges for the hero and the villain in the
   notation of DECON-04, per-street equity figures, and free-text notes.
10. The **pre-shuffle deck**: the full 52-card deck as it stood before a single
    card was dealt.

### The pre-shuffle deck and its consumption order

11. The deck is captured *after* shuffling and *before* any card is drawn.
12. Cards are consumed in exactly this order: **hole cards dealt one at a time,
    clockwise from the seat immediately left of the button, round by round,
    skipping seats not in the hand; then one burn and three flop cards; then
    one burn and the turn; then one burn and the river.** Three burns total,
    one immediately before each community-card event.
13. Because the deck and the order are both recorded, the hand's entire card
    distribution is reconstructible from the record without any source of
    randomness. This is what makes replay deterministic rather than merely
    repeatable.

### Replay

14. Replay builds a fresh table from the record's stakes, betting structure,
    variant, and button, seats every recorded player at its recorded seat with
    its recorded starting stack, and posts forced bets through the engine.
15. Seat numbers are **physical seat indices**. A record's seat numbering must
    survive a table where some seats are empty and where the button may sit on
    a seat no player occupies, so the seating structure is sized to cover both
    the highest occupied seat and the button seat.
16. Forced bets are **re-posted by the engine**, not replayed from the recorded
    posting actions. Recorded postings are skipped during replay; they exist so
    the record is readable and auditable, not so it can drive the engine.
17. Hole cards are injected from the record. For stud-family variants, per-card
    visibility is restored before the bring-in is taken, because both the
    bring-in seat and the action order depend on which cards are face up.
18. Actions are applied in recorded order, street by street. Between streets
    the engine's street-advance is invoked and the street's community cards are
    taken from the record.
19. Replay ends by completing the hand through the ordinary settlement path of
    DECON-07 and collecting each recorded seat's final stack.
20. **Consistency.** A replay is consistent when, for every result entry that
    records a net change, the seat's recorded starting stack plus its recorded
    net change equals its replayed final stack. A record with no results is
    vacuously consistent.
21. Replay fails, loudly and with a named error, when the recorded sequence is
    not playable: an action out of turn, an illegal action, an unparseable
    card, or a street whose betting the recorded actions do not resolve. A
    silent divergence is not permitted.

### Deriving the per-street breakdown

22. The per-street breakdown is derived from the raw event log in a **single
    forward pass**. Actions land in whichever street is current when they are
    seen; nothing is reordered.
23. The three community-deal events are the street boundaries and supply each
    post-flop street's cards.
24. The last pot-size event seen within a street becomes that street's pot.
25. Event-to-action mapping is exhaustive and total: every forced-bet event
    becomes a posting; check, bet, call, raise, and fold map to themselves; an
    all-in maps to an all-in action flagged as such; every other event —
    dealing, seating, pot bookkeeping, narration — is dropped.
26. The preflop street is emitted when it has at least one action or a recorded
    pot. Each post-flop street is emitted **if and only if** its deal event was
    seen — a street with cards but no actions is emitted with an empty action
    list, which is exactly what an all-in run-out looks like.
27. No breakdown is produced from an empty log.
28. Each derived action carries the acting seat's identity, resolved from the
    seating events or from an explicitly supplied seat-to-identity mapping. A
    per-hand slice of a session log contains no seating events, so the mapping
    must be suppliable from outside; otherwise every identity comes out absent.

### Round-trip and compatibility

29. **Round-trip.** Serializing a record, parsing it, and serializing again
    yields a stable result, and the reparsed record is structurally equal to
    the original. Equality is over the whole tree, including nested streets,
    results, and analysis.
30. **Optional-field omission.** Every optional field that is absent must be
    absent from the serialized form. Writing a record parsed from an older
    generation must not introduce a key that record never had.
31. **Backward compatibility.** A record lacking player identity — on player
    entries, on actions, or both — parses without error with identity absent,
    re-serializes without emitting an identity key anywhere, and reparses to a
    value equal to the first parse. The same holds for every other
    later-added optional field.
32. **Defaulted-not-optional fields.** Format version and betting structure are
    defaulted when absent and always written. This is deliberate and distinct
    from omission: a consumer can always read them.
33. **Extensible action vocabulary.** The set of action kinds must be
    extensible without breaking consumers that were compiled or written against
    an earlier set.

### Collections and session accounting

34. A **collection** holds records in insertion order and carries the
    authoritative producing-library version and format version.
35. **Hands by player.** Yields, in order, every record in which some player
    entry carries the given identity. Records written before identity existed
    are silently skipped — they cannot match.
36. **Hands by position.** Yields records whose *table shape* would seat a
    player at the given position. This is a shape filter, not a player filter:
    occupied seats are sorted, the button and each player are mapped to their
    index within that sorted list, and position is derived from that logical
    index. Records without a button, or without players, match nothing. The
    query's purpose is to exclude hands too short-handed to have the position
    at all.
37. **Showdowns only.** Yields records whose results exist and contain two or
    more entries whose outcome is not a fold. A hand won by folding out the
    field is not a showdown.
38. **Replay all.** Yields one replay outcome per record, in order, each
    independently succeeding or failing.
39. **Net conservation.** Within any recorded hand, the per-seat net chip
    changes sum to zero.
40. **Pot completeness.** Within any recorded hand, the amounts won sum to the
    hand's final pot — the pot recorded by the latest street that recorded one.
41. **Session replay.** Every record in a recorded session replays without
    error and lands on stacks consistent with its recorded net changes.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| **Format version** | Stamped on write, defaulted on read, always emitted | `roundtrip.json` |
| **Hand metadata** | Identifier, variant, optional timestamp/source/description | `roundtrip.json` |
| **Table information** | Seats, button, stakes, betting structure (defaulted) | `roundtrip.json` |
| **Player entry** | Seat, name, starting stack, optional identity, hole cards, visibility | `roundtrip.json` |
| **Street breakdown** | Per-street cards, ordered actions, end-of-street pot | `roundtrip.json`, `replay.json` |
| **Action** | Seat, optional identity, kind, optional amount, all-in flag | `roundtrip.json`, `replay.json` |
| **Per-seat result** | Seat, best hand and rank, outcome, net change, amount won | `replay.json` |
| **Analysis context** | Ranges in DECON-04 notation, per-street equities, notes | `roundtrip.json` |
| **Pre-shuffle deck** | 52 cards captured before dealing, consumed in a fixed order | `replay.json` |
| **Replay** | Recorded actions re-driven to the recorded final stacks | `replay.json` |
| **Round-trip** | Serialize → parse → serialize stable; reparsed equals original | `roundtrip.json` |
| **Optional-field omission** | Absent stays absent through a write | `roundtrip.json` |
| **Collection queries** | By player, by position, showdowns only | `roundtrip.json` |
| **Session accounting** | Nets sum to zero; amounts won equal the final pot | `replay.json` |

## Design

### Why the record is built at hand end from three sources

The record is assembled after the hand completes, from three inputs that no
single moment of the hand possesses at once:

- a **snapshot taken before forced bets** — seats, names, starting stacks,
  identities — because starting stacks are unrecoverable once blinds are in;
- the **hole cards captured immediately after the deal**, because a folded
  seat's cards would otherwise be gone;
- the **per-hand slice of the event log** and the **settlement result**, which
  only exist once the hand is over.

A rebuild is free to assemble the record differently, but it must capture
starting stacks before forced bets and hole cards after the deal. Those two
capture points are domain constraints, not implementation details: the
information ceases to exist otherwise.

### Deriving results

Per-seat results are computed, not copied:

- **Amount won** is the sum over settlement layers awarded to that seat;
  recorded only when positive.
- **Net change** is the seat's ending stack minus its recorded starting stack.
  This is an observed stack delta, which matters: it stays correct even when
  some other part of the record — recorded stakes, say — has drifted from what
  the hand actually played.
- **Outcome** is a fold when the seat folded, otherwise a win when it won
  chips, otherwise a loss.
- **Best hand and rank** are computed only when the board reached five cards.
  Otherwise both are absent.

### Replay is a re-run, not a re-render

```
replay(record):
    table = new table from record's variant, stakes,
                            betting structure, and button
    seat every recorded player at its recorded seat index,
        with its recorded starting stack
    post forced bets through the engine

    inject each player's recorded hole cards
    for stud-family variants:
        restore per-card visibility, then take the bring-in

    for street in [preflop, flop, turn, river]:
        if street is absent: break
        if street is post-flop:
            set the board to the recorded cards for this street
        for action in street's actions in order:
            if action is a forced posting: skip
            apply action to the table          # may reject → error
        if the hand is over: break
        advance the street through the engine  # may reject → error

    settle the hand through the ordinary settlement path
    return per-seat final stacks and a consistency verdict
```

The engine does the work. The record only steers. That is why replay is a real
test of the record's completeness: anything the record omits, the engine
notices.

Two replay subtleties are domain-load-bearing and easy to get wrong.

**The button may point at an empty seat.** When a player busts, the button can
advance onto their now-vacant position. Seating must be sized to cover the
button's seat even when no player occupies it, or forced-bet posting computes
the wrong action order for the whole street.

**A frozen all-in run-out has no actions.** Once every remaining player is
all-in, the remaining streets deal out with no action to record. A replay that
resets players to "yet to act" at the start of every street will then stall,
because nothing in the record resolves that state. Reset only when the street
actually has actions to apply; an empty action list means the frozen state must
be preserved.

### Analysis provenance is inert

Records may carry, per action, how an automated actor produced it — the raw
response, whether it had to be coerced into legality, the intended action and
amount, token counts, the model and prompt. This is analysis material only.
Replay ignores it entirely. A rebuild must round-trip it faithfully and must
never let it change what replay does.

### Consistency, exactly

Rule 20 is stated as exact equality: starting stack plus recorded net equals
replayed final stack. The original permits a one-chip tolerance, a hedge
against fractional-looking splits. With the division rule of DECON-07 — whole
chips, remainder distributed deterministically — that tolerance is never
needed, and a rebuild should not grant itself one. A one-chip drift is a real
defect wearing a rounding costume.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | — | The record's information content and the consumption order of the recorded deck are fixed by the library. A consumer may not invent a street, a variant, or a dealing order. |
| **Administrative** | Choose where records are kept, name a session, group records into collections, read the producing-library and format versions | Edit a completed record's results or actions and still call it a record of that hand | An operator decides what is kept and where; not what happened. |
| **User/client** | Read records it is entitled to, including its own hole cards | Rely on a record to reveal an opponent's hole cards that were never shown | A record reveals exactly what the hand revealed, plus what its keeper chose to record. |
| **Observer/operator** | Reconstruct any completed hand in full from the record alone, re-derive the per-street breakdown from the raw log, and re-run the hand without disturbing the original | Watch a hand as it is played through this slice; there is no live stream here | Anything that already happened can be reconstructed and re-examined without altering it. |
| **Agent** | Have its decision provenance recorded alongside its actions and read it back later | Have that provenance change how the hand replays | An agent's reasoning is evidence about the hand, never part of it. |
| **Trainer/researcher** | Replay any recorded hand or whole session and obtain identical final stacks, filter a corpus by player, table shape, or showdown, and audit a session's accounting | — | A researcher can rerun any recorded experiment and get the same answer. |
| **Spectator** | — | — | N/A for this slice — redaction of a live view belongs to the table engine; a stored record is not a spectator surface. |
| **Trustless/cryptographic peer** | — | — | N/A for this slice — a record is evidence of what a trusted dealer did, and carries no commitment or proof. |

## Work Items

### Phase 0 — Vectors and harness

- [ ] **0a.** Build loaders for `vectors/hand-history/roundtrip.json` and
      `vectors/hand-history/replay.json`, failing loudly on unparseable cases.
- [ ] **0b.** Stand up a serialize/parse harness and a replay harness, both
      initially failing every vector case.

### Phase 1 — The record and its round-trip

- [ ] **1a.** Write failing tests asserting that each record in
      `roundtrip.json` parses with every documented piece of information
      present and correctly typed.
- [ ] **1b.** Define the record's information content per Scope rules 1–10 and
      make those tests pass.
- [ ] **1c.** Implement serialization and parsing; assert
      serialize → parse → serialize stability and structural equality of the
      reparse. Proven by `roundtrip.json`.
- [ ] **1d.** Implement optional-field omission and assert, per case, that an
      absent field produces no key. Proven by `roundtrip.json`.

### Phase 2 — Backward compatibility

- [ ] **2a.** Write the failing legacy test: a record with no player identity
      anywhere parses, every entry and every action reports identity absent.
- [ ] **2b.** Assert the re-serialized form contains no identity key anywhere,
      and that reparsing it yields a value equal to the first parse.
- [ ] **2c.** Extend the same three-step check to every later-added optional
      field the rebuild carries.
- [ ] **2d.** Assert that format version and betting structure default when
      absent and are always written.

### Phase 3 — Deriving the per-street breakdown

- [ ] **3a.** Write failing tests over synthetic event logs: empty log,
      preflop-only, full hand to the river, all-in run-out with no post-flop
      actions.
- [ ] **3b.** Implement the single forward pass with deal events as boundaries
      and the last pot-size event per street winning.
- [ ] **3c.** Implement the exhaustive event-to-action mapping, dropping every
      non-action event.
- [ ] **3d.** Implement the emission rules: preflop on actions-or-pot,
      post-flop streets on the deal event alone, no breakdown from an empty
      log.
- [ ] **3e.** Implement seat-to-identity resolution from seating events *and*
      from an externally supplied mapping; prove a per-hand log slice still
      stamps identities when the mapping is supplied.

### Phase 4 — The recorded deck

- [ ] **4a.** Capture the full deck after the shuffle and before any draw, and
      assert 52 distinct cards.
- [ ] **4b.** Document and test the consumption order: hole cards clockwise
      from the button, then burn+flop, burn+turn, burn+river.
- [ ] **4c.** Assert the recorded deck round-trips exactly, and that its
      absence produces no key. Proven by `roundtrip.json`.
- [ ] **4d.** Prove that reconstructing the hand's card distribution from the
      recorded deck and the consumption order reproduces the recorded hole
      cards and board. Proven by `replay.json`.

### Phase 5 — Replay

- [ ] **5a.** Write the failing replay test for each case in `replay.json`,
      asserting the recorded final stacks.
- [ ] **5b.** Implement table construction from the record, including the
      seating that covers a button on an unoccupied seat.
- [ ] **5c.** Implement forced-bet re-posting and the skipping of recorded
      postings.
- [ ] **5d.** Implement hole-card injection, and stud-family visibility
      restoration before the bring-in.
- [ ] **5e.** Implement street-by-street action application, board restoration,
      and the frozen-run-out rule for streets with no actions.
- [ ] **5f.** Implement the consistency verdict as exact equality of starting
      stack plus net against replayed final stack.
- [ ] **5g.** Add negative tests: an out-of-turn action, an unresolved street,
      and an unparseable card each fail with a named error rather than a
      silent divergence.

### Phase 6 — Collections and session accounting

- [ ] **6a.** Implement the collection: insertion order preserved, one
      authoritative producing-library version, per-record copies cleared on
      insertion.
- [ ] **6b.** Implement hands-by-player, including the skipping of records
      without identity. Proven by `roundtrip.json`.
- [ ] **6c.** Implement hands-by-position as a table-shape filter over sorted
      occupied seats, returning nothing when the button is absent.
- [ ] **6d.** Implement showdowns-only as two-or-more non-fold outcomes.
- [ ] **6e.** Implement replay-all over a collection, one independent outcome
      per record.
- [ ] **6f.** Assert the session invariants: per-hand nets sum to zero, amounts
      won sum to the final recorded pot, and every record replays consistently.
      Proven by `replay.json`.

## Test Plan

**Round-trip stability.** *Given* each record in `roundtrip.json`, *when* it is
parsed, serialized, and parsed again, *then* the second parse is structurally
equal to the first and the two serialized forms agree. (`roundtrip.json`)

**Full-record fidelity.** *Given* a record carrying metadata, table
information, players with hole cards, a board, all four streets, results with
hand ranks, and analysis context, *when* round-tripped, *then* every field
survives with its exact value. (`roundtrip.json`)

**Omission of absent optionals.** *Given* a record with no analysis context, no
recorded deck, and no per-action provenance, *when* serialized, *then* the
output contains no key for any of them. (`roundtrip.json`)

**Legacy record without identity.** *Given* a record written before player
identity existed, *when* parsed, *then* every player entry and every action
across every street reports identity absent; *and when* re-serialized, *then*
the output contains no identity key anywhere; *and when* reparsed, *then* it
equals the first parse. (`roundtrip.json`)

**Defaulted fields.** *Given* a record with neither format version nor betting
structure, *when* parsed, *then* both take their defaults; *and when*
re-serialized, *then* both are present. (`roundtrip.json`)

**Street derivation — empty log.** *Given* an empty event log, *when* the
breakdown is derived, *then* no breakdown is produced. (`roundtrip.json`)

**Street derivation — boundaries and pots.** *Given* a log with forced bets,
preflop action, two pot-size events preflop, and a flop deal followed by
checks, *when* derived, *then* preflop carries its actions and the *second*
pot-size value, and the flop carries its cards and its actions.
(`roundtrip.json`)

**Street derivation — all-in run-out.** *Given* a log whose post-flop streets
carry deal events but no actions, *when* derived, *then* each such street is
emitted with an empty action list rather than omitted. (`roundtrip.json`)

**Street derivation — identity stamping.** *Given* a per-hand log slice with no
seating events and an externally supplied seat-to-identity mapping, *when*
derived, *then* every action carries its actor's identity; *and given* no
mapping, *then* every identity is absent. (`roundtrip.json`)

**Deck consumption order.** *Given* a record's pre-shuffle deck and its
recorded button and player count, *when* the documented consumption order is
applied, *then* the reconstructed hole cards and board equal the recorded ones.
(`replay.json`)

**Replay to recorded stacks.** *Given* each recorded hand in `replay.json`,
*when* replayed, *then* the replay succeeds and every seat's starting stack
plus recorded net change equals its replayed final stack. (`replay.json`)

**Replay — button on an empty seat.** *Given* a record whose button sits past
the highest occupied seat, *when* replayed, *then* forced bets post in the
correct order and the hand replays consistently. (`replay.json`)

**Replay — frozen run-out.** *Given* a record whose turn and river carry cards
but no actions, *when* replayed, *then* the streets advance without demanding
an action and the hand settles to the recorded stacks. (`replay.json`)

**Replay — provenance is inert.** *Given* two otherwise identical records, one
carrying per-action analysis provenance and one without, *when* both are
replayed, *then* the final stacks are identical. (`replay.json`)

**Replay — rejection.** *Given* a record whose action sequence is illegal for
the variant and structure, *when* replayed, *then* replay fails with a named
error rather than producing stacks. (`replay.json`)

**Collection — by player.** *Given* a collection mixing records with and
without identity, *when* queried for an identity, *then* only records with a
matching player entry are yielded, in insertion order. (`roundtrip.json`)

**Collection — by position.** *Given* a six-handed record and a heads-up
record, *when* queried for the cutoff, *then* only the six-handed record is
yielded; *and given* a record with no button, *then* nothing is yielded.
(`roundtrip.json`)

**Collection — showdowns only.** *Given* records ending in a win plus a fold,
and in a win plus a loss, *when* queried, *then* only the second is yielded.
(`roundtrip.json`)

**Session — nets sum to zero.** *Given* every recorded hand in a session,
*when* the per-seat net changes are summed, *then* the sum is zero.
(`replay.json`)

**Session — amounts won equal the final pot.** *Given* every recorded hand
carrying a street pot, *when* the amounts won are summed, *then* the sum equals
the pot of the latest street that recorded one. (`replay.json`)

## Not specified (implementer's choice)

- **Serialization syntax.** Text or binary, any encoding. See **SD-10**.
- **Field names and nesting.** The information is normative; the spelling is
  not. A rebuild must be able to state its own schema.
- **Identity representation.** Any stable, comparable identifier suffices; the
  original uses a universally-unique identifier, which is one choice among
  many.
- **Timestamp format.** The record's timestamp is free-text provenance; nothing
  in this slice parses it. A rebuild should pick one format and state it. The
  original is internally inconsistent here — its own documentation names one
  format while its writer emits another — which is precisely why nothing should
  depend on it.
- **Seat numbering base.** Zero-based or one-based, provided the record's
  numbering matches the engine's physical seat indices and round-trips.
  (The original's documentation and behavior disagree; the behavior is
  zero-based physical indices.)
- **Amount representation.** Whole chips are the domain reality. The original
  stores recorded amounts as floating-point and compares session invariants
  within a hundredth of a chip; a rebuild storing whole chips and comparing
  exactly is strictly better and equally conformant.
- **How replay obtains community cards.** This spec requires that the record
  make the hand reconstructible from the recorded deck plus the consumption
  order, *and* that replay reach the recorded final stacks. Whether replay
  deals from the recorded deck or reads each street's recorded cards is free.
  (The original records the deck, documents the order, and then takes board
  cards from the recorded streets — the deck-driven path is stated but not
  implemented. Either satisfies this spec; a rebuild choosing the deck-driven
  path should verify both agree.)
- **Version policy.** Whether a rebuild rejects a record whose format version
  it does not recognize. The original never checks. Rejecting is permitted and
  arguably better, provided the default-on-absent behavior of rule 32 holds.
- **Storage.** Where records live — files, a database, memory — is out of scope
  pack-wide.
- **Query implementation.** Eager collections, lazy sequences, or streaming
  iterators, provided order is preserved and the semantics of rules 35–38 hold.

## Spec decisions

> **Spec decision SD-10:** Are the hand record's field names normative, or only
> the information they carry? **Options:** pin the field names and serialized
> shape / pin only the information and its round-trip and replay properties.
> **Chosen:** pin only the information — field names bind a rebuild to one
> serialization format for no domain reason.

What is normative: the *information* enumerated in Scope rules 1–10; the
consumption order of rule 12; the replay and consistency properties of rules
14–21; the derivation rules 22–28; and the round-trip and compatibility
properties of rules 29–33.

What is not: the names of fields, their nesting, the ordering of keys, and the
serialization syntax. The original's names are one legible choice made in one
text format; a rebuild in another language or another format has no reason to
inherit them, and pinning them would force a wire format on a slice whose
domain content is "the record is complete and stable".

Two obligations survive the freedom, and they are the reason this is a decision
rather than a shrug:

1. **A rebuild must state its own schema.** "Whatever my serializer emits" is
   not a schema. The point of a hand record is that a second party can read it.
2. **Optional-field omission is normative regardless of names.** Whatever a
   rebuild calls its optional fields, an absent one must be absent from the
   output, and a record parsed from an earlier generation must write back
   without acquiring fields it never had. This is the one place where the
   serialized *shape* is load-bearing, because backward compatibility is a
   property of the shape and not of the information.

## Verification

Any implementation must reproduce every file under `vectors/hand-history/`:

1. Every record in `vectors/hand-history/roundtrip.json` parses, serializes,
   and reparses to a structurally equal value, with the two serialized forms
   agreeing.
2. Every absent optional field in those cases is absent from the serialized
   output, and every record lacking player identity writes back with no
   identity key anywhere and reparses equal to its first parse.
3. Format version and betting structure default when absent and are always
   present in the output.
4. Every per-street breakdown derived from the event logs in the vector files
   matches the recorded breakdown — street assignment, action order, action
   kinds and amounts, street pots, and the presence or absence of each street.
5. Every recorded hand in `vectors/hand-history/replay.json` replays without
   error, and for every seat, starting stack plus recorded net change equals
   the replayed final stack exactly.
6. Reconstructing card distribution from each record's pre-shuffle deck using
   the documented consumption order reproduces that record's hole cards and
   board.
7. For every recorded hand, the per-seat net changes sum to zero and the
   amounts won sum to the final recorded pot.
8. Collection queries — by player, by position, showdowns only — return exactly
   the recorded result sets, in insertion order.
9. Records carrying per-action analysis provenance replay to the same final
   stacks as records without it.
10. A record whose action sequence is unplayable fails replay with a named
    error; no case produces silently divergent stacks.
11. The rebuild publishes its own record schema, naming every field, its
    optionality, and its omission behavior.

## Dependencies

**Builds on:** DECON-06 (Table Engine) for the event log, seating, dealing
order, forced bets, and street advance; DECON-07 (Pot Accounting) for
settlement, without whose determinism replay cannot land on identical stacks.
Analysis context uses the notation of DECON-04 (Range Notation); recorded hand
ranks use DECON-02 (High Hand Ranking) and DECON-03 (Lowball Ranking).

**Blocks:** DECON-12 (Player Statistics) — behavioural statistics are derived
from recorded hands, and their correctness depends on the breakdown derivation
and the collection queries specified here.

## Provenance (non-normative)

- `src/hand_history.rs:85` — the format-version constant; stamped on write,
  defaulted on read, never validated.
- `src/hand_history.rs:128` — the hand record and its fields, including the
  optional producing-library version cleared on insertion into a collection.
- `src/hand_history.rs:167` — the recorded pre-shuffle deck and the documented
  consumption order: hole cards clockwise from the button, then burn+flop,
  burn+turn, burn+river.
- `src/hand_history.rs:242` — record assembly from a pre-forced-bets snapshot,
  post-deal hole cards, the settlement result, the per-hand event-log slice,
  and ending stacks.
- `src/hand_history.rs:311` — the identity-carrying assembly path and the
  derivation of outcome, net change, amount won, and best hand.
- `src/hand_history.rs:547` — replay: table construction sized for a button on
  an unoccupied seat, forced-bet re-posting, hole-card injection, stud
  visibility restoration, per-street action application, and the frozen
  run-out rule.
- `src/hand_history.rs:915` — the collection, its authoritative version, and
  its queries.
- `src/hand_history.rs:1069` — hands by player, skipping identity-less records.
- `src/hand_history.rs:1096` — hands by position as a table-shape filter over
  sorted occupied seats.
- `src/hand_history.rs:1115` — showdowns as two or more non-fold outcomes.
- `src/hand_history.rs:1452` — player entries: starting stack, optional
  identity, hole cards, per-card visibility, cumulative withdrawal.
- `src/hand_history.rs:1654` — the per-street breakdown; every street optional.
- `src/hand_history.rs:1715` — derivation of the breakdown from the event log
  in a single forward pass.
- `src/hand_history.rs:1763` — the externally supplied seat-to-identity mapping
  needed for per-hand log slices.
- `src/hand_history.rs:1849` — the exhaustive event-to-action mapping.
- `src/hand_history.rs:2229` — per-action analysis provenance, explicitly
  ignored by replay.
- `src/hand_history.rs:2286` — actions and the extensible action vocabulary.
- `src/hand_history.rs:2379` — per-seat results: best hand, rank, outcome, net
  change, amount won, mucked.
- `src/hand_history.rs:2489` — analysis context carrying ranges and per-street
  equities.
- `src/hand_history.rs:2611` — replay across a whole collection.
- `src/hand_history.rs:2688` — the consistency verdict; the original's one-chip
  tolerance.
- `src/casino/session.rs:331` — the deck captured after the shuffle and before
  any draw.
- `src/casino/table.rs:1216` — hole cards dealt one at a time clockwise from
  the seat left of the button.
- `src/casino/table.rs:1315` — the burn before each community-card deal.
- `tests/replay_consistency.rs` — self-play round-trips for no-limit,
  fixed-limit, and pot-limit; betting structure and variant survive the
  round-trip; stud and razz round-trip their per-card visibility but defer
  replay, because one-shot card injection cannot reproduce per-street visible
  sets.
- `tests/hand_history_legacy_yaml.rs` — the three-step legacy contract: parse
  without error, re-serialize without injecting an identity key, reparse equal.
- `tests/pkarena0_session.rs` — the session invariants: nets sum to zero,
  amounts won equal the final street pot, every hand replays consistently.
