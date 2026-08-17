# V0 — Product decisions and calibration closure

Status: `V0A COMPLETE / V0B OPEN / BLOCKS CODE`.

## Outcome

V0A records the selected product, authority, representation, evidence and
budget decisions. V0B freezes every numerical input that could otherwise be
chosen by an implementer after seeing a result: exact graph/taper bytes,
synthetic wood constants, fixtures, numeric scales, candidate cadence ladders,
curve hashes, remaining thresholds and capacities. Both parts are
documentation-only and prove no physics.

## V0A selected decision profile

| ID | Selected answer | Frozen decision |
|---|---|---|
| 1A | Trail-tree vertical | One pinned-active tree beside a trail; production axe notch/back-cut, computed fall and road blockage. |
| 2A | Synthetic reference | `Next Engine Reference Conifer V1` is an engine test profile with plausible response and no species-realism claim. |
| 3B | Medium bounded tree | 10 m height, 12 major branches and a hard capacity of 128 structural segments. |
| 4A | Fixed root | Root is a rigid clamp; translation, rotation and uprooting are excluded. |
| 5A | Formulation bake-off | Compare a shearable Cosserat/Timoshenko formulation with a constrained/implicit discrete-rod or corotational beam baseline. |
| 6A | Fixed cadence | Outer step is 1/240 s; V0B freezes bounded candidate cadence ladders and V1 selects one fixed internal profile from the corpus. No adaptive or variable step. |
| 7A | Profiled fixed-point continuation | `f64` is private to one outer step under the exact ADR-081 execution profile; the next step begins only from ties-to-even fixed-point canonical state. |
| 8A | Axe-only cutting | One authored felling zone supports a directional axe notch and back cut through the production command path. |
| 9B | Section resolution | Exactly 8 radial rings by 32 angular sectors, or 256 cells, in the felling zone. |
| 10A | Canonical cut work | One exact contact/load identity is consumed once; its receipt allocates rigid reaction, structural work, damage/fracture and dissipation. |
| 11A | Geometry-derived failure | Remaining area, centroid and moments plus directional stress/strength drive failure; at most one split per structure per substep. |
| 12A | Outcome-owned rigid handoff | Stage 8 publishes `PendingFracture`; stage-9 Outcome owns the topology transaction and the PhysX body activates next substep. |
| 13A | Stable collision shape | Standing-tree collision uses at most 32 tapered proxies, each mapped to one stable PhysX kinematic body/shape pair. |
| 14A | Forcing fixtures | Deterministic analytical calm, steady-wind and gust fixtures plus prescribed pull/release. |
| 15A | LOD ladder | `AuthoredStatic -> ShaderWind -> ModalStructural -> ActiveStructural -> RefinedSection`, driven only by canonical facts. |
| 16B | Forest gate | 1,000 visible, 128 modal, 8 active, 1 refined and at most 2 falling trees. |
| 17A | Performance budget | 2/3 ms p95/p99 is a standalone THOTH stop target; integrated promotion uses the successor combined `world-dynamics-step` row. |
| 18A | Accuracy targets | Static curves <=2%; natural frequencies <=5%; aggregate normalized RMSE <=5%; maximum curve error <=10%; work/impulse residual <=1%; mass exact; handoff CoM <=1 mm and momentum residual <=1%. |
| 19A | Exactness ladder | Same-target roots are exact in V1; Windows/Linux roots are exact before production; exact active save precedes lossy modal/sleep persistence. |

V0A is complete. It does not close the numeric/profile/corpus items below and
does not authorize V1 code.

## V0B required exact tree definition

V0B must record canonical bytes and hashes for:

- the selected 10 m total height, exact trunk centreline/taper and root frame;
- complete node/segment IDs, rest directors/curvature and the selected 12-
  branch graph;
- every segment length, start/end cross-section, mass/drag allocation and
  collision-proxy mapping;
- the felling-zone axial extent and its relation to the trunk section;
- visual asset references only where needed to prove projection alignment;
- exact node count within the selected capacities of 128 segments, 32
  collision proxies and 256 felling-zone cells.

The branch count and capacities are selected, but the exact graph is not.
Source order must not affect canonical bytes.

## V0B required synthetic wood profile

Assign the selected `Next Engine Reference Conifer V1` engine test material
exact values, units, fixed-point source values and provenance for:

- density and mass-distribution rules;
- longitudinal, radial and tangential Young/shear/Poisson closure needed by
  the selected candidate formulations;
- directional tension, compression, shear and fracture/hinge criteria;
- structural damping and a reference moisture condition;
- the rigid-clamp boundary and explicit exclusion of root failure;
- tolerance from source material values to the reduced beam/section model.

The USDA Wood Handbook describes clear straight-grained specimens and cannot
by itself calibrate a complete living tree or junction. This synthetic profile
makes no species-realism claim.

## V0B required canonical numeric profile

Freeze exact integer widths, scales, bounds and ties-to-even conversions for:

- node position/orientation and linear/angular velocity;
- strain/curvature/twist, section resultant and damage/history variables;
- wind samples, forces, impulses, work and energy diagnostics;
- section-cell geometry/state and derived area/centroid/moments;
- topology/handoff mass, CoM and linear/angular momentum;
- residuals, iteration counts and stable failure codes.

Also freeze the ADR-081 target/toolchain feature baseline, FMA contraction,
rounding/subnormal behavior, deterministic math primitives, factorization/
reduction/tie order and convergence branches plus adversarial cross-target
fixtures.

The outer cadence is exactly 1/240 s. Freeze a bounded internal
substep/iteration ladder for each V1 candidate, convergence rules and hard
iteration/capacity limits before running the bake-off. V1 may select only a
predeclared fixed profile from that ladder. No wall-time deadline may select
iteration count or LOD.

## V0B required wind, pull and cutting fixtures

Wind profile:

- exact mean vector, deterministic gust function, duration and evaluation
  points for calm, steady and gust cases;
- drag areas/coefficients and how foliage mass/area is aggregated;
- external-work accounting and output sample cadence.

Structural pull/release profile:

- exact node/point, direction, load curve, hold/release ticks and expected
  analytical/reference outputs.

Cut profile:

- axe/contact/load identity, one-use consumption ledger, geometry and
  production command trace;
- exact mapping from canonical relative velocity/impulse, blade direction,
  grain coefficient and tool profile to cut work;
- the selected 8-ring by 32-sector cell geometry and closed cell states;
- notch/back-cut command sequence and permitted felling-zone bounds;
- failure/hinge criterion and deterministic tie-breaking.
- exact work-allocation receipt across rigid reaction, structural work,
  section damage/fracture and dissipation.

Direct test mutation or a `fell_now` command is forbidden.

## V0B reference corpus and metrics

| Family | Required cases | Metrics to freeze before V1 |
|---|---|---|
| Beam/rod | static/dynamic cantilever, torsion, axial load, buckling, taper | displacement/rotation curves, mode frequencies, force/moment balance, energy/work residual |
| Junction | one Y-branch under gravity and prescribed load | junction continuity, branch-tip curves, reaction balance |
| Tree/wind | gravity sag, calm, steady wind, gust, pull/release | node trajectories, first modes, damping envelope, external-work-aware residual |
| Cutting | progressive one-sided notch, back cut, asymmetric hinge, invalid/stale command | removed area/moments, load-displacement/failure tick/direction, exact rejection |
| Coupling | body impact/push, ground contact after handoff, capacity/stale faults | impulse closure, penetration, mass/CoM/momentum handoff and atomicity |
| Persistence/LOD | save/resume and every adjacent tier cycle | exact active root; declared discontinuity/history/energy bounds for approximate tiers |

Every scenario needs exact geometry, horizon, output cadence, formula and curve
source/hash. The selected general thresholds are static curves <=2%, natural
frequencies <=5%, aggregate normalized RMSE <=5%, maximum curve error <=10%,
external-work-aware work/impulse residual <=1%, exact mass, handoff CoM <=1 mm
and momentum residual <=1%. V0B must still freeze metric normalization,
per-case applicability, collision penetration and LOD-transition bounds.
Visual plausibility is diagnostic and cannot replace a curve.

## Independent evidence

- [Discrete Elastic Rods](https://doi.org/10.1145/1360612.1360662) supplies
  buckling, stability and coupled bending/twist validation cases.
- [PyElastica v1.0.0](https://github.com/GazzolaLab/PyElastica/tree/v1.0.0)
  at tag commit `b087f1399f9be2fdd2fcf3768689f7735a96f7ab` is the pinned external
  Cosserat comparator. Its aggregate curves are evidence, not exact node
  identity and not linked/copied engine code.
- The [USDA Wood Handbook 2021](https://research.fs.usda.gov/fpl/wood-handbook)
  supplies orthotropic and moisture/property provenance, not a ready tree
  profile.
- Published wind-tree modal measurements may supply frequency ranges only when
  exact specimen/geometry assumptions and curve extraction are recorded.

Heavy external runs and raw measurements live in the configured research
store. Checked-in summaries bind their exact input/output hashes.

## Selected performance fixture and V0B remainder

The production gate is 1,000 visible, 128 modal, 8 active, one refined and at
most two falling trees. Active trees have at most 128 structural segments and
32 tapered-capsule proxies; a refined felling section has 256 cells. The
incremental vegetation CPU stop target is 2/3 ms p95/p99 on THOTH. The combined
successor `world-dynamics-step` budget is frozen separately before promotion.

Before V6 code, V0B must still freeze:

- the static/shader split within the selected visible count and exact
  node/segment counts per representation tier;
- representation cadence and integer transition budget;
- player/tool/rigid-body interaction trace;
- THOTH memory and transition budgets;
- report-only stress profiles and their non-gating status.

## Exit and stop conditions

V0A is complete with decisions 1 through 19 recorded above. V0 overall is
complete only when every V0B cell is exact, reviewed and linked from the
roadmap. Any missing wood constant, curve/hash, state scale, command trace,
capacity, collision/LOD threshold, memory budget or transition budget is a
blocker rather than an implementation choice.

V1 stops at the first nonfinite, overflow, non-convergence, capacity or exact-
root failure. A candidate formulation may be rejected, but thresholds and
fixtures cannot be widened after its result. If neither candidate can meet the
fixed outer step under a bounded internal cadence, the program remains
research-only until a new explicit cadence/product decision.
