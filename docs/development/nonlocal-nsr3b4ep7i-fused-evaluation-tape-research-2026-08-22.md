# NSR3-B4EP7I fused evaluation/tape research -- 2026-08-22

Status: `COMPLETE / EXACT_FUSION_AB_SELECTED / NO_IMPLEMENTATION_YET`

## Selected mechanism

B4EP7D proves that the B4EP5 transaction repeats 217,703,380 radius and
131,987,230 gradient-kernel evaluations across evaluation and tape creation.
The narrow candidate is a transaction-only fused workspace builder for the
already selected flat-CSR/topology/coefficient path.

The pair pass retains canonical `neighborhood.pairs` order. For each pair it:

1. computes and stores radius;
2. performs the unchanged density contribution and endpoint additions;
3. computes and stores the already selected gradient and second coefficients.

After every density is complete, the center pass retains center and flat-CSR
slot order. It computes compression once, performs unchanged energy/gradient
accumulation from stored binary64 radius/gradient, and writes the identical
compression into the pressure tape. Only then is the validated flat CSR moved
to the tape.

## Exactness boundary

The candidate must not:

- reorder pair, center, adjacency or endpoint accumulation;
- store/reuse density weights or vector pair contributions;
- change multiplication/division grouping in the gradient expression;
- cache displacement, normal or Hessian matrices;
- change active-center branching, tolerances or HVP schedule;
- fall back to the old builder after opting into fusion.

Pure kernel evaluations may occur earlier relative to unrelated pairs, but
their arguments and binary64 results are identical. Strict FP flags remain
`-ffp-contract=off -fno-fast-math`.

## Ownership and failure

The full-state parent and every default/old command stay unchanged. Fusion is
enabled only through the internal transaction trace. It accepts only valid
flat adjacency, enforces the existing capacities, allocates the same final
tape arrays and fails closed before publication. The work-only query-chain
payload must remain byte-identical to B4EP5.

## Gate

Require two independent Release builds, two byte-identical fused reports,
exact physical/work roots, exact frozen fused counters, and byte-identical
B4EP1/B4EP3/B4EP3I/B4EP5/B4EP7D reports. Only then run three balanced process
pairs against B4EP5. All three must win and median paired speedup must be at
least `1.10x`.

## Decision

Freeze one B4EP7I implementation/A-B. PASS selects the exact fused workspace
candidate and authorizes only B4EP8 residual profiling/design. Failure keeps
B4EP7D and forbids stacking another optimization.
