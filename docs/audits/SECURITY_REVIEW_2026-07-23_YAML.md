# pkcore Security Review — YAML Deserialization Paths

_Date:_ 2026-07-23
_Repo:_ `pkcore` v0.3.2 (HEAD `e498826`, `main`, clean tree)
_Model:_ Claude Fable 5 (`/security-review` command)
_Review basis:_ Two-phase agent review — a vulnerability-identification pass
over every YAML deserialization entry point and its downstream data flow,
followed by an adversarial false-positive filter. Findings below the filter's
8/10 confidence threshold are recorded but not reported as vulnerabilities.

---

## Verdict

**No high-confidence vulnerabilities found.** One candidate finding was
identified and eliminated during false-positive filtering (confidence 2/10).

## Scope Examined

All YAML deserialization entry points using `serde_yaml_bw` 2.5, feature-gated
behind `bot-profiles`, `hand-histories`, `player-stats-persistence`, and
`bot-training`:

| Path | Verdict |
|---|---|
| `src/hand_history.rs` — `HandHistory::from_yaml`, `HandCollection::from_yaml` | Clean |
| `src/bot/profile.rs` — `BotProfile::from_yaml_str`, `from_file` | Clean |
| `src/analysis/player_stats_store.rs` — `YamlPlayerStatsStore` | Clean |
| `src/bot/weighted_range.rs` — custom `Deserialize` impls | Clean |
| `examples/*` (replay_play, yaml_audit, interactive_play*, bot_selfplay, …) | One filtered candidate |

## Why the Library Paths Are Clean

- **No deserialization gadgets.** All YAML deserializes into predeclared typed
  Rust structs via serde — there is no arbitrary-type instantiation (no
  Python-pickle/PyYAML equivalent), so no deserialization-to-RCE path exists.
- **No path traversal.** `YamlPlayerStatsStore::path_for` builds filenames
  exclusively from `Uuid` values (canonical UUID form cannot contain `/` or
  `..`), and `load_all` skips files whose stems don't parse as UUIDs.
  `HandCollection::save` and `BotProfile::to_file` take caller-supplied path
  arguments that are hardcoded literals at every call site — no deserialized
  field ever drives path construction.
- **No dangerous sinks.** Deserialized values flow only into validating
  card-string parsers (returning `Result`), the in-memory replay engine (seat
  indices bound-safe by construction), and console output. No `Command`/shell
  execution, no SQL, no `unsafe`, and no WASM/DOM surface receives untrusted
  YAML (`BotProfile` file I/O is `#[cfg(not(target_arch = "wasm32"))]`).

## Filtered Candidate (not reported as a vulnerability)

**Terminal escape-sequence injection** in `examples/yaml_audit.rs:150` and
`examples/replay_play.rs:87-96`: deserialized fields (`hand.id`, `p.name`) are
printed verbatim, so a crafted hand-history YAML could embed ANSI/OSC sequences
(e.g., SGR conceal to hide a `FAIL` verdict, OSC 52 clipboard writes).

**Filtered out (confidence 2/10)** because:

- The files are cargo example binaries excluded from the published crate
  (`Cargo.toml` `exclude` contains `examples/*`), making them developer-only
  tooling.
- Unsanitized input in human-readable diagnostic output is the excluded
  log-spoofing class.
- The audit tool's exit code is computed from the pass count and cannot be
  spoofed, so automated consumers always see the true result.

Stripping control characters at the print sites would be reasonable hygiene but
is not a reportable vulnerability.

## Out-of-Scope Exclusions Applied

Per the review's ground rules: DoS (YAML billion-laughs/alias expansion,
deep-nesting stack overflow, panic-on-malformed-input in example `unwrap()`s),
resource exhaustion, memory safety (safe Rust), and third-party dependency
currency were not assessed as vulnerabilities.
