# EPIC-90: Superuser Countermeasures — the Leak Test (PEEK)

> **One-line:** Give a cheated player — and the room that hosts them — a pure,
> local, statistically honest way to ask *"did this opponent play as if he could
> see my cards?"*, answered from hand histories the player already owns, with a
> simulated superuser to prove the test works and a calibrated false-positive
> rate to prove it doesn't accuse the merely good.

> **Kata framing.** Every superuser scandal arrives through a different door —
> an insider's admin account (2000s), a spectator token (pkdealer EPIC-70
> Vector A), a tampered update package (September 2026). The doors cannot be
> enumerated. What every door leaves behind is the same footprint: **decisions
> that depend on information the decider was not dealt.** This EPIC tests for
> the footprint, not the door.

---

## Why now

On 2026-09-29 an anonymous security researcher posting as `@wolfsec0x0`
alleged that a covert remote-access agent had been planted on the Windows PCs
of roughly thirty high-stakes online players through compromised third-party
poker software, letting whoever ran the server watch the victim's screen —
hole cards included — in real time, and drive the mouse and keyboard. The
next day one of the affected vendors, Jurojin, confirmed it: between June 2025
and June 2026 an attacker intermittently swapped the update package served to
one specific group of its users for a tampered build, some of which carried a
remote-access tool. The vendor describes a hand-run, targeted operation by a
known cheater aimed at specific (mostly high-stakes) opponents, names
IntuitiveTables as a second targeted application, and says the same actor ran
phishing sites impersonating poker rooms and tools. The vendor states it holds
logs of every compromised version and the dates each was served.
(Sources in [References](#references).)

Three facts from that account shape this EPIC:

1. **The poker client was never breached.** The leak happened on the victim's
   own screen. No server-side fix — not EPIC-79's mental poker, not EPIC-79b's
   sealed deck — can stop it, because the victim must be shown their cards.
   *Prevention has a floor; detection does not.*
2. **It was targeted.** ~30 victims, chosen by hand. A superuser who cheats only
   a handful of opponents can post an unremarkable aggregate win rate. Any
   detector that looks at a suspect in aggregate will miss him; the unit of
   analysis must be the **(suspect, victim) pair**.
3. **Compromise windows are known.** The vendor can tell each victim *when*
   they were served a tampered build. That is a natural experiment: the same
   opponent, the same victim, inside the window versus outside it.

---

## Status

Status as of `main` @ `78a45ab` (0.15.3), **2026-10-01**. Nothing has landed.
Every row below is honest aspiration.

| Component | Status |
|---|---|
| EPIC-90–94 block opened in `ROADMAP.md`; EPIC-89 (LITE) registered | Planned (Phase 0) |
| `integrity` feature (pure, no new dependencies) + purity gate coverage | Planned (Phase 0) |
| `VictimView` — the typed seat-tier view, single constructor | Planned (Phase 1) |
| `LeakStratum` — public-state stratification incl. `BoardClass` | Planned (Phase 1) |
| `VictimStrength` — victim equity vs. a random hand, per decision | Planned (Phase 1) |
| `LeakTest` — stratified Cochran–Mantel–Haenszel statistic + MH odds ratio | Planned (Phase 2) |
| `LeakReport` — evidence record, multiplicity-adjusted, never a verdict | Planned (Phase 2) |
| `CompromiseWindow` — provenanced input + inside/outside contrast | Planned (Phase 2) |
| Targeting index — per-suspect spread of leak evidence across victims | Planned (Phase 2) |
| `PeekingDecider<D>` superuser simulator + `LeakSchedule` textures | Planned (Phase 3, `superuser-sim` feature, off by default) |
| Calibration: Pluribus null corpus + honest-bot nulls + power curves | Planned (Phase 4) |
| PokerStars-format importer (hero + shown cards only) | Planned (Phase 5) |
| `peek_audit` CLI shell (local, offline) | Planned (Phase 5) |
| Browser delivery via EPIC-86 (device-local audit) | Planned (Phase 5, blocked on EPIC-86) |
| Sequential (live) mode for pkdealer's Boss | Pointer only — pkdealer `EPIC-71` |
| `pkcore:companion` WIT world — capability-confined HUD/table-manager logic | 🔒 Gated — design only (Phase 6) |
| Signed, attested pkcore release artifacts | Planned (Phase 7) |

---

## Context

### The threat classes, and which EPIC owns each

| # | Class | Where the leak happens | Prevention | Detection |
|---|---|---|---|---|
| T1 | **Insider superuser** — staff or admin account sees all hole cards (the 2000s scandals) | Server | EPIC-79 / 79a / 79b / 84 — the house never holds plaintext | **PEEK** |
| T2 | **Endpoint compromise** — malware on the victim's machine reads the screen (this incident) | Victim's PC | None in protocol; Phase 6 narrows the third-party-tool vector | **PEEK** |
| T3 | **Stream sniping / ghosting** — a viewer watches a streamer with too little delay, or someone stands behind the victim | Broadcast / room | Operational (delay) | **PEEK** — same footprint |
| T4 | **Collusion** — partners share their own cards | Between partners | EPIC-79 cannot help (side channel) | pkdealer **EPIC-70** (COLLUDE) |
| T5 | **Puppeting** — attacker drives the victim's mouse to dump chips | Victim's PC | — | EPIC-70's chip-flow asymmetry; recorded here, not built |
| T6 | **RTA** — real-time solver assistance | Suspect's PC | — | **Out of scope**, and PEEK must *not* flag it (see Decision 1) |

T1, T2 and T3 differ in mechanism and are identical in effect: one seat acts on
another seat's hidden cards. That is why a single test serves all three.

### What EPIC-70 already built, and the gap it leaves

pkdealer's EPIC-70 (`pkdealer/docs/EPIC-70_Collusion_and_Cheat_Detection.md`)
built a **blind** detector — the Boss — whose input type `RedactedHand` cannot
hold a hole card, plus an **oracle** scorer that may read every card but only to
grade the Boss. Two tiers: *public* and *omniscient*.

The superuser footprint lives in neither:

- In the **public** tier it is nearly invisible. The signal is "S folds when V
  is strong and calls when V is bluffing" — and V's strength is exactly what the
  blind tier deletes. Showdowns leak a biased sliver of it (you only see V's
  cards in the hands S didn't fold).
- In the **omniscient** tier it is visible but only the house stands there, and
  in T1 the house is the suspect.

There is a **middle tier** EPIC-70 never needed: **one seat's view.** The victim
holds their own cards for every hand they played — legitimately — and that is
precisely the information that leaked. The test PEEK runs is the one only the
victim (or an operator acting for the victim) can run. This EPIC adds that tier.

### What pkcore already has that this stands on

- **The record.** `HandHistory` (`src/hand_history.rs:132`) carries per-hand
  metadata with timestamp (`hand: HandMeta`, `:146`), `players: Vec<PlayerEntry>`
  (`:152`) with stable `player_id: Option<Uuid>` (`:1590`) and
  `hole_cards: Option<String>` documented as `None` when mucked or unknown
  (`:1599`), per-street `streets: Option<Streets>` (`:163`, `:1784`) of `Action`s
  that also carry `player_id` (`:2446`), and — dangerously for this EPIC —
  `shuffled_deck: Option<String>` (`:180`), which reveals every card.
  `HandCollection` (`:1005`) groups them; `showdowns_only()` (`:1205`) exists.
- **A per-principal view, at the live tier.** `PokerSession::view(viewer:
  Option<Principal>)` (`src/casino/session.rs:892`) returns a `SessionView`
  (`:1042`) redacted for one principal; `Principal` is a `Uuid` newtype
  (`src/casino/principal.rs:33`). PEEK needs the same idea for *completed*
  hands.
- **An honest null with every card known.** The Pluribus corpus,
  `data/pluribus/converted_logs/` — 92 files, **10,000 hands** of six-max
  NLHE between Pluribus and professionals, in PokerStars text format with a
  `Dealt to` line for **every** seat — parsed by `Pluribus::parse_log`
  (`src/analysis/nubibus.rs:1353`). No one in it could see anyone else's cards.
  Every ordered (S, V) pair in it is a labelled negative.
- **A deterministic arena.** `SimTable` (`src/bot/sim.rs:214`) runs seeded
  hands (`with_seed`, `:475`) over `Box<dyn BotDecider>` seats (`new`, `:270`);
  `BotDecider::decide_seeded` (`src/bot/decider.rs:77`) is the one required
  method and gets a `TableSnapshot` (`src/bot/table_snapshot.rs:116`) holding
  only the deciding seat's `hole_cards` (`:124`).
- **Equity.** `analysis::equity::compute` (`src/analysis/equity/engine.rs:91`)
  and the embedded heads-up chart (`hup-charts`, default-on).
- **Public stats.** `PlayerStats` / `StatsRegistry`
  (`src/analysis/player_stats.rs:61,277`) and the sample-size `Confidence` band
  (`:232`).
- **The purity gate.** `make check-purity` (`Makefile:325`) checks the
  `--no-default-features` and default trees.

### What does not exist

- No completed-hand view keyed to one seat (`rg 'for_victim|VictimView' src/` →
  zero hits).
- No board-texture classifier (`rg 'BoardTexture' src/` → zero hits; the
  `suit_texture` masks in `src/arrays/matchups/masks/` are heads-up matchup
  canonicalization, not board classes).
- No statistical test anywhere in the crate, and no statistics dependency
  (`Cargo.toml` has none).
- No way for a `BotDecider` to receive another seat's cards.
- No importer for real-site hand histories that show only hero + shown cards.

### What this EPIC explicitly does NOT do

- **No endpoint security.** PEEK does not scan for Mesh Agent services, hidden
  files or indicators of compromise. That is EDR territory; affected players
  should follow their vendor's and a security professional's guidance.
- **No verdicts, no names.** The kernel emits evidence with stated error rates.
  Deciding that someone cheated is a human judgment made with context PEEK does
  not have (Decision 2).
- **No RTA detection.** A solver-assisted player does not know hidden cards; PEEK
  is specifically constructed to stay silent about him (Decision 1).
- **No collusion detection.** EPIC-70 owns it.
- **No change to `Table`, the evaluators, or the `HandHistory` schema.** PEEK
  consumes histories; it does not alter how they are written.
- **No hosted service.** No code path uploads anyone's hand history (Decision 9).

---

## Goals

- A **leak test**: for one (suspect, victim) pair, measure whether the suspect's
  decisions depend on the victim's hidden holding *after accounting for
  everything the suspect could legitimately know*.
- Run it from the **victim's own history** — hero cards plus whatever was shown —
  with no operator cooperation required.
- Report **evidence, not verdicts**: an effect size, an interval, a p-value
  adjusted for how many opponents were tested, and the count of decisions that
  actually carried information.
- Exploit **known compromise windows** as a within-pair natural experiment.
- Expose **targeting**: a superuser who hits few victims shows a lopsided
  per-victim profile even when his aggregate looks clean.
- Ship a **simulated superuser** with schedules that model real behaviour
  (intermittent, big-pots-only, rate-limited) so detection power is *measured*,
  and a **calibrated false-positive rate** on honest elite play (Pluribus).
- Keep it **pure**: an opt-in feature, no I/O, no new dependencies, runnable in
  Wasm on the victim's own device.

---

## Decisions

1. **Test the effect, not the vector.** The null hypothesis is conditional
   independence: *the suspect's action is independent of the victim's hidden
   holding, given the public state.* It is mechanism-agnostic (T1/T2/T3 alike)
   and, importantly, **skill-agnostic**: a great player — or an RTA user — reads
   public information better, which changes how he acts *within* a public state
   but not how his action covaries with cards he cannot see. Skill raises his win
   rate; it does not create dependence on the hidden holding. That is what keeps
   T6 and plain excellence out of the positives.
2. **Evidence, never verdict.** No type in the module is named for cheating, and
   nothing returns a `bool` about a person. The output is `LeakReport` — numbers
   with provenance. This is the house principle *open-texture judgments are
   inputs*, applied in the other direction: the judgment is downstream of the
   kernel, made by a person, and the kernel's job is to make that person's
   evidence auditable and replayable.
3. **The victim tier is the headline.** Three observer tiers, typed: `Public`
   (showdowns only), `Seat(victim)` (the victim's cards always, others when
   shown), `Omniscient` (operator with all cards). PEEK's test is defined on
   `Seat`. `Omniscient` is a power upgrade (it can also condition on the
   suspect's own cards); `Public` is best-effort and documented as biased.
4. **A typed view is the firewall.** `VictimView` has one constructor, which
   drops `shuffled_deck`, `analysis`, and every non-victim hole card that was
   not shown at showdown. This is not about purity — it is about **correctness**:
   a seat-tier result computed with an unshown card silently becomes an
   omniscient-tier result and its calibration no longer applies. Same
   construction as EPIC-70's `redact()`, one tier up.
5. **Stratified CMH is the headline statistic.** Forensics is batch: the victim
   has a fixed window of history. The Cochran–Mantel–Haenszel test over
   public-state strata gives a pooled Mantel–Haenszel odds ratio that reads in
   one sentence — *"at identical public states, S retreated against your strong
   hands N times more often than against your weak ones."* Sequential (SPRT)
   detection belongs to the live shell and is deferred to pkdealer (EPIC-71).
6. **Compromise windows are provenanced inputs.** `CompromiseWindow` carries who
   asserted it and from what source (e.g. a vendor's served-version log). PEEK
   never infers a window.
7. **Multiplicity correction is mandatory, not optional.** A victim auditing 400
   opponents will find a "significant" one by chance. Holm–Bonferroni across the
   victim's opponents; Benjamini–Hochberg FDR for operator-scale scans across all
   pairs. The unadjusted p-value is reported but never headlined.
8. **Pure, opt-in, dependency-free.** A new `integrity` feature, off by default,
   that pulls in no crate. The one special function needed — the normal tail via
   `erfc` — is implemented in-crate (≈30 lines, tested against known values)
   rather than adding a statistics dependency for it. `make check-purity` stays
   green with the feature on.
9. **Local-only delivery.** A victim's history contains every hole card they
   ever played — their whole strategy. Uploading it to a "cheat checker" is
   itself a leak. PEEK ships as a CLI and (via EPIC-86) a device-local browser
   build; the docs say plainly not to use hosted alternatives for this.
10. **The superuser simulator is a defaulted trait method.** `BotDecider` gains
    `decide_seeded_informed(&self, profile, state, leaked, rng)`, whose default
    body ignores `leaked` and calls `decide_seeded`. Every existing decider is
    therefore honest *by construction*; only `PeekingDecider` overrides it.
    Adding a defaulted method is non-breaking (minor bump).
11. **Heads-up-at-decision spots are the headline sample.** When a third player
    is live, the suspect's fold can be about him. v1 tests decisions where S and
    V are the only players left; multiway decisions are tallied and reported but
    excluded from the headline statistic.

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| Who is looking, and what they may see | `ObserverTier { Public, Seat(Uuid), Omniscient }` | ❌ absent |
| The victim's legitimate view of past hands | `VictimView` / `VictimHand` (single `for_victim` constructor) | ❌ absent |
| A decision point where S faces V | `PairDecision` | ❌ absent |
| "Same public situation" | `LeakStratum` (street, context, size bucket, V's line, `BoardClass`) | ❌ absent |
| Board texture | `BoardClass` (paired / flush-possible / straight-possible) | ❌ absent |
| How strong V really was | `VictimStrength` (equity vs. random, bucketed) | ❌ absent |
| Did S press or retreat | `Response { Retreat, Press }` | ❌ absent |
| The test | `LeakTest` → `LeakReport` | ❌ absent |
| When V was exposed | `CompromiseWindow` + `Provenance` | ❌ absent |
| Victim-selectivity of a suspect | `TargetingIndex` | ❌ absent |
| A simulated superuser | `PeekingDecider<D>` + `LeakSchedule` | ❌ absent |
| Completed-hand record | `HandHistory` / `HandCollection` | ✅ reuse |
| Per-principal live view | `PokerSession::view` / `SessionView` | ✅ reuse (pattern) |
| Honest null corpus | `data/pluribus/` + `Pluribus::parse_log` | ✅ reuse |
| Deterministic arena | `SimTable` + `BotDecider` | ✅ reuse, one defaulted method added |
| Equity | `analysis::equity::compute`, `hup-charts` | ✅ reuse |

---

## Design

### Module layout

```text
src/analysis/integrity/          # feature = "integrity"
├── mod.rs                       # re-exports; module docs = the honest-limits summary
├── view.rs                      # ObserverTier, VictimView, VictimHand, for_victim()
├── decision.rs                  # PairDecision extraction from VictimHand
├── stratum.rs                   # LeakStratum, BoardClass, SizeBucket
├── strength.rs                  # VictimStrength
├── cmh.rs                       # 2×2 tables, MH odds ratio, CMH z, RBG interval, erfc
├── report.rs                    # LeakReport, Provenance, multiplicity adjustment
├── window.rs                    # CompromiseWindow, inside/outside contrast
└── targeting.rs                 # TargetingIndex
src/bot/peeking_decider.rs       # feature = "superuser-sim" (implies "integrity")
```

### The view

```rust
/// Which seat of knowledge a test is run from.
pub enum ObserverTier {
    /// Showdowns only. Biased: V's cards are seen only in hands S did not fold.
    Public,
    /// One seat's history — that seat's cards in every hand, others when shown.
    Seat(Uuid),
    /// The house: every card. A power upgrade, and a different calibration.
    Omniscient,
}

/// One completed hand as `victim` legitimately saw it.
///
/// There is no field for the deck and none for an unshown opponent card.
/// `for_victim` is the only constructor.
pub struct VictimHand {
    pub hand_id: String,
    pub timestamp: Option<String>,          // from HandMeta; used by CompromiseWindow
    pub victim: Uuid,
    pub victim_cards: Two,                  // always present, or the hand is skipped
    pub shown: Vec<(Uuid, Two)>,            // cards revealed at showdown, nothing else
    pub board: Option<Board>,
    pub streets: Streets,                   // public actions, unchanged
    pub big_blind: f64,
}

pub fn for_victim(collection: &HandCollection, victim: Uuid) -> VictimView;
```

`VictimView` also records what it *dropped* (`hands_without_victim_cards`,
`hands_without_ids`, `decks_discarded`) so a report can say how much of the
input it could use.

### The decision, the stratum, the strength

For every action by suspect `S` taken while `V` was live, with S and V the
only two players remaining (Decision 11):

```rust
pub struct PairDecision {
    pub suspect: Uuid,
    pub victim: Uuid,
    pub stratum: LeakStratum,
    pub strength: VictimStrength,   // V's real holding, from V's own cards
    pub response: Response,         // what S did
    pub pot_bb: f64,                // for the pot-weighted secondary statistic
    pub hand_id: String,
    pub timestamp: Option<String>,
}

pub enum Response {
    /// Folded to a bet, or checked when checked to.
    Retreat,
    /// Called or raised a bet, or bet when checked to.
    Press,
}

pub struct LeakStratum {
    pub street: Street,
    pub context: Context,           // FacingBet(SizeBucket) | CheckedTo
    pub victim_line: VictimLine,    // V's aggression on this and the prior street
    pub board: BoardClass,
    pub in_position: bool,
}
```

**Why these strata.** V's hidden strength reaches S *legitimately* through V's
betting. The stratum must therefore hold V's line and V's sizing, or an honest
player who simply believes big bets will look like a superuser. Sizing is
bucketed (≤⅓ pot, ≤⅔, ≤pot, overbet) so strata stay populated; the residual
risk of finer sizing tells is Limit 1.

**`VictimStrength`** is V's equity against a uniformly random opposing hand on
the current board (excluding V's cards and the board), split at a configurable
threshold — default 0.60, **an estimate pending Phase 4**. Preflop it reads a
169-class vs-random table derived once from the embedded heads-up chart;
postflop it enumerates exactly (turn and river are cheap; flop cost is bounded
in a Phase 2 test).

**Orientation.** The leak hypothesis predicts S *retreats more against strong
V and presses more against weak V* — in both contexts. With rows
`Strong/Weak` and columns `Retreat/Press`, a leak shows as an odds ratio above 1
in every stratum, so the two contexts pool without sign flips.

### The statistic

For stratum *k* with cells `a` (Strong∧Retreat), `b` (Strong∧Press),
`c` (Weak∧Retreat), `d` (Weak∧Press) and total `n`:

```text
OR_MH   = Σ_k (a·d / n)  /  Σ_k (b·c / n)

E[a_k]  = (a+b)(a+c) / n
V[a_k]  = (a+b)(c+d)(a+c)(b+d) / (n² (n−1))

z       = (Σ a_k − Σ E[a_k]) / √(Σ V[a_k])        one-sided; p = ½·erfc(z/√2)
CI      = exp( ln OR_MH ± 1.96 · SE_RBG )           Robins–Breslow–Greenland
```

Strata with `n < 2` or a zero margin contribute nothing; their count is
reported as `uninformative_strata`. A **pot-weighted** variant (each decision
weighted by `pot_bb`) is computed alongside, because a careful superuser spends
his edge where the money is (`LeakSchedule::BigPotsOnly`), and is reported as
secondary.

### The report

```rust
pub struct LeakReport {
    pub suspect: Uuid,
    pub victim: Uuid,
    pub tier: ObserverTier,
    pub decisions_tested: u32,          // heads-up decisions in informative strata
    pub decisions_multiway: u32,        // tallied, excluded (Decision 11)
    pub informative_strata: u32,
    pub uninformative_strata: u32,
    pub mh_odds_ratio: Option<f64>,     // None when no stratum is informative
    pub ci95: Option<(f64, f64)>,
    pub z: Option<f64>,
    pub p_raw: Option<f64>,
    pub p_adjusted: Option<f64>,        // filled by `adjust(&mut [LeakReport], Method)`
    pub pot_weighted_or: Option<f64>,
    pub window: Option<WindowContrast>,
    pub provenance: Provenance,         // pkcore version, config, input digest
}
```

`Display` writes the one-sentence reading and the caveats; it never uses the
words "cheat", "cheater" or "guilty". A report with too few decisions says
*insufficient evidence* — not *clean*.

### Compromise windows

```rust
pub struct CompromiseWindow {
    pub victim: Uuid,
    pub start: String,                  // ISO-8601, compared against HandMeta.timestamp
    pub end: String,
    pub provenance: Provenance,         // e.g. "vendor served-version log, 2026-09-30"
}

pub struct WindowContrast {
    pub inside: Option<f64>,            // OR_MH inside the window(s)
    pub outside: Option<f64>,
    pub decisions_inside: u32,
    pub decisions_outside: u32,
    pub homogeneity_p: Option<f64>,     // Breslow–Day across the inside/outside split
}
```

A superuser whose access came and went with the tampered update predicts
`inside ≫ outside ≈ 1`. An honest strong player predicts `inside ≈ outside ≈ 1`.
This is the strongest single piece of evidence PEEK can produce, and it needs
nothing but the victim's history and the vendor's dates.

### Targeting

`TargetingIndex` takes all `LeakReport`s for one suspect (operator tier, or a
pooled set of victims who choose to share *reports* — not histories) and reports
the spread of `ln OR_MH` across victims: the fraction of victims with
adjusted-significant evidence, and the dispersion. Targeted superusing
predicts a few extreme victims and many null ones.

### The simulated superuser

```rust
/// Cards another seat holds, delivered to a decider by the harness.
pub struct LeakedHand { pub seat: u8, pub cards: Two }

// Added to BotDecider (src/bot/decider.rs) — defaulted, non-breaking:
fn decide_seeded_informed(
    &self,
    profile: &BotProfile,
    state: &TableSnapshot,
    leaked: &[LeakedHand],
    rng: &mut dyn rand::RngCore,
) -> PlayerAction {
    self.decide_seeded(profile, state, rng)   // honest by default
}

/// Wraps any honest decider; acts on a victim's cards per `schedule`.
pub struct PeekingDecider<D: BotDecider> {
    pub inner: D,
    pub victims: Vec<u8>,
    pub schedule: LeakSchedule,
}

pub enum LeakSchedule {
    Always,
    Intermittent { on: Vec<std::ops::Range<u64>> },   // hand-number ranges — the Jurojin shape
    BigPotsOnly { min_pot_bb: u32 },                  // camouflage
    RiverOnly,
    UseRate { per_mille: u16 },                       // acts on the leak this often
}
```

`SimTable` gains `with_leak(from: u8, to: u8)`; when set, it passes the
victim's cards to the leak recipient's `decide_seeded_informed` and an empty
slice to everyone else. The peek rule is simple and deterministic: when the
schedule fires, compute S's equity against V's *actual* hand (exact on the
river, the equity engine otherwise) and press if it beats the pot odds, retreat
if it does not; otherwise defer to `inner`. These schedules are the PEEK
**data textures** in the kernel-testkit sense.

---

## Work Items

### Phase 0 — Registration & feature skeleton

- [ ] **0a.** `ROADMAP.md` numbering policy: record `EPIC-89 Bare-Metal Kernel
  (LITE)` in the 80-block (now full) and open **EPIC-90 through EPIC-94** as the
  pkcore third internal block, `EPIC-90` claimed by this EPIC. Next free
  `EPIC-91`. (95–99 stay meta.)
- [ ] **0b.** `integrity` feature in `Cargo.toml` (no dependencies);
  `superuser-sim = ["integrity"]`; both excluded from `default`, both included
  in `full`.
- [ ] **0c.** `src/analysis/integrity/mod.rs` skeleton; `make check-purity`
  extended to run with `--features integrity` and stay green.
- [ ] **0d.** `make check-wasm` passes with `--features integrity`.

### Phase 1 — View, decisions, strata, strength

- [ ] **1a.** `ObserverTier`, `VictimHand`, `VictimView`, `for_victim`.
  Test: a hand with a full `shuffled_deck` and all six hole cards yields a view
  holding the victim's two cards, the shown cards, and nothing else.
- [ ] **1b.** `PairDecision` extraction from `Streets`, honoring Decision 11.
  Fixture tests for fold-to-bet, call, raise, check-behind, bet-when-checked-to.
- [ ] **1c.** `BoardClass` and `SizeBucket`, each with exhaustive small tests.
- [ ] **1d.** `VictimStrength`: the 169-class preflop vs-random table, generated
  once and checked in (`generated/`), and postflop exact enumeration. Test: AA
  preflop and the river nuts are `Strong`; 72o preflop is `Weak`.
- [ ] **1e.** `LeakStratum` assembly; doc-test showing two decisions that differ
  only in V's hidden cards land in the same stratum.

### Phase 2 — The test and the report

- [ ] **2a.** In-crate `erfc` with tests against tabulated values (|err| < 1e-7).
- [ ] **2b.** `cmh.rs`: MH odds ratio, CMH z, RBG interval. Golden tests against
  a published textbook worked example (Agresti, *Categorical Data Analysis*).
- [ ] **2c.** `LeakTest::run(&VictimView, suspect, &Config) -> LeakReport`,
  including the pot-weighted variant and multiway tally.
- [ ] **2d.** `adjust(&mut [LeakReport], Method::{Holm, BenjaminiHochberg})`.
- [ ] **2e.** `CompromiseWindow` + `WindowContrast` (Breslow–Day homogeneity).
- [ ] **2f.** `TargetingIndex`.
- [ ] **2g.** `Display` for `LeakReport`; test that the rendered text contains no
  verdict vocabulary.
- [ ] **2h.** Flop `VictimStrength` cost bound: a test asserting a 1,000-decision
  audit completes within a stated budget on CI hardware (budget set from the
  first measurement; recorded as such).

### Phase 3 — The superuser simulator (`superuser-sim`)

- [ ] **3a.** `decide_seeded_informed` defaulted method on `BotDecider`; test
  that every in-crate decider produces byte-identical actions through both
  entry points under the same seed.
- [ ] **3b.** `LeakedHand`, `SimTable::with_leak`.
- [ ] **3c.** `PeekingDecider<D>` + all five `LeakSchedule` textures.
- [ ] **3d.** Behavioural tests: with `Always`, the peeker folds a made hand to a
  bet from a victim holding the nuts and calls the same bet against air; with
  `Intermittent`, behaviour outside the ranges is byte-identical to `inner`.

### Phase 4 — Calibration (the part that earns trust)

- [ ] **4a.** **Pluribus null.** Run `LeakTest` at `Seat` tier for every ordered
  (S, V) pair across all 10,000 hands. Report the distribution of `p_raw` (it
  should be ≈ uniform) and the empirical false-positive rate at α = 0.01 and
  0.001 after Holm. Any departure is a finding, written up before thresholds
  are set.
- [ ] **4b.** **Honest-bot nulls.** Same, over seeded `SimTable` runs of the
  EPIC-18 styles and `ExploitativeDecider` (which reads opponents' *stats* —
  legitimate information that must not register).
- [ ] **4c.** **Card-removal bias.** Measure the baseline odds ratio among honest
  bots (S's own cards weakly predict V's); record it as the empirical null
  offset if it is distinguishable from 1.
- [ ] **4d.** **Power curves.** For each `LeakSchedule`, decisions-to-detection
  at 80% power vs. `UseRate`. Publish the floor: below what use-rate and sample
  size PEEK cannot see, stated as a number.
- [ ] **4e.** **Window contrast power.** Intermittent superuser, known windows:
  how much faster does `WindowContrast` reach significance than the pooled test?
- [ ] **4f.** **State coverage.** Fraction of reachable `LeakStratum` cells the
  calibration corpora populate; strata never populated are listed.
- [ ] **4g.** `docs/PEEK_CALIBRATION.md` — every number from 4a–4f with seed and
  commit, and `PEEK_POTENTIAL_LIMITS.md` (see below) updated from results.
  Finalize the `VictimStrength` threshold here.

### Phase 5 — Shells

- [ ] **5a.** PokerStars-format importer producing `HandCollection` from real
  histories that show **hero cards and shown cards only** — generalizing what
  `Pluribus::parse_log` does for all-cards logs. Behind `hand-histories`.
- [ ] **5b.** `examples/peek_audit.rs` — reads a directory of the user's
  histories and optional window file, runs every opponent, prints Holm-adjusted
  reports ranked by evidence. Offline; no network code in the example.
- [ ] **5c.** Browser delivery through EPIC-86 bindings (blocked on EPIC-86):
  histories are read by the page from local files and never leave the tab.
- [ ] **5d.** Pointer doc for pkdealer `EPIC-71`: a sequential PEEK signal for
  the live Boss at the operator tier, beside EPIC-70's collusion signals.

### Phase 6 — 🔒 Gated: capability-confined companion tools (design only)

The T2 vector in September was a third-party table manager's update channel.
This phase drafts a `pkcore:companion` WIT world for the *logic* of such tools
— HUD stats, overlay layout, hotkey bet-sizing — so it can ship as a Wasm
component whose imports are a seat-tier event stream and a `StatsRegistry`
read, and whose exports are overlay draw lists and action *intents*. No
`wasi:sockets`, no `wasi:filesystem`, no process spawn. A tampered component
delivered through the update channel can make the HUD lie; it cannot open a
remote-access session, because nothing it can import reaches the network or
the OS. The privileged host shrinks to a small, rarely-updated, signed binary
that verifies each component's digest against a public append-only log —
which also makes *targeted* serving (one group of users getting a different
build) detectable, the exact pattern in the Jurojin account.

Deliverable: `docs/COMPANION_WORLD.md` + `wit/companion.wit` draft + an
honest-limits section (host compromise, OS compromise, installer tampering,
and screen capture by unrelated malware all remain). Status stays 🔒 until the
design is reviewed against at least one real tool's feature list.

### Phase 7 — pkcore's own supply chain

pkcore is a library that tools of exactly this kind embed.

- [ ] **7a.** Release artifacts (crates.io, the npm package from EPIC-85,
  the Wasm build from EPIC-86, `pkcore.py`) published through CI with
  build-provenance attestations; verify each registry's current
  trusted-publishing support at implementation time.
- [ ] **7b.** Reproducible Wasm build with the digest published in the release
  notes, so a downstream host (Phase 6) can pin it.
- [ ] **7c.** `AI-BOM.md` and `deny.toml` reviewed for the new features.

---

## Honest limits (seed for `PEEK_POTENTIAL_LIMITS.md`)

1. **Legitimate reads beyond the strata.** Timing, sizing to the chip, and
   session-level history are public signals the strata do not hold. A player
   who reads them well can show mild dependence on V's strength. Sizing buckets
   absorb the largest part; Phase 4 measures what remains on Pluribus, where
   timing is absent — so the *live* residual is likely larger than the
   calibrated one.
2. **Card removal.** S's own cards make some V holdings less likely. The effect
   is small and measured (4c), not assumed away.
3. **Volume.** High-stakes pairs may share only hundreds of heads-up decisions.
   PEEK will often say *insufficient evidence*, and that is a correct answer.
4. **A disciplined superuser is hard to see.** Using the leak rarely, only in
   big pots, buys camouflage at the cost of edge. Phase 4d publishes the floor;
   the pot-weighted statistic narrows it; nothing closes it.
5. **Mechanism is not identified.** A positive result means *dependence on the
   victim's hidden cards*. Malware, an insider, a stream without delay, or a
   person behind the victim all produce it. PEEK cannot say which.
6. **Not every leak is a hole-card leak.** A remote-access tool also sees the
   victim's notes, HUD and tells about their tilt. Those leaks are invisible to
   a hole-card test.
7. **Pluribus is a narrow null.** Play-money, six-max, 10,000-chip stacks reset
   every hand, one bot and a fixed field. A clean result there bounds the false
   positive rate for that texture of play, not for every game.
8. **The public tier is biased.** Showdown-only data sees V's cards selectively
   (only when S didn't fold). `ObserverTier::Public` reports carry that caveat
   in their `Display` and are excluded from headline claims.

---

## Test Plan

| Area | Kind | What it proves |
|---|---|---|
| `for_victim` | unit + doc | No unshown opponent card or deck survives the view |
| Decision extraction | fixture | Each betting line maps to the right `Response` and stratum |
| `BoardClass`, `SizeBucket` | exhaustive unit | Classes are total and disjoint |
| `VictimStrength` | unit + golden | Known hands land in known buckets; table regenerates identically |
| `erfc`, CMH, RBG | golden | Match published values to tolerance |
| Multiplicity | unit | Holm and BH match hand-computed examples |
| `LeakTest` null | property (proptest) | Shuffling V's cards across hands within strata yields ≈ uniform p |
| `LeakTest` power | sim | `PeekingDecider::Always` flagged within the 4d budget at seed *s* |
| Honest-by-default | sim | All deciders identical through `decide_seeded_informed` |
| Report text | unit | No verdict vocabulary in `Display` |
| Purity | `make check-purity` | `integrity` adds no forbidden crate |
| Wasm | `make check-wasm` | Feature builds for the browser target |

Test modules follow house naming (`analysis__integrity__cmh_tests`, etc.), no
`test_` prefixes, colocated `#[cfg(test)]` modules, no `unwrap` outside tests.

---

## Key Files

| File | Role |
|---|---|
| `src/analysis/integrity/*` | New — the kernel |
| `src/bot/peeking_decider.rs` | New — the simulated superuser |
| `src/bot/decider.rs:77` | One defaulted method added to `BotDecider` |
| `src/bot/sim.rs:214` | `SimTable::with_leak` |
| `src/hand_history.rs:132,1005,1574,2436` | Read only — the input |
| `src/analysis/nubibus.rs:1353` | Read only — the null corpus parser |
| `data/pluribus/converted_logs/` | The null corpus |
| `Cargo.toml` `[features]` | `integrity`, `superuser-sim` |
| `Makefile:325` | Purity gate extended |
| `ROADMAP.md` numbering policy | EPIC-89/90 registration |
| `examples/peek_audit.rs` | New — CLI shell |

## Reuse (do NOT recreate)

- `HandHistory` / `HandCollection` / `Streets` / `Action` — the record.
- `Principal` and the `PokerSession::view` pattern — the shape of a per-seat view.
- `analysis::equity::compute` and the heads-up chart — strength.
- `Pluribus::parse_log` — the null corpus, and the starting point for 5a.
- `SimTable`, seeded RNG, existing `BotDecider`s — the arena and the honest controls.
- EPIC-70's design vocabulary (typed firewall, oracle tier, labels, FP study).

## Compatibility

Minor bump. Two opt-in features, one defaulted trait method, no field or
signature changes to existing public types, no change to the `HandHistory`
format. Default-feature builds are byte-identical in behaviour.

## Dependencies

- **Uses:** the `HandHistory` recorder, EPIC-26 (player ids, stats), EPIC-87
  (Pluribus corpus), EPIC-18/27 (honest controls).
- **Blocks on, for Phase 5c:** EPIC-86 (browser bindings).
- **Siblings:** pkdealer EPIC-70 (collusion; shares vocabulary, not code);
  EPIC-79/79b/84 (prevention for T1 — complementary, not overlapping).
- **Feeds:** pkdealer EPIC-71 (live sequential signal); the kernel-testkit
  skill (PEEK's `LeakSchedule` textures and stratum coverage are a worked
  example of data textures and state coverage for a detector rather than a
  kernel).

## Verification

```bash
cargo test  --features integrity
cargo test  --features superuser-sim
cargo test  --features full
make check-purity
make check-wasm
cargo run --example peek_audit --features "integrity hand-histories" -- \
    --histories ./my_hands --victim <uuid> [--windows windows.yaml]
cargo clippy --features full -- -D warnings
```

---

## Open Questions

1. **Threshold or rank test?** The `Strong/Weak` split at 0.60 equity is
   simple and readable but throws away information. A stratified rank test
   (van Elteren) on V's equity uses all of it. Default: CMH in v1 for
   readability; revisit if 4d shows the power loss is material.
2. **Which real-site formats after PokerStars?** Each site's export differs.
   Default: PokerStars only in this EPIC; others as follow-ups, ideally
   contributed by people who hold such histories.
3. **Omniscient tier in v1?** Conditioning on S's own cards raises power for
   operators. Default: type exists, test is seat-tier only; omniscient
   conditioning is a follow-up once seat-tier calibration is published.
4. **Should victims pool reports?** `TargetingIndex` across victims needs them
   to share results (not histories). A signed `LeakReport` export would let a
   group of affected players compare notes without exposing strategy. Default:
   out of scope; the report is already history-free.
5. **Name of the CLI.** `peek_audit` example now; a `pkpeek` binary crate if it
   grows a UI.
6. **Multiway.** Decision 11 drops most decisions in six-max. A multiway
   extension stratifying on the number and line of other live players is
   possible; default is to report the tally and leave it.

---

## References

- PokerNews, *Is a New Superuser Targeting High-Stakes Online Poker Players?*,
  2026-09-29 — https://www.pokernews.com/news/2026/09/superuser-targets-online-poker-players-52500.htm
- PokerNews, *Third-Party Software Company Impacted By Online Poker Cyber
  Attack Issues Statement*, 2026-09-30 — https://www.pokernews.com/news/2026/09/online-poker-scandal-statement-released-52508.htm
- Mantel, N. & Haenszel, W. (1959), *Statistical aspects of the analysis of
  data from retrospective studies of disease*, JNCI 22.
- Robins, J., Breslow, N. & Greenland, S. (1986), *Estimators of the
  Mantel–Haenszel variance consistent in both sparse data and large-strata
  limiting models*, Biometrics 42.
- Agresti, A., *Categorical Data Analysis* — CMH and Breslow–Day worked examples.
- Holm, S. (1979); Benjamini, Y. & Hochberg, Y. (1995) — multiplicity.
- Brown, N. & Sandholm, T. (2019), *Superhuman AI for multiplayer poker*,
  Science — source of the Pluribus hands in `data/pluribus/`.
- pkdealer `docs/EPIC-70_Collusion_and_Cheat_Detection.md` — the blind/oracle
  tier design this EPIC extends.
