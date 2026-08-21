# NSR3-B4C4B immutable static-support-index evidence

Status: `PASS / TIMING DISCRIMINATOR REQUIRED`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --static-support-index-self-test
```

Two parent-gated reports executed concurrently and are byte-identical:

```text
status                 PASS / ONE_MACRO_STATIC_INDEX_ONLY
raw JSON + LF          a0829e6c8e70839a45c9b330fb43ada22910490f59190e70f628e59c91ce5f28
raw JSON without LF    2a593ec11b2a7122ea2d8fafae80db1e9c1548678005988f86229be4562ac1ee
semantic result        40f181c646d2bbc208a46915f09ae9c49dfcd29f22f7969b21940e26f6e2720b
wall time              181.57 s / 182.31 s
CPU utilization        241% / 243%
maximum RSS            16,592 KiB / 15,956 KiB
parent B4C4A1 exact     true
```

The isolated probe passes twice byte-identically in `1.71/1.75 s` at
raw-with-LF
`5d3a900d0b6230d78f5166a0606cae96c38c8c2c74ca21df8ec9efa9f3f2dc35`,
raw-without-LF
`187cff865ea739f96c1440f3041e7fd1f6de1a8c95436ae810f604850d80c90e`
and the same semantic result. Rebuilt B4C4A and B4C4A1 probes preserve their
exact historical hashes.

## Exact correspondence

Legacy, candidate and repeated candidate one-macro transactions are bit-exact
for neighborhood discovery/final pairs, pair hash, adjacency rows, degree and
distance counts, evaluation/density/gradient, pressure tape, workspace hashes,
all KKT states and HVPs, physical diagnostics, schedules, contact/topology,
publication, durable roots and retained-workspace receipts.

Candidate identities and receipts repeat:

| Case | Static index | Work receipt | Query chain |
|---|---|---|---|
| P1 | `62e075ec...c0d4` | `1ef54208...b2e9` | `7108e9c9...c62f` |
| P2 | `53c3d1a8...f281` | `1025e98d...4d9` | `74bfe7de...b446` |

## Exact structural work

| Case | Legacy index builds | Candidate | Legacy sort records | Candidate | Removed |
|---|---:|---:|---:|---:|---:|
| P1 | `264` | `1` | `156,288` | `13,216` | `143,072` (`91.54%`) |
| P2 | `9` | `1` | `11,187` | `1,459` | `9,728` (`86.96%`) |

Dynamic fluid records remain exact at `12,672/243`; support records fall from
`143,616->544` and `10,944->1,216`. Workspace, neighborhood, tape, pair,
directed-record and distance-test counts do not change.

The candidate performs two cell-range lookups where legacy performs one:
P1 `342,144` combined becomes `342,144 + 342,144` split lookups; P2 `6,561`
becomes `6,561 + 6,561`. The contract intentionally has no wall-time gate, so
the net performance effect remains unclassified.

## Failure and invalidation

All frozen negatives pass: missing source, identity mismatch and corrupt layout
fail before pair/adjacency publication; a one-bit support mutation changes
identity, rejects the stale index and matches legacy after rebuild; permuted
input canonicalizes identically; capacity, duplicate, non-finite and invalid
cell-coordinate failures retain legacy classification.

## Decision

Select `ONE_MACRO_IMMUTABLE_STATIC_SUPPORT_INDEX_CANDIDATE` for correctness
and exact structural work only. Before complete-lane application, freeze a
threshold-free interleaved A/B timing discriminator that isolates combined
versus split neighborhood construction. B4C4C, B4D, nominal corpus, CUDA,
runtime and production remain blocked.
