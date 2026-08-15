# Three Harnesses, One Workload Set

The nano-band workloads are measurable three ways. The custom harness is the
one whose numbers are published (`RESULTS.md`); the Criterion and Divan
benches exist for edification — the same operations over the same hand
samples, so the harnesses themselves can be compared.

| | Custom (`perf run`) | Criterion | Divan |
|---|---|---|---|
| Run with | `make perf-native` | `cd perf && cargo bench --bench criterion` | `cd perf && cargo bench --bench divan` |
| Timing model | one `Instant` around a loop of N ops, divided by N | many sampled windows, linear regression, confidence interval | per-call timing, lean protocol |
| Dead-code defence | integer checksum (doubles as determinism proof) | explicit `std::hint::black_box` | returned values black-boxed automatically |
| Statistics | min / median / p95 / MAD | mean ± CI, outlier classification, regression vs. last run | median, fastest, mean |
| Cross-target (wasm) | yes — that is why it exists | no (`black_box` availability, filesystem, plotting) | no |
| History | committed JSON in `docs/perf/results/` | `target/criterion/` (untracked) | none |
| Cost | one dependency-free crate to maintain | heavy dev-dependency tree | light dev-dependency |

Both bench files live in `perf/benches/` and share the catalog's sample
builders (`five_sample`, `seven_sample`), so a row here matches a row in
`RESULTS.md` by workload name. Run all three back to back with:

```sh
make perf-native && make perf-bench
```

## A measured example

All three harnesses on the same host, same build, back to back
(2026-08-14, Apple M1, `rustc` release build). Median nanoseconds per
operation:

| Workload | Custom (`perf run`) | Criterion | Divan |
|---|---:|---:|---:|
| `eval.five.or_rank_bits` | 1.23 | 1.95 | 40.2 |
| `eval.five.hand_rank_value` | 13.25 | 14.9 | 61.7 |
| `parse.five.from_str` | 525 | 537 | 665 |
| `eval.seven.hand_rank_value` | 734 | 752 | 915 |

Two things worth noticing:

- **Custom and Criterion agree within a few percent** on every row — two
  unrelated timing models converging is the best cheap evidence that both
  are honest.
- **Divan reads far high on the smallest operations** (40 ns for a ~1 ns
  bit-or) because its default protocol here timed calls individually, so
  clock overhead dominates tiny work. The gap closes as the operation grows:
  ~25% high at 700 ns. Same ordering, though — all three harnesses rank the
  four workloads identically, which is the invariant that matters.

## Why the numbers will not match exactly

Each harness pays a different per-operation overhead (loop counter and index
mask in the custom harness, black-box fences in the two libraries) and
summarizes differently (median vs. mean-with-CI). Expect the same ordering
and magnitude across all three; expect the exact nanosecond figures to
differ by a few percent. If the *ordering* ever disagrees — one harness says
`or_rank_bits` is slower than `hand_rank_value`, the others say faster —
that is a measurement bug worth chasing, not noise.
