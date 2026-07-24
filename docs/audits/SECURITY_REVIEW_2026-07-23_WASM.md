# pkcore Security Review — WASM Surface

_Date:_ 2026-07-23
_Repo:_ `pkcore` v0.3.2 (HEAD `e498826`, `main`, clean tree)
_Model:_ Claude Fable 5 (`/security-review` command)
_Review basis:_ Two-phase agent review — a vulnerability-identification pass
over all 17 wasm-cfg-gated files, the wasm32 dependency resolution in
`Cargo.toml`/`Cargo.lock`, every shuffle/RNG call path, and the serialization
surface consumed by the sibling web apps (pkgto-web, pkkuhn-web, pkarena0-web).
The identification pass reported no findings, so no false-positive filtering
round was needed.

---

## Verdict

**No vulnerabilities found.** pkcore has no wasm-bindgen exports of its own;
its WASM surface is cfg-gated library code, `getrandom` 0.3 with the `wasm_js`
feature, and the game-loop APIs the web apps call.

## What Was Examined and Why It's Clean

### Cryptographic randomness of the shuffle (the highest-value target)

Every production shuffle path — `Cards::shuffle_in_place` (`src/cards.rs:466`),
`CardsCell::shuffle_in_place`, `TableCelled::act_shuffle_deck`
(`src/casino/table_celled.rs:620`), `Session::start_hand`
(`src/casino/session.rs:331`), `Deck::poker_cards_shuffled`, and `SimTable`'s
unseeded path (`src/bot/sim.rs:551`) — converges on `rand::rng()` (rand 0.9.4,
thread-local ChaCha12, reseeded from OS entropy). On wasm32 this resolves
through `getrandom` 0.3.4 with the `wasm_js` feature (`Cargo.toml:99`) to the
browser's `crypto.getRandomValues` — cryptographically strong. A missing
backend cfg fails the build rather than silently degrading, so there is no
path to a weak fallback RNG.

Deterministic-seed APIs (`Cards::shuffle_in_place_with`,
`SimTable::with_seed`/`with_rng`, and the equity engine's
`SmallRng::seed_from_u64` at `src/analysis/equity/engine.rs:205`) are explicit
opt-ins never used as a default on any game-loop path, and the seeded equity
sampling only drives Monte Carlo estimation — never which cards are dealt.
UUID v4 player IDs are also getrandom-backed.

### cfg-gate correctness

Every `#[cfg(target_arch = "wasm32")]` gate merely excludes SQLite/zstd
storage, file/terminal I/O, solver caching, or rayon threading from wasm
builds. The only wasm-specific substitutes are fixed emoji in
`Terminal::random_happy`/`random_sad` (`src/util/terminal.rs:56-77`) —
cosmetic. No gate substitutes weakened randomness, validation, or auth
behavior.

### Data exposure through serialized state

`Table` derives only `Clone, Debug` — no `Serialize`. The
`HandHistory`/`HandCollection` YAML surface (which includes `shuffled_deck`
and hole cards) is a caller-populated **post-hand** record used for local
download in the single-user practice apps, where the browser legitimately runs
all seats — no trust boundary is crossed. Per `ROADMAP.md`, the multiplayer
trust boundary lives server-side in pkdealer/pkgate with a planned
`SessionView::for_principal` redaction seam (EPIC-50/52); the documented
spectator-token card-reveal gap belongs to those sibling repos, not pkcore's
WASM surface.

## Sub-Threshold Observation (not a vulnerability)

`Table::reset` (`src/casino/table.rs:1470-1477`) returns the deck to sorted
order without reshuffling — a consumer that forgets to call
`shuffle_in_place()` each hand would deal predictably. All current callers
(`Dealer::start_hand`, `Session::start_hand`, the web apps' documented game
loop) shuffle correctly, so this is a consumer footgun worth a doc note, not
an exploitable finding.
