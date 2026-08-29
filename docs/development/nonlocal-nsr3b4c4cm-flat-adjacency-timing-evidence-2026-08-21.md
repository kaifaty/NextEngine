# B4C4CM flat-adjacency timing evidence

Status: `PASS / COMPLETE-LANE DESIGN AUTHORIZED`

Date: `2026-08-21`

## Result

Three independent sequential processes pass the frozen measurement protocol
and reproduce deterministic result SHA-256
`472c437876c6f4b78e17d2b31733bc87c3d45fafd77253cd5f1bebaaf7b91600`.
Raw reports differ only in timing fields, as expected.

The candidate is faster in all `63/63` paired rounds for each fixture. Median
process-level speedup is `1.2274x` for P1 and `1.2055x` for P2 over the complete
timed path: neighborhood discovery/construction, pressure evaluation, tape
construction/transfer and identical checksum consumption.

## Deterministic inputs and checksums

| Case | States | Corpus SHA-256 | Output checksum |
|---|---:|---|---:|
| P1 | `264` | `04b72b013b4c88709b3dca83ed1454b56c85acb278f233ed03aee1d96fab3755` | `7096961597763734428` |
| P2 | `9` | `446daff2b5e78531dd7ca1c96bf75c1adf669ff38359a591d23d96e746f960f6` | `616825836938601862` |

These are the exact B4C4BM recorded-state corpus identities. Every state
passes pair/row/evaluation/untaped-HVP/tape correspondence outside timing.

## Timing observations

Times are medians of 21 alternating paired rounds after three warmups.

| Process | P1 legacy / candidate | P1 ratio / wins | P2 legacy / candidate | P2 ratio / wins |
|---|---:|---:|---:|---:|
| 1 | `108.896 / 87.960 ms` | `1.2380x`, `21/21` | `0.825 / 0.686 ms` | `1.2012x`, `21/21` |
| 2 | `104.316 / 84.990 ms` | `1.2274x`, `21/21` | `0.803 / 0.662 ms` | `1.2131x`, `21/21` |
| 3 | `105.317 / 85.993 ms` | `1.2247x`, `21/21` | `0.807 / 0.670 ms` | `1.2055x`, `21/21` |

Process wall/CPU/peak RSS observations were:

```text
run 1   7.00 s / 100% / 9,516 KiB
run 2   6.82 s / 100% / 10,116 KiB
run 3   6.79 s / 100% / 10,288 KiB
```

Raw report hashes with final LF:

```text
6f47aa3bb76218d62d21ce6e2e4a081917540719636fb837c5168c1b81049aff
2780e8d9cf8eef39df0c741036e073c8df17d5f3ddbd278ee9c69a335b650a73
b82d86fc47c4891f75a01f92ec7cae4785575aa5fda201b62032eb809b928e64
```

Without final LF:

```text
dceb1836236e7b68be9515fd1bf98584d6b2389c28139405c73fbbbe08880976
c24fcb3d5a8e9a147b431c99087dd7d0452c4b74873438b4e861d3d478598df0
139dc45e2bbb46b48dba412d76e94fc5387e55e13424d8ed29103e510da650a2
```

## Interpretation

This validates a local serial CPU construction optimization, not a
`1.2x` whole-solver gain. HVP iterations, contact, canonical publication and
other solver work are outside the timed region. The consistent paired result
is sufficient to justify complete-lane integration under the existing exact
physics and rollback gates.

## Decision

Freeze B4C4C1 complete-lane flat-CSR ownership. Preserve B4C4C's opt-in
legacy path as rollback. B4D, nominal corpus, CUDA, runtime/schema and
production remain blocked until the complete-lane gate passes.
