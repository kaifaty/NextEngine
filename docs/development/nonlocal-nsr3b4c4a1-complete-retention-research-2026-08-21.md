# B4C4A1 complete-lane retained-workspace design

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Why this is a separate stage

B4C4A proves exact ownership on one P1/P2 macro transaction. Complete adaptive
replay changes committed state at every macro boundary, while fixed reference
lanes repeat the same ownership over `48/96/192` substeps per frame. Applying
the optimization there without an independent gate would turn local evidence
into an untested trajectory claim.

B4C4A1 changes no ownership rule. It only composes the selected one-substep
protocol over the already selected complete lanes:

```text
adaptive macro replay: P1 8 frames, P2 16 frames
macro-fixed reference: P1/P2 at 48, 96 and 192 substeps per frame
```

The per-substep canonical lane is not selected as the reference and remains
out of scope.

## Predeclared work deltas

Every completed private substep owns exactly one retained diagnostic read.
Existing schedules therefore determine the removed builds before execution:

| Lane | P1 removed builds | P2 removed builds |
|---|---:|---:|
| adaptive | `444` | `124` |
| fixed 48 | `384` | `768` |
| fixed 96 | `768` | `1,536` |
| fixed 192 | `1,536` | `3,072` |

Candidate build totals are measured, not guessed. For each lane, legacy minus
candidate builds must equal the table; transfers, reads, releases and receipt
sequence must equal the same value. Maximum retained ownership is one and all
final live counts are zero.

## Correspondence

Candidate and legacy runs must match bit-exactly in schedules, attempts,
accepted/discarded work, private physical diagnostics, contact/onset state,
published position/velocity, per-frame canonical roots, ledger entries,
trajectory roots and aggregate physical budgets. The adaptive candidate must
also preserve the selected B4C3MAR roots, while the combined accuracy parent
remains exact through B4C4A.

Each complete lane receives a deterministic aggregate receipt. Query-policy
roots intentionally change and must repeat separately from the preserved
legacy transcript.

## Failure composition

Repeat the existing adaptive forced-prepublication rollback with retained
ownership. It must finish the private selection, publish nothing, preserve the
committed prefix and roots, and leave zero retained and total live workspaces.

## Decision

Freeze the
[B4C4A1 contract](../plans/nonlocal-nonlinear-solver-research/03b4c4a1-complete-retention-contract.md).
A PASS completes only the retained-workspace portion of packaging. B4C4B
immutable static-support indexing and B4C4C flat-only CSR remain mandatory and
separate. B4D, nominal corpus, CUDA, runtime and production remain blocked.
