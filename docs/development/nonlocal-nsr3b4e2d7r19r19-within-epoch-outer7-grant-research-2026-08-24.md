# NSR3-B4E2D7R19R19 within-epoch outer-7 grant research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / WITHIN-EPOCH GRANT SELECTED / IMPLEMENTATION NEXT`

## Question

How should the exact, non-admissible R18 outer-6 result authorize outer 7
without incorrectly resetting its still-active budget epoch?

## Decision

Issue a new one-use outer grant while preserving epoch 1 and every resource
counter. R18 consumed its active resume owner, so the exact R18 state receives
a separate trusted source owner. A copy-on-write transition consumes that
source owner and creates a grant, receipt and unconsumed outer owner.

```text
R18 state (not admissible)
epoch 1, slice/cumulative 50/573
              |
              v
consume R18 state owner once
              |
              v
outer-7 grant, still epoch 1, still 50/573
```

Creating epoch 2 or resetting slice HVP here is rejected: the epoch-1 soft
slice has consumed only 50 of 512 HVP. Epoch transition is a slice-exhaustion
operation, not an outer-update operation.

## Bound source

R19 binds exact R18 state/receipt/history roots
`5ad2f99d...07bc`, `5a029f24...8a64`, `a36fa9c0...a5b1`, the complete used
ledger `7,2,25,1,50,573,37,23,552,21,2,23,0`, and the non-admissible outer-6
facts:

```text
primal bits       0x3e5afe281c000000
stationarity bits 0x3d5eabe1f7af94f6
admissible        false
next outer        7
```

## Canonical objects

All formats are private fixed-order little-endian schema-1 objects with raw
32-byte digests and no native padding:

```text
source owner  NEALROW1  44 / 60 bytes
outer grant   NEALOGT1 276 / 292 bytes
receipt       NEALOGR1 116 / 132 bytes
active owner  NEALOOW1 108 / 124 bytes
owner state   NEALOGS1 164 / 180 bytes
```

Exact roots are frozen in the contract. The grant includes R18 state, resume
receipt and successor-history roots, epoch/next-outer, all limits and all used
counters. The receipt additionally binds the frozen within-epoch policy root.

## Atomicity and controls

The transition validates R18 first, prepares all objects locally, decodes and
re-encodes them, then replaces one private owner state. Source/owner/
duplicate/stale, epoch/slice/cumulative/context drift, candidate corruption
and injected abort controls all fail with exact rollback. Duplicate replay
after success is idempotent and executes no work.

## Authority boundary

R19 performs metadata encoding/hashing only. It may issue one private outer-7
grant, but may not consume that new grant, execute outer 7, reset an epoch,
run another substep/macro/trajectory/timing, mutate world/public state or
change live budget/production policy.

The executable gate is frozen by the
[D7R19R19 contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r19-within-epoch-outer7-grant-contract.md).
