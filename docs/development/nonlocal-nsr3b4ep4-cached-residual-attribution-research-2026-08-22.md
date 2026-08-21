# NSR3-B4EP4 cached residual-attribution research -- 2026-08-22

Status: `COMPLETE / EXACT_OUTPUT_GPROF_SELECTED / NO_OPTIMIZATION_YET`

## Question

After B4EP3I reduces the exact work-only nominal Hydro macro from median
16.71 s to 10.40 s, which remaining serial stage should own the next bounded
optimization?

## Competing hypotheses

The physical work remains 42 attempted substeps, 221 outer trials, 411
nonlinear HVPs, 48 spectral HVPs and 226 state queries. B4EP3I changes only
topology discovery: one superset build plus 225 certified reuses replaces 226
canonical cell builds. Four explanations remain falsifiable:

1. **HVP traversal is now dominant.** All 459 HVPs still traverse the exact
   active pressure tape and perform state-vector work.
2. **Filtered workspace refresh remains dominant.** Every query still filters
   the superset, materializes exact flat CSR, evaluates density/energy and
   rebuilds the pressure tape.
3. **Evaluation/tape, rather than filtering/CSR, owns workspace cost.** The
   topology cache cannot reuse state-dependent radii, density, compression or
   Hessian coefficients.
4. **Nonlinear bookkeeping is material.** 221 serial trials, trust-region
   controls, ownership and publication may leave no single data kernel clearly
   dominant even after the first two mechanical optimizations.

Counts cannot rank these paths. The B4EP2 profile predates the cache and must
not be arithmetically subtracted from new Release time.

## Measurement design

Use one external GCC 15.2 `-O3 -DNDEBUG -g -pg` build of implementation
`65ce739e17def07266f0d2f72b00178451a1fb3e`. Run only:

```text
nonlocal-formula-reclosure --nominal-hydro-cached-topology-ablation
```

The instrumented stdout must remain byte-identical to B4EP3I before any
sample is admitted. Linux performance events remain unavailable under the
unchanged host policy; no sysctl is modified. Instrumented wall time is only a
completion diagnostic and is never compared with Release throughput.

Attribute top-level inclusive CPU separately to exact HVP application,
complete cached workspace construction and residual nonlinear bookkeeping.
Within workspace construction, split superset/filter/CSR from evaluation/tape
using call-graph children and exact call counts. Optimized or folded symbol
names are not interpreted literally when caller/child structure contradicts
the displayed name.

## Frozen selection rule

- If one top-level category is at least `1.20x` the runner-up, select only its
  next mechanical design.
- If complete workspace wins, apply the same `1.20x` rule between
  filter/CSR and evaluation/tape.
- If no leader clears the ratio, or symbol attribution cannot separate the
  paths, select scoped internal phase timing instead of an optimization.
- Solver-policy changes, parallelism, GPU work and B4E2 remain forbidden in
  this discriminator.

## Decision

Freeze B4EP4 as one exact-output cached gprof run. It may authorize one B4EP5
design only; it makes no source, formula, solver-policy or production change.
