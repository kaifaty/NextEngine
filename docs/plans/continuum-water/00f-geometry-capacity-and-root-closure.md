# W0F — Geometry, capacity and successor-root closure

Status: `SUCCESSOR_PROFILE_ROOTS_FROZEN / W1_AUTHORIZED / RESEARCH_ONLY`.

## Purpose

W0E proves that the constraint-separated profile can solve the bounded static
outer-box hydro problem. W0F is the minimum profile-closure package required
before that candidate may enter the full W1 corpus. It does not add runtime or
public contracts and cannot award ProductCheck credit.

The W0E implementation and
[evidence](../../development/continuum-water-w0e-constraint-separated-redesign-2026-08-18.md)
are the baseline. The candidate remains
`LOCAL_PROFILE_DISCRIMINATOR_SURVIVED / NOT_SELECTED` until every exit gate
below passes.

## Frozen recommended decisions

### One rooted geometry source

Use an exact integer, axis-aligned geometry manifest containing:

- one closed outer box;
- zero or more internal plane patches with stable feature IDs;
- zero or more closed rectangular opening sets cut from a named plane patch.

The solid set is the plane-patch union minus its declared openings. Opening
membership, closest-feature distance, stable ordering and duplicate handling
must be defined once and consumed by density support, contact and validation.
Do not maintain separate “solver wall” and “collision wall” descriptions.

The first manifest supports only the already selected outer box and
`CW-ORIFICE-001` internal wall. General triangle meshes, moving topology,
curved surfaces and CSG are non-goals for W0F.

### Density support remains discrete and reconstructible

Retain the W0E two-layer, `REST_VOLUME` lattice complement for outer boxes.
Extend the generator to both sides of an internal plane patch while excluding
the opening and deduplicating all edge/corner overlaps in stable integer
coordinate order. The exact rule must be independently reimplemented and
must establish face, aperture-edge and aperture-corner density/gradient
fixtures before any trajectory credit.

If the internal-patch generator cannot meet those fixtures without placing
support samples in the declared opening, stop and return to an implicit
integrated boundary field. Do not fit per-feature multipliers or silently
change the aperture dimensions.

### Contact uses swept analytical features

Generalize the W0E pre-integration velocity constraint to swept particle
spheres against the same stable plane-patch, aperture-edge and aperture-corner
features. Candidate constraints are generated and solved in feature-ID then
sample-ID order. A crossing outside the closed opening must stop at one
particle radius; a centre may cross the wall plane only through the opening
while retaining radius clearance from its edges.

The implementation must report fluid impulse per feature and include its sum
in momentum accounting. No post-integration clamp, teleport, reflected
velocity repair or retry is allowed. Whether multiple simultaneous feature
constraints need a small deterministic active-set solve is decided by exact
face/edge/corner and high-speed crossing tests, not by wall-clock convergence.

### Capacity is explicit

Select `maximum_static_boundary_samples=32768` for the successor research
profile, subject to checked generation of every W1 scenario and exact
threshold cases at `32767`, `32768` and `32769`.

The selected `4 × 2 × 1 m` extent alone needs

```text
84 × 44 × 24 - 80 × 40 × 20 = 24,704
```

two-layer exterior samples. The old `16,384` limit is therefore invalid for
the surviving representation. Dynamic rigid-body samples belong to a
separate W3 capacity and must not consume this static admission budget by
accident.

### The solver profile stays minimal

Freeze these W0E choices unless a named W0F/W1 gate falsifies them:

- projected diagonally preconditioned active-set PCG, maximum 50 density
  iterations and minimum 2;
- unchanged `100,000 ppb` mean positive compression threshold;
- analytical contact after pressure and before integration;
- regular cold lattice, zero velocity and no continuation state;
- stabilization `NONE`: no density map, XSPH, viscosity, surface tension,
  settling output, retry or positional repair;
- canonical stable ID plus integer position/velocity as the complete published
  water state.

## Roots to issue

W0F must produce new, domain-separated SHA-256 roots for:

1. this successor profile document;
2. the float/numeric profile including projected-PCG and contact ordering;
3. the execution profile and admitted capacities;
4. the geometry/corpus manifest;
5. every scenario manifest, including unchanged free-fall semantics;
6. bounded independent fixtures used before the external corpus.

The same coherent closure must update the candidate max-20/profile wording in
Proposed SPEC-38 and ADR-076 to the successor operations before those roots are
used by W1. Both documents remain `Proposed`; the update does not activate a
runtime owner or ProductCheck.

The W0B roots stay byte-for-byte immutable and labelled rejected-profile
evidence. A new root may not inherit W0B validation by name or implication.

## Required discriminators

1. Production and independent geometry paths match all solid/opening
   classifications, sample records, closest features and roots.
2. Initial density and gradient fixtures match independently at an outer face,
   internal face, aperture edge and aperture corner.
3. Swept contact matches an independent calculator for separating, resting,
   direct impact, high-speed crossing, aperture pass, edge graze and
   simultaneous corner contacts.
4. The outer-box 24/1200 W0E transcript remains exact under the rooted profile.
5. An orifice preflight proves no false wall crossing, no blocked legal
   crossing, exact left/right count conservation and reaction closure.
6. Static boundary capacity and per-row neighbor capacities pass their exact
   below/equal/above threshold cases.
7. Two identical clean runs produce the same bounded roots under the exact
   execution flags.

## Exit and stop rules

W0F exits only with `SUCCESSOR_PROFILE_ROOTS_FROZEN / W1_AUTHORIZED`. That
status authorizes the serial W1 corpus; it is not `CONTINUUM-WATER-REF-P1` and
does not activate the main roadmap.

- If aperture density support fails local partition/gradient fixtures, reject
  the explicit internal-patch complement and evaluate one implicit boundary
  field as a new candidate. Do not tune feature scales.
- If swept contact cannot resolve simultaneous features with a bounded exact
  schedule, stop and specify a deterministic local complementarity solve.
- If the outer W0E transcript changes without an intentional rooted operation,
  treat it as a regression.
- If `32,768` is insufficient for a selected W1 static scenario, recompute one
  admitted power-of-two capacity from all frozen scenarios and issue a new
  execution root; do not allocate opportunistically.
- If any independent path disagrees, issue no successor roots.

W2, WG, dynamic PhysX coupling, persistence, public schemas and production
promotion remain blocked until the complete W1 gate passes.

## Successor root projections

The following byte projections are the complete W0F successor inputs. Hashes
include the marker lines and their final LF, but exclude the Markdown fences.
They define a research oracle only. SPEC-38 and ADR-076 remain `Proposed`, and
`CONTINUUM-WATER-REF-P1` receives no credit from these roots.

```text
SUCCESSOR_FLOAT_PROFILE_V1_BEGIN
profile.id=constraint-separated-support-pcg-v1
scalar=binary64-private-within-one-substep
canonical_state=sample-id,position-i64-um,velocity-i64-um-per-s
publication=round-to-nearest-ties-to-even-once-per-accepted-substep
kernel=cubic-spline;support-radius-bits=0x3fb999999999999a
rest-volume-bits=0x3f20624dd2f1a9fc
time-step-bits=0x3f71111111111111
gravity-bits=0x40239eb851eb851f;toward-negative-y
divergence=relaxed-jacobi;minimum=1;maximum=20;mean-positive-error-ppb=1000000
density=projected-diagonal-pcg-active-set-restart;minimum=2;maximum=50;mean-positive-error-ppb=100000
contact=analytical-swept-sphere-after-pressure-before-integration
contact.outer=component-active-set;feature-order=0,1,2,3,4,5
contact.internal=plane-then-aperture-edge-then-aperture-corner;feature-id-order;maximum-constraints=8
contact.direction-rounding-guard-bits=0x3d00000000000000
contact.response=frictionless-nonpenetrating-velocity-projection
initialization=regular-cold-lattice;velocity=zero;continuation-state=none
stabilization=none;density-map=off;xsph=off;viscosity=off;surface-tension=off;retry=off;position-repair=off
SUCCESSOR_FLOAT_PROFILE_V1_END
```

```text
SUCCESSOR_EXECUTION_PROFILE_V1_BEGIN
toolchain=rustc-1.97.1-8bab26f4f68e0e26f0bb7960be334d5b520ea452;llvm=22.1.6
targets=x86_64-unknown-linux-gnu,x86_64-pc-windows-msvc
rustflags=-Ctarget-cpu=x86-64|-Ctarget-feature=-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,-fma|-Cllvm-args=-fp-contract=off
cargo-profile=water-oracle;opt-level=3;codegen-units=1;lto=false;incremental=false;overflow-checks=true;debug-assertions=false;panic=abort
sample-order=ascending-stable-sample-id
neighbor-membership=integer-squared-distance-less-than-or-equal-to-10000000000-um2
fluid-neighbor-visibility=exact-rational-segment-crossing-against-rooted-solid-set
boundary-order=ascending-position-then-oriented-support-then-feature-id
contact-order=feature-id-then-sample-id
maximum-samples=50000
maximum-static-boundary-samples=32768
maximum-dynamic-rigid-boundary-samples=separate-w3-capacity-not-admitted-here
maximum-fluid-row-neighbors=128
maximum-directed-fluid-neighbors=6400000
maximum-boundary-row-neighbors=128
maximum-steps=7200
maximum-report-bytes=16777216
maximum-reference-input-bytes=33554432
maximum-decoded-heap-bytes=536870912
SUCCESSOR_EXECUTION_PROFILE_V1_END
```

```text
SUCCESSOR_CORPUS_MANIFEST_V1_BEGIN
geometry.schema=nextengine.continuum-water.axis-aligned-geometry.v1
geometry.outer-feature-ids=0,1,2,3,4,5
geometry.internal-patch-feature-id=16
geometry.aperture-edge-feature-ids=17,18,19,20
geometry.aperture-corner-feature-ids=21,22,23,24
geometry.opening-membership=closed
geometry.density-support=two-layer-rest-volume-lattice-complement
geometry.internal-support=two-sided-oriented-and-opening-excluded
scenario.CW-HYDRO-001.kind=hydrostatic-column
scenario.CW-HYDRO-001.box_um=(0,0,0)..(1000000,1000000,1000000)
scenario.CW-HYDRO-001.boundary=successor-closed-outer-box
scenario.CW-HYDRO-001.fluid=20,15,20;(25000,25000,25000);(0,0,0)
scenario.CW-HYDRO-001.steps=1200
scenario.CW-HYDRO-001.outputs=0..1200/every=24
scenario.CW-HYDRO-001.reference=analytical-mass-symmetry-work-and-splishsplash-aggregate
scenario.CW-FREEFALL-001.kind=pre-impact-free-fall
scenario.CW-FREEFALL-001.box_um=(0,0,0)..(1000000,2000000,1000000)
scenario.CW-FREEFALL-001.boundary=successor-closed-outer-box
scenario.CW-FREEFALL-001.fluid=10,10,10;(275000,1275000,275000);(0,0,0)
scenario.CW-FREEFALL-001.steps=96
scenario.CW-FREEFALL-001.outputs=0..96/every=1
scenario.CW-FREEFALL-001.recurrence=publish-vy-plus-gravity-then-publish-y-plus-published-vy-over-240
scenario.CW-FREEFALL-001.reference=exact-canonical-semi-implicit-gravity-recurrence
scenario.CW-DAMBREAK-001.kind=three-dimensional-dam-break
scenario.CW-DAMBREAK-001.box_um=(0,0,0)..(4000000,1000000,1000000)
scenario.CW-DAMBREAK-001.boundary=successor-closed-outer-box
scenario.CW-DAMBREAK-001.fluid=20,15,20;(25000,25000,25000);(0,0,0)
scenario.CW-DAMBREAK-001.steps=720
scenario.CW-DAMBREAK-001.outputs=0..720/every=4
scenario.CW-DAMBREAK-001.reference=splishsplash-aggregate
scenario.CW-STILL-001.kind=long-horizon-still-tank
scenario.CW-STILL-001.box_um=(0,0,0)..(1000000,1000000,1000000)
scenario.CW-STILL-001.boundary=successor-closed-outer-box
scenario.CW-STILL-001.fluid=20,10,20;(25000,25000,25000);(0,0,0)
scenario.CW-STILL-001.steps=7200
scenario.CW-STILL-001.outputs=0..7200/every=240
scenario.CW-STILL-001.reference=analytical-mass-symmetry-work-and-repeat-root
scenario.CW-ORIFICE-001.kind=sealed-two-chamber-orifice-transfer
scenario.CW-ORIFICE-001.box_um=(0,0,0)..(2000000,1000000,1000000)
scenario.CW-ORIFICE-001.boundary=successor-outer-box-plus-patch-16
scenario.CW-ORIFICE-001.aperture_um=x=1000000;y=200000..400000;z=400000..600000;opening-set-closed
scenario.CW-ORIFICE-001.fluid=20,15,20;(25000,25000,25000);(0,0,0)
scenario.CW-ORIFICE-001.steps=720
scenario.CW-ORIFICE-001.outputs=0..720/every=4
scenario.CW-ORIFICE-001.reference=splishsplash-aggregate-and-exact-partition-accounting
scenario.CW-SEALED-001.kind=nominal-product-boundary-stress
scenario.CW-SEALED-001.box_um=(-2000000,0,-1000000)..(2000000,1000000,1000000)
scenario.CW-SEALED-001.boundary=successor-closed-outer-box;static-support-count=24704
scenario.CW-SEALED-001.fluid=80,15,40;(-1975000,25000,-975000);(500000,0,0)
scenario.CW-SEALED-001.steps=480
scenario.CW-SEALED-001.outputs=0..480/every=8
scenario.CW-SEALED-001.reference=exact-count-mass-boundary-and-repeat-root
scenario.CW-ORDER-001.kind=input-storage-order-invariance
scenario.CW-ORDER-001.box_um=(0,0,0)..(600000,600000,600000)
scenario.CW-ORDER-001.boundary=successor-closed-outer-box
scenario.CW-ORDER-001.fluid=12,8,12;(25000,25000,25000);(0,0,0)
scenario.CW-ORDER-001.steps=240
scenario.CW-ORDER-001.outputs=0..240/every=24
scenario.CW-ORDER-001.storage-orders=identity,reverse,affine-257k-plus-17-mod-1152
scenario.CW-ORDER-001.reference=exact-trajectory-root-equality
case.CW-CAP-SAMPLES=49999-pass,50000-pass,50001-WATER_SAMPLE_CAPACITY_EXCEEDED
case.CW-CAP-STATIC-BOUNDARY=32767-pass,32768-pass,32769-WATER_BOUNDARY_CAPACITY_EXCEEDED
case.CW-CAP-FLUID-ROW=127-pass,128-pass,129-WATER_NEIGHBOR_CAPACITY_EXCEEDED
case.CW-CAP-BOUNDARY-ROW=127-pass,128-pass,129-WATER_BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED
case.CW-CAP-STEPS=7199-pass,7200-pass,7201-WATER_STEP_CAPACITY_EXCEEDED
SUCCESSOR_CORPUS_MANIFEST_V1_END
```

```text
SUCCESSOR_FIXTURE_MANIFEST_V1_BEGIN
fixture.geometry=outer-face,internal-solid,closed-opening,aperture-edge,aperture-corner
fixture.density=outer-face:sample-3000,internal-face:sample-1019,aperture-edge:sample-1819,aperture-corner:sample-1779
fixture.contact=face-separating,face-resting,face-direct-impact,face-high-speed-crossing,aperture-pass,aperture-edge-graze,aperture-edge-impact,aperture-corner-simultaneous
fixture.hydro-regression=steps-24-and-1200;w0e-canonical-state-transcript-exact
fixture.orifice-preflight=24-steps;strict-radius-clearance;legal-transfer-positive;left-plus-right-equals-6000;reaction-residual-at-most-1-percent
fixture.repeatability=two-clean-identical-profile-runs-have-identical-bounded-roots
SUCCESSOR_FIXTURE_MANIFEST_V1_END
```
