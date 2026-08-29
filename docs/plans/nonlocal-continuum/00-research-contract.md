# NR0 — Nonlocal continuum research contract

Status: `EXECUTED / NONLOCAL_48K_RECLOSURE_CANDIDATE / REPORT_ONLY / NO_W2_CREDIT`

## Outcome

Determine whether the published Nonlocal/SISSM formulation deserves a new,
separately rooted Next Engine continuum profile. The experiment must distinguish
mathematical reproduction, semantics-preserving GPU engineering and changes to
the nonlinear algorithm. It must stop without integration when the measured
evidence does not justify the next stage.

This document selects an experiment, not a production solver. SPEC-38's current
candidate authority and ADR-076's solver-family decision remain in force.

## Falsifiable hypotheses

| ID | Hypothesis | Falsifier | Smallest discriminator |
|---|---|---|---|
| `HN-1` | The published pair terms and one SISSM update can be reproduced independently | any sign, symmetry, finite-value or tolerance failure in a tiny exact fixture | CPU `f64` pair and one-iteration oracle |
| `HN-2` | The source-shaped baseline is dominated by repeated density/pair traversal, atomics or state copies rather than launch overhead alone | profiler attributes no material share to those stages | NR1 stage timing and one profiler capture |
| `HN-3` | Persistent buffers, term specialization and a better accumulation layout can provide at least `2.0x` fixed-iteration speedup | optimized fixed-iteration geometric-mean speedup is below `2.0x` on both declared workloads | adjacent `source-atomic-v0`/NR2 reports on the water-48k and viscous-16k profiles where that denominator passed correctness |
| `HN-4` | Clean-water 48k can approach the existing product scale | optimized fixed five-iteration p95 including neighbor construction remains above `8 ms` | 48k water feasibility window |
| `HN-5` | If 48k fails, Nonlocal can still be useful as a local high-fidelity material domain | neither declared 16k material profile fits `8 ms` without correctness failure | 16k water and viscous/surface profiles |

No hypothesis asserts that Nonlocal already supports plasticity, thermal state,
phase change or solids. Those are future constitutive-model questions.

## Frozen research inputs

| Input | Exact research identity |
|---|---|
| Primary paper | DOI `10.1145/3799902.3811196`; inspected PDF SHA-256 `610047ef32e895026c3661c57999f14ae550d8e2719ba2fcd95742371ad031c1` |
| Reference implementation | PeriDyno commit `1aa892bb296fe766d2f9249c881b8605af23a69b` |
| License | PeriDyno repository default `Apache-2.0`; paper marked `CC BY 4.0` |
| Active host | Linux x86_64; NVIDIA GeForce RTX 3080, compute capability `8.6` |
| Existing comparison | W2 direct-port report: APG40 p95 `37.398 ms` `f64`, `18.015 ms` mixed; not an equivalent algorithm |
| Product scale | nominal `48,000`, hard `50,000` water samples; `100,000` remains stress/report-only |

The paper PDF, external checkout, build tree, binaries and raw reports remain
outside Git. The report records their hashes and exact commands. External code
is inspected before execution and is never made a transitive project dependency.

## Research profiles

All profiles use fixed particle count, fixed time step, fixed neighbor horizon,
fixed material coefficients and fixed iteration count. Exact constants are
emitted in the NR1 machine-readable report before timings are accepted.

| Profile ID | Lattice / samples | Active terms | `kappa/lambda/mu/gamma` | Fixed iterations |
|---|---:|---|---|---:|
| `nuv-tiny-oracle.v0` | named `1..=256` fixtures | individually selected and full | per named case | `1..=4` |
| `nuv-water-16k.v0` | `40 x 20 x 20 = 16,000` | incompressibility + bulk viscosity | `1 / 1.5 / 0 / 0` | `5` |
| `nuv-water-48k.v0` | `80 x 40 x 15 = 48,000` | incompressibility + bulk viscosity | `1 / 1.5 / 0 / 0` | `5` |
| `nuv-viscous-16k.v0` | `40 x 20 x 20 = 16,000` | incompressibility + bulk/shear viscosity | `1 / 200 / 1 / 0` | `20` |
| `nuv-surface-16k.v0` | `40 x 20 x 20 = 16,000` | incompressibility + surface tension | `1 / 0.2 / 0 / 1000` | `20` |

Common performance constants are rest density `1000 kg/m^3`, spacing
`0.005 m`, uniform mass `0.000125 kg`, support/horizon `0.015 m`, time step
`0.001 s` and gravity `[0, -9.81, 0] m/s^2`. Samples are generated in stable
lexicographic `(z, y, x)` order as a free-surface rectangular block. The
performance profile has no solid boundary; analytical-boundary behavior is
covered only by named tiny oracle cases so boundary cost cannot be mistaken
for the published fluid-kernel cost. These are Next Engine research fixtures,
not reproductions of a paper figure.

The material coefficients are taken from the paper's Table 1 water, high-
viscosity and high-surface-tension profiles, while sample counts and the
bounded 20-iteration stress schedule are Next Engine research choices. Any
change creates a `v1` profile and cannot be compared as the same workload.

The primary performance gate uses fixed iterations. An adaptive convergence
experiment receives a different `nuv-adaptive-*.v0` identity and cannot improve
the fixed-iteration score by relabelling less work as the same workload.

## Correctness requirements

NR1 and every retained NR2 change must satisfy all applicable checks:

- valid, symmetric neighbor membership for the declared fixture;
- finite energy, density, matrix, source, position and velocity values;
- CPU `f64` pair contribution and one-iteration comparisons under the
  predeclared absolute-plus-relative tolerance;
- equal-and-opposite pair impulse within the declared normalized momentum
  residual bound;
- identical sample count and mass before/after every closed fixture step;
- no position outside the declared analytical boundary allowance;
- repeated cold reports have the same fixture/profile/input hashes;
- an optimization cannot change fixed iteration count or active term set.

The `v0` oracle tolerances are fixed before implementation:

| Field | Acceptance |
|---|---|
| density | absolute `<= 0.1 kg/m^3` or relative `<= 1e-4` |
| normalized energy/source/matrix component | absolute `<= 2e-5` or relative `<= 5e-5` after the report's declared physical scale normalization |
| position after one iteration | absolute `<= 2e-6 m` |
| reconstructed velocity after one iteration | absolute `<= 2e-3 m/s` |
| normalized total pair momentum residual | `<= 1e-5` |
| repeated CUDA output spread for one fixed input | each field remains inside the same applicable CPU-oracle bound |

The report must emit both absolute and relative discrepancy and the exact
normalization scales. Tolerances may not be widened after a failure. A
paper/result mismatch is reported rather than tuned away. If condition-number
analysis proves one bound meaningless before implementation, NR0 must be
revised and committed before the affected kernel is written.

Energy decrease is recorded but is not a universal pass rule because the
published method lacks an unconditional global line-search guarantee. Any NR3
line-search or early-exit candidate must define its own residual, accepted-step
and failure semantics before implementation.

## Performance measurement contract

The Linux RTX 3080 report records:

- tool commit, compiler, CUDA toolkit/driver, GPU identity and clocks/power
  mode available to the process;
- profile/input/source hashes and fixed iteration count;
- allocation/setup, neighbor construction, density, each enabled energy term,
  local `3x3` update, state handoff and total time separately;
- five warm-ups and at least 50 measured runs for a decision p95;
- minimum, median, p95, p99 and mean from raw unrounded samples;
- peak device memory, directed/undirected pair counts and average/max degree;
- one profiler capture for stage attribution, kept outside Git with SHA-256.

Setup that is legitimately persistent across substeps is reported separately,
but the primary total includes all work required by one ordinary step after
capacity admission, including neighbor construction. Report I/O and external
process startup are excluded.

Device utilization is diagnostic. High occupancy is neither a pass nor proof
that the algorithm is optimal. Runs with competing compute load or thermal
throttling are invalidated and repeated, not silently averaged.

## Stage gates

NR1 may enter NR2 only when:

- every tiny oracle case passes;
- both repeated fixed-iteration controls produce finite, hash-identical input
  closures;
- the baseline report separates the declared stages;
- a profiler capture supports or falsifies `HN-2`;
- no upstream framework dependency is required by the standalone binary.

The NR1 surface mismatch prevents direct entry. The separately identified
[NR1-RC1 accumulation reclosure](03-nr1-deterministic-accumulation-reclosure.md)
may satisfy this gate only as `nuv-gather-directed-r0`; it cannot relabel
`source-atomic-v0` as passing. The original HN-3 denominator remains usable
only for the two declared speed workloads on which the atomic implementation
passed correctness. Surface timing from that implementation receives no
correctness or performance credit.

NR2 continues to NR4 when its ordered optimization ladder is complete or an
early stop rule fires. NR3 is optional and begins only after the fixed-iteration
result is known; it cannot delay the fixed-iteration decision.

## Decision states

NR4 selects exactly one state:

### `NONLOCAL_48K_RECLOSURE_CANDIDATE`

All correctness gates pass, retained NR2 work is at least `2.0x` faster than
NR1, and `nuv-water-48k.v0` total p95 is `<= 8 ms`. This authorizes drafting a
new Proposed solver/profile reclosure and full independent corpus plan. It is
not W2 PASS and does not authorize GPU authority.

### `NONLOCAL_LOCAL_DOMAIN_CANDIDATE`

The 48k cutoff fails, but correctness passes and both `nuv-water-16k.v0` and at
least one coupled viscous/surface 16k profile fit total p95 `<= 8 ms`. The only
admissible follow-up is a consumer decision for a local high-fidelity domain
alongside a separately specified coarse far field. The 50k product gate is not
silently reduced.

### `NONLOCAL_RESEARCH_ONLY_STOP`

Correctness fails, fixed-iteration speedup stays below `2.0x`, both scale gates
fail, or implementation complexity/resource bounds eliminate a credible
integration path. Useful source audit and microbenchmark code may remain, but
no further solver integration or long corpus run is funded.

### `NONLOCAL_EVIDENCE_INCOMPLETE`

Required hardware, primary source, compiler/tool support or a reproducible
baseline is unavailable. This is a blocked research result, not a rejection or
performance miss.

## NR4 closure

NR4 selected `NONLOCAL_48K_RECLOSURE_CANDIDATE` on 2026-08-20. The retained
identity passes its applicable correctness gates, reaches `3.27688x` HN-3
geometric-mean fixed-work speedup and records `4.019520 ms` water-48k total
p95 against the `8 ms` research cutoff. The complete gate interpretation and
scope boundary are recorded in the
[NR4 decision](../../development/nonlocal-continuum-nr4-decision-2026-08-20.md).

This closure authorizes a later Proposed reclosure draft and fresh corpus only.
It grants no W2, runtime, GPU-authority or production credit.

## Explicit non-goals

- copying or building all of PeriDyno inside Next Engine;
- changing SPEC-38, ADR-076 or current W1 roots during NR0–NR3;
- runtime/public types, PhysX reaction, persistence or rendering;
- GPU replay/canonical authority;
- Pairwise Descent without its public equations;
- ML initial guesses, adaptive split/merge, multi-GPU or world-scale LOD;
- claiming water, mud, clay and solids from one particle/state schema.
