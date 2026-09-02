# Continuum water — standalone implementation roadmap

Status: `RESEARCH_ONLY / ACTIVE_W2_RESEARCH / INTEGRATION_BLOCKED / ADR-100_PROPOSED`; post-v1 isolated program. Governing candidate
architecture: [SPEC-38](../../architecture/38-continuum-material-physics.md)
and [ADR-076](../../architecture/adr/076-continuum-material-physics-track.md),
with [ADR-081](../../architecture/adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md)
as the promotion guardrail. Proposed
[ADR-100](../../architecture/adr/100-authoritative-water-volume-and-presentation-only-gpu-water.md)
reframes water V1 as an exact CPU `WaterVolume` for gameplay plus
non-authoritative presentation dynamics; the CPU lane below stays the
research oracle, and the Nonlocal GPU game candidate is tracked in
`docs/development/task-state/nonlocal-gpu-full-step-performance.md`.
The original W0B numeric/profile/corpus closure remains hash-frozen evidence,
but W1-RC1 independently reproduced its hydro non-convergence and requires a
W0C calibration reclosure. W0C diagnostics reject the ceiling-only repair,
cell-centred ghost boundary and independently reproduced settling generator.
The final analytical volume-map discriminator also fails local partition and
the first hydro step while matching its independent calculator exactly. W0C
is closed `RESEARCH_ONLY`. W0D then falsifies one-layer support truncation as
the cause: a support-complete two-layer complement matches independently but
fails on the same second step. W0E separates density support, unilateral
geometric contact and the pressure algorithm; its projected-PCG candidate
matches independent implementations and passes a 1200-step local soak with
zero penetration. W0F now closes shared outer/internal geometry, oriented
density support, swept aperture contact, the `32,768` static capacity and new
domain-separated roots. W1 then exposes an inherited contradiction between
inelastic impact projection and a two-sided conservation gate. W0G keeps the
solver fixed and roots reversible versus static-impact energy semantics. The
next W1 failure shows that the inherited active-set PCG globally restarts its
direction on every observed sealed-48k iteration. W0H retains the physical QP
and operation ceiling but roots a fixed accelerated projected-gradient solve
with compression plus projected-KKT acceptance. The successor profile is
authorized only for the Linux W1 serial corpus; it is not selected for
production. W0I rejects the earlier geometry-violating external trajectories,
freezes three twice-reproduced hard-clearance reference hashes and requires
exact attestation before production-credit execution. Clean full-corpus W1
now passes twice at one clean Linux commit with exact target-local equality:
`CONTINUUM-WATER-REF-P1=PASS / LINUX_W1_PASS`. The result remains
`RESEARCH_ONLY`; every later continuum ProductCheck is still `NOT_RUN`.

The W1 evidence and W2 feasibility checkpoints were merged into the mainline
history by merge commit `fe223f9`. The main [Next Engine roadmap](../../roadmap.md)
therefore records an active isolated W2 research track, not production
integration. The measured CPU and direct-GPU profiles still miss the 50k stop
target, so W3 and every runtime/public-contract change remain blocked. The
explicitly selected [Nonlocal research branch](../nonlocal-continuum/README.md)
is report-only and cannot inherit W1 roots or W2 credit.

## Selected product result

One sealed `4 × 2 × 1 m` basin contains `0.75 m` of clean water and one
`0.5 m`, `50 kg` PhysX crate. The nominal production fixture uses `48,000`
samples with a hard capacity of `50,000`. The player acts through the normal
production command path; deterministic headless consumes the recorded command
trace. Debug particles and diagnostic overlays are sufficient presentation.

If continuum capability cannot activate, the project loads its authored dry
basin variant. An active region never silently changes to dry, decorative or
frozen water.

## Stage graph

```text
W0A Product and evidence scope                    COMPLETE / DOCUMENTATION
 └─ W0B Original numeric/corpus closure           INVALIDATED_BY_RC1 / ROOTS_RETAINED
     └─ W0C Hydro calibration reclosure            CLOSED / RESEARCH_ONLY
         └─ W0D Support-complete discriminator     CLOSED / CANDIDATE_REJECTED
             └─ W0E Constraint-separated redesign CLOSED / LOCAL_SURVIVOR
                 └─ W0F Geometry + successor roots CLOSED / W1_AUTHORIZED
                     └─ W0G Impact energy semantics CLOSED / W1_AUTHORIZED
                         └─ W0H Accelerated pressure profile CLOSED / W1_AUTHORIZED
                             └─ W0I External reference attestation CLOSED / W1_AUTHORIZED
                                 └─ W1 Serial CPU oracle + external corpus COMPLETE / LINUX_PASS
                                     ├─ W2 Performance architecture research ACTIVE / CURRENT PROFILES MISS
                                     │   └─ W3 One-pass PhysX coupling            NOT_STARTED
                                     │       └─ W4 Basin + crate + debug view     NOT_STARTED
                                     │           └─ W5 Exact active persistence   NOT_STARTED
                                     │               └─ W6 Production promotion   NOT_STARTED
                                     ├─ WG Optional GPU correspondence mirror     DIRECT-PORT REPORT / NON_BLOCKING
                                     └─ NR Nonlocal continuum spike              SPECIFIED / NO W2 CREDIT
```

| Stage | Specification | Exit evidence | Blocks |
|---|---|---|---|
| W0A | [Product and evidence contract](00-product-and-evidence-contract.md) | Product fixture, evidence categories, authority and non-goals are selected | W0B |
| W0B | [Original numeric execution and corpus closure](00b-numeric-execution-and-corpus-closure.md) | Immutable rejected-profile evidence; RC1 invalidates promotion use | W0C |
| W0C | [Hydro calibration reclosure](00c-hydro-calibration-reclosure.md) | Exit not achieved; candidate ladder exhausted and explicit successor decision required | W1 resume |
| W0D | [Support-complete boundary discriminator](00d-support-complete-boundary-discriminator.md) | Two-layer production/independent equality; candidate rejected and joint profile reclosure required | W1 resume |
| W0E | [Constraint-separated profile reclosure](00e-constraint-separated-profile-reclosure.md) | Independent operator equality and 24/1200-step local survivor; no corpus credit | W0F |
| W0F | [Geometry, capacity and successor-root closure](00f-geometry-capacity-and-root-closure.md) | Outer/internal geometry semantics, admitted capacities and successor roots frozen | W1 resume |
| W0G | [Impact energy-contract reclosure](00g-impact-energy-contract-reclosure.md) | Reversible absolute drift and static-impact energy-excess semantics rooted without changing W0F operations | W1 resume |
| W0H | [Accelerated pressure-profile reclosure](00h-accelerated-pressure-profile-reclosure.md) | Same W0F pressure QP closes under a rooted fixed APG schedule with independent compression/KKT equality | W1 resume |
| W0I | [External reference geometry attestation](00i-external-reference-geometry-attestation.md) | Geometry-safe external profile and three exact reference hashes frozen without changing W0F/G/H | W1 resume |
| W1 | [Serial CPU DFSPH oracle](01-serial-cpu-dfsph-oracle.md) | `CONTINUUM-WATER-REF-P1 = PASS` on the same-target reference profile | W2, WG and bounded alternative research |
| W2 | [Deterministic parallel CPU and performance](02-deterministic-parallel-and-performance.md) | Worker/order exactness and standalone `50k` THOTH stop-target PASS | W3 |
| W3 | [One-pass PhysX coupling](03-one-pass-physx-coupling.md) | `CONTINUUM-COUPLING-P1 = PASS` under one composition DAG, exact exchange tuple and one PhysX integration | W4 |
| W4 | [Basin, crate and debug presentation](04-basin-crate-debug-presentation.md) | Production command loop and presentation-independence pass | W5 |
| W5 | [Exact active persistence](05-exact-active-persistence.md) | `CONTINUUM-PERSISTENCE-P1 = PASS` with scheduled checkpoint epochs | W6 |
| W6 | [Production promotion](06-production-promotion.md) | Consumer-backed Accepted decision, explicit fault/capacity profiles and successor combined budget PASS | shipped claim |
| WG | [GPU correspondence mirror](wg-gpu-correspondence.md) | `CONTINUUM-MIRROR-P1` report for named devices | no authority or promotion stage |
| NR | [Nonlocal continuum research](../nonlocal-continuum/README.md) | one NR4 decision from independently reproduced fixed-work evidence | no direct W2 or promotion credit |

## Program invariants

- CPU DFSPH is the only canonical water solver candidate for V1.
- Authoritative private `f64` requires the exact ADR-081 execution profile and
  adversarial Windows/Linux root gate before solver code can promote.
- The accepted state is stable sample ID plus fixed-point position/velocity;
  the next substep starts from that state.
- One physical substep produces one water candidate and one reaction batch;
  water and PhysX publish together or neither publishes.
- Exchange keys use namespace/owners/world/revisions/roots/participants and an
  operation slot, never a generic world or checkpoint generation.
- The production profile pre-admits worst-case capacities, binds the primary
  session fault domain and rebuilds PhysX at fixed checkpoint epochs.
- The first region is sealed and pinned active. Cross-region transfer, halos,
  streaming eviction and sleep conversion are not hidden implementation work.
- Public contracts appear only at W4/W6 with the runtime-bearing basin
  consumer and an Accepted promotion ADR.
- GPU completion, renderer cadence, wall time, worker count and cache warmth do
  not select an authoritative state.
- Heavy corpora, comparison trajectories, captures and performance reports
  stay outside Git; checked-in evidence is bounded hashes/summaries only.

## Frozen W0B inputs

The authoritative W0B document SHA-256 is
`d357bca64983fbd2074961a462743a4fb5d3fedc2af09631d32ec178bd299550`.
Its machine-facing float-profile and corpus roots are recorded inside
[W0B](00b-numeric-execution-and-corpus-closure.md). W1 preflight binds all
three roots. W1-RC1 invalidates their promotion use but does not rewrite this
evidence. W0C closed without successor roots; any newly authorized profile
must issue its own roots rather than editing history in place.

## Current W1 discriminator

The W0F-rooted Linux runner passes hydro and exact free-fall, then first
breached the inherited absolute-energy rule in dam-break at step 72. A bounded
continuation showed a `63.8407503%` final mechanical deficit with every other
internal dam-break invariant passing. Pressure/contact order and a
contact-aware Jacobi counterfactual did not remove the defect. The
[W1 successor energy report](../../development/continuum-water-w1-successor-energy-discriminator-2026-08-18.md)
therefore routes the metric meaning—not a threshold value or solver
coefficient—to W0G.

W0G is root-frozen. Under its static-impact rule the unchanged dam-break
completes `720/720`, has zero positive energy excess and publishes the full
deficit/stage accounting. At that checkpoint it remained
`REFERENCE_PENDING`; W0I and the later clean W1 closure resolve that historical
gap. Current execution is Linux-only; Windows exactness is deferred, not
waived, for promotion.

The next first failure is `CW-SEALED-001` step 2: projected active-set PCG
ends its 50th operator application at `194,545 ppb`. A PCG-200 control accepts
that step only at application 96, and exact traces show that active-set changes
globally reset the conjugate direction on every iteration. Standard MPRGP and
projected-CG expansion also reach no feasible CG step within the same budget.
The bounded
[solver research report](../../development/continuum-water-w1-sealed-pressure-solver-research-2026-08-18.md)
selects fixed-step `0.25` accelerated projected gradient after rejecting step
`0.5` on its hydro curvature guard.

W0H now freezes that algorithm, its zero-diagonal rule, one-operator schedule,
curvature bound and dual compression/projected-KKT acceptance. Production and
an independent first-step calculator match exactly. The candidate completes
the entire internal discriminator, including two identical still and sealed
runs and all three storage orders. Those were diagnostic pre-freeze runs and
provide no W1 corpus credit.

The first external dam-break comparison then failed only because the pinned
SPlisHSPlasH trajectory crossed the analytical wall: it exceeded the canonical
penetration allowance at step 4 and escaped at step 28. Geometry-safe solver
counterfactuals could not match that invalid splash height. W0I therefore
recloses the reference profile rather than the W0H equations. Independently
implemented predictive hard contact produces twice-identical hydro,
dam-break and orifice files. Unchanged W0H passes dam-break front/height at
`0.3157% / 0.8419%` and `2.3296% / 6.4657%` RMSE/maximum, and orifice transfer
at `0.1624% / 0.3000%`, under the original `5% / 10%` gates. The next action
was two clean W0I-attested full Linux corpus runs. See the
[hard-clearance reference report](../../development/continuum-water-w1-hard-clearance-reference-reclosure-2026-08-18.md).

Commit `e00999e96f0f55ae02426e806457f625d0a4844f` implements exact W0I
attestation and is the clean executable checkpoint for the final gate. Two
complete seven-scenario runs both produce
`CONTINUUM-WATER-REF-P1=PASS / LINUX_W1_PASS`, attest all three mandatory
references and reproduce corpus root
`d38d6bc8a8e98e87402202a926685dbe4867e3de6a7d8679362885be46e96835`.
After removing only diagnostic wall-clock fields, both reports are
byte-identical with normalized SHA-256
`2dffa4e3ea12638c3cb3c5f1be43c3ebddea3213a967c4ed3bb8054d5252a0bf`.
The next stage is W2 deterministic parallel equality and standalone
performance; this W1 result supplies no production, coupling or persistence
claim.

Final source-layout commit `fa12d3956a2a5d5b6127a9e31cbb664221b01afc`
extracts the unchanged attestation policy into a private module to satisfy the
repository file-size boundary. One additional complete clean corpus reproduces
all seven scenario roots and `d38d6bc8...e96835`. Its second serial repetition
was stopped by explicit user decision because a run takes about 42 minutes on
one core; it receives no evidence status. W2 now owns resource utilization and
worker-count equality rather than more redundant serial corpus repetition.

W2 begins with the Linux-only `profile-w2-linux` diagnostic. Its fixed
`continuum-water-50k-stage-profile.v0` projection runs one warm-up and three
measured sealed-48k substeps, reports decode-through-publication stage times,
and records a short trajectory root. Timing is outside canonical state and
roots; the diagnostic is explicitly `NO_W2_CREDIT`. The clean serial baseline records
`100%` of one CPU, about `1.212 s` per measured sealed-48k substep,
reconstruction at `61.15%` and density at `34.90%`. W2 cycle 1 now removes the
duplicate canonical neighbor discovery and per-row temporary allocations; its
clean measurement preserves every short-run root, makes reconstruction
`1.801×` and the whole step `1.375×` faster. Cycle 2 adds a private local pool
with `64` stable logical partitions. A clean serial/worker-`1/2/4/8` matrix
preserves every short root and iteration vector; eight workers reduce the
adjacent serial mean from `837.264` to `269.643 ms` (`3.105×`) and raise
whole-command utilization to `372%`. Density scales only `2.578×`, and the
best short mean remains `67.41×` above the `4 ms` ceiling. Formal full-corpus
worker equality and percentile runs are not claimed or run; W2 now requires an
explicit stop-versus-new-profile decision before more expensive execution.

Commit `74730e208cfeb70b05a3ec44b2bb9c2f5002fe97` implements the serial oracle.
On a clean exact-profile run, `CW-FREEFALL-001` passes all 97 canonical frames
and two repeats produce the same trajectory root. `CW-HYDRO-001` then stops at
its first density solve with `WATER_DENSITY_NONCONVERGENCE`: iteration 20 ends
at `74,482,699 ppb` against the `100,000 ppb` threshold. The bounded
[evidence report](../../development/continuum-water-w1-evidence-2026-08-17.md)
records roots and artifact hashes.

Commit `d54e10e55bb20528c4cf485bc3bc6be5a0a6b5c3` adds a clean-tree independent
brute-force audit. It reports `EXACT_MATCH` for all inputs, boundary volumes,
the 20-value global residual curve and four representative complete row
traces. The bounded [RC1 report](../../development/continuum-water-w1-rc1-audit-2026-08-17.md)
therefore routes the next work to W0C rather than a W1 repair.

Commit `328d01c70004bd0c9572ca4dc49891d141f6ba7f` adds exact W0C contribution,
extended-curve and boundary-candidate diagnostics. The bounded
[W0C report](../../development/continuum-water-w0c-hydro-calibration-2026-08-17.md)
rejects a larger iteration ceiling and `ghost-cell-shell-v1`: the candidate
fixes the initial partition and passes step 1, but fails the unchanged-ceiling
soak on step 2; ceilings 100 and 160 only defer failure. That result routed the
next cycle to deterministic pre-equilibrated initialization rather than
another ceiling or boundary-scale variant.

Commit `0c3acb4969f32cac1bbe1205a6e07307a10cc2c0` adds the independently
reproduced `zero-velocity-settle-v1` generator. Its iteration demand and
penetration increase through four passes, triggering the adverse-trend guard;
the last diagnostic state then fails the first unchanged-ceiling hydro step at
`162,015 ppb`. No generator output or successor root is selected. Persistent
failure now routes W0C to an analytically specified non-particle boundary
candidate, not another settling or ceiling variant.

Commit `ea122208ca102c5fd63febc229f7d14b72de5b19` implements the final
analytical volume-map discriminator from pinned paper/reference semantics.
Production and an independent calculator match exactly, including field
probes and the 320-iteration trace; free-fall remains exact. The candidate
nevertheless reconstructs face/corner density as `2.139 / 2.600`, fails the
first hydro step at `70,690,915 ppb` and accepts zero of 24 soak steps. The
bounded [volume-map evidence](../../development/continuum-water-w0c-volume-map-2026-08-18.md)
therefore closes W0C `RESEARCH_ONLY`. No fitted scale, larger ceiling or
successor roots are authorized.

Commit `89567e14a6205fbec4705c8bf5de40be674eb234` implements the separately
authorized W0D support-completeness discriminator. Its two exterior layers
cover every boundary lattice centre that can enter support before the
unchanged clearance gate. Production and independent calculations match
exactly and initial partition remains `-27,534 ppb`, but the trajectory still
accepts one step and fails step 2 at `192,430 ppb`; even ceilings 100 and 160
accept only three and four steps. The bounded
[W0D evidence](../../development/continuum-water-w0d-support-complete-boundary-2026-08-18.md)
rejects the candidate and stops particle-shell variants.

Commit `7ee1651b6c1bfcef575af1bd6952ac36f9fca661` implements the W0E
constraint-separated discriminator. It retains the two-layer complement only
for density, replaces relaxed Jacobi with deterministic projected diagonally
preconditioned PCG, and adds an analytical particle-radius velocity constraint
for the outer box. Independent pressure-action, first-PCG-step and contact
calculators match exactly. Contact/Jacobi-20 still fails on step 2, whereas
contact/Jacobi-160 passes 24 steps, confirming that weak pressure convergence
and missing non-penetration were separate defects. The candidate passes all
24 local and 1200 extended steps with at most 48 density iterations, zero
penetration and the existing reaction/work gates. The bounded
[W0E evidence](../../development/continuum-water-w0e-constraint-separated-redesign-2026-08-18.md)
records the clean report and limits.

W0F closed the profile-level blockers without activating the main roadmap.
Production and independent paths match the exact geometry manifest, all
`10,880` orifice support records, density/gradient fixtures and swept contact
fixtures. Every selected W1 geometry fits the `32,768` static capacity; the
product extent uses `24,704`. The W0E 24/1200-step roots and free-fall remain
exact, while the 24-step orifice preflight transfers 24 samples legally with
strict radius clearance and closes momentum to `23 ppb`. The new roots
authorize the full serial W1 corpus. They do not supply corpus credit or a
ProductCheck PASS, and dynamic rigid geometry remains W3 work. The evidence
does not currently require a density map, XSPH, warm pressure state or
settling generator. At the W0F checkpoint W2, WG, PhysX and GPU work remained
blocked; only the later W1 closure releases W2.

The dated [W0F evidence](../../development/continuum-water-w0f-geometry-capacity-root-closure-2026-08-18.md)
records the frozen roots, independent comparisons and two clean identical
bounded report projections.

W0G leaves those operations and roots immutable. It adds a child metric/corpus
root: equilibrium/control scenarios retain two-sided `1%` drift, while named
static-impact scenarios block positive energy creation and expose dissipation.
External hydro/dam-break/orifice curves remain mandatory, so the reclosure
cannot manufacture reference credit.

W0H leaves W0F and W0G immutable and adds a child pressure-solver profile. Its
fixed diagonal scaling and APG schedule solve the same non-negative QP within
the same 50-operator ceiling. Compression and projected-KKT residuals are both
blocking. External hydro/dam-break/orifice curves and clean rooted corpus runs
were mandatory downstream evidence; W0H itself did not manufacture W1 or
ProductCheck credit. W0I plus the two clean W1 runs now close that downstream
gate.

## Worktree and main-roadmap protocol

1. Create the water worktree from the documentation checkpoint containing W0A
   and the hash-frozen W0B closure.
2. Record stage state and exact evidence links in this roadmap/task-state;
   never mark a PASS from a type, fixture, compilation or report-only run.
3. The W1/W2 checkpoint is merged at `fe223f9`; the main R8 row may therefore
   show active isolated research. Integration remains blocked because W2 has
   no performance PASS. A report-only solver spike does not change that state.
4. Each stage is a coherent commit boundary. Failed evidence leaves the stage
   open and records the smallest discriminator; it does not relax thresholds.
5. Treat the `4/6 ms` 50k number as a standalone stop target. Production also
   requires the mutually exclusive successor `world-dynamics-step` row across
   every substep in one gameplay tick. If that combined gate misses after two
   evidence-backed optimization
   cycles, stop the roadmap as `RESEARCH_ONLY`. GPU authority, a smaller
   production gate or a larger budget requires a new explicit decision.

## Whole-program non-goals

Ocean/weather simulation, fluid networks, cross-region particle transport,
adaptive particles, surface-quality rendering, foam/spray, generic plugin
solver ABI, arbitrary gameplay fluid queries, wet soil, terrain deformation,
full vehicles and production sleep/wake conversion.
