# NSR3-B4C4BM static-support timing evidence

Status: `PASS / COMPLETE-LANE ROLLOUT DESIGN AUTHORIZED`

Date: `2026-08-21`

## Reproducible result

Command:

```text
nonlocal-formula-reclosure --static-support-index-benchmark
```

Three sequential processes pass. Raw reports intentionally differ because
they contain nanosecond samples:

```text
run 1 raw + LF   8c9bc42e96b6966f18bb50ebf5c7e5c594dfaaf1edbcb26aeed25822816fe029
run 2 raw + LF   5ac8612ef1b229ba6b52929faef523990d414a44e73d04befa4eeb36a8bd5dba
run 3 raw + LF   df943e699e50c59f64854dec5a571b4c0a447e120c568b732bf61284eb0e722d
deterministic     66cccf12ffebff98a0ef904907bb04aa015a8e57102f1f8dd7bf1ef5a7663465
wall time         5.59 s / 5.57 s / 5.46 s
CPU utilization   100% / 100% / 100%
maximum RSS       9,388 / 9,300 / 10,036 KiB
```

All reports reproduce the B4C4B isolated parent exactly. Ordered corpus hashes,
static-index identities and checksums are identical:

| Case | States | Corpus | Checksum |
|---|---:|---|---:|
| P1 | `264` | `04b72b01...3755` | `7224013671075326178` |
| P2 | `9` | `446daff2...60f6` | `13329469309062717860` |

Every recorded state passes full untimed neighborhood/pair-hash
correspondence. Warmups and all timed passes reproduce the frozen checksum.

## Timing result

Each row is a median of 21 alternating AB/BA paired corpus passes:

| Run | Case | Legacy median | Candidate median | Candidate wins | Speedup |
|---|---|---:|---:|---:|---:|
| 1 | P1 | `71.657 ms` | `61.757 ms` | `21/21` | `1.1603x` |
| 2 | P1 | `69.921 ms` | `61.844 ms` | `21/21` | `1.1306x` |
| 3 | P1 | `69.465 ms` | `60.939 ms` | `21/21` | `1.1399x` |
| 1 | P2 | `1.362 ms` | `0.526 ms` | `21/21` | `2.5883x` |
| 2 | P2 | `1.366 ms` | `0.526 ms` | `21/21` | `2.5982x` |
| 3 | P2 | `1.352 ms` | `0.521 ms` | `21/21` | `2.5952x` |

The median process-level speedup is `1.1399x` P1 and `2.5952x` P2. This is a
neighborhood-construction result, not a whole-solver or production speedup.
Index construction is outside timing because the selected lifecycle amortizes
one index over the transaction.

## Interpretation

The candidate is faster in every paired round despite exactly doubling
cell-range binary searches. P1 benefits less because its dynamic fluid work and
large pair payload dominate more of construction; P2's larger static-support
ratio makes repeated combined sorting much more expensive.

No fitted timing threshold was used. B4C4BM passes because the measurement is
valid and classifies both cases as observed `CANDIDATE_FASTER`.

## Decision

Authorize a separately frozen B4C4B1 complete adaptive/macro-fixed static-index
rollout with exact correspondence, index lifetime, rebuild and failure gates.
B4C4C, B4D, nominal corpus, CUDA, runtime and production remain blocked.
