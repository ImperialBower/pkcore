# DECON-11: Agent Model

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

A poker table needs occupants. This epic specifies the **automated seat
occupant**: a thing that, given a description of how it likes to play and a
picture of the table taken from one seat, produces an **action**.

This is the pack's most confident slice. The manifest rates the **Agent**
perspective **Full**, and it earns that rating on four counts: the decision
contract is published and third-party implementable; the agent is handed a
**seat view** rather than the live table; a **seeded** decision path exists so
a whole simulation reproduces exactly; and behaviour is **data**, not code —
nine named archetypes and per-variant profile sets, all loadable from files,
so a new personality requires no new code.

One honesty note, stated up front because a rebuild should improve on it: in
the original the seat view is **conventional, not sealed**. The view is built
by copying the fields a seat is entitled to see, and it carries no deck at
all — but nothing structurally prevents a caller from assembling a view by
hand and populating it with information the seat could not know. The
invariant below is the requirement; the original enforces it by construction
discipline rather than by an unforgeable boundary. A rebuild that makes the
view unforgeable satisfies this spec strictly better.

A second note: two of the optional decision capabilities (draw/outs
augmentation, preflop chart source) are declared in the original's profile
data but never consulted by the decision procedure. They are documented here
as deferred, with the reason: they need information the decider is never
handed.

## Status

| Component | Status |
|---|---|
| Decision contract and per-hand lifecycle notification | Planned |
| Seat view (bounded information) | Planned |
| Seeded determinism across shuffle, lifecycle, and decision | Planned |
| Profile model: ranges, betting parameters, decision capabilities | Planned |
| Nine named archetypes | Planned |
| Profile loading from files | Planned |
| Table-size classification and position sets | Planned |
| Positional range charts (playbook) | Planned |
| Positional betting strategy | Planned |
| Per-variant profile sets | Planned |
| Optional decision capabilities as layered toggles | Planned |
| Multi-hand simulation harness | Planned |

## Goals

- A **decision contract** narrow enough that a third party can implement a
  seat occupant without access to the table engine's internals.
- **Bounded information** as a domain invariant: an agent decides only what to
  do with the cards it holds.
- **Reproducibility**: same seed plus same state yields the same action, so
  agent-driven research is repeatable.
- **Behaviour as data**: a new personality is a new file, never new code.
- **Positional literacy**: what an agent opens, three-bets, and bets is a
  function of table size, position, and the action it faces.

## Scope

A rebuild must obey these rules.

1. **The decision contract.** A seat occupant answers one question: given a
   **profile** and a **seat view**, which **action** do you take? The action
   vocabulary is the engine's (DECON-06): fold, check, call, bet an amount,
   raise to a total, go all in. The occupant returns an action; it never
   applies one.
2. **Per-hand lifecycle.** Before any action in a hand, every seated occupant
   is notified that a new hand is beginning. An occupant may use that
   notification to re-randomise per-hand state. Doing nothing is a valid
   response.
3. **Seeded twins.** Both the decision call and the lifecycle notification
   have a seeded form that draws all randomness from a supplied generator.
   When a run is seeded, every occupant is driven through the seeded form and
   the deck is shuffled from the same generator.
4. **The seat view carries exactly**: the seat's own index and its position;
   the current street; the community cards; the seat's own hole cards; the
   total pot; the amount required to call; the current highest bet on the
   street; the minimum legal raise increment; the seat's remaining stack; the
   big blind; the table's betting structure and the current bet tier; whether
   this seat has already checked on this street; the dealer button's logical
   index; the number of occupied seats; and, for every occupied seat, its
   identity, index, display name, stack, current-street commitment, and
   whether it is still contesting the hand. Optionally it carries a read-only
   handle to opponent statistics (DECON-12).
5. **The seat view carries no undealt card and no opponent hole card.** Ever.
   A view built for a seat that has not been dealt in shows no hole cards.
6. **Determinism.** For a fixed profile, a fixed seat view, and a fixed
   generator state, the produced action is fixed. No decision may consult a
   clock, a process-global generator, an address, or iteration order over an
   unordered collection.
7. **Profiles are data.** A profile is loadable from a file and writable back
   to one, and a profile that round-trips through a file produces identical
   decisions.
8. **Nine archetypes** exist by name with the parameters given in Design, plus
   per-variant sets for fixed-limit hold'em, pot-limit Omaha, seven-card stud
   hi, and razz.
9. **Fallback is total.** Whenever a positional lookup misses — unclassified
   table size, position with no chart, absent playbook — the agent falls back
   to the profile's flat range and flat betting parameters and still returns a
   legal action. A missing chart is never an error.
10. **Sizing is legal by construction.** Every bet and raise the agent
    produces is clamped to its own stack, floored at the structure's minimum,
    and — under a fixed-limit structure — set to the street's tier increment
    rather than a pot fraction.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| **Decision contract** | Profile + seat view → action; never mutates either | `vectors/agent-model/seeded-decisions.json` |
| **Lifecycle notification** | Fires once per hand before any action; seeded form uses the run's generator | `vectors/agent-model/seeded-decisions.json` |
| **Seat view** | Bounded projection of the table from one seat | `vectors/agent-model/seeded-decisions.json` |
| **Profile** | Named, file-loadable bundle of ranges, betting parameters, playbook, decision capabilities | `vectors/agent-model/profiles.json` |
| **Archetype** | Nine named parameterisations with fixed values | `vectors/agent-model/profiles.json` |
| **Playbook** | Table size → position → action → weighted range | `vectors/agent-model/profiles.json` |
| **Positional betting** | Table size → position → betting parameters | `vectors/agent-model/profiles.json` |
| **Table-size class** | Seat count → recognised class → ordered position set | `vectors/agent-model/profiles.json` |
| **Decision capabilities** | Layered toggles whose defaults reproduce the base behaviour | `vectors/agent-model/profiles.json` |
| **Seeded run** | Seed + starting table → identical hand-by-hand outcome | `vectors/agent-model/seeded-decisions.json` |

`profiles.json` contains, for every named archetype, the defining parameters:
range strings, betting percentages, preferred bet sizes, continuation-bet
frequency, playbook contents where present, and decision-capability defaults.
`seeded-decisions.json` contains, for each case, a seed and a fully specified
seat view, and the action produced. Conformance means: load the profile named
in the case, feed it the given view with a generator initialised from the
given seed, and produce the recorded action exactly — including the recorded
amount for a bet or raise.

## Design

### The decision procedure

The reference decision procedure is a total function. It is written here as
ordered rules; the first matching rule wins.

```
decide(profile, view, generator):
  if view.my_chips == 0:              return Check

  if profile.exploit is enabled and view carries opponent statistics:
      profile := adjust(profile, view)          # DECON-12; pure, no mutation

  strategy := profile.betting_for(view.seat_count, view.position)
              or profile.flat_betting            # fallback when unclassified

  aggression := strategy.aggression_for(view.street)
  roll       := generator.uniform()             # one draw, reused below

  # 1. Check-raise: we checked earlier this street and now face a bet.
  if view.to_call > 0 and view.checked_this_street:
      if roll < strategy.check_raise_frequency:
          target := sized_raise_to(view, strategy, generator)
          if target > view.current_bet:         return RaiseTo(target)

  strength := hand_strength(profile, view, generator)   # may be undefined

  if strength is defined:
      if view.to_call > 0:
          if view.to_call >= view.my_chips:
              return AllIn if strength > 0.5 else Fold
          pot_odds  := view.to_call / (view.pot + view.to_call)
          threshold := pot_odds * profile.pot_odds_discipline
          if strength > pot_odds * 2:
              if generator.uniform() < max(aggression, 0.5):
                  target := sized_raise_to(view, strategy, generator)
                  if target > view.current_bet: return RaiseTo(target)
              return Call
          if strength > threshold:              return Call
          if generator.uniform() < strategy.bluff_frequency:
              target := sized_raise_to(view, strategy, generator)
              if target > view.current_bet:     return RaiseTo(target)
          return Fold
      else:
          if strength > strategy.value_threshold:
              return Bet(sized_bet(view, strategy, generator))
          if view.street is not preflop and generator.uniform() < strategy.bluff_frequency:
              return Bet(sized_bet(view, strategy, generator))
          return Check
  else:
      # No hole cards yet: aggression-only fallback.
      if view.to_call > 0:
          if view.to_call >= view.my_chips:
              return AllIn if roll < aggression * 0.6 else Fold
          if roll < aggression * 0.25:
              target := sized_raise_to(view, strategy, generator)
              if target > view.current_bet:     return RaiseTo(target)
          return Call if roll < aggression else Fold
      else:
          bet_gate := profile.cbet_frequency if view.street is flop else aggression
          if roll < bet_gate:                   return Bet(sized_bet(view, strategy, generator))
          if view.street is not preflop and generator.uniform() < strategy.bluff_frequency:
              return Bet(sized_bet(view, strategy, generator))
          return Check
```

Two properties of this shape are domain-load-bearing and must survive a
rebuild. First, **the number and order of generator draws is part of the
observable behaviour** under a fixed seed: a rebuild that reorders draws will
not reproduce `seeded-decisions.json`. Second, **strength is compared against
pot odds, not against an absolute bar**, when facing a bet — an agent that
ignores price is not a poker agent.

The default value threshold, used when a profile does not override it, is
`0.55`.

### Hand strength

Strength is a number in `[0, 1]`, or undefined when the seat holds no cards.

| Situation | Rule |
|---|---|
| No hole cards | Undefined — the aggression-only fallback runs |
| Preflop | Look up the hole cards' **frequency weight** in the profile's open range (DECON-04 weighted notation). Draw once; strength is `1.0` with that probability and `0.0` otherwise. A hand at weight `0.7` therefore plays as a premium 70% of the time — this is how a mixed strategy is realised |
| Stud family, 3rd/4th street, 3 or 4 known cards | Coarse bucket: trips on 3rd `0.90`; pair on 3rd `0.65`; quads on 4th `0.98`; trips on 4th `0.85`; two pair on 4th `0.75`; one pair on 4th `0.55`; otherwise `0.20 + 0.25 × (top_rank − 2)/12` |
| Postflop, equity capability off | **Rank proxy**: take the best five-card hand from hole cards plus board, and map its rank position on the 7,462-step ladder (DECON-02) to `1 − rank/7462`, so the nut hand is `1.0` |
| Postflop, equity capability on | Compute the seat's equity against the remaining active opponents as unknown hands (DECON-09), sampled or exhaustive per the capability's setting; fall back to the rank proxy when the equity engine cannot answer |

The rank proxy is a **hand-strength** measure, not an equity. Naming it
honestly matters: a rebuild must not silently substitute true equity where the
proxy is specified, because the vectors were produced with the proxy.

### Sizing

| Structure | Bet (no outstanding bet) | Raise (facing a bet) |
|---|---|---|
| Fixed-limit | The street's tier increment (small bet or big bet), capped at the stack | `current_bet + tier increment`, capped at the stack |
| No-limit / pot-limit | A pot fraction chosen uniformly from the profile's preferred sizes, floored at the big blind, capped at the stack | `current_bet + pot × fraction`, floored at `current_bet + minimum raise`, capped at the stack |

When the profile lists no preferred sizes, half pot is used.

### Table-size classification and positions

Only these seat counts are classified. Seven and eight seats are **not**
recognised, and at those sizes every agent falls back to flat strategy — an
observable behaviour, not an oversight to be quietly fixed.

| Class | Seats | Positions, in order |
|---|---|---|
| Heads-up | 2 | BB, BTN |
| Three-handed | 3 | BTN, SB, BB |
| Four-handed | 4 | UTG, BTN, SB, BB |
| Five-handed | 5 | UTG, CO, BTN, SB, BB |
| Six-max | 6 | LJ, HJ, CO, BTN, SB, BB |
| Nine-max | 9 | UTG, UTG+1, EP, LJ, HJ, CO, BTN, SB, BB |

Position is derived from the seat's **logical** index among occupied seats and
the button's logical index (DECON-05), so eliminations that leave gaps in
physical seat numbering never mis-assign a position.

### Profiles

A profile is a named bundle:

| Part | Contents |
|---|---|
| Identity | Name, human description, style label |
| Ranges | Open-raise range, three-bet range, call-a-three-bet range, continuation-bet frequency |
| Betting | Aggression, bluff frequency, check-raise frequency, preferred bet sizes as pot fractions, optional per-street aggression overrides, optional value threshold |
| Playbook | Optional: table size → position → action → weighted range, and table size → position → betting parameters |
| Structure marker | Optional: the betting structure this profile was authored for |
| Capabilities | The layered decision toggles below |

Percentages are whole numbers in `0..=100`, clamped on construction.

### The nine archetypes

Aggression, bluff, and check-raise are percentages; sizes are pot fractions;
continuation-bet frequency is a percentage.

| Archetype | Aggr | Bluff | Ck-raise | Sizes | Open range | C-bet |
|---|---|---|---|---|---|---|
| Tight-passive | 25 | 5 | 3 | 1/2 | `QQ+, AKs` | 30 |
| Loose-aggressive | 75 | 35 | 20 | 2/3, pot | `22+, AT+, 54s+` | 75 |
| Game-theory-optimal | 50 | 33 | 15 | 1/3, pot | `QQ+, JJ:0.95, TT:0.8, AKs, AQs, AJs:0.7, AKo, AQo:0.85, KQs:0.9` | 50 |
| Tight-aggressive | 70 | 20 | 15 | 2/3, pot | `JJ+, AQs+, KQs, AKo` | 65 |
| Loose-passive | 15 | 3 | 2 | 1/2 | `22+, AKs-A2s, KTs+, QTs+, J9s+, T8s+, 98s, ATo+, KTo+` | 15 |
| Maniac | 90 | 55 | 30 | pot, 2×pot | `22+, AT+, 54s+` | 90 |
| By-the-book | 65 | 0 | 5 | 2/3 | `QQ+, AKs, AKo` | 60 |
| Short-stack specialist | 95 | 45 | 40 | pot, 2×pot | `77+, ATs+, KQs, AJo+, KQo` | 100 |
| Wildcard | — | — | — | — | — | — |

Three-bet and call-a-three-bet ranges per archetype are in `profiles.json`.
Three archetypes — tight-passive, loose-aggressive, and game-theory-optimal —
additionally carry six-max and nine-max playbooks; the rest have none and
therefore always use flat strategy.

The by-the-book archetype's zero bluff frequency is deliberate and
behaviourally visible: it never bets a hand below its value threshold.

**The wildcard** is different in kind. It holds no strategy of its own. On
every new-hand notification it selects one of the **eight** other archetypes
uniformly at random and plays that archetype faithfully for the whole hand;
its in-hand behaviour is indistinguishable from the archetype it drew. Only
the style changes between hands.

> The selection set is the eight named archetypes **in a fixed order**. The
> order is part of the seeded behaviour: a seeded run reproduces only if the
> index drawn from the generator maps to the same archetype. The order in the
> vectors is: game-theory-optimal, tight-passive, loose-aggressive,
> tight-aggressive, loose-passive, maniac, by-the-book, short-stack
> specialist.

### Per-variant profile sets

For each of the four non-Hold'em variants a profile is derived from a base
archetype: the derived profile keeps the base's ranges and betting parameters,
takes a name and description tagged with the variant, and records the
variant's betting structure as a marker. Concrete bet amounts are **never**
read from the marker — they come from the table's structure via the seat view
— so the marker's numeric fields may be placeholders. Deriving from any
unrecognised style yields the game-theory-optimal base.

This is the honest state of the original: the per-variant sets are correct in
structure and legality but reuse Hold'em hand selection. Stronger per-variant
ranges are an acknowledged gap, not a hidden one.

### Layered decision capabilities

Every capability's default reproduces the base decision procedure exactly, so
a profile that declares none plays identically to one that declares all
defaults.

| Capability | Settings | Effect | State |
|---|---|---|---|
| Equity awareness | Off (default) / sampled with a budget (default budget 2,000) / exhaustive | Postflop strength source: rank proxy, or real multi-way equity with proxy fallback | Active |
| Range model | Flat (default) / position-aware | Preflop frequency source: flat open range, or the playbook chart for this table size and position | Active |
| Pot-odds discipline | A factor in `[0,1]`, default `1.0` | Scales the call threshold; `1.0` is strict break-even, `0.0` ignores price | Active |
| Exploit mode | Off (default) / light / heavy | Adjusts the profile from aggregate opponent statistics before deciding; a no-op when no statistics are attached. Light uses the default sample gates; heavy lowers them to 15 and 25 hands (DECON-12) | Active |
| Draw/outs augmentation | Off (default) / on | Intended to raise strength for live draws on flop and turn | **Deferred** |
| Preflop chart source | Off (default) / precomputed heads-up table / solver-generated charts | Intended to replace range-membership with a chart lookup | **Deferred** |

The two deferred capabilities are declared in profile data and round-trip
through files, but no decision consults them. The reason is domain-real: both
need information the decider is never handed — a chart keyed by the exact
action sequence faced, and a draw classification that requires knowing which
opponents are live in a form the seat view does not express. A rebuild may
implement them, but must not let doing so change the vectors: a profile with
both toggles at their defaults must decide identically.

### Determinism

A seeded multi-hand run is a single generator threaded through everything that
consumes randomness, in this order per hand:

1. Shuffle the deck from the generator.
2. Notify every occupant of the new hand, passing the same generator.
3. Play the hand; every decision call passes the same generator.

The consequence is the guarantee the research tooling depends on: **same seed,
same starting table, same profiles → identical hands, identical actions,
identical final stacks.** An unseeded run behaves the same way except that
each occupant draws from an unspecified source; unseeded runs are not
reproducible and no vector covers them.

The harness also supports a fixed-stack mode in which every stack is restored
to a buy-in before each hand and the per-hand chip delta is accumulated
instead. This exists so that a strategy comparison measures skill without
survivorship bias — no seat is ever eliminated, so the full requested number
of hands always plays out.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **Agent** | Read its own hole cards, the board, the pot, every stack and commitment, its own position and the button, the structure and tier, and whether it has checked this street; decide any action; re-randomise between hands | See any other seat's hole cards, see any undealt card, learn the deck's order, alter the table, or carry information from one hand into the next except through its own declared state | An agent decides only what to do with the cards it holds; it can neither see nor change anything its seat would not know |
| **Trainer/researcher** | Fix a seed and reproduce a run exactly; swap profiles; run many hands; measure per-seat chip outcomes and action counts | Depend on unseeded runs for any reproducible claim | Any seeded experiment reruns to the identical answer |
| **Administrative** | Author, load, and save profiles as files; seat any profile at any seat; choose the table size and stakes | Change what an action means, or make an agent able to see more by editing its profile | An operator tunes how a seat plays without changing what the seat can know |
| **User/client** | Substitute its own occupant implementation for any seat | Receive more than the seat view when it does so | A third-party occupant is bounded by exactly the same view as a built-in one |
| **Observer/operator** | Read the per-hand and cumulative action counts a run produces | Watch a decision as it is made — there is no live subscription | Everything an agent did is reconstructable afterwards; nothing observes it mid-decision |
| **God-mode** | — | — | N/A — an agent cannot add an action, a variant, or a card. |
| **Spectator** | — | — | N/A — this slice has no delivery surface. |
| **Trustless/cryptographic peer** | — | — | N/A — recorded as a designed absence. |

## Work Items

### Phase 0 — Contract and seat view

- [ ] **0a.** Write the seat-view construction test first: build a table, deal,
      and assert that a view for one seat shows that seat's hole cards, an
      empty board before the flop, every occupied seat's stack and
      commitment, and **no** other seat's cards. Proven by
      `seeded-decisions.json` view fields.
- [ ] **0b.** Implement the seat view with exactly the fields in Scope §4.
- [ ] **0c.** Implement position derivation from logical seat and logical
      button, including the gapped-seating case.
- [ ] **0d.** Define the decision contract and the lifecycle notification,
      each with a seeded twin whose default delegates to the unseeded form.

### Phase 1 — Profiles as data

- [ ] **1a.** Write the round-trip test: every archetype written to a file and
      read back compares equal and decides identically. Proven by
      `profiles.json`.
- [ ] **1b.** Implement the profile model: identity, ranges, betting
      parameters, optional playbook, optional structure marker, capabilities.
- [ ] **1c.** Implement percentage clamping to `0..=100` on construction.
- [ ] **1d.** Populate the eight strategy archetypes with the Design values.
      Proven by `profiles.json`.
- [ ] **1e.** Implement file load and save.

### Phase 2 — The decision procedure

- [ ] **2a.** Write the zero-stack test: a seat with no chips always checks.
- [ ] **2b.** Write the no-cards test: a view with no hole cards still returns
      a legal action via the aggression-only path.
- [ ] **2c.** Implement the ordered decision procedure exactly as in Design,
      including draw order. Proven by `seeded-decisions.json`.
- [ ] **2d.** Implement the rank-proxy strength and the preflop
      frequency-roll strength.
- [ ] **2e.** Implement fixed-limit and pot-fraction sizing with the stack cap,
      the minimum-raise floor, and the big-blind floor.
- [ ] **2f.** Assert every produced action is legal for the view it was
      produced from, across all archetypes and all streets.

### Phase 3 — Positional strategy

- [ ] **3a.** Write the classification test: seat counts 2–6 and 9 classify;
      7 and 8 do not, and an agent at those sizes still returns a legal action.
- [ ] **3b.** Implement table-size classification and the ordered position sets.
- [ ] **3c.** Implement the playbook: table size → position → action →
      weighted range, and table size → position → betting parameters.
- [ ] **3d.** Populate six-max and nine-max playbooks for the three archetypes
      that carry them. Proven by `profiles.json`.
- [ ] **3e.** Implement total fallback to flat strategy on every lookup miss.

### Phase 4 — Determinism

- [ ] **4a.** Write the reproducibility test first: two seeded runs of the same
      length with the same seed and profiles produce identical per-seat final
      stacks and identical action counts.
- [ ] **4b.** Thread one generator through shuffle, lifecycle notification, and
      every decision.
- [ ] **4c.** Implement the wildcard: reselect from the eight in fixed order on
      each new-hand notification, then play that archetype for the hand.
      Proven by `seeded-decisions.json` cases naming the wildcard.
- [ ] **4d.** Implement fixed-stack mode and assert the full hand count always
      completes.

### Phase 5 — Capabilities and variants

- [ ] **5a.** Write the defaults test: a profile declaring no capabilities and
      one declaring all defaults decide identically for every vector case.
- [ ] **5b.** Implement equity awareness, range model, and pot-odds discipline.
- [ ] **5c.** Record draw augmentation and preflop chart source as declared,
      round-tripping, and inert.
- [ ] **5d.** Implement per-variant derivation for the four non-Hold'em
      variants, with the structure marker and the unrecognised-style default.
      Proven by `profiles.json`.

## Test Plan

- **Given** a seat with zero chips, **when** it is asked to decide, **then**
  it checks — regardless of profile or street. (`seeded-decisions.json`)
- **Given** a view whose seat has not been dealt cards, **when** the agent
  decides, **then** it returns a legal action through the aggression-only
  path and never inspects hole cards. (`seeded-decisions.json`)
- **Given** a seed and a fully specified view, **when** any archetype decides,
  **then** the action and its amount match the recorded case exactly.
  (`seeded-decisions.json`)
- **Given** the same seed, table, and profiles, **when** a run of N hands is
  executed twice, **then** final stacks and action counts are identical.
  (`seeded-decisions.json`)
- **Given** a preflop range containing a hand at weight `w`, **when** that
  hand is held over many seeded draws, **then** it is treated as premium at
  frequency `w`. (`profiles.json`)
- **Given** a fixed-limit table, **when** the agent bets or raises, **then**
  the amount equals the street's tier increment (or current bet plus that
  increment), never a pot fraction. (`seeded-decisions.json`)
- **Given** any archetype, **when** written to a file and read back, **then**
  it compares equal and produces identical decisions. (`profiles.json`)
- **Given** a seven- or eight-seat table, **when** an agent decides, **then**
  no positional lookup succeeds, flat strategy is used, and the action is
  legal. (`profiles.json`)
- **Given** a wildcard occupant under a fixed seed, **when** a run proceeds,
  **then** the archetype adopted at each hand matches the recorded sequence.
  (`seeded-decisions.json`)
- **Given** a profile with all capabilities at their defaults, **when** it
  decides, **then** its action equals that of a profile declaring no
  capabilities at all. (`profiles.json`)

## Not specified (implementer's choice)

- **The profile file format.** The original uses a text serialization; any
  format is acceptable provided a profile round-trips and decides identically.
  Field names, nesting, and whether defaults are omitted are all free.
- **How ranges are stored inside a profile** — as strings re-parsed on demand,
  as pre-expanded holdings, or as a compiled table. Only the frequency a given
  holding resolves to is observable.
- **The generator algorithm.** Any generator reproduces the vectors provided
  the same seed yields the same draw sequence *for that implementation*;
  vector cases record the action for the reference draw sequence, so a rebuild
  that uses a different generator must supply its own equivalent cases and
  demonstrate the reproducibility property rather than the exact actions.
- **Memory layout, ownership, and mutability** of profiles, views, and
  occupants; whether a view owns or borrows its card collections.
- **Error representation** for a profile that fails to load.
- **Concurrency.** Whether occupants are shared across threads, and how, is
  free — but an occupant's per-hand state must not leak between concurrent
  hands.
- **Module structure and naming** throughout.
- **How the equity capability computes equity** (DECON-09 governs the values,
  not the mechanism).
- **Whether the two deferred capabilities are implemented**, so long as their
  default settings leave every vector unchanged.

## Spec decisions

None. The manifest's spec-decision index records no open decision for this
epic: the archetype parameters, the position sets, the classification
boundaries, and the wildcard's selection order are all pinned by the vectors.

## Verification

Any implementation must reproduce every file under `vectors/agent-model/`:

1. Every archetype in `profiles.json` is constructible by name and every
   listed parameter matches exactly.
2. Every case in `seeded-decisions.json` produces the recorded action,
   including the exact amount for a bet or raise.
3. A seat view built for any seat of a dealt table exposes that seat's hole
   cards and no other seat's, and exposes no undealt card.
4. Two seeded runs with identical seed, table, and profiles produce identical
   final stacks and action counts.
5. Every action any archetype produces, on every street, under every betting
   structure, is legal for the view it was produced from.
6. Seat counts 2, 3, 4, 5, 6, and 9 classify and yield the position sets in
   Design; 7 and 8 do not classify, and agents at those sizes still act
   legally.
7. Every archetype round-trips through a file with identical decisions.
8. A profile with all decision capabilities at their defaults decides
   identically to one declaring none.
9. The wildcard's per-hand archetype sequence under a fixed seed matches the
   recorded sequence, and its in-hand actions match those of the adopted
   archetype.
10. Fixed-stack mode completes the full requested hand count with no seat
    eliminated.

## Dependencies

**Builds on:** DECON-01 (cards), DECON-02 (the rank ladder used by the
strength proxy), DECON-04 (range notation and weighted ranges), DECON-05
(betting structures, tiers, positions), DECON-06 (the table engine, action
vocabulary, and legality). Optionally DECON-09 when the equity capability is
enabled.

**Blocks:** DECON-12 (behavioural statistics and counter-strategy adjustment
close the loop back onto this epic).

## Provenance (non-normative)

- `src/bot/decider.rs:71` — the decision contract, with the lifecycle hook and
  both seeded twins.
- `src/bot/decider.rs:146`, `src/bot/decider.rs:165` — the profile-driven
  decider and the full ordered decision procedure.
- `src/bot/decider.rs:346`, `src/bot/decider.rs:420` — the wildcard occupant
  and its per-hand reselection.
- `src/bot/decider.rs:463`, `:501`, `:521`, `:618` — hand strength: dispatch,
  rank proxy, preflop frequency roll, stud partial-hand buckets.
- `src/bot/decider.rs:656`, `:669`, `:683`, `:699` — bet-size selection,
  fixed-limit tier increment, raise target, bet amount.
- `src/bot/table_snapshot.rs:50`, `:105`, `:193`, `:325`, `:354` — per-seat
  information, the seat view's fields, its construction, the statistics-bearing
  construction, position derivation.
- `src/bot/profile.rs:46`, `:202`, `:294`–`:501` — style labels, the profile
  model, the eight archetypes plus the wildcard placeholder and the fixed
  ordering of the default set.
- `src/bot/profile.rs:547`, `:591`, `:634`, `:683` — per-variant derivation
  for fixed-limit hold'em, pot-limit Omaha, stud hi, razz.
- `src/bot/profile.rs:723`–`:807` — playbook attachment and the
  range/betting resolution helpers with their fallbacks.
- `src/bot/profile.rs:836`–`:909` — file serialization and load/save.
- `src/bot/playbook.rs:37`, `:90`, `:147` — table-size-keyed strategy entries.
- `src/bot/position_ranges.rs:112`, `:193`, `:265` — position → action →
  weighted range, six-max and nine-max charts.
- `src/bot/positional_betting.rs:34`, `:119`, `:166` — position → betting
  parameters, six-max and nine-max.
- `src/bot/betting_strategy.rs:218`, `:260`, `:287`–`:394`, `:437`, `:442` —
  betting parameters, clamping, archetype values, default value threshold,
  per-street aggression override.
- `src/bot/range_strategy.rs:36`, `:86`–`:203`, `:267` — range fields,
  archetype ranges, weighted open-range frequency lookup.
- `src/bot/table_size.rs:28`, `:58`, `:108` — classification and position sets.
- `src/casino/position.rs:8`, `:95`–`:141` — the position vocabulary and the
  per-size ordered sets.
- `src/bot/decision_config.rs:25`–`:141` — the layered capabilities and their
  defaults; the draw and preflop-chart settings have no reader elsewhere in
  the tree.
- `src/bot/sim.rs:67`, `:184`, `:426`, `:539`, `:609`, `:679` — action
  counts, the harness, seeding, the per-hand order of shuffle/notify/act,
  multi-hand runs, fixed-stack mode.
- `data/bots/*.yaml` and `data/bots/{flhe,plo,stud_hi,razz}/` — file-loadable
  profiles, including per-variant sets.
