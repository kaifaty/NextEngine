# B4C4B immutable static-support-index research

Status: `PASS / TIMING DISCRIMINATOR REQUIRED`

Date: `2026-08-21`

## Question

Can the immutable boundary-support portion of every joint-neighborhood build
be canonicalized and cell-sorted once per transaction without changing pair
order, floating-point arithmetic, solver state or durable roots?

B4C4A1 is the selected parent. Its retained candidate still builds `264/9`
P1/P2 workspaces in the isolated one-macro controls. Each build repeats the
same `544/1,216` support points even though only fluid positions change.

## Current algorithm

The legacy builder canonicalizes fluid and support independently, assigns
participants as `[0,F)` and `[F,F+S)`, creates one combined cell-record array,
sorts by `(cell_x, cell_y, cell_z, participant)` and searches its cell ranges.

For any cell, participant numbering proves this exact order:

```text
fluid records in canonical fluid-id order
support records in canonical support-id order
```

All fluid participants are smaller than every support participant. Therefore
a split index can reproduce the legacy callback sequence by visiting the
dynamic fluid range first and the immutable support range second for each of
the same 27 cells. Fluid self/lower-triangle rejection remains before distance
counting; every support record is then tested in the same canonical order.

Pairs, rows and pressure-tape records are sorted again downstream, but the
candidate does not rely on that fact: pre-sort discovery order must also be
exact so capacity failures occur at the same first pair.

## Candidate ownership

Build one research-only `JointStaticSupportIndex` from the fixture boundary at
transaction entry. It owns:

- canonical support points;
- support-local cell records and ranges;
- horizon/format identity and a hash over exact point ids and binary64 bits;
- bounded build, canonicalization, record and range counters.

Every workspace continues to own a canonical support copy in its existing
`JointNeighborhood`. This deliberately leaves point-copy/layout optimization
out of B4C4B. Only canonicalization, cell-coordinate construction and support
sorting/range construction are reused.

Workspace construction receives the immutable index plus its expected
identity. An absent, failed, mismatched or structurally invalid index fails
before pair publication. Changing support geometry requires an explicit new
index; a one-bit geometry mutation must change identity, reject the stale
index and pass after rebuild.

## Predeclared work effect

The isolated discriminator composes with B4C4A retention:

| Measurement | P1 legacy | P1 candidate | P2 legacy | P2 candidate |
|---|---:|---:|---:|---:|
| workspace/neighborhood builds | `264` | `264` | `9` | `9` |
| static-index builds | `264` | `1` | `9` | `1` |
| support records admitted to sorting | `143,616` | `544` | `10,944` | `1,216` |
| dynamic fluid records admitted to sorting | `12,672` | `12,672` | `243` | `243` |
| total records admitted to sorting | `156,288` | `13,216` | `11,187` | `1,459` |

Thus exact removed support records are `143,072` P1 and `9,728` P2. The
candidate performs a second range lookup for each visited cell. Distance-test,
pair, directed-record and workspace-build counts must remain exact, so this is
a threshold-free structural result rather than a wall-time claim.

## Rejected combinations

- General state memoization does not address P1's 195 distinct trial states.
- Merging cached support records with dynamic records and sorting the combined
  array still repeats the dominant sort input.
- Hash-table cell storage introduces iteration/order and allocation variables
  that are unnecessary for this discriminator.
- Eliminating the per-workspace support copy or nested adjacency rows belongs
  to the separate B4C4C layout stage.
- A mutable global cache would hide ownership and invalidation; the index is an
  explicit transaction-scoped immutable input.

## Decision

The frozen
[B4C4B contract](../plans/nonlocal-nonlinear-solver-research/03b4c4b-static-support-index-contract.md)
passes exact one-macro correspondence and every invalidation control; see the
[dated evidence](nonlocal-nsr3b4c4b-static-support-index-evidence-2026-08-21.md).
The candidate removes `91.54%/86.96%` of P1/P2 sort-record admissions but
doubles cell-range lookups. Freeze a threshold-free timing discriminator
before complete-lane application. B4C4C, B4D, nominal corpus, CUDA, runtime and
production remain blocked.
