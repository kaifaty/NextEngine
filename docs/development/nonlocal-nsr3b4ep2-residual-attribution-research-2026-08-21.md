# NSR3-B4EP2 residual cost-attribution research -- 2026-08-21

Status: `COMPLETE / WORK_ONLY_GPROF_SELECTED / NO_OPTIMIZATION_YET`

## Question

After B4EP1 removes transient full-state hashes and reduces one nominal Hydro
macro from 48.74 s to 16.15 s, which remaining serial stage should own the
next controlled optimization?

## Residual work facts

B4EP1 changes no solver work. One work-only transaction still performs:

- 42 attempted and 28 accepted private substeps;
- 221 outer trust-region trials;
- 226 neighborhood/evaluation/tape workspace builds plus one full parent;
- 411 nonlinear HVPs and 48 spectral HVPs;
- 151,461,068 directed flat adjacency records materialized across all builds.

The elapsed-time change proves that evidence serialization was separable, but
counts alone still cannot rank the residual costs. Four hypotheses remain:

1. **HVP traversal.** The nonlinear and spectral paths execute 459 exact HVPs,
   with vector construction and repeated active-pair traversal.
2. **Topology/CSR construction.** Each current/trial state rebuilds cells,
   pairs and flat adjacency, even when particle motion may remain inside a
   reusable superset neighborhood.
3. **Evaluation/tape refresh.** Density/gradient evaluation and the pressure
   tape are rebuilt with every workspace; topology reuse alone would not
   remove this state-dependent work.
4. **Nonlinear-control overhead.** 221 serial trials can keep the macro slow
   even if individual kernels are locally efficient. Changing trial count,
   warm starts or globalization would be a mathematical-policy experiment,
   not a mechanical optimization.

## Measurement design

Process-local Linux performance events remain unavailable under
`perf_event_paranoid=4`; the host policy stays unchanged. Reuse GNU gprof 2.46
with a separate GCC 15.2 `-O3 -DNDEBUG -g -pg` build of the exact B4EP1
implementation. Run only:

```text
nonlocal-formula-reclosure --nominal-hydro-query-evidence-ablation
```

Require stdout to equal the frozen 5,780-byte B4EP1 candidate exactly before
using any samples. Keep instrumented wall time descriptive only. Generate
flat and call-graph artifacts and attribute both leaf self time and inclusive
call-graph time. Optimized/folded symbol names must be interpreted using call
counts and child structure, as already required by B4EP0.

## Routing rule

- HVP dominant: design a deterministic allocation/reduction or cached-input
  HVP ablation before parallelism;
- topology/CSR dominant: design exact conservative superset/Verlet topology
  with filtered current pairs and an explicit rebuild certificate;
- evaluation/tape dominant: design in-place state refresh over exact current
  pairs while preserving every result bit/root;
- no clear kernel dominant or control overhead dominant: add scoped internal
  phase timing before any solver-policy change.

Select exactly one next experiment from observed residual attribution. A
correspondence failure or empty profile selects nothing. No B4E2, CUDA,
runtime or production work is authorized by this profile.

## Decision

Freeze B4EP2 as a work-only residual gprof run. This stage makes no source or
solver change. Its only possible credit is one B4EP3 optimization design.
