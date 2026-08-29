# B4C4C1 complete-lane flat-adjacency evidence

Status: `PASS / B4C4 PACKAGING COMPLETE`

Date: `2026-08-21`

## Result

All eight complete adaptive and macro-fixed P1/P2 lanes pass with one
immutable support index per lane, retained accepted workspaces and one
flat-CSR-to-tape ownership transfer per workspace. Legacy nested and candidate
flat variants preserve every physical, schedule, recovery, topology,
canonical publication, ledger, query and durable-root value exactly.

Forced prepublication rollback also passes with exact committed prefix and
cumulative totals. It performs `540` candidate transfers and ends with zero
retained and workspace owners.

## Deterministic evidence

Two probe reports are byte-identical:

```text
raw JSON with final LF     cd2d5279776c4a1dce2c98038e3ca8a13bc9d1defba4685a95abb1a71d33882f
JSON without final LF      b9aa613f1b1c096b103aff1163c4079ae7e82b3b9c70970c028d1d4e618318b7
wall / CPU / peak RSS      58.76 s / 125% / 9,420 KiB
                            58.95 s / 125% / 9,324 KiB
```

Two full reports including the B4C4CM timing parent are byte-identical:

```text
raw JSON with final LF     50bf779b3bcd43ab4cda4dc7defcc00bc0432faafbaef44b2dc1c57123de5e60
JSON without final LF      2f9a510099f4f1e9eb03667b707e6802d6f69052629842143799dd8676728e8a
semantic result            b4d5260012f4208026b814411891a221dda2d5823b242027d890d4066e69550c
wall / CPU / peak RSS      60.65 s / 124% / 9,492 KiB
                            60.61 s / 124% / 9,592 KiB
```

## Complete-lane work

| Lane | Workspaces/transfers | Removed rows | Directed records built once instead of twice |
|---|---:|---:|---:|
| P1 adaptive | `1,924` | `92,352` | `8,798,828` |
| P1 fixed 48 | `1,557` | `74,736` | `7,116,685` |
| P1 fixed 96 | `2,937` | `140,976` | `13,416,146` |
| P1 fixed 192 | `4,631` | `222,288` | `21,176,919` |
| P2 adaptive | `323` | `8,721` | `397,753` |
| P2 fixed 48 | `900` | `24,300` | `1,044,482` |
| P2 fixed 96 | `1,751` | `47,277` | `2,036,185` |
| P2 fixed 192 | `3,449` | `93,123` | `3,986,244` |

All predeclared workspace, row, offset and transfer counts are exact.
Candidate nested/tape-reconstruction counters are zero. Final tape directed
counts equal legacy for every lane.

Across the four executed P1 lane comparisons, duplicate construction removed
`50,508,578` `u32` writes (`202,034,312` cumulative bytes) and `1,060,704`
row-sort calls. Across P2 it removed `7,464,664` writes (`29,858,656` bytes)
and `346,842` row-sort calls. These are cumulative gate work totals, not peak
memory or a production workload estimate.

Rollback records `2,501,252` directed entries, `25,920` removed rows,
`26,460` offsets and exactly `540` successful transfers. Its static index
still builds once and sorts only 544 support records.

## Decision

Select `COMPLETE_LANE_FLAT_ADJACENCY_CANDIDATE`. B4C4 packaging is complete:
retention, immutable static support and single-owner flat adjacency all pass
their isolated timing, complete-lane and rollback gates. B4D may now re-attest
its frozen reference inputs. This does not yet authorize nominal corpus,
runtime/schema, CUDA or production integration.
