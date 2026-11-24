# Aikido Trial Evaluation

**Evaluated:** 2026-11-23 at `9d5049a4` (`0.22.0`)
**Trial window:** findings acted on 2026-11-22 to 2026-11-23
**Scope:** every Aikido finding against `pkcore` that left a record in this
repo (commits, PRs #171–#173, `CHANGELOG.md`, `DEFECT_051`), plus the live
Aikido feed read through the Aikido MCP server on 2026-11-23.
**Method:** each finding was checked against the code on `main` and, for the
dependency finding, against the RustSec advisory database and the
`cargo deny check advisories` CI runs.

> **Headline:** the **AI pentest** is the part worth paying for. It found six
> real betting-rule bugs in `Table` that our own tests, CodeQL and
> `cargo-deny` all missed. The **scanner** findings were correct but small.
> The one **dependency** finding (`rand` 0.9.5) could not be confirmed and
> cost a breaking release. The **feed** does not close fixed issues by itself.

---

## Summary

| Source | Finding | Real? | Value | Cost to fix | Verdict |
|---|---|---|---|---|---|
| AI pentest | F1 — gated seat can still raise (`1156678529`) | Yes | High | Medium | Keep |
| AI pentest | F2 — full-stack `Bet`/`Raise` skips PL/FL ceiling (`1156678495`) | Yes | High | Medium | Keep |
| AI pentest | F3 — actions accepted after the street closes (`1156678538`) | Yes | High | Medium | Keep |
| AI pentest | F4 — actions accepted before the hand starts (`1156678483`) | Yes | High | Medium | Keep |
| AI pentest | F5 — equal blinds: big blind acts before UTG (`1156678532`) | Yes | Medium | Low | Keep |
| AI pentest | F6 — `NaN` net passes Money checks (`1156678551`) | Yes | Medium | Low | Keep |
| AI pentest | `table0` example prints the password (`1156678479`) | Yes | Low | Trivial | Keep, advice incomplete |
| CI scan | `persist-credentials` left on in 16 checkouts | Yes | Low | Trivial | Keep |
| Dependency | `rand` 0.9.5 (`AIKIDO-2026-875754`, issue `1155700015`) | Not confirmed | Unclear | **High (breaking)** | Verify first next time |

Fixes: F1–F6 in `0.21.0` (PR #172, [`DEFECT_051`](../defects/DEFECT_051_aikido_pentest_table_guards.md));
password and checkouts in `0.20.14` (PR #171); `rand` in `0.22.0` (PR #173).

---

## What went well

### The AI pentest found real domain bugs

All six table findings were checked by hand before they went into
`DEFECT_051`. All six were real. None was a false positive.

These are not pattern matches. They need an understanding of poker rules:

- F1 needs TDA Rule 47-A (a short all-in does not reopen the betting).
- F2 needs the Pot-Limit and Fixed-Limit ceilings.
- F5 needs to know that 1/1 blinds are a real structure.

Each one is a place where `legal_actions` and the `act_*` methods disagreed.
The `validate_raise` doc said they "cannot drift". They had drifted. A caller
that drives `Table` directly (pkdealer, pkarena0-web) could break the rules.

"Medium" severity was a fair call for a library.

### The pentest reports were good enough to work from

Each finding named the file, the input and the wrong result. The plan in
`DEFECT_051` was written straight from them. Each one became a failing test
first. No finding had to be re-diagnosed.

### The CI-hardening finding was correct

`persist-credentials: false` is standard hardening. The real risk was small,
because no job pushes back. It was cheap to fix.

---

## What went less well

### 1. The `rand` finding could not be confirmed, and it was expensive

Aikido flagged `rand` 0.9.5 under its own ID `AIKIDO-2026-875754`, patched
in 0.10.2. We upgraded in `0.22.0`. That was a **breaking change**:
`BotDecider` now takes `&mut dyn rand::Rng`, and every downstream crate must
move to `rand` 0.10.

What the evidence says:

- The only public advisory for `rand` is `RUSTSEC-2026-0097`
  (`GHSA-cq8v-f236-94qc`). It lists **0.9.3 and later in the 0.9 line as
  patched**. So 0.9.5 was already patched for it.
- That advisory also needs the `log` feature and a custom logger that calls
  `rand::rng()`. pkcore enabled `std`, `std_rng`, `small_rng`, and (under
  `entropy`) `os_rng` and `thread_rng`. It never enabled `log`.
- `cargo deny check advisories` passed on `main` with `rand` 0.9.5
  (CI run on `0d077b2c`, 2026-11-23).
- The Aikido issue is now closed, and the MCP server returns `404` for it.
  We could not read what `AIKIDO-2026-875754` describes.

So we cannot say the finding was wrong. We also cannot say it was right. It
was acted on without that check.

**Rule for next time:** before a breaking upgrade for an `AIKIDO-*` ID,
find the matching RUSTSEC or GHSA entry and check the affected versions and
functions. If there is none, ask Aikido for the source.

### 2. The fix advice was sometimes too narrow and sometimes too wide

- **Too narrow:** the password finding said "do not include `password` in
  output". The same `println!` also printed the username. Our first fix
  followed the advice word for word. CodeQL then flagged the line again
  (commit `96451fbb`).
- **Too wide:** for F6, Aikido also said to enforce whole-chip amounts.
  `DEFECT_051` refused this on purpose: `net` stays `f64` to allow fractional
  blinds later. For the password, it said to add hidden password input to an
  example program.

The findings were right. The remediation text needs a human read.

### 3. The feed does not close fixed issues

On 2026-11-23, the feed showed two issues as **open**: `1156678551` (F6) and
`1156678479` (password). Both fixes are on `main`:

- `src/hand_history/check.rs:1555` and `src/hand_history.rs:1146` refuse
  non-finite values.
- `examples/table0.rs:97` reads the password into `_password` and does not
  print it.

Either the next scan closes them, or they must be closed by hand. Until then
the feed over-counts open work.

### 4. Closed issues cannot be read back

The MCP server lists only open issues. A lookup by ID of a closed issue
returns `404`. So the history of what was found, fixed or ignored lives only
in the Aikido web app. We could not measure a false-positive rate from the
tool. The repo record (`DEFECT_051`, CHANGELOG) is the durable copy.

### 5. The MCP server can time out on start

The Aikido MCP server (`npx -y @aikidosec/mcp@1.0.35`) missed the 30-second
start limit once, with the package already cached. `/mcp` → Reconnect fixed
it.

---

## Compared with what we already run

| Tool | Found F1–F6? | Found password echo? | Flagged `rand` 0.9.5? |
|---|---|---|---|
| Our test suite | No | — | — |
| GitHub CodeQL | No | Yes (and the username) | No |
| `cargo deny check advisories` | — | — | No |
| Aikido | **Yes** | Yes (password only) | Yes, not confirmed |

The pentest is the only thing here that found logic bugs. The scanner
overlaps with CodeQL and `cargo-deny`, which we already run for free.

---

## Recommendation

- **Keep the AI pentest**, if the price is right. Run it after big changes
  to `casino::table`, `hand_history` or any rule-heavy module.
- **The scanner alone is not worth a paid plan** for this repo. CodeQL and
  `cargo-deny` already cover most of it.
- **Verify `AIKIDO-*` dependency IDs** against RustSec or GHSA before any
  breaking upgrade.
- **Read the remediation text**; do not apply it word for word.
- **Close fixed issues by hand**, or confirm a rescan closes them, before
  judging the feed.

## Open questions

- What does `AIKIDO-2026-875754` describe, and does it touch code pkcore
  calls? Ask Aikido support, or read it in the web app.
- Does a rescan of `main` close `1156678551` and `1156678479` by itself?
- Does the next pentest of `main` raise F1–F6 again? `DEFECT_051` exit
  criterion 3 says it must not.
