# NSR3-B4 physical-corpus research -- 2026-08-21

Status: `COMPLETE / DECOMPOSED_PHYSICAL_RECLOSURE / B4A_SELECTED`

## Question

What is the smallest physical-corpus sequence that can falsify the selected
owned-residual Nonlocal solver without mixing pressure, boundary topology,
viscous wall coupling, surface calibration, canonical publication and
performance into one uninterpretable long run?

## Audited evidence

The Nonlocal paper validates one coupled position objective for
incompressibility, viscosity and surface tension, but its quantitative corpus
does not directly close our engine profile. Its dam-break study is primarily
qualitative, its convergence reference is iteration 200, and its Poiseuille
case is a two-dimensional channel bounded by ghost solid particles and driven
by a constant inlet velocity. Fluid-solid collision remains future work
([Liu et al., 2026](https://doi.org/10.1145/3799902.3811196)).

The authors' public PeriDyno implementation was audited at commit
`1aa892bb296fe766d2f9249c881b8605af23a69b`. In
`SemiImplicitUnifiedFluidSolver.cu`, fixed ghost particles are present in the
same neighbor lists used by the viscosity term; the fixed state is then held
during the position iteration. The viscosity example constructs a sampled
ghost floor and separately applies an SDF volume boundary. This is materially
different from Next Engine's current split model, where virtual wall samples
enter only fluid-centred pressure density and hard contact is a frictionless
post-solve sweep. A Poiseuille run on the current wall would therefore test a
different boundary condition, not the paper's viscosity claim.

The authors' official publications page still lists **Semi-Implicit Pairwise
Descent for Nonlocal Continuum Mechanics** as `Paper (to appear)` and
`Code (to appear)` on 2026-08-21. No formula or implementation can be inferred
from the title.

The accepted DFSPH comparator remains useful independent evidence. Its exact
geometry, aggregate definitions and complete-file hashes are frozen, but the
large files were deliberately kept outside Git and are no longer present at
their recorded `/tmp/cwref-*-hard-contact-final.bin` locations. Reference
rehydration and hash attestation must therefore precede any comparison run;
missing files cannot be treated as a physics failure.

## Capability audit

| Concern | Selected evidence | Missing before physical credit |
|---|---|---|
| pressure objective | normalized density/gradient/HVP and owned-residual trust trajectory pass | hydrostatic and free-surface trajectories |
| static support/contact | two wall-owned layers, virtual reaction and lower face/corner composition pass | free-surface-aware closed box; upper faces; larger topology |
| gravity | manufactured free flight passes | publication through the physical runner |
| viscosity | boundary-free formula derivatives/HVP pass | fixed-wall viscous coupling and inlet/no-slip contract |
| surface tension | boundary-free formula derivatives/HVP pass | macroscopic coefficient calibration and physical observable |
| internal wall | no selected implementation | plane patch, closed aperture, edge/corner CCD and sided support |
| neighborhood | fluid-only canonical cell pairs and Hessian tape pass | joint fluid/static-support operator in the B3R solver |
| publication | independent binary64 publisher and roots exist | accepted-step integration and decode-next-step ownership |
| references | scenario manifests and SHA-256 values frozen | external file rehydration and exact attestation |

This audit rejects a monolithic `NSR4` run. A failure there could currently
mean at least six unrelated things.

## Current brute-force cost boundary

`boundary_reference.cpp` scans every fluid-fluid and fluid-boundary candidate
in each pressure evaluation and repeats analogous work in HVPs. Before any
distance rejection, the nominal two-layer closed-box counts are:

| Scenario topology | fluid | outer support | candidate checks / evaluation |
|---|---:|---:|---:|
| hydro, `20 x 20 x 20` box | 6,000 | 5,824 | 52,941,000 |
| dam break, `80 x 20 x 20` box | 6,000 | 16,384 | 116,301,000 |
| orifice outer box, before internal wall | 6,000 | 9,344 | 74,061,000 |
| sealed product, `80 x 20 x 40` box | 48,000 | 24,704 | 2,337,768,000 |

These are per objective evaluation, not per step; a trust solve and embedded
controller execute many evaluations/HVPs. Tiny physical controls may use this
oracle, but nominal execution is forbidden until the already selected cell
neighborhood and tape are extended to static support.

## Decomposed B4 roadmap

1. **B4A closed-box/free-surface eligibility.** Prove that a box-owned shell
   is independent of the current fluid extent, contains no fictitious support
   in the air region, preserves the two-layer identity and handles all six
   swept faces. Publish the nominal brute-force cost boundary.
2. **B4B tiny pressure-water corpus.** Run bounded hydrostatic support and
   release/impact fixtures with `lambda=mu=gamma=0`, the B3R owned solver and
   independent fixed refinement. Compare aggregate mass, COM, height/front,
   pressure strain, energy and reactions; do not compare particle identity.
3. **B4C scalable canonical pressure runner.** Integrate joint fluid/support
   cell neighborhoods, Hessian tape, accepted-step canonical publication,
   decode-next-step ownership, exact repeat/storage-order roots and capacity
   failure. No nominal run precedes this gate.
4. **B4D reference rehydration.** Restore the three external files, require
   exact complete-file hashes and reject a one-byte mutation before executing
   a trajectory.
5. **B4E nominal pressure-water corpus.** Run free fall, hydro, dam break,
   still tank, sealed boundary and order controls. Orifice remains excluded
   until B4O.
6. **B4O internal aperture topology.** Add sided internal support plus swept
   plane-patch/edge/corner contact, then run smoke and nominal transfer.
7. **B4V viscosity lineage.** Freeze fixed-wall/inlet viscous coupling, reclose
   derivative and momentum/dissipation ledgers, then use analytical
   Poiseuille. This is a separate material profile.
8. **B4S surface lineage.** First calibrate a boundary-free macroscopic
   observable (planar interface or droplet oscillation/Laplace control), then
   test free-surface dynamics. Wall adhesion is a later separate term.

Pressure-water success cannot donate credit to B4V/B4S, and B4V/B4S cannot
rescue a pressure-water failure.

## Decision

Freeze and execute
[B4A](../plans/nonlocal-nonlinear-solver-research/03b4a-closed-box-eligibility-contract.md).
It is an inexpensive geometry/model-eligibility discriminator, not a physical
water validation. A PASS authorizes only the bounded B4B contract design.

