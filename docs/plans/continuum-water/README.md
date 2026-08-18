# Continuum water — standalone implementation roadmap

Status: `RESEARCH_ONLY / NOT_ACTIVE`; post-v1 isolated program. Governing candidate
architecture: [SPEC-38](../../architecture/38-continuum-material-physics.md)
and [ADR-076](../../architecture/adr/076-continuum-material-physics-track.md),
with [ADR-081](../../architecture/adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md)
as the promotion guardrail.
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
successor profile is authorized only for the Linux W1 serial corpus; it is not
selected for production. Required external references are still missing and
no `CONTINUUM-*` ProductCheck has run.

This directory is the resume and execution surface for a dedicated water
worktree. The main [Next Engine roadmap](../../roadmap.md) keeps the track
inactive until `CONTINUUM-WATER-REF-P1 = PASS`. Progress here cannot change
the current PhysX, save/replay or public-contract baseline by implication.

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
                         └─ W1 Serial CPU oracle + external corpus IMPLEMENTED / IN_PROGRESS
                             ├─ W2 Deterministic parallel CPU + benchmark NOT_STARTED
                             │   └─ W3 One-pass PhysX coupling            NOT_STARTED
                             │       └─ W4 Basin + crate + debug view     NOT_STARTED
                             │           └─ W5 Exact active persistence   NOT_STARTED
                             │               └─ W6 Production promotion   NOT_STARTED
                             └─ WG Optional GPU correspondence mirror     NOT_STARTED / NON_BLOCKING
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
| W1 | [Serial CPU DFSPH oracle](01-serial-cpu-dfsph-oracle.md) | `CONTINUUM-WATER-REF-P1 = PASS` on the same-target reference profile | main-roadmap activation, W2, WG |
| W2 | [Deterministic parallel CPU and performance](02-deterministic-parallel-and-performance.md) | Worker/order exactness and standalone `50k` THOTH stop-target PASS | W3 |
| W3 | [One-pass PhysX coupling](03-one-pass-physx-coupling.md) | `CONTINUUM-COUPLING-P1 = PASS` under one composition DAG, exact exchange tuple and one PhysX integration | W4 |
| W4 | [Basin, crate and debug presentation](04-basin-crate-debug-presentation.md) | Production command loop and presentation-independence pass | W5 |
| W5 | [Exact active persistence](05-exact-active-persistence.md) | `CONTINUUM-PERSISTENCE-P1 = PASS` with scheduled checkpoint epochs | W6 |
| W6 | [Production promotion](06-production-promotion.md) | Consumer-backed Accepted decision, explicit fault/capacity profiles and successor combined budget PASS | shipped claim |
| WG | [GPU correspondence mirror](wg-gpu-correspondence.md) | `CONTINUUM-MIRROR-P1` report for named devices | no authority or promotion stage |

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

W0G is now root-frozen. Under its static-impact rule the unchanged dam-break
completes `720/720`, has zero positive energy excess and publishes the full
deficit/stage accounting. It remains `REFERENCE_PENDING`, so W1 and
`CONTINUUM-WATER-REF-P1` remain incomplete. Current execution is Linux-only;
Windows exactness is deferred, not waived, for promotion.

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

W0F closes the profile-level blockers without activating the main roadmap.
Production and independent paths match the exact geometry manifest, all
`10,880` orifice support records, density/gradient fixtures and swept contact
fixtures. Every selected W1 geometry fits the `32,768` static capacity; the
product extent uses `24,704`. The W0E 24/1200-step roots and free-fall remain
exact, while the 24-step orifice preflight transfers 24 samples legally with
strict radius clearance and closes momentum to `23 ppb`. The new roots
authorize the full serial W1 corpus. They do not supply corpus credit or a
ProductCheck PASS, and dynamic rigid geometry remains W3 work. The evidence
does not currently require a density map, XSPH, warm pressure state or
settling generator. W2, WG, PhysX and GPU work remain blocked.

The dated [W0F evidence](../../development/continuum-water-w0f-geometry-capacity-root-closure-2026-08-18.md)
records the frozen roots, independent comparisons and two clean identical
bounded report projections.

W0G leaves those operations and roots immutable. It adds a child metric/corpus
root: equilibrium/control scenarios retain two-sided `1%` drift, while named
static-impact scenarios block positive energy creation and expose dissipation.
External dam-break/orifice curves remain mandatory, so the reclosure cannot
manufacture reference credit.

## Worktree and main-roadmap protocol

1. Create the water worktree from the documentation checkpoint containing W0A
   and the hash-frozen W0B closure.
2. Record stage state and exact evidence links in this roadmap/task-state;
   never mark a PASS from a type, fixture, compilation or report-only run.
3. After W1 PASS, merge the evidence checkpoint and change the main R8 row from
   `PLANNED / NOT_ACTIVE` to an active integration track. Before that event the
   main roadmap contains only this experimental pointer.
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
