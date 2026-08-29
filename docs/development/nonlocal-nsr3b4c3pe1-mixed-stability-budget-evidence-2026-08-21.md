# NSR3-B4C3PE1 mixed stability budget evidence

Status: `PASS / MACRO_BOUNDARY_CANONICAL_FIXED_REFERENCE_CANDIDATE`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --mixed-stability-budget-self-test
```

Two parent-gated reports executed concurrently and are byte-identical:

```text
status                 PASS / REFERENCE_RESEARCH_ONLY
raw JSON + LF          bd48c77938873e54a51f9832780745ffc70d28d4ade32ab59b6b24a490f834ed
raw JSON without LF    eb4d82300653d779baf00620cb83a2526d164347c1b97a65b487f1955a3b8d60
semantic result        830c1613a65e92954744b104818a63f7be1f6e8f6fa3bf9580009a83265d5d48
wall time              65.65 s / 65.71 s
CPU utilization        302% / 305%
maximum RSS            13,428 KiB / 13,496 KiB
parent B4C3PE exact     true
```

The isolated probe also passes:

```text
raw JSON + LF          65fa1d36eb426860ccddfda5de2098cf2d5dbb28c1080b47b187b4b5164dec1a
raw JSON without LF    12e25ef1f0f396caf680405d2101ae82c57f3fa75e1b5d27dd47c872c10f4b5e
semantic result        830c1613a65e92954744b104818a63f7be1f6e8f6fa3bf9580009a83265d5d48
wall time              21.71 s
CPU utilization        299%
maximum RSS            12,116 KiB
```

Refactoring the old physical gate did not alter its serialized report. The
B4C3PE parent retains raw-without-LF SHA-256
`aebe7fbe218b507ae0ca8ebe7fde5ecafc38b5ec66fc894d669043649c51ea51`
and semantic SHA-256
`584db48c73b11131db26c5a76758111bd47a7569d85599936fb007951d089395`.

## Admission coverage

Every one of 144 position/velocity frame fields selects exactly one branch;
none is rejected:

| Case | Field | Temporal | Absolute | Rejected |
|---|---|---:|---:|---:|
| P1 supported | position | 19 | 5 | 0 |
| P1 supported | velocity | 24 | 0 | 0 |
| P2 released | position | 43 | 5 | 0 |
| P2 released | velocity | 3 | 45 | 0 |
| **Total** | both | **89** | **55** | **0** |

The largest utilization of a selected temporal branch is `0.976902`
(P1 position, 48 substeps, frame 1). The largest utilization of a selected
absolute branch is `0.045817` (P2 position, 192 substeps, frame 4). Thus the
result is not created by a marginal absolute fallback. All raw temporal and
absolute utilizations remain present in the report, including the unselected
branch.

## Unchanged evidence

- P1 canonical final convergence ratios remain `2.016654` for position and
  `2.115541` for velocity.
- P2 ratios remain `1.992165` and `2.167876`.
- Binary references, all six private solver transactions, non-tube physical
  gates, exact contact time/sets, canonical geometry, macro ledgers and roots
  pass.
- Forced prepublication failure preserves state, step count, trajectory root,
  legacy/policy ledger roots and cumulative totals.
- The obsolete `32*P*q` tube remains visible: legacy B4C3P still fails at
  P1/192 and was not silently reclassified.

## Decision

Select `MACRO_BOUNDARY_CANONICAL_FIXED_REFERENCE_CANDIDATE` for the next
research boundary. This is a validated fixed-reference representation policy,
not a runtime or production solver.

Authorize design of an adaptive macro-boundary transaction that keeps all
trial substeps and comparisons private in binary64 and publishes exactly once
after accepting a macro frame. Do not resume per-substep canonical publication,
do not use the obsolete tube as an admission gate, and do not authorize B4C3TC,
nominal-corpus, CUDA, runtime/schema or production work yet.
