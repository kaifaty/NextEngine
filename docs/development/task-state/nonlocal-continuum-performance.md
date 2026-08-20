# Nonlocal continuum performance reclosure — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE_IMPLEMENTATION / NP1_P1 / REPORT_ONLY` |
| Updated | `2026-08-20` |
| Task key | `nonlocal-continuum-performance` |
| Scope | Design and execute a representative exact-50k Nonlocal performance reclosure |
| Definition of done | NP4 selects one frozen terminal state from reproducible v1 correctness and p95/p99 evidence |
| Authority | Working context only; SPEC-38, ADR-076/081, roadmap and frozen profiles/evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** NP0 is complete and reproducible. Exact-50k p99
  passes 6 ms, but conditioned p95 remains `4.849–4.869 ms` coherent and
  `4.470–4.495 ms` advected, so the 4 ms target is not met.
- **Why:** coherent/permuted physics and canonical topology match, yet the
  permutation makes p95 `1.94x` slower. Pair traversal/memory remains the
  dominant first target; the v1 trace changes membership after step two.
- **Next action:** specify the arithmetic association for
  `fused-owner-terms-p1`, implement it behind a selectable rollback identity,
  then run tiny/stiff/full correspondence before an adjacent tournament.
- **Current blocker:** None for P1.
- **Do not retry:** O3 endpoint pre-addition or coherent-lattice O4 tuning;
  their numeric/performance failures are closed evidence.
- **Reconsider when:** a v1 dynamic profile exposes materially different
  pair/locality distributions and passes the retained oracle.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [NR4 decision](../nonlocal-continuum-nr4-decision-2026-08-20.md) | `NONLOCAL_48K_RECLOSURE_CANDIDATE` | fresh Proposed reclosure/corpus may be researched; no W2 credit |
| [Final NR2 evidence](../nonlocal-continuum-nr2-o4-evidence-2026-08-20.md) | water-48k `4.019520 ms` p95; HN-3 `3.27688x`; pair kernels dominate | prioritize pair work; retain all negative boundaries |
| [Next-performance research](../nonlocal-continuum-performance-roadmap-research-2026-08-20.md) | primary sources audited and candidates ranked | NP0/NP1 immediate; NP2 separate; NP3 conditional |
| [Performance roadmap](../../plans/nonlocal-continuum-performance/README.md) | `SPECIFIED / NP0_NEXT` | stage, order and stop rules frozen |
| [Research contract](../../plans/nonlocal-continuum-performance/00-performance-research-contract.md) | `SPECIFIED` | v1 families, measurement tiers and NP4 states frozen |
| [NP0 corpus specification](../../plans/nonlocal-continuum-performance/01-np0-corpus-and-baseline.md) | `COMPLETE / NP1_INPUT` | seven generators and hashes frozen |
| [NP0 evidence](../nonlocal-continuum-np0-evidence-2026-08-20.md) | `NP0_COMPLETE / NP1_P1_AUTHORIZED` | conditioned denominator, HP-1 confirmation and stiff-dynamic negative frozen |

## Decisions that still constrain the work

### D-NP-001 — New corpus, no relabelled v0 profile

- **Observation:** v0 is 48k, coherent, boundary-free and single-state.
- **Evidence:** frozen profile JSON and final O4 evidence.
- **Decision:** create v1 exact-50k/permuted/advected identities; never modify
  v0 in place.
- **Rejected alternatives:** infer 50k/p99 from 48k p95.
- **Consequences:** NP1 waits for a fresh denominator.
- **Uncertainty:** the dynamic baseline may improve or regress.
- **Reconsider when:** never for v0 identity; only a new version may change it.

### D-NP-002 — Pair traversal is the first optimization target

- **Observation:** density/incompressibility/viscosity own almost all retained
  GPU kernel time.
- **Evidence:** final Nsight Systems hashes in NR2 evidence.
- **Decision:** test compatible owner-term fusion and compact CSR before launch
  cleanup or exotic spatial structures.
- **Rejected alternatives:** CUDA Graphs as first rescue; launch work is not a
  measured dominant stage.
- **Consequences:** density's global dependency remains explicit.
- **Uncertainty:** memory versus arithmetic limitation without privileged
  counters.
- **Reconsider when:** an NP1 profiler assigns at least `15%` elsewhere.

### D-NP-003 — Dynamic locality is a new discriminator, not an O4 retry

- **Observation:** stable lattice order was already spatially coherent and O4
  worsened tail distance/cost.
- **Evidence:** O4 exact negative tournament.
- **Decision:** cell/radix storage may return only for permuted/advected v1
  inputs with coherent O4 as negative control.
- **Rejected alternatives:** tune Morton/Hilbert/LBVH on the same lattice.
- **Consequences:** NP0 must emit storage-distance and pair distributions.
- **Uncertainty:** amount of disorder in a representative trajectory.
- **Reconsider when:** HP-1 records a `>=5%` locality or timing difference.

### D-NP-004 — Algorithm changes have separate roots

- **Observation:** adaptive exit/Anderson changes accepted work and possibly
  the stationary point; O3 showed surface sensitivity to association alone.
- **Evidence:** NR0 contract, O3 failure and Anderson primary paper.
- **Decision:** NP2 begins with a residual/stationary corpus and cannot claim
  fixed-work speedup.
- **Rejected alternatives:** position delta, hidden warm start or widened
  tolerance.
- **Consequences:** fixed and adaptive reports cannot share identity.
- **Uncertainty:** whether SISSM benefits from safeguarded Anderson.
- **Reconsider when:** a residual specification and high-iteration oracle land.

### D-NP-005 — Scale reduction is separately credited

- **Observation:** active domains/adaptivity reduce represented work rather
  than optimize the same 50k solve.
- **Evidence:** multi-scale SPH and Dual-SPH sources are method-specific.
- **Decision:** NP3 is conditional and cannot pass HP-5 by using fewer samples.
- **Rejected alternatives:** port Dual-SPH fission/fusion as Nonlocal material
  split/merge.
- **Consequences:** conservation/identity/error receipts precede NP3 code.
- **Uncertainty:** conservative Nonlocal coarse/fine coupling.
- **Reconsider when:** NP1/NP2 close and a consumer justifies the separate lane.

### D-NP-006 — Condition GPU before formal warmups

- **Observation:** unconditioned duplicate decision p95 differed by about
  `12%` after long CPU trace canonicalization left the GPU idle.
- **Evidence:** NP0 diagnostic and conditioned A/B reports.
- **Decision:** run 256 untimed workload executions, reset the seed, then run
  the contract's 32/64 formal warmups.
- **Rejected alternatives:** select the fastest run or infer boost state.
- **Consequences:** conditioned p95 spread is `0.41–0.54%`; NP1 uses the same
  preconditioning.
- **Uncertainty:** privileged per-kernel counters remain unavailable.
- **Reconsider when:** runtime clock telemetry becomes directly available.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| HP-1 dynamic disorder matters | confirmed: permuted p95 `1.94x`, storage p95 `39247` vs `5199` | coherent O4 still negative | P3 only after P1/P2 |
| HP-2 pair fusion/compact CSR gives `>=10%` total | repeated compatible CSR walks own most time | hardware counters unavailable | fused owner-term micro/oracle then adjacent 50k |
| HP-3 skin reuse gives `>=5%` amortized | established neighbor-list method; 11.8% water neighbor share | extra candidate pairs/filter/rebuild | 32-substep advected trace |
| HP-4 convergence acceleration gives `>=25%` coupled gain | 20+ iterations and related fixed-point evidence | no SISSM residual proof yet | residual spec and CPU stationary corpus |
| HP-5 exact 50k reaches `4/6 ms` | 48k p95 is `4.019520 ms` | exact 50k/p99/dynamic absent | NP0 then final decision campaign |

## Required context

Read these sources in precedence order before acting:

1. [continuum routing](../../architecture/agent-routing.md),
   [SPEC-38](../../architecture/38-continuum-material-physics.md),
   [ADR-076](../../architecture/adr/076-continuum-material-physics-track.md)
   and [ADR-081](../../architecture/adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md);
2. [main roadmap](../../roadmap.md) continuum R8 row;
3. [performance roadmap](../../plans/nonlocal-continuum-performance/README.md)
   and [contract](../../plans/nonlocal-continuum-performance/00-performance-research-contract.md);
4. [research report](../nonlocal-continuum-performance-roadmap-research-2026-08-20.md),
   [NR4 decision](../nonlocal-continuum-nr4-decision-2026-08-20.md) and
   [final NR2 evidence](../nonlocal-continuum-nr2-o4-evidence-2026-08-20.md).

## Next action

1. Freeze P1's per-term accumulators and retained combine order in a short
   implementation specification.
2. Implement one-CSR owner fusion without crossing density's global barrier.
3. Pass CPU term oracle, stiff i2, exact fixed-frame output/CSR and dynamic
   trace correspondence.
4. Run conditioned 32/96 alternating adjacent tournaments on coherent,
   permuted, advected and viscous controls; retain only if the NP1 gate passes.

## Do not retry

- source-atomic stiff surface — repeated association instability;
- O3 endpoint pre-addition — first mismatch at stiff-surface i2;
- O4 on coherent v0 — exact but slower on every profile;
- long percentile run before tiny/dynamic preflight or GPU conditioning;
- Pairwise Descent implementation — primary paper/code still to appear;
- hidden warm start, ML, multi-GPU or fewer particles as an NP1 rescue.

## Handoff

- **Workspace state:** NP0 implementation/evidence are ready for checkpoint;
  P1 code has not started.
- **Checks:** CPU, CPU-gather, retained CUDA self-tests, v0 repeatability, all
  v1 adjacent controls and duplicate exact-50k decisions pass.
- **Remaining risk:** fixed-work p95 misses by `11–18%`; P1 arithmetic
  association and later P2 memory encoding are not yet proven.
- **Promotion needed:** none for NP0 research; later Proposed solver/profile
  reclosure only after NP4 evidence.
