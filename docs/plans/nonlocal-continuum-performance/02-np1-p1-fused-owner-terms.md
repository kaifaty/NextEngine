# NP1-P1 — fused owner-term traversal

Status: `COMPLETE / RETAINED / P2_INPUT / EXACT_WORK / REPORT_ONLY`

## Hypothesis

`fused-owner-terms-p1` removes redundant directed-CSR reads across compatible
post-density terms while preserving the retained owner-only arithmetic. It
should improve pair-stage p95 by at least `10%` or total exact-50k p95 by at
least `5%` without regressing coherent or advected total p95 by more than `2%`.
P1 and later P2 together must satisfy HP-2's stronger `15%` pair / `10%` total
threshold.

## Frozen identity

P1 changes only traversal. Its complete identity is:

```text
nuv-gather-directed-r0
+ pointer-swap-o1
+ nuv-terms-specialized-o2
+ stable-sample-v0
+ fused-owner-terms-p1
```

The retained NP0 stack remains selectable as rollback. No iteration count,
neighbor membership/order, storage order, coefficient, tolerance, accepted
iterate or f32 mode changes.

## Arithmetic association

Density remains a separate kernel and global barrier because
incompressibility consumes the complete density field. After density, one
thread owns one sample and visits its directed CSR row exactly once.

For every neighbor slot, it evaluates enabled terms in this fixed order:

1. incompressibility;
2. bulk/shear viscosity;
3. surface tension.

Each term keeps independent f32 source and matrix accumulators. Within a term,
slot order, local/incoming endpoint order and every add are copied from the
retained specialized owner kernel. After the row finishes, accumulators are
committed to global source/matrix in the same term order above. Algebraic
factoring across terms, multiplying duplicated endpoints by two, FMA changes,
atomics, unique-pair fragments and reduction-tree changes are forbidden.

The candidate is instantiated at compile time for the active masks used by
the frozen profiles:

- incompressibility + bulk viscosity;
- incompressibility + bulk/shear viscosity;
- incompressibility + surface tension.

Any future mask requires a new instantiation and the same oracle gate. A
single active post-density term continues to use the retained specialized
kernel because fusion would remove no traversal.

## Timing and capacity

The report exposes `fused_owner_terms` as its own stage. Pair-stage comparison
is:

```text
retained = incompressibility + viscosity + surface_tension
candidate = fused_owner_terms
```

No new per-edge or per-sample device allocation is allowed. Register/local
spill is measured through compile diagnostics and wall time; a spill-driven
regression rejects the profile even if launch count falls.

Both identities receive the NP0 256-execution conditioning window, then run
32 formal warm-up and 96 measured alternating rounds in one process. Dynamic
instances advance the same frozen 32-step trace in lockstep and reset only at
epoch boundaries. Setup, capture and report hashing remain outside timing.

## Correctness gate

Before adjacent timing:

1. retained CPU/CUDA self-tests pass;
2. P1 passes every independent tiny CPU f64 gather fixture;
3. stiff surface `gamma=1000`, i2 is exact to the retained GPU output/CSR;
4. coherent, permuted and every advected trace step have exact ordered output,
   active logical CSR, handoff state and capacity correspondence;
5. finite, symmetric topology and normalized momentum bounds pass;
6. v0 profile/input/output/CSR hashes remain unchanged.

Any repeated association mismatch stops P1. Tolerance widening or promotion
to an algorithm identity is not allowed for this exact-work candidate.

## Retention

Retain P1 only if all correctness/capacity gates pass and either its pair-stage
p95 improves by at least `10%` or exact-50k total p95 improves by at least
`5%`, while neither coherent nor advected total regresses by more than `2%`.
Otherwise record the negative result, keep the retained NP0 stack and proceed
to P2 only if the roadmap's consecutive-low-gain stop rule still permits it.

P1 passed this gate on 2026-08-20 and is retained. See the
[dated evidence](../../development/nonlocal-continuum-np1-p1-evidence-2026-08-20.md).
