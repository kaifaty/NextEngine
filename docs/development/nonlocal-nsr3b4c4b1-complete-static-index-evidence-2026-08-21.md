# NSR3-B4C4B1 complete-lane static-index evidence

Status: `PASS / FLAT-ONLY CSR DESIGN AUTHORIZED`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --complete-static-support-index-self-test
```

Two parent-gated reports are byte-identical:

```text
status                 PASS / COMPLETE_RESEARCH_LANES_ONLY
raw JSON + LF          2dc13154936077e8148e5308344b1418e51e0a44841839a8719a469b174b6146
raw JSON without LF    7d93828a21473fd6af6534107199e844387f41b8b3d86ef18b7141a854cd74b9
semantic result        05c4b74a330fc23f598bbae687e68e84403da3f69ee0bf9bbdb05b70add818c8
wall time              62.05 s / 62.71 s
CPU utilization        125% / 125%
maximum RSS            9,488 KiB / 9,956 KiB
B4C4BM parent exact     true
```

The isolated probe passes at raw-with-LF
`cf7f4355043dd83a2e64436ce441f2101c8cc375c632421958056517f35b6567`,
raw-without-LF
`8a04973d683f64a52a9d66e1a0fa9f4430d8e3db00767772b284ca303495efb2`
and the same semantic result.

## Exact complete-lane work

| Lane | Workspace/index builds legacy | Candidate index builds | Removed support records |
|---|---:|---:|---:|
| P1 adaptive | `1,924` | `1` | `1,046,112` |
| P1 fixed 48 | `1,557` | `1` | `846,464` |
| P1 fixed 96 | `2,937` | `1` | `1,597,184` |
| P1 fixed 192 | `4,631` | `1` | `2,518,720` |
| P2 adaptive | `323` | `1` | `391,552` |
| P2 fixed 48 | `900` | `1` | `1,093,184` |
| P2 fixed 96 | `1,751` | `1` | `2,128,000` |
| P2 fixed 192 | `3,449` | `1` | `4,192,768` |

Candidate support sorting is exactly `544` records for each P1 lane and
`1,216` for each P2 lane. Dynamic records, workspace/neighborhood/tape builds,
pairs, directed records, distance tests and retained transfers/reads/releases
remain exact. Split fluid and support lookup counts each reproduce the legacy
combined lookup count.

## Complete correspondence

Legacy, candidate and repeated candidate match bit-exactly for adaptive
schedules/attempts, macro-fixed runs, private physical diagnostics,
contact/onset, aggregate and energy budgets, canonical publications,
trajectory/ledger/query roots and retention receipts. Candidate index/work
receipts repeat for every lane.

The forced two-macro rollback builds one P1 index, uses it for all `540`
workspaces, reproduces legacy prefix/failure/roots and ends with zero retained
and total live workspaces.

## Decision

Select `COMPLETE_LANE_IMMUTABLE_STATIC_SUPPORT_INDEX_CANDIDATE`. This completes
only static-index packaging and authorizes B4C4C flat-only CSR design. The
reported gate wall time is not an A/B whole-solver speedup. B4D, nominal corpus,
CUDA, runtime and production remain blocked.
