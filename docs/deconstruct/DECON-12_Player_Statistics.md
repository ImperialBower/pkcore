# DECON-12: Player Statistics

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

Poker is a game of incomplete information about cards and *complete*
information about behaviour. Every hand a player plays is evidence. This epic
specifies the **behavioural statistics** derived from recorded hands
(DECON-08), the **confidence** those statistics carry given how much has been
seen, and the loop that closes DECON-11's agent model onto this evidence:
**counter-strategy adjustment**, in which an opponent's observed tendencies
transform a base profile into an adjusted one.

Three requirements shape everything below.

**A rate with no sample is not zero.** A player who has never faced a
continuation bet has no fold-to-continuation-bet rate. Reporting `0.0` there
is a modelling error, not a rounding convenience — it says "this player never
folds", which is the opposite of "we do not know". Every derived statistic is
therefore **undefined** when its denominator is zero, and every consumer must
be able to distinguish that from a genuine zero.

**Gathering is side-effect-free with respect to play.** Statistics are
computed from completed hand records, never from live table state. Watching a
table cannot change what happens at it.

**Adjustment is a transformation, not a mutation.** Deriving an adjusted
profile from an opponent's statistics produces a *new* profile. The base
profile is untouched, so the same base can be adjusted differently against
different opponents in the same session, and an adjustment can always be
discarded.

Two honesty notes, stated plainly because a rebuild should fix both:

- The original's own consistency test **disclaims** being a regression test on
  exact ratios. Its bands are deliberately wide because the agent producing
  the hands draws from an unseeded generator, so the ratios move run to run.
  A rebuild should drive statistics generation from a seeded agent (DECON-11
  provides one) and pin exact ratios.
- There is an **open defect**: a smoke test that plays a thousand unseeded
  heads-up hands through the exploitative-play path intermittently fails with
  a betting-completion error, at roughly a 3% rate, with the root cause
  unconfirmed. The path is not deterministic and the failure is not
  understood. A rebuild should make this path deterministic first, which will
  either fix the flake or make it reproducible.

## Status

| Component | Status |
|---|---|
| Raw per-player counters | Planned |
| Derived statistics with undefined-on-no-sample semantics | Planned |
| Confidence banding by sample size | Planned |
| Ingestion from hand records | Planned |
| Per-identity registry | Planned |
| Persistence as a pluggable seam | Planned |
| Counter-strategy profile adjustment | Planned |
| Adjusting decider wrapper | Planned |
| Parameter-space encoding and decoding | Planned |
| Evolutionary search over adjustment configurations | Planned |
| Fitness evaluation against a field | Planned |

## Goals

- Turn recorded hands into a **behavioural read** on each player identity.
- Keep "no sample" and "a rate of zero" **distinguishable everywhere**.
- Attach **confidence** to every read so consumers can suppress noise.
- Make statistics **survive a session** through a seam that does not dictate
  a storage technology.
- Let an agent **adjust** to an opponent purely, and let a search procedure
  **tune** that adjustment automatically.

## Scope

1. **Statistics are keyed by player identity**, not by seat. Seats change
   between hands; identity does not. A hand record whose players carry no
   identity is skipped entirely — there is nothing to accumulate against.
2. **Statistics accumulate across a session** and across sessions when
   persisted. Ingesting the same hand twice double-counts it; deduplication is
   the caller's concern.
3. **Every derived statistic is a ratio of two raw counters** and is undefined
   when the denominator is zero.
4. **Confidence** is a function of hands dealt alone: below 50 hands is low,
   50 through 199 is medium, 200 or more is high.
5. **Ingestion reads records only.** It never reads or writes live table state
   and never influences a decision in progress.
6. **Persistence is a seam.** The requirement is that statistics survive
   across sessions and reload to an identical value. Where and how they are
   stored is free.
7. **Adjustment is pure**: base profile plus opponent statistics plus an
   adjustment configuration yields a new profile. No rule may push a
   parameter negative; all scaling is multiplicative on existing values and
   clamped to the parameter's legal range.
8. **Adjustment targets one opponent**: the active opponent, other than the
   deciding seat, holding the largest stack. When no active opponent exists,
   or the target has no recorded statistics, the base profile is returned
   unchanged.
9. **An adjustment configuration encodes to a fixed-length bounded numeric
   vector of 16 dimensions and decodes back**, with decoding clamping every
   dimension into its bounds and repairing the ordering constraint between
   the two sample gates.
10. **Search is reproducible**: a training run with a fixed configuration and
    seed produces an identical best configuration.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| **Raw counters** | Incremented by ingestion; never derived | `vectors/player-statistics/derivations.json` |
| **Derived statistic** | Ratio of named counters; undefined at zero denominator | `vectors/player-statistics/derivations.json` |
| **Aggression factor** | Postflop aggressive actions relative to postflop calls | `vectors/player-statistics/derivations.json` |
| **Aggression frequency** | Postflop aggressive actions relative to all postflop actions of four kinds | `vectors/player-statistics/derivations.json` |
| **Confidence** | Hands dealt → low / medium / high | `vectors/player-statistics/confidence.json` |
| **Registry** | Identity → statistics; iterable; reconstructable from precomputed rows | `vectors/player-statistics/derivations.json` |
| **Ingestion** | Hand record → counter increments, by the counting conventions below | `vectors/player-statistics/derivations.json` |
| **Persistence seam** | Save and reload to an identical value | `vectors/player-statistics/derivations.json` |
| **Adjustment** | Base profile + opponent statistics + configuration → new profile | `vectors/player-statistics/derivations.json` |
| **Parameter vector** | 16 bounded dimensions; encode/decode round-trip | `vectors/player-statistics/derivations.json` |

`derivations.json` contains, for each case, a complete set of raw counters and
the value of every derived statistic — including the cases where a statistic
is undefined. `confidence.json` maps sample sizes, including the exact band
boundaries, to their confidence band. Conformance means: load the counters,
compute each statistic, and match the recorded value or recorded
undefinedness exactly; and band each sample size to the recorded level.

## Design

### Raw counters

Per player identity:

| Counter | Meaning |
|---|---|
| Hands dealt | Hands the player was dealt into |
| Hands voluntarily played | Hands where the player put money in preflop by choice — a call, bet, raise, or all-in — excluding forced posts |
| Went to showdown | Contested hands the player reached showdown in |
| Won at showdown | Of those, hands the player won or tied |
| Per-street action counts | Folds, checks, calls, bets, raises, all-ins, for each of preflop, flop, turn, river |
| Per-position action counts | The same six counts, keyed by the position the player occupied |
| Preflop-raise opportunities / count | Chances to be the preflop raiser, and raises taken |
| Three-bet opportunities / count | Spots facing a single open raise, and three-bets taken |
| Four-bet opportunities / count | Spots facing a three-bet, and four-bets taken |
| Fold-to-three-bet opportunities / count | Spots where the player open-raised and then faced a three-bet, and folds taken |
| Continuation-bet opportunities / count | Flops where the player was the preflop aggressor acting with no bet yet made, and bets taken |
| Fold-to-continuation-bet opportunities / count | Flops where the player faced the aggressor's continuation bet, and folds taken |
| Check-raise opportunities / count | Postflop streets where the player checked first and someone then bet, and raises taken |

Every opportunity/occurrence pair exists so the derived rate has an honest
denominator. That pairing is the whole point of the raw layer.

### Derived statistics

Each is undefined when its denominator is zero.

| Statistic | Definition |
|---|---|
| Voluntarily put money in pot | hands voluntarily played ÷ hands dealt |
| Preflop raise | preflop-raise count ÷ preflop-raise opportunities |
| Three-bet percentage | three-bet count ÷ three-bet opportunities |
| Four-bet percentage | four-bet count ÷ four-bet opportunities |
| Fold to three-bet | fold-to-three-bet count ÷ fold-to-three-bet opportunities |
| Continuation-bet percentage | continuation-bet count ÷ continuation-bet opportunities |
| Fold to continuation-bet | fold-to-continuation-bet count ÷ fold-to-continuation-bet opportunities |
| Aggression factor | (postflop bets + postflop all-ins + postflop raises) ÷ postflop calls |
| Aggression frequency | (postflop bets + all-ins + raises) ÷ (postflop bets + all-ins + raises + calls + checks) |
| Went to showdown | showdowns reached ÷ hands dealt |
| Won at showdown | showdowns won or tied ÷ showdowns reached |

Three conventions inside these definitions are domain-load-bearing:

- **Aggression measures are postflop only.** Preflop is excluded by
  convention, because preflop aggression is already captured by the
  preflop-raise and three-bet rates and including it swamps the postflop
  signal.
- **An all-in counts as a bet** for aggression purposes.
- **Aggression factor is undefined when there are no postflop calls**, not
  infinite. A player who has bet ten times and never called has no ratio.

### Counting conventions during ingestion

These determine what the counters mean and must be reproduced.

**Identity.** Build the seat → identity map from the record. If no player in
the record carries an identity, skip the hand entirely.

**Position.** Translate physical seats to logical, button-relative indices
before deriving positions (DECON-05), so gapped seating after eliminations
never mis-assigns a position.

**Hands dealt and preflop-raise opportunity.** Every identified player in the
record gets one hand dealt. Every identified player also gets one
preflop-raise opportunity whenever the record has street detail — including
the big blind, who by convention always has the option to raise their own
blind.

**Bet levels preflop.** The big-blind post counts as the implicit first bet.
An open raise is therefore the second bet, a re-raise the third (the
three-bet), and the next the fourth. A player's **first voluntary action** on
the street determines which opportunity they are credited with: facing one
open raise is a three-bet spot; facing a three-bet is a four-bet spot. Forced
posts are skipped and never count as voluntary.

**Fold to three-bet.** When someone three-bets, the original open-raiser — if
a different player — is credited with a fold-to-three-bet opportunity. The
count increments when that same open-raiser subsequently folds.

**Continuation bet.** The preflop aggressor is the last preflop raiser. On the
flop, their **first** action, taken while no one has yet bet, is a
continuation-bet opportunity; a bet or an all-in there is a continuation bet.

**Fold to continuation bet.** Once a continuation bet has been made, every
other seat's first subsequent action on that street is a
fold-to-continuation-bet opportunity, and a fold there increments the count.

**Check-raise.** On any postflop street, a player who checks as their first
action before anyone has bet is marked. When a bet or all-in then occurs,
every marked player is credited with a check-raise opportunity. A marked
player who later raises increments the count. Continuation-bet bookkeeping is
flop-only; check-raise bookkeeping runs on flop, turn, and river.

**Showdown.** A showdown requires at least two contestants who did not fold.
Each such contestant is credited with a showdown reached; a win or a tie also
credits a showdown won. A hand won uncontested is not a showdown.

### Confidence

| Hands dealt | Band |
|---|---|
| 0 – 49 | Low |
| 50 – 199 | Medium |
| 200 or more | High |

Confidence is a property of the whole player read, derived from hands dealt
alone — not per statistic. A player with 300 hands but only two
continuation-bet opportunities carries a high-confidence read containing a
near-worthless continuation-bet rate; distinguishing those is the consumer's
job, and the raw opportunity counts are exposed so the consumer can.

The band boundaries are exact and observable: 49 is low, 50 is medium, 199 is
medium, 200 is high.

### The registry

A registry maps player identity to statistics. It must support: reading one
player's statistics, iterating all pairs, reporting how many players it
tracks, ingesting one hand, ingesting a whole collection of hands, and
**inserting precomputed statistics directly**. That last one matters: it is
the reconstruction path, by which a registry assembled elsewhere — from
storage, from a batch aggregation, from another process — is rebuilt without
re-ingesting hands. A registry rebuilt that way must be indistinguishable
from one reached by ingestion.

A registry must be transportable: it can be written to a neutral
representation and rebuilt into an equal value. Its persistence attachment,
if any, does **not** travel with it — a rebuilt registry arrives unattached,
and attaching storage is an explicit act on the receiving side.

### Persistence

The manifest records this as the one genuinely open administrative seam in the
original, and a rebuild should keep it open.

The seam is four operations: read one player's record; read every record;
write one player's record, overwriting; and flush anything buffered. The
registry loads every known record when storage is attached and writes them all
back on an explicit flush; the original also flushes best-effort when the
registry is discarded, swallowing errors because a discard cannot report one.

> A rebuild should prefer an **explicit** flush and treat implicit
> flush-on-discard as a convenience, not a durability guarantee. The
> requirement is only this: statistics written and then reloaded compare
> identical.

The original's default backing is one text file per player identity in a
configured directory, named by the identity. That arrangement is an
implementer's choice. So is the file format, the directory layout, whether
loading is eager or lazy, and whether the backing is a filesystem at all.

### Counter-strategy adjustment

Adjustment reads the largest-stack active opponent's statistics and applies
each rule whose sample gate is met and whose statistic exceeds (or falls
below) its threshold. Each rule scales one profile parameter by a factor.

Two sample gates exist: a **light** gate for statistics that stabilise quickly
and a **heavy** gate for those that do not.

| Rule | Gate | Condition | Effect |
|---|---|---|---|
| Opponent folds to continuation bets often | Light | fold-to-c-bet > 0.60 | Continuation-bet frequency × 1.4 |
| Opponent rarely folds to continuation bets | Light | fold-to-c-bet < 0.30 | Continuation-bet frequency × 0.6 |
| Opponent goes to showdown often | Light | went-to-showdown > 0.35 | Bluff frequency × 0.4 |
| Opponent is a calling station | Heavy | voluntarily-played > 0.40 | Bluff frequency × 0.5 |
| …and is also passive | Heavy | additionally preflop-raise < 0.10 | Preferred bet sizes replaced with two-thirds pot and pot |
| Opponent is a nit | Heavy | preflop-raise < 0.08 | Aggression × 0.75 |
| Opponent is hyper-aggressive | Heavy | aggression factor > 4.0 | Value threshold becomes max(current × 0.85, 0.30), where the default current value is 0.55 |
| Opponent three-bets often | Light | three-bet percentage > 0.12 | Aggression × 0.80 |

Default gates: light at 30 hands dealt, heavy at 50. A "heavy" *intensity*
setting on the agent side lowers those to 15 and 25 so the agent adjusts
sooner (DECON-11).

Percentage scaling clamps the result into `[0, 100]` after scaling and rounds
to the nearest whole percent. A parameter already at zero stays at zero — no
rule can create aggression where the base profile declared none.

An undefined statistic fires no rule. That is the direct payoff of the
undefined-on-no-sample requirement: without it, an unobserved opponent would
read as an ultra-nit (zero preflop-raise) and every agent would misadjust
against strangers.

Adjustment returns a new profile carrying the base's decision capabilities
unchanged, so the other capability settings survive the transformation. An
adjusting decider is then just a wrapper: adjust, then delegate to an inner
decider with the adjusted profile and the original view. With no statistics
attached, the wrapper is a transparent pass-through — its decisions are
identical to the inner decider's.

### Parameter-space encoding

An adjustment configuration encodes to a **16-dimension** bounded numeric
vector, in a fixed order: the eight thresholds, then the six multipliers, then
the two sample gates.

| # | Dimension | Lower | Upper |
|---|---|---|---|
| 1 | Fold-to-c-bet high threshold | 0.30 | 0.90 |
| 2 | Fold-to-c-bet low threshold | 0.10 | 0.60 |
| 3 | Calling-station voluntarily-played threshold | 0.20 | 0.80 |
| 4 | Passive preflop-raise threshold | 0.05 | 0.30 |
| 5 | Nit preflop-raise threshold | 0.03 | 0.20 |
| 6 | Aggression-factor threshold | 1.00 | 8.00 |
| 7 | Went-to-showdown threshold | 0.20 | 0.60 |
| 8 | Three-bet percentage threshold | 0.05 | 0.25 |
| 9 | Fold-to-c-bet high multiplier | 1.00 | 2.50 |
| 10 | Fold-to-c-bet low multiplier | 0.20 | 1.00 |
| 11 | Bluff-vs-station multiplier | 0.10 | 1.00 |
| 12 | Bluff-vs-showdown multiplier | 0.10 | 1.00 |
| 13 | Aggression-vs-nit multiplier | 0.30 | 1.00 |
| 14 | Aggression-vs-three-bettor multiplier | 0.30 | 1.00 |
| 15 | Light sample gate | 5 | 100 |
| 16 | Heavy sample gate | 10 | 200 |

Decoding **clamps** each dimension into its bounds, rounds the two gate
dimensions to whole hands, and enforces that the heavy gate is at least the
light gate. Consequently a search procedure may propose any vector at all and
still receive a valid configuration — invalid configurations are unreachable
by construction, which is what lets an unconstrained optimiser drive this.

The gates are integers in the domain but continuous in the search space; that
relaxation is deliberate and is why decoding rounds.

### Search

Tuning is an evolutionary search: start from a baseline configuration, and
each generation produce a number of offspring by perturbing the current best
with Gaussian noise scaled per dimension by that dimension's range. The
fittest offspring replaces the parent when it improves the score. Step size
adapts by the one-fifth success rule — it grows when at least a fifth of the
offspring improve on the parent and shrinks otherwise. Search stops at a
generation limit or when the step size reaches a floor.

**Fitness** is the mean big blinds won per hundred hands, measured over a
field of opponents with several independent replicates each. The reference
field is the eight strategy archetypes of DECON-11.

Two properties are essential, both learned the hard way in the original:

- **Common random numbers.** Each opponent-and-replicate session gets a seed
  derived from the master seed but **independent of the candidate being
  scored**, so every candidate is measured on the same hands. Without this the
  optimiser chases sampling noise.
- **Failure is not neutral.** A session that errors or completes zero hands
  scores a large finite negative sentinel, far below the legitimate range, so
  the search selects decisively against error-prone candidates rather than
  treating a failure as break-even. It must stay finite so the mean-fitness
  diagnostic and the step-size arithmetic never see a non-number.

Given a fixed search configuration and seed, two runs must produce the
identical best configuration.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **Observer/operator** | Derive every statistic from recorded hands; iterate every tracked identity; read confidence | Learn anything about a hand that the record does not contain, or change a hand by observing it | Watching costs the game nothing — statistics are a function of the record alone |
| **Administrative** | Attach, detach, and choose a persistence backing; flush; reload a prior session; reconstruct a registry from precomputed rows | Fabricate counters that no record supports | An operator decides where reads are kept, never what they say |
| **Agent** | Consult aggregate opponent tendencies and adjust its own profile | Learn opponent identity, hole cards, or anything about a hand in progress beyond its own seat view | An agent may know how an opponent tends to play; it never learns what that opponent holds |
| **Trainer/researcher** | Encode, perturb, decode, and search adjustment configurations; measure a candidate against a fixed field with a fixed seed | Draw a reproducible conclusion from an unseeded run | Two searches with the same seed reach the same answer |
| **User/client** | Read its own statistics | Alter recorded counters to flatter itself | Statistics are derived, never asserted |
| **God-mode** | — | — | N/A — no statistic changes the rules of the game. |
| **Spectator** | — | — | N/A — this slice has no delivery surface. |
| **Trustless/cryptographic peer** | — | — | N/A — recorded as a designed absence. |

## Work Items

### Phase 0 — Counters and derivations

- [ ] **0a.** Write the undefined-on-no-sample test first: a fresh player has
      **no** value for any of the eleven derived statistics — not zero.
      Proven by `derivations.json`.
- [ ] **0b.** Implement the raw counter set from Design.
- [ ] **0c.** Implement all eleven derived statistics as ratios, undefined at
      zero denominator. Proven by `derivations.json`.
- [ ] **0d.** Implement the postflop-only, all-in-counts-as-a-bet aggregation
      behind aggression factor and aggression frequency. Proven by
      `derivations.json`.

### Phase 1 — Confidence

- [ ] **1a.** Write the boundary test first: 0, 49, 50, 199, 200, and a large
      value band to low, low, medium, medium, high, high. Proven by
      `confidence.json`.
- [ ] **1b.** Implement banding from hands dealt.

### Phase 2 — Ingestion

- [ ] **2a.** Write the skip test: a record whose players carry no identity
      changes nothing.
- [ ] **2b.** Implement seat-to-identity mapping and logical-seat position
      derivation.
- [ ] **2c.** Implement preflop walking: bet levels, first-voluntary-action
      opportunity crediting, voluntary-play detection, preflop-raise,
      three-bet, four-bet, and fold-to-three-bet. Proven by
      `derivations.json`.
- [ ] **2d.** Implement flop walking with continuation-bet and
      fold-to-continuation-bet detection.
- [ ] **2e.** Implement check-raise detection across flop, turn, and river.
- [ ] **2f.** Implement showdown crediting, including the two-contestant
      requirement and tie-counts-as-won.
- [ ] **2g.** Assert ingestion never touches live table state.

### Phase 3 — Registry and persistence

- [ ] **3a.** Write the reconstruction test first: a registry built by
      inserting precomputed rows is indistinguishable from one built by
      ingestion.
- [ ] **3b.** Implement the registry: read, iterate, count, ingest one, ingest
      a collection, insert precomputed.
- [ ] **3c.** Define the persistence seam: read one, read all, write one,
      flush.
- [ ] **3d.** Write the round-trip test: ingest, persist, reload, compare
      identical. Proven by `derivations.json`.
- [ ] **3e.** Implement one reference backing and demonstrate a second is
      substitutable without touching the registry.

### Phase 4 — Counter-strategy adjustment

- [ ] **4a.** Write the purity test first: adjusting leaves the base profile
      unchanged and returns a new one.
- [ ] **4b.** Write the no-stats test: with no statistics attached, or no
      active opponent, or an unknown opponent, the output equals the input.
- [ ] **4c.** Implement target selection: the largest-stack active opponent
      other than the deciding seat.
- [ ] **4d.** Implement percentage scaling with clamping to `[0,100]` and
      rounding.
- [ ] **4e.** Implement each of the eight rules with its gate and threshold.
      Proven by `derivations.json`.
- [ ] **4f.** Implement the adjusting wrapper and assert it is a transparent
      pass-through with no statistics.

### Phase 5 — Parameter space and search

- [ ] **5a.** Write the round-trip test first: encode then decode reproduces
      every field of a default configuration.
- [ ] **5b.** Write the repair test: an out-of-bounds vector decodes to a
      valid configuration, and a vector whose heavy gate is below its light
      gate decodes with the ordering repaired.
- [ ] **5c.** Implement the 16-dimension encoding with the bounds in Design.
      Proven by `derivations.json`.
- [ ] **5d.** Implement fitness evaluation with common random numbers and the
      failure sentinel.
- [ ] **5e.** Implement the evolutionary search with the one-fifth success
      rule and both stopping conditions.
- [ ] **5f.** Assert two searches with the same seed produce the identical
      best configuration.

### Phase 6 — Determinism repair

- [ ] **6a.** Drive all statistics-generating runs from a seeded agent so
      derived ratios are exactly reproducible, replacing the original's
      wide-band smoke test with pinned values.
- [ ] **6b.** Reproduce the exploitative-play failure deterministically under
      a seed, then fix it. Assert chip conservation (DECON-07) across a long
      seeded session on that path.

## Test Plan

- **Given** a player with no recorded hands, **when** each derived statistic
  is requested, **then** every one is undefined and none is zero.
  (`derivations.json`)
- **Given** counters with a non-zero denominator and a zero numerator,
  **when** the statistic is requested, **then** it is defined and equals zero
  — distinguishable from the previous case. (`derivations.json`)
- **Given** a player with postflop bets and raises but no postflop calls,
  **when** aggression factor is requested, **then** it is undefined.
  (`derivations.json`)
- **Given** sample sizes 0, 49, 50, 199, 200, **when** banded, **then** the
  results are low, low, medium, medium, high. (`confidence.json`)
- **Given** a hand record in which a player open-raises and folds to a
  three-bet, **when** ingested, **then** that player gains one
  fold-to-three-bet opportunity and one fold-to-three-bet count.
  (`derivations.json`)
- **Given** a hand record with a preflop aggressor who bets the flop,
  **when** ingested, **then** the aggressor gains a continuation-bet
  opportunity and count, and each other seat's first subsequent action
  registers a fold-to-continuation-bet opportunity. (`derivations.json`)
- **Given** a hand won without a showdown, **when** ingested, **then** no
  player is credited with a showdown reached. (`derivations.json`)
- **Given** a record whose players carry no identity, **when** ingested,
  **then** the registry is unchanged. (`derivations.json`)
- **Given** an ingested registry, **when** persisted and reloaded, **then**
  every player's counters compare identical. (`derivations.json`)
- **Given** an opponent with a fold-to-continuation-bet rate above the high
  threshold and enough hands to pass the light gate, **when** a base profile
  is adjusted, **then** the adjusted continuation-bet frequency is the base's
  scaled by the high multiplier and clamped, and the base is unchanged.
  (`derivations.json`)
- **Given** an opponent below every sample gate, **when** a profile is
  adjusted, **then** the output equals the input. (`derivations.json`)
- **Given** a default adjustment configuration, **when** encoded and decoded,
  **then** every field is preserved; **and given** an out-of-bounds vector,
  **when** decoded, **then** every dimension lands in bounds and the heavy
  gate is at least the light gate. (`derivations.json`)
- **Given** a fixed search configuration and seed, **when** a search is run
  twice, **then** both runs report the identical best configuration.

## Not specified (implementer's choice)

- **How "undefined" is represented** — an absent value, a sentinel, a
  two-valued return. Only the ability to distinguish it from zero is required.
- **Counter widths and overflow behaviour**, provided realistic session
  volumes never saturate.
- **The registry's internal indexing** and whether iteration order is stable.
  No behaviour may depend on iteration order.
- **The persistence backing entirely**: filesystem, database, network,
  in-memory; one record per file or one file for all; the serialization
  format; eager or lazy loading; whether discarding a registry flushes it.
- **Whether statistics are computed on demand or cached.** Only the values
  are observable.
- **The neutral representation used to transport a registry.**
- **Error representation** for storage failures.
- **The random-number generator and the Gaussian sampling method** used by the
  search, provided the reproducibility property holds.
- **Search parallelism.** Evaluations across the field are independent;
  running them concurrently is free, provided the result is unchanged.
- **The exact failure sentinel value**, provided it is finite and far below
  the legitimate fitness range.
- **Module structure and naming** throughout.

## Spec decisions

None. The manifest's spec-decision index records no open decision for this
epic: the ratio definitions, the confidence thresholds, the adjustment
thresholds and multipliers, and the 16 dimensions with their bounds are all
pinned by the vectors.

## Verification

Any implementation must reproduce every file under
`vectors/player-statistics/`:

1. Every case in `derivations.json` yields the recorded value for every
   derived statistic, and yields *undefined* — never zero — wherever the case
   records undefined.
2. Every sample size in `confidence.json` bands to the recorded level,
   including the exact boundaries at 49/50 and 199/200.
3. Ingesting a hand record produces exactly the recorded counter increments,
   under all the counting conventions in Design.
4. A record whose players carry no identity leaves the registry unchanged.
5. A registry rebuilt from precomputed rows is indistinguishable from one
   built by ingestion.
6. Statistics persisted and reloaded compare identical, through at least one
   backing, with a second backing substitutable without changing the registry.
7. Adjustment leaves the base profile unchanged and returns a new profile
   whose parameters match the recorded adjusted values.
8. Adjustment with no attached statistics, no active opponent, or an unknown
   opponent returns an unchanged copy of the base.
9. Encoding and decoding round-trips a valid configuration; decoding an
   arbitrary vector always yields a configuration in bounds with the heavy
   gate at least the light gate.
10. Two searches with the same configuration and seed report the identical
    best configuration.
11. A long seeded session on the adjusting path completes without error and
    conserves chips (DECON-07).

## Dependencies

**Builds on:** DECON-08 (hand records are the sole input), DECON-11 (profiles,
the seat view that carries the statistics handle, and the seeded agents that
generate hands to measure). DECON-05 supplies positions; DECON-07 supplies the
chip-conservation property asserted in verification.

**Blocks:** nothing in this pack — this epic closes the agent loop.

## Provenance (non-normative)

- `src/analysis/player_stats.rs:55` — the raw counter set.
- `src/analysis/player_stats.rs:119`–`:201` — the eleven derived statistics
  and the confidence accessor.
- `src/analysis/player_stats.rs:765`, `:773`, `:780` — the ratio helper that
  returns nothing at a zero denominator, the postflop aggression aggregation
  (all-ins counted as bets), and the action-count increment that ignores
  forced posts.
- `src/analysis/player_stats.rs:220`–`:233` — the confidence bands and their
  boundaries.
- `src/analysis/player_stats.rs:265` — the identity-keyed registry, its
  transport behaviour, and the deliberately untransported storage attachment.
- `src/analysis/player_stats.rs:351` — hand ingestion: identity mapping,
  logical-seat position derivation, hands dealt, preflop-raise opportunity,
  showdown crediting.
- `src/analysis/player_stats.rs:447`, `:534`, `:622` — preflop bet-level
  walking, flop continuation-bet and fold-to-continuation-bet detection, and
  the generic turn/river walker with check-raise detection.
- `src/analysis/player_stats.rs:718`, `:736`, `:748` — attach-with-storage,
  explicit flush, and best-effort flush on discard.
- `src/analysis/player_stats_store.rs:63` — the four-operation storage seam.
- `src/analysis/player_stats_store.rs:117`, `:128`, `:135`, `:140` — the
  reference one-file-per-identity backing.
- `src/bot/exploit.rs:36`, `:73` — the adjustment configuration and its
  default thresholds, multipliers, and sample gates.
- `src/bot/exploit.rs:117` — percentage scaling with clamping and rounding.
- `src/bot/exploit.rs:151` — largest-stack active-opponent selection.
- `src/bot/exploit.rs:197`, `:223`–`:312` — the pure adjustment entry point
  and the five rule groups.
- `src/bot/exploitative_decider.rs:44` — the adjusting wrapper and its
  pass-through behaviour.
- `src/bot/decider.rs:597` — the agent-side exploit intensity settings and the
  lowered gates for the heavy setting.
- `src/bot/training/encoding.rs:11`, `:14`, `:22`, `:41`, `:77`, `:112` — the
  dimension count, the per-dimension bounds, encoding, clamping/repairing
  decode, and the per-dimension ranges used for step scaling.
- `src/bot/training/trainer.rs:39`, `:151`, `:199`, `:207`, `:250` — search
  hyper-parameters, the search itself, and the one-fifth success rule.
- `src/bot/training/evaluator.rs:22`, `:31`, `:48`, `:80`, `:103`, `:111` —
  session stakes and stacks, the failure sentinel, the default field of eight
  archetypes, fitness evaluation, and the config-independent per-session seed
  derivation that gives common random numbers.
- `tests/player_stats_consistency.rs:1`–`:33` — the source's own note that
  this is a differentiation smoke test with deliberately loose bands, not a
  regression test on exact ratios, because the agent's generator is unseeded.
- `tests/player_stats_persistence.rs:31` — the persist-then-reload round trip.
- `docs/DEFECT_exploit_smoke_flake.md` — the open, non-deterministic failure
  on the exploitative-play path.
