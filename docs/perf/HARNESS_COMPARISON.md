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

## Why the numbers will not match exactly

Each harness pays a different per-operation overhead (loop counter and index
mask in the custom harness, black-box fences in the two libraries) and
summarizes differently (median vs. mean-with-CI). Expect the same ordering
and magnitude across all three; expect the exact nanosecond figures to
differ by a few percent. If the *ordering* ever disagrees — one harness says
`or_rank_bits` is slower than `hand_rank_value`, the others say faster —
that is a measurement bug worth chasing, not noise.
