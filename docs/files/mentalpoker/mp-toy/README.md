# mp-toy

Companion exercises for the **mental-poker cryptography lesson plan**
(`mental-poker-crypto-lesson-plan.md`). One crate, six modules, 22 tests.
Your job: replace the `todo!()`s until everything is green.

**This crate is deliberately insecure** — tiny numbers (p = 467), a
non-cryptographic hash, no constant time, a toy RNG. That's the point: every
intermediate value is small enough to inspect and check on paper. It exists
for X-ray vision into the algebra and retires after module 8 of the plan.
Never let any of it near real code.

## Workflow

```bash
cargo test                       # runs YOUR implementations (starts: 22 failing)
cargo test m1_                   # just module 1's tests
cargo test --features solutions  # reference solutions — verify the tests, or peek
```

Each function body is an `exercise!("hint", { reference })` macro: by default
you hit a `todo!()` with a hint; with `--features solutions` the reference
implementation runs (all 22 tests pass — verified). Work module by module, in
order; each builds on the previous.

## The modules

| Module | You implement | The test that teaches |
|--------|---------------|----------------------|
| `m1_groups` | `mod_mul`, `mod_exp`, `element_order`, `subgroup_elements` | G generates a prime-order-233 subgroup of ℤ*₄₆₇; every non-identity element is a generator (Lagrange) |
| `m2_dh` | `keypair`, `shared_secret`, `baby_step_giant_step` | **`m2_break_the_exchange`** — steal a DH secret from public data in ~15 steps; extrapolate to 2^128 |
| `m3_elgamal` | `encode/decode_card`, `mask`, `remask`, `unmask_full` | **`m3_why_shuffling_without_remask_hides_nothing`** — track cards through a permute-only "shuffle" by equality |
| `m4_threshold` | `aggregate_key`, `reveal_token`, `apply_tokens` | n−1 of n tokens never yield the true card; the staged unmask *is* dealing a hole card |
| `m5_sigma` | Schnorr + Chaum–Pedersen (interactive), **`schnorr_forge`** | pass verification for a key you don't know, given a predictable challenge — why challenges must be unpredictable |
| `m6_fiat_shamir` | NIZK Schnorr/CP, `shuffle_and_remask` | the module-5 forgery is dead; **`m6_full_protocol_end_to_end`** runs the whole deal: keys→aggregate→mask→shuffle×3→hole card→board |

The attack exercises (`m2_break_the_exchange`, `m5_fixed_challenge_forgery`,
`m3_why_shuffling_without_remask_hides_nothing`) are as important as the
constructive ones — every primitive earns its place by the attack its absence
permits.

## Small-group honesty

Two artifacts of tiny parameters, called out where they bite:

- The subgroup has 233 elements and 52 are card encodings, so a *partially*
  unmasked value collides with **some** card ~22% of the time by accident. The
  threshold tests therefore assert "not the *true* plaintext," which is the
  claim that survives at real sizes (where accidental collisions are ~2^-244).
- `toy_hash` is FNV-1a, not SHA-256. The *shape* of Fiat–Shamir is identical;
  the collision resistance is not. The forgery test's `assert_ne!` holds for
  this hash and these inputs, and overwhelmingly for a real hash.

## Where it ends

After module 6 the crate runs the complete Barnett–Smart deal **except** the
verifiable shuffle — the tests say so explicitly. That gap is module 7 of the
plan (read Bayer–Groth, study `geometryxyz/mental-poker`), and module 8 swaps
these u64s for Ristretto points via `curve25519-dalek` to implement the real
`CardCrypto` backend for `pkcore-mp`. The API you built here maps onto that
trait almost 1:1 — by design.
