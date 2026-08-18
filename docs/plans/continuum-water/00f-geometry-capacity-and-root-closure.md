# W0F — Geometry, capacity and successor-root closure

Status: `REQUIRED / NOT_STARTED / RESEARCH_ONLY`.

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
