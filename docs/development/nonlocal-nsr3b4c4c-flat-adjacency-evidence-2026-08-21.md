# B4C4C flat-adjacency evidence

Status: `PASS / ONE_MACRO CANDIDATE SELECTED`

Date: `2026-08-21`

## Result

The frozen B4C4C discriminator passes P1/P2 legacy, candidate and repeated
candidate one-macro transactions. The candidate constructs canonical CSR pair
indices once, evaluates through them and transfers their allocations into the
pressure tape. It publishes the exact legacy tape and leaves the neighborhood
with no adjacency owner.

Every row, density, gradient, untaped HVP, taped HVP, KKT choice, physical
diagnostic, adaptive schedule, canonical publication, ledger and durable root
is bit-exact. All malformed-CSR, capacity, failed-transfer, repeated-transfer
and permutation controls pass.

## Deterministic evidence

Two isolated reports are byte-identical:

```text
raw JSON with final LF     ac4b6a74bf596ac3d4de667b1cb1da9ac3cdd02989bcb916fa3cfb0234683537
JSON without final LF      45f340a34f2e970179bf30358c126dc1ea30302c8bcca70c1c190715836f5a7b
semantic result            22007ffd36f7189ea4aca544c93f7a0b3193ec6962438c626be0a03179dabb8f
wall / CPU / peak RSS      1.41 s / 102% / 7,468 KiB
                            1.40 s / 102% / 7,408 KiB
```

Two full reports, each including the complete B4C4B1 parent, are also
byte-identical:

```text
raw JSON with final LF     2559f1c8daa11b4eb070b70bf7a89a20c91711fb48b7c54b66c288cd2d39d425
JSON without final LF      fe9600e57754cfb5e7f630f5326a30c9959114c4a569562a7121d2a49ce31493
semantic result            22007ffd36f7189ea4aca544c93f7a0b3193ec6962438c626be0a03179dabb8f
wall / CPU / peak RSS      62.85 s / 125% / 10,180 KiB
                            62.96 s / 124% / 9,288 KiB
```

The historical B4C4B isolated regression remains exact at
`187cff865ea739f96c1440f3041e7fd1f6de1a8c95436ae810f604850d80c90e`
without the final LF and semantic
`40f181c646d2bbc208a46915f09ae9c49dfcd29f22f7969b21940e26f6e2720b`.

## Exact work result

| Counter | P1 legacy | P1 candidate | P2 legacy | P2 candidate |
|---|---:|---:|---:|---:|
| workspaces | `264` | `264` | `9` | `9` |
| nested row objects | `12,672` | `0` | `243` | `0` |
| nested participant records | `1,223,663` | `0` | `9,623` | `0` |
| nested row sorts | `12,672` | `0` | `243` | `0` |
| flat offsets built | `0` | `12,936` | `0` | `252` |
| flat pair-index records built | `0` | `1,223,663` | `0` | `9,623` |
| duplicate tape offsets built | `12,936` | `0` | `252` | `0` |
| duplicate tape pair indices built | `1,223,663` | `0` | `9,623` | `0` |
| duplicate tape row sorts | `12,672` | `0` | `243` | `0` |
| CSR ownership transfers | `0` | `264` | `0` | `9` |

Thus the candidate removes one complete directed-record construction:
`4,894,652` P1 bytes and `38,492` P2 bytes of cumulative `u32` record writes
over the isolated transactions, plus nested allocation metadata and both
row-sort passes. This is cumulative construction traffic, not a peak-memory
or whole-solver speed claim.

## Failure controls

Offset count/start/monotonicity, out-of-range/duplicate/unrelated/unordered
pair indices, directed capacity and payload capacity all fail with an empty
tape, unchanged source arrays and zero transfers. A valid source transfers
exactly once; reuse fails. Canonical permutation reproduces evaluation and
tape exactly.

## Decision

Select `ONE_MACRO_FLAT_ADJACENCY_TRANSFER_CANDIDATE`. Freeze an interleaved
construction timing discriminator and a separate complete-lane ownership
rollout before selecting the representation beyond the isolated corpus. B4D,
nominal corpus, CUDA, runtime/schema and production remain blocked.
