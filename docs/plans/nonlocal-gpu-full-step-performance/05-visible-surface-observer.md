# NCGP8 corrected-axis visible-surface observer contract

| Field | Value |
| --- | --- |
| Research ID | `NCGP8` revision 1 |
| Status | `FROZEN / DIAGNOSTIC_ONLY / REPORT_ONLY` |
| User decision | Explicitly authorized on 2026-08-31 after NCGP7 |
| Frozen parent | NCGP7 `H7C_INCONCLUSIVE`, exact NCGP6 hydrostatic step-112 witness |
| Architecture status | SPEC-38/ADR-076 remain Proposed; ADR-081 guardrails remain Accepted |
| Engineering consumer | decide whether the corrected standalone solver may resume the remaining 4k correctness corpus under a visible-water quantity of interest |
| Claim class | finite/profile-bound presentation diagnostic; no 50k or performance claim |

## Correction of the parent apparatus

The retained corrected profile has gravity
`(0, 0, -9.81) m/s^2`; therefore its physical vertical axis is `z` and the
horizontal basin plane is `x-y`. NCGP7 instead deposited surface columns in
`x-z` and treated `y` as height. Its three-dimensional bulk mass, density,
velocity and centre-of-mass observables remain valid, but its wet-column and
surface-height observables do not measure the physical free surface and cannot
select a product gate.

This correction does not rewrite the exact NCGP7 execution or reinterpret the
NCGP6 stable-ID p99 failure. NCGP8 freezes a different observer before reading
its result. A vertical-axis control must reject the NCGP7 `y`-height mutation.

## Exact question and claim ceiling

On the exact accepted hydrostatic step-112 CPU/GPU states, do the two particle
sets produce the same visible water geometry when observed through the
mandatory SPEC-38 debug sphere presentation?

NCGP8 may support or reject only this finite quantity of interest. It does not
change FCR2 physics, solver rules, the 128-HVP ceiling, NCGP4/NCGP6 results, or
any public/runtime/PhysX/renderer contract. It does not run dam-break, orifice,
16k, 50k or timing. CPU DFSPH remains the product fallback.

## Frozen witness

Replay the exact NCGP6 hydrostatic input for 112 accepted steps with:

```text
profile_id        = nonlocal-water-50k-v1
dt                = 1/240 s
spacing           = 0.05 m
horizon           = 0.15 m
mass              = 0.125 kg
gravity           = (0, 0, -9.81) m/s^2
solver             = unpreconditioned Steihaug--Toint
total_hvp_budget   = 128
gpu arithmetic     = corrected compensated FCR2 binary32 (hi,lo)
cpu arithmetic     = independent binary64/long-double reference
```

The stable-ID RMSE/p99/maximum witness, corrected/permuted GPU state identity,
particle count, mass, containment, momentum, work and all source/input/binary
identities must reproduce exactly before surface interpretation.

## Canonical visible-sphere observer

Observe the basin orthographically from above, along `-z`. The image plane is
`x-y` over `[0,3.0] x [0,2.5] m`. Each water sample is a presentation sphere:

```text
sphere radius = spacing / 2 = 0.025 m
pixel pitch   = spacing / 4 = 0.0125 m
image size    = 240 x 200 pixels
pixel centre  = ((ix + 0.5) * pitch, (iy + 0.5) * pitch)
```

For a particle centre `(x_i,y_i,z_i)` and pixel centre `(x_p,y_p)`, emit a
visible-depth contribution exactly when

```text
d2 = (x_i-x_p)^2 + (y_i-y_p)^2 <= radius^2
depth = z_i + sqrt(radius^2-d2)
```

The wet mask is the union of covered pixels. The surface depth at a wet pixel
is the maximum contribution. Contributions are sorted by pixel followed by
the binary64 physical value tuple, excluding `SampleId`; reductions use
`long double` intermediates and round the selected depth once to binary64.
This is an unsmoothed diagnostic of the sphere depth buffer, not a new
renderer or an implicit-surface reconstruction claim.

The pixel pitch is fixed from the physical sample spacing, not selected from
the NCGP7 output. No second resolution, alignment search, image translation,
sample reassignment, filtering, hole filling or retry is permitted.

## Visible geometry and topology observables

For CPU image `C` and GPU image `G`, compute:

```text
silhouette_symmetric_difference = |wet_C xor wet_G| / |wet_C union wet_G|

depth_rmse = sqrt(mean_common((depth_G-depth_C)^2))
depth_p95  = nearest-rank p95 of common-pixel absolute depth error
depth_p99  = nearest-rank p99 of common-pixel absolute depth error
depth_max  = maximum common-pixel absolute depth error (diagnostic only)
```

Flood-fill each wet mask with 8-connectivity in increasing pixel order.
`material_component_count` counts components containing at least four pixels;
four pixels are one quarter of the approximately 13-pixel projected area of
one frozen presentation sphere. Also report the largest-component fraction and
the complementary satellite-area fraction. Empty union/common support or an
invalid component partition is an apparatus failure.

## Predeclared classification

The bands are tied to the 25 mm presentation radius and existing 1% physical
field ceiling, not to a measured NCGP8 value.

### `H8A_VISIBLE_SURFACE_SUPPORTED_BOUNDED`

All of these must hold:

```text
silhouette symmetric difference <= 1%
depth RMSE                       <= 6.25 mm   // radius / 4
depth p95                        <= 12.5 mm   // radius / 2
depth p99                        <= 25 mm     // radius
material component count        identical
largest-component fraction diff <= 1%
CPU satellite-area fraction     <= 1%
GPU satellite-area fraction     <= 1%
```

The NCGP7 bulk fields on both frozen grids, exact GPU permutation, mass,
momentum, containment and complete identity/work closure must also pass.

### `H8B_VISIBLE_SURFACE_DIVERGENCE_SUPPORTED_BOUNDED`

Select H8B when the valid observer reaches any clear-error band:

```text
silhouette symmetric difference >= 5%
depth RMSE                       >= 25 mm
depth p95                        >= 50 mm
depth p99                        >= 100 mm
material component mismatch with either satellite-area fraction >= 5%
```

### `H8C_VISIBLE_SURFACE_INCONCLUSIVE`

Every other valid result is H8C. An apparatus or identity failure is
`APPARATUS_INCONCLUSIVE`, not physical evidence.

## Mandatory controls

Before the witness replay:

1. identical input and stable-ID relabelling preserve image, topology and
   result roots exactly;
2. a whole-state `+0.05 m` `z` translation, kept inside the basin, rejects
   H8A through depth while preserving silhouette;
3. a whole-state `+0.05 m` `x` translation rejects H8A through silhouette;
4. an anisotropic fixture whose `z` surface changes while `y` does not rejects
   the `y`-height/wrong-plane mutation;
5. deleting one sample fails exact particle/mass closure;
6. a sparse one-pixel depth tail remains diagnostic through `depth_max`, while
   a complete top-sheet `+0.025 m` mutation rejects through the distribution;
7. strict sphere radius `<`, changed pixel pitch, omitted topology, depth
   record, component record, work count and result-root mutations each change
   or invalidate the result;
8. corrected/permuted GPU image and topology roots are exact on the witness.

Controls execute the real observer/comparator. Binding only a variant label or
expected hash is insufficient.

## Work and evidence closure

Seal separately for CPU, corrected GPU and permuted GPU:

- state, density, active-signature and observer-profile input roots;
- validated samples, pixel candidate tests, emitted/sorted/reduced depth
  records, wet/depth comparisons, quantile reads, flood-fill pixel/edge reads,
  component reductions and root derivations;
- mask, depth, component, work, comparison and final result roots;
- complete retained step receipts and NCGP7 bulk-field roots;
- contract/profile/input/source commit/tree/binary/compiler/environment
  identities, exact command, snapshot transfers and allocated bytes.

The JSON schema is versioned and fail-closed. Two fresh Release builds and
byte-identical executions are required. A positive result additionally
requires retained controls, Compute Sanitizer memcheck/initcheck/synccheck and
one independent read-only review before it can authorize a successor corpus.

## Research basis and bounded implication

- SPHERIC Test 02 defines a 3-D dam-break benchmark by the evolution of the
  free surface and supplies experimental data:
  <https://www.spheric-sph.org/tests/test-02>.
- van der Laan, Green and Sainz, *Screen Space Fluid Rendering with Curvature
  Flow* (I3D 2009, DOI `10.1145/1507149.1507164`) constructs the visible
  particle-fluid surface from sphere depth before optional smoothing:
  <https://research.rug.nl/files/14497408/05c5.pdf>.
- Yu and Turk, *Reconstructing Surfaces of Particle-Based Fluids Using
  Anisotropic Kernels* (TOG 2013, DOI `10.1145/2421636.2421641`) shows that
  higher-quality implicit reconstruction is a separate representation choice:
  <https://diglib.eg.org/items/2d966af2-5428-41ca-b90c-2cf76c1f4b53>.

These sources support measuring visible free-surface geometry and keeping the
observer explicit. They do not validate the Nonlocal formulas, the thresholds,
or production renderer quality.

## Stop and successor boundary

- NCGP4 and NCGP6 remain failed under their own frozen stable-ID gates.
- NCGP7 bulk evidence remains valid; its surface classification is invalidated
  by the axis error and grants no authority.
- H8A plus exact review may authorize only a separately frozen restart of the
  remaining 4k correctness corpus using this visible-surface quantity of
  interest alongside strict same-state and physical-invariant gates.
- H8B blocks the corrected solver on visible geometry. H8C requires a new
  product decision, not threshold or resolution tuning.
- No NCGP8 result is a 50k/full-step/frame-time measurement or runtime
  promotion. CPU DFSPH remains the fallback.
