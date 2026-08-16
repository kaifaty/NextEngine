# V0 — Product, profile and evidence closure

Status: `OPEN / BLOCKS CODE`.

## Outcome

Freeze every input that could otherwise be chosen by an implementer after
seeing a result: the product tree, wood and root-anchor profiles, wind and cut
fixtures, numerical state, formulation candidates, corpora, metrics,
thresholds, capacities, external evidence and stop conditions. V0 is
documentation-only and proves no physics.

## Already selected

- one procedural trail-side tree, pinned active and fully loaded;
- one trunk plus bounded major-branch rooted graph; visual twigs/leaves are not
  structural degrees of freedom;
- right-handed MKS, current physical cadence as the outer step and scenario-
  bound analytical wind;
- one authored felling zone, directional notch and back cut through the
  production command path;
- bounded polar section cells, no scalar HP;
- one load-bearing split per substep and one detached PhysX compound body;
- debug structural presentation; no production foliage/fire rendering gate;
- CPU authority and optional GPU correspondence only;
- authored static rigid-tree fallback only before activation.

## Required exact tree definition

V0 must record canonical bytes and hashes for:

- total height, trunk centreline/taper and root-anchor frame;
- complete node/segment IDs, rest directors/curvature and branch graph;
- every segment length, start/end cross-section, mass/drag allocation and
  collision-proxy mapping;
- the felling-zone axial extent and its relation to the trunk section;
- visual asset references only where needed to prove projection alignment;
- maximum nodes, segments, section zones/cells and collision proxies.

The source brief's `5..20` branch suggestion is not a profile. V0 chooses one
exact count and graph. Source order must not affect canonical bytes.

## Required wood and anchor profile

Choose one calibrated living-tree or explicitly synthetic reference profile.
It must include, with units, fixed-point source values and provenance:

- density and mass-distribution rules;
- longitudinal, radial and tangential Young/shear/Poisson closure needed by
  the selected candidate formulations;
- directional tension, compression, shear and fracture/hinge criteria;
- structural damping and any reference moisture condition;
- root-anchor translational/rotational stiffness and failure exclusion;
- tolerance from source material values to the reduced beam/section model.

The USDA Wood Handbook describes clear straight-grained specimens and cannot
by itself calibrate a complete living tree, junction or root anchor. If a
synthetic profile is selected, V0 must name it as an engine test material and
make no species-realism claim.

## Required canonical numeric profile

Freeze exact integer widths, scales, bounds and ties-to-even conversions for:

- node position/orientation and linear/angular velocity;
- strain/curvature/twist, section resultant and damage/history variables;
- wind samples, forces, impulses, work and energy diagnostics;
- section-cell geometry/state and derived area/centroid/moments;
- topology/handoff mass, CoM and linear/angular momentum;
- residuals, iteration counts and stable failure codes.

Freeze outer cadence, internal substeps/iterations for each V1 candidate,
convergence rules and hard iteration/capacity limits. No wall-time deadline may
select iteration count or LOD.

## Required wind, pull and cutting fixtures

Wind profile:

- exact mean vector, gust function, duration and evaluation points;
- drag areas/coefficients and how foliage mass/area is aggregated;
- external-work accounting and output sample cadence.

Structural pull/release profile:

- exact node/point, direction, load curve, hold/release ticks and expected
  analytical/reference outputs.

Cut profile:

- tool/contact identity, geometry and production command trace;
- exact mapping from canonical relative velocity/impulse, direction, grain and
  tool profile to cut work;
- polar ring/sector counts, cell geometry and closed cell states;
- notch/back-cut command sequence and permitted felling-zone bounds;
- failure/hinge criterion and deterministic tie-breaking.

Direct test mutation or a `fell_now` command is forbidden.

## Reference corpus and metrics

| Family | Required cases | Metrics to freeze before V1 |
|---|---|---|
| Beam/rod | static/dynamic cantilever, torsion, axial load, buckling, taper | displacement/rotation curves, mode frequencies, force/moment balance, energy/work residual |
| Junction | one Y-branch under gravity and prescribed load | junction continuity, branch-tip curves, reaction balance |
| Tree/wind | gravity sag, calm, steady wind, gust, pull/release | node trajectories, first modes, damping envelope, external-work-aware residual |
| Cutting | progressive one-sided notch, back cut, asymmetric hinge, invalid/stale command | removed area/moments, load-displacement/failure tick/direction, exact rejection |
| Coupling | body impact/push, ground contact after handoff, capacity/stale faults | impulse closure, penetration, mass/CoM/momentum handoff and atomicity |
| Persistence/LOD | save/resume and every adjacent tier cycle | exact active root; declared discontinuity/history/energy bounds for approximate tiers |

Every scenario needs exact geometry, horizon, output cadence, formula and curve
source/hash. Relative/RMSE and maximum thresholds are set before implementation.
Visual plausibility is diagnostic and cannot replace a curve.

## Independent evidence

- [Discrete Elastic Rods](https://doi.org/10.1145/1360612.1360662) supplies
  buckling, stability and coupled bending/twist validation cases.
- [PyElastica v0.3.3](https://github.com/GazzolaLab/PyElastica/releases/tag/v0.3.3)
  at release commit `aef9e24` is the pinned external Cosserat comparator. Its
  aggregate curves are evidence, not exact particle/node identity and not
  linked/copied engine code.
- The [USDA Wood Handbook 2021](https://research.fs.usda.gov/fpl/wood-handbook)
  supplies orthotropic and moisture/property provenance, not a ready tree
  profile.
- Published wind-tree modal measurements may supply frequency ranges only when
  exact specimen/geometry assumptions and curve extraction are recorded.

Heavy external runs and raw measurements live in the configured research
store. Checked-in summaries bind their exact input/output hashes.

## Required performance fixture

Before V6 code, V0 must freeze:

- visible/static/shader/modal/active/refined tree counts;
- node/segment/proxy/section-cell counts per tier;
- cadence and integer transition budget;
- player/tool/rigid-body interaction trace;
- THOTH p95/p99 CPU, integrated frame, memory and transition budgets;
- report-only stress profiles and production gate.

No target count is inferred from the source brief's phrases “large forest” or
“small subset”.

## Exit and stop conditions

V0 is complete only when all cells above are exact, reviewed and linked from
the roadmap. Any missing wood constant, curve threshold, state scale, command
trace, capacity or performance gate is a blocker rather than an implementation
choice.

V1 stops at the first nonfinite, overflow, non-convergence, capacity or exact-
root failure. A candidate formulation may be rejected, but thresholds and
fixtures cannot be widened after its result. If neither candidate can meet the
fixed outer step under a bounded internal cadence, the program remains
research-only until a new explicit cadence/product decision.

