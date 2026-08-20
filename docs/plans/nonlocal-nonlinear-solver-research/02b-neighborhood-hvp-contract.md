# NSR2-B -- deterministic neighborhood objective/HVP correspondence

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

Selected solver inputs: unpreconditioned full-HVP trust-region Newton-CG and
the NSR2-A1 scale-aware stopping criterion. The block preconditioner is
rejected evidence and is not part of this stage.

## Purpose

The current oracle evaluates every particle pair. NSR2-B introduces the first
scalable data structure but makes no timing claim. It must reproduce the exact
all-pairs objective, gradient and Hessian-vector product before the trust solver
may use it.

## Frozen pair structure

- Cell edge is `max(horizon, 3*spacing)`.
- Cell coordinates are mathematical `floor(position/cell_edge)` signed
  integers; nonfinite or integer-overflow input fails before allocation.
- Sort cell records by `(cell_x, cell_y, cell_z, sample_index)`.
- For each canonical sample index, inspect exactly the 27 lexicographically
  ordered adjacent cells and retain `j>i` only when distance is inside the
  requested support.
- Sort final unique pairs by `(i,j)` and reject a duplicate or missing endpoint.
- Build current-position pairs for density/pressure/surface and independent
  reference-position pairs for viscosity.
- Per-center pressure adjacency is reconstructed from the unique current-pair
  list and sorted by neighbor index.

The reference all-pairs loops and neighborhood loops therefore accumulate in
the same `(i,j)` or `(center,neighbor)` order. Hash-map iteration, worker order
and pointer address are forbidden ordering inputs.

## Controls

Use:

1. NSR0 compressed, combined, repulsive-surface and attractive-surface
   fixtures with their canonical directions;
2. NSR2-A1 8/27/64 lattices with one deterministic direction
   `v_i = normalize((0.31+0.01*i, -0.27+0.02*(i mod 5), 0.11-0.015*(i mod 7)))`.

At each predicted point compare:

- complete pair lists against an independently enumerated all-pairs support;
- total/inertia/pressure/viscosity/surface objective components bit-for-bit;
- every gradient component bit-for-bit;
- density extrema and active count bit-for-bit;
- every analytic HVP component bit-for-bit;
- internal momentum residual bit-for-bit;
- pair-list SHA-256 across two builds and two process executions.

## Gates and selection

Every comparison is exact binary64 equality; finite/capacity errors fail before
report publication. Two full runs must be byte-identical, and all historical
NSR/FCR hashes remain unchanged.

- PASS selects `canonical-cell-neighborhood-v1` and authorizes NSR2-C
  neighborhood trust-solver scaling.
- One implementation-defect repair may rerun the same contract. A second
  mismatch selects `NSR_STOP` because performance on a different operator is
  irrelevant.
- No GPU, thread parallelism or preconditioner is introduced here.

