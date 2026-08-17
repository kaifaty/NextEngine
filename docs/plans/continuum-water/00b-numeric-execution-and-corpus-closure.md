# W0B — Numeric execution and corpus closure

Status: `COMPLETE / DOCUMENTATION`; W1 is `READY / NOT_STARTED`.

## Outcome and authority

This document removes the remaining pre-code choices from W0A. It freezes the
serial `Continuum Water V1` research profile, not a generic world-physics
service and not a production runtime backend. The first implementation may now
be the private safe-Rust DFSPH oracle described by
[W1](01-serial-cpu-dfsph-oracle.md).

W0B proves no solver result. Every `CONTINUUM-*` ProductCheck remains
`NOT_RUN`; SPEC-38 and ADR-076 remain Proposed; no public schema, schedule,
save format, PhysX behavior or shipped claim changes. A code change to any
frozen value below creates a new profile and reopens W0B before evidence is
interpreted.

The authoritative specification input is the complete byte content of this
file. Its SHA-256 is recorded by the
[water roadmap](README.md#frozen-w0b-inputs). The two smaller roots below are
machine-facing projections for preflight and reports.

## Canonical float execution profile

The canonical bytes are the ASCII lines in the following block, including the
`BEGIN`/`END` lines and the final LF, excluding the Markdown fences. The root
is `SHA-256("nextengine.continuum-water.float-profile.v1\0" || bytes)`.

```text
FLOAT_PROFILE_V1_BEGIN
profile_id=nextengine.continuum-water.dfsph.serial.v1
profile_revision=1
operation_precision=ieee-754-binary64
rustc_release=1.97.1
rustc_commit=8bab26f4f68e0e26f0bb7960be334d5b520ea452
cargo_release=1.97.1
cargo_commit=c980f4866
llvm_release=22.1.6
targets=x86_64-unknown-linux-gnu,x86_64-pc-windows-msvc
cargo_profile=water-oracle
opt_level=3
codegen_units=1
lto=off
incremental=false
overflow_checks=true
debug_assertions=false
panic=abort
target_cpu=x86-64
target_features=-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,-fma
llvm_fp_contract=off
rounding=nearest-ties-to-even
subnormals=gradual-underflow
fma=forbidden
simd=forbidden
fast_math=forbidden
external_libm=forbidden
sqrt=rust-f64-sqrt-with-startup-golden-probes
transcendentals=forbidden
reduction=serial-left-fold
vector_component_order=x,y,z
neighbor_order=fluid-sample-id-then-boundary-sample-id
jacobi_update=old-vector-to-new-vector
publication=checked-ieee-decode-nearest-ties-to-even
rho0_bits=0x408f400000000000
particle_radius_bits=0x3f9999999999999a
lattice_spacing_bits=0x3fa999999999999a
support_radius_bits=0x3fb999999999999a
uniform_mass_bits=0x3fc0000000000000
rest_volume_bits=0x3f20624dd2f1a9fc
dt_bits=0x3f71111111111111
gravity_magnitude_bits=0x40239eb851eb851f
pi_bits=0x400921fb54442d18
solver_epsilon_bits=0x3ee4f8b588e368f1
relaxation_bits=0x3fe0000000000000
micrometres_per_metre_bits=0x412e848000000000
kernel_outside_support=zero-if-q-greater-than-one
pressure_acceleration_threshold=strict-abs-greater-than-solver-epsilon
inverse_step=precomputed-reciprocal-multiply
density_min_iterations=2
density_max_iterations=20
density_threshold_ppb=100000
divergence_min_iterations=1
divergence_max_iterations=20
divergence_threshold_ppb=1000000
FLOAT_PROFILE_V1_END
```

Float-profile SHA-256:
`d6152c575fd88bb53d0d63d0e1e8b2e86a82465268fc8d102ebdb1d77c092d63`.

### Build and process contract

W1 adds a Cargo profile named `water-oracle` with exactly the profile values
above. The logical rustflags, in order, are:

```text
-Ctarget-cpu=x86-64
-Ctarget-feature=-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,-fma
-Cllvm-args=-fp-contract=off
```

The evidence invocation uses the repository's pinned toolchain, `--locked`,
`--profile water-oracle`, and one of the two exact `--target` triples. It
rejects ambient `RUSTFLAGS`, `RUSTDOCFLAGS` or Cargo profile overrides unless
their normalized values are exactly the frozen values. `target-cpu=native`,
runtime feature dispatch, SIMD intrinsics, `mul_add`, inline assembly, unsafe
floating-environment mutation and GPU execution are forbidden.

The oracle uses only binary64 `+`, `-`, `*`, `/`, comparisons, bit inspection,
`abs` and `f64::sqrt`. Cubes and squares are named repeated multiplications;
`powf`, `powi`, trigonometric, exponential and other library mathematics are
forbidden. Every clamp is an explicit finite-checked comparison, so NaN
behavior of `min`/`max` never selects a result. Parentheses and named
temporaries below are operation-order requirements, not illustrative algebra.
The notation `max(value, 0)` means
`if value > 0.0 { value } else { 0.0 }`, so its zero is always positive.

At process start, values assembled from runtime inputs passed through
`std::hint::black_box` MUST pass all of these bit checks before a scenario is
decoded:

| Probe | Required result bits |
|---|---:|
| `1 + 2^-53` | `0x3ff0000000000000` |
| `1 + 3*2^-53` | `0x3ff0000000000002` |
| `MIN_POSITIVE * 0.5` | `0x0008000000000000` |
| `(MIN_SUBNORMAL * 2) / 2` | `0x0000000000000001` |
| `(1 + 2^-27) * (1 - 2^-27) - 1` | positive zero, `0x0000000000000000` |
| `sqrt(0)`, `sqrt(1)`, `sqrt(2)`, `sqrt(4)` | `0`, `0x3ff0000000000000`, `0x3ff6a09e667f3bcd`, `0x4000000000000000` |

The multiplication and subtraction in the non-contraction probe are separate
named operations. Any mismatch returns
`WATER_FLOAT_ENVIRONMENT_MISMATCH`; it is never a skipped warning.

### Exact constants and publication conversion

Profile constants are constructed with `f64::from_bits`, not reparsed decimal
text. With `H = support_radius`, the required startup derivation order and
golden bits are:

```text
H2 = H * H                                      0x3f847ae147ae147c
H3 = H2 * H                                     0x3f50624dd2f1a9fd
kernel_denominator = PI * H3                    0x3f69bc65b68b71c4
kernel_k = 8.0 / kernel_denominator             0x40a3e4f54b370dcf
kernel_l = 48.0 / kernel_denominator            0x40cdd76ff0d294b6
rest_volume = uniform_mass / rho0               0x3f20624dd2f1a9fc
dt2 = dt * dt                                   0x3ef23456789abcdf
inv_dt = 1.0 / dt                               0x406e000000000000
inv_dt2 = 1.0 / dt2                             0x40ec200000000000
```

Canonical positions are limited to `[-16_000_000, +16_000_000] µm` per axis;
velocities are limited to `[-64_000_000, +64_000_000] µm/s` per axis. These
integers convert exactly to binary64 before division by the exact binary64
integer `1_000_000`. A value outside the lab bound is
`WATER_NUMERIC_OVERFLOW` even if it fits in `i64`.

Publication does not use a float-to-integer cast or a platform rounding
function. For a finite binary64 value `v`, conversion to units per metre is:

1. decode sign, raw exponent and fraction from `v.to_bits()`; for a subnormal
   use `significand = fraction`, `exponent = -1074`, and for a normal use
   `significand = 2^52 + fraction`,
   `exponent = raw_exponent - 1023 - 52`, so that
   `abs(v) = significand * 2^exponent` exactly;
2. checked-multiply the significand by the integer scale (`1_000_000` for
   canonical state or `1_000_000_000` for ppb metrics);
3. shift when the exponent is non-negative; otherwise divide by the exact
   power of two, compare twice the remainder with the divisor and round a tie
   to an even quotient;
4. apply the sign, normalize either zero to integer zero and checked-convert to
   the destination range.

Nonfinite input is `WATER_NONFINITE_VALUE`; any checked intermediate or result
overflow is `WATER_NUMERIC_OVERFLOW`. Each position and velocity component is
converted once after the complete substep. The next substep decodes only that
published integer state; no private float, multiplier or warm-start value
survives.

## Canonical geometry, neighborhood and boundary profile

### Fluid identity and lattice

All scenario coordinates below are signed integer micrometres. A fluid block
with dimensions `(nx, ny, nz)` has centres

```text
x = first_x + 50_000 * ix
y = first_y + 50_000 * iy
z = first_z + 50_000 * iz
SampleId = ((iy * nz + iz) * nx + ix)
```

for ascending `iy`, then `iz`, then `ix`; `ix` is the fastest coordinate.
`SampleId` is a scenario-local `u32`. Input storage order is never identity.
Duplicate IDs are rejected before allocation; coincident positions with
different IDs are legal inputs and are handled by a zero gradient.

### Sealed analytical boundary

Every outer box is closed on all six faces, including the non-rendered top.
Its exact plane set is authoritative geometry. W1 deterministically derives a
one-layer Akinci-style quadrature cache at every `50_000 µm` lattice point on
the planes. For an internal wall, it derives points on that exact plane except
inside the declared aperture. It then sorts the union lexicographically by
`(x, y, z)`, removes coordinate duplicates and assigns consecutive
`BoundarySampleId` values from zero. Boundary velocity is exactly zero.

For each boundary sample `b`, canonical boundary neighbors are every other
boundary sample within the inclusive exact integer radius
`dx² + dy² + dz² <= 100_000²`. In ascending `BoundarySampleId` order:

```text
denom_b = W(0)
denom_b = left_fold(denom_b + W(x_b - x_neighbor))
V_b = 1.0 / denom_b
```

`V_b` must be positive and finite. Boundary samples and volumes are
reconstructible cache, not water state. The product basin generates exactly
`11_202` boundary samples. The `CW-ORIFICE-001` union generates exactly
`4_338`.

The ideal centre-clearance surface is one particle radius from every solid
plane or solid aperture edge. For a canonical integer centre, squared distance
to the union of axis-aligned solid plane patches is computed with checked
`i128`; inside the aperture projection it includes squared distance to the
nearest aperture edge. The pass branch is the exact integer comparison
`distance_squared >= 22_500² µm²`, equivalent to penetration
`<= 2_500 µm`. Reported penetration may use the frozen square root and exact
ties-to-even conversion, but cannot select the branch. An outer-box centre
outside the box is the stronger `WATER_BOUNDARY_ESCAPE`. Crossing the internal
wall is legal only through the aperture; otherwise the clearance rule applies.
No projection, teleport, clamp or reflected-velocity repair is allowed after
the solve.

### Neighborhood

Grid-cell width is exactly `100_000 µm`. Cell coordinates are Euclidean
integer division of the prior canonical position by that width. Entries sort
by `(cell_x, cell_y, cell_z, SampleId)`. For a fluid sample, candidates come
from the 27 offsets ordered by `dz = -1..1`, then `dy = -1..1`, then
`dx = -1..1`; the final fluid candidate set is sorted by `SampleId` and
deduplicated. The sample itself is excluded. Boundary candidates are sorted
separately by `BoundarySampleId`.

Neighbor admission uses the same inclusive integer squared-distance test as
the boundary cache. Float distance never decides membership. Each fluid row
contains at most `128` fluid-plus-boundary neighbors. All scenarios reserve at
most `6_400_000` directed fluid-row entries. Boundary-volume construction
reserves at most `128` neighbors per boundary sample and `2_097_152` directed
boundary-row entries. An excess fails before a partial row is published.

## Exact serial DFSPH formulation

The following is the W1 formulation. It follows the hash-bound DFSPH sources
below while making the selected no-warm-start, one-fluid, fixed-boundary case
and operation order explicit.

### Kernel and reductions

For exact integer displacement `d = x_i - x_j`, decode its components to
binary64 metres and compute:

```text
r2 = ((dx * dx) + (dy * dy)) + (dz * dz)
r = sqrt(r2)
q = r / H
```

Membership is exact integer geometry. After binary64 decoding, the cubic
kernel and gradient are both zero if `q > 1`; otherwise the kernel is:

```text
q <= 0.5: q2 = q*q; q3 = q2*q; W = K * (((6*q3) - (6*q2)) + 1)
q >  0.5: t = 1 - q; t2 = t*t; t3 = t2*t; W = K * (2*t3)
```

`gradW(d)` is zero when integer `r2 == 0`; otherwise
`gradq = (d / r) / H` component by component, then:

```text
q <= 0.5: gradW = ((L * q) * ((3*q) - 2)) * gradq
q >  0.5: t = 1 - q; t2 = t*t; gradW = (-(L * t2)) * gradq
```

All vector dot products are
`((a_x*b_x) + (a_y*b_y)) + (a_z*b_z)`. A row starts with its self term where
specified, folds fluid neighbors by ascending `SampleId`, then boundary
neighbors by ascending `BoundarySampleId`. Global means fold rows by ascending
`SampleId`. No pairwise, tree, compensated or library sum may replace these
folds in W1.

### Reconstructed density and factor

Let `V = 0.000125 m³`, `G_ij = gradW(x_i - x_j)`, and `V_b` be the frozen
boundary pseudo-volume:

```text
rho_ratio_i = V * W(0)
rho_ratio_i += sum_fluid_j(V * W_ij)
rho_ratio_i += sum_boundary_b(V_b * W_ib)
rho_i = rho0 * rho_ratio_i

sum_sq_i = 0
central_i = [0, 0, 0]
for fluid j:
    g = -(V * G_ij)
    sum_sq_i += dot(g, g)
    central_i -= g
for boundary b:
    g = -(V_b * G_ib)
    central_i -= g
denom_i = sum_sq_i + dot(central_i, central_i)
alpha_i = (1 / denom_i) when denom_i > 1e-5, otherwise 0
```

Every scalar and vector is finite-checked at the end of each named assignment.

### Pressure acceleration and matrix action

For a complete old multiplier vector `k`, compute all acceleration rows before
updating any multiplier:

```text
a_i = [0, 0, 0]
for fluid j:
    p_sum = k_i + k_j
    if abs(p_sum) > solver_epsilon:
        a_i += (-(V * p_sum)) * G_ij
if abs(k_i) > solver_epsilon:
    a_i += sum_boundary_b((-(V_b * k_i)) * G_ib)

A_i = 0
A_i += sum_fluid_j(V * dot(a_i - a_j, G_ij))
A_i += sum_boundary_b(V_b * dot(a_i, G_ib))
```

Fluid and boundary terms are accumulated in the canonical orders above. A
Jacobi iteration writes every `k_next_i` from the complete old `k`/`a` arrays,
then swaps arrays after the global metric is computed. Relaxation is exactly
`0.5`.

### One substep

One accepted substep uses this fixed sequence:

1. decode prior canonical samples; validate identity/bounds; reconstruct the
   grid, neighbors, density, boundary volumes and `alpha`;
2. run the divergence solve on prior velocities;
3. add gravity as
   `v_y = v_y + (dt * -gravity_magnitude)`; there are no other forces;
4. run the density solve;
5. update positions component-wise as `x = x + (dt * v)`;
6. validate finite values, bounds, outer escape and clearance penetration;
7. publish every position and velocity through the one exact integer
   conversion, sort by `SampleId`, compute roots and discard float scratch.

The divergence source for `i` is:

```text
d_i = sum_fluid_j(V * dot(v_i - v_j, G_ij))
d_i += sum_boundary_b(V_b * dot(v_i, G_ib))
```

If the total fluid-plus-boundary neighbor count is below `20`, set `d_i = 0`
for the divergence solve only. Otherwise clamp negative `d_i` to zero for its
initial multiplier. `dt2 = dt*dt`, `inv_dt = 1/dt` and
`inv_dt2 = 1/dt2` are computed once with the golden bits above. With
`factor_v = alpha_i * inv_dt`:

```text
k_i = max(d_i, 0) * factor_v
each iteration:
    compute a and A from old k
    s_i = -d_i
    k_next_i = max(k_i - 0.5 * ((s_i - (dt*A_i)) * factor_v), 0)
    error_i = max(d_i + (dt*A_i), 0)
```

After each complete iteration, convert
`dt * (left_fold(error_i) / sample_count)` to ppb with the exact integer-scale
converter. Stop only when at least one iteration completed and the value is
`<= 1_000_000`; iteration 20 above threshold is
`WATER_DIVERGENCE_NONCONVERGENCE`. Recompute acceleration from the accepted
final multiplier and apply `v = v + dt*a`.

After gravity, compute

```text
delta_i = sum_fluid_j(V * dot(v_i - v_j, G_ij))
delta_i += sum_boundary_b(V_b * dot(v_i, G_ib))
rho_adv_i = rho_ratio_i + (dt * delta_i)
factor_p = alpha_i * inv_dt2
k_i = max(rho_adv_i - 1, 0) * factor_p
```

The density iterations are:

```text
compute a and A from old k
s_i = 1 - rho_adv_i
k_next_i = max(k_i - 0.5 * ((s_i - (dt2*A_i)) * factor_p), 0)
error_i = max(rho_adv_i + (dt2*A_i) - 1, 0)
```

After each complete iteration, convert
`left_fold(error_i) / sample_count` to ppb. Stop only after at least two
iterations and when the value is `<= 100_000`; iteration 20 above threshold is
`WATER_DENSITY_NONCONVERGENCE`. Recompute and apply the accepted final pressure
acceleration before position integration.

Warm start, viscosity, vorticity, surface tension, emitter/sink behavior,
adaptive sampling, CFL, variable step, repair projection and retries are not
merely disabled defaults: they are absent from this profile.

## Canonical roots

All integers use little-endian two's-complement bytes. Hash fields are raw 32
bytes, not hex text. `w0b_document_root` is the SHA-256 recorded in the water
roadmap after this file is final.

```text
execution_profile_root = SHA-256(
  "nextengine.continuum-water.execution-profile.v1\0" ||
  w0b_document_root || float_profile_root || corpus_root)

frame_root = SHA-256(
  "nextengine.continuum-water.frame.v1\0" ||
  execution_profile_root || scenario_root || step_u32 || sample_count_u32 ||
  repeated ascending SampleId {
    sample_id_u32 || position_i64[3] || velocity_i64[3]
  })

trajectory_root = SHA-256(
  "nextengine.continuum-water.trajectory.v1\0" ||
  frame_count_u32 ||
  repeated every accepted step including step zero {
    step_u32 || frame_root
  })
```

The scenario root uses domain
`nextengine.continuum-water.scenario.v1\0` followed by the exact LF-terminated
ASCII lines beginning `scenario.<ID>.` in the corpus block, in document order.
The trajectory includes every substep even when metrics are sampled at a lower
cadence. Root construction is streaming and never requires a trajectory dump.

## Frozen corpus manifest

Tuple order is always `(x,y,z)`. `box` is inclusive plane coordinates in
micrometres. `fluid` is `nx,ny,nz;first-centre;initial-velocity-µm/s`.
`outputs=a..b/every=n` includes both endpoints. All runs start at step zero.

The canonical bytes are the ASCII lines in this block, including markers and
the final LF. Its root is
`SHA-256("nextengine.continuum-water.corpus.v1\0" || bytes)`.

```text
CORPUS_MANIFEST_V1_BEGIN
corpus_id=nextengine.continuum-water.reference.v1
corpus_revision=1
profile_id=nextengine.continuum-water.dfsph.serial.v1
maximum_samples=50000
maximum_boundary_samples=16384
maximum_neighbors_per_fluid_row=128
maximum_directed_fluid_neighbors=6400000
maximum_neighbors_per_boundary_row=128
maximum_directed_boundary_neighbors=2097152
maximum_steps_per_run=7200
maximum_report_bytes=16777216
maximum_reference_input_bytes=33554432
maximum_decoded_heap_bytes=536870912
stress_maximum_samples=100000
stress_maximum_directed_fluid_neighbors=12800000
stress_maximum_decoded_heap_bytes=1073741824
scenario.CW-HYDRO-001.kind=hydrostatic-column
scenario.CW-HYDRO-001.box_um=(0,0,0)..(1000000,1000000,1000000)
scenario.CW-HYDRO-001.boundary=closed-outer-shell
scenario.CW-HYDRO-001.fluid=20,15,20;(25000,25000,25000);(0,0,0)
scenario.CW-HYDRO-001.steps=1200
scenario.CW-HYDRO-001.outputs=0..1200/every=24
scenario.CW-HYDRO-001.reference=analytical-mass-symmetry-work-and-splishsplash-aggregate
scenario.CW-FREEFALL-001.kind=pre-impact-free-fall
scenario.CW-FREEFALL-001.box_um=(0,0,0)..(1000000,2000000,1000000)
scenario.CW-FREEFALL-001.boundary=closed-outer-shell
scenario.CW-FREEFALL-001.fluid=10,10,10;(275000,1275000,275000);(0,0,0)
scenario.CW-FREEFALL-001.steps=96
scenario.CW-FREEFALL-001.outputs=0..96/every=1
scenario.CW-FREEFALL-001.reference=exact-canonical-semi-implicit-gravity-recurrence
scenario.CW-DAMBREAK-001.kind=three-dimensional-dam-break
scenario.CW-DAMBREAK-001.box_um=(0,0,0)..(4000000,1000000,1000000)
scenario.CW-DAMBREAK-001.boundary=closed-outer-shell
scenario.CW-DAMBREAK-001.fluid=20,15,20;(25000,25000,25000);(0,0,0)
scenario.CW-DAMBREAK-001.steps=720
scenario.CW-DAMBREAK-001.outputs=0..720/every=4
scenario.CW-DAMBREAK-001.reference=splishsplash-aggregate
scenario.CW-STILL-001.kind=long-horizon-still-tank
scenario.CW-STILL-001.box_um=(0,0,0)..(1000000,1000000,1000000)
scenario.CW-STILL-001.boundary=closed-outer-shell
scenario.CW-STILL-001.fluid=20,10,20;(25000,25000,25000);(0,0,0)
scenario.CW-STILL-001.steps=7200
scenario.CW-STILL-001.outputs=0..7200/every=240
scenario.CW-STILL-001.reference=analytical-mass-symmetry-work-and-repeat-root
scenario.CW-ORIFICE-001.kind=sealed-two-chamber-orifice-transfer
scenario.CW-ORIFICE-001.box_um=(0,0,0)..(2000000,1000000,1000000)
scenario.CW-ORIFICE-001.boundary=closed-outer-shell-plus-wall-x-1000000
scenario.CW-ORIFICE-001.aperture_um=x=1000000;y=200000..400000;z=400000..600000;opening-set-closed
scenario.CW-ORIFICE-001.fluid=20,15,20;(25000,25000,25000);(0,0,0)
scenario.CW-ORIFICE-001.steps=720
scenario.CW-ORIFICE-001.outputs=0..720/every=4
scenario.CW-ORIFICE-001.reference=splishsplash-aggregate-and-exact-partition-accounting
scenario.CW-SEALED-001.kind=nominal-product-boundary-stress
scenario.CW-SEALED-001.box_um=(-2000000,0,-1000000)..(2000000,1000000,1000000)
scenario.CW-SEALED-001.boundary=closed-outer-shell
scenario.CW-SEALED-001.fluid=80,15,40;(-1975000,25000,-975000);(500000,0,0)
scenario.CW-SEALED-001.steps=480
scenario.CW-SEALED-001.outputs=0..480/every=8
scenario.CW-SEALED-001.reference=exact-count-mass-boundary-and-repeat-root
scenario.CW-ORDER-001.kind=input-storage-order-invariance
scenario.CW-ORDER-001.box_um=(0,0,0)..(600000,600000,600000)
scenario.CW-ORDER-001.boundary=closed-outer-shell
scenario.CW-ORDER-001.fluid=12,8,12;(25000,25000,25000);(0,0,0)
scenario.CW-ORDER-001.steps=240
scenario.CW-ORDER-001.outputs=0..240/every=24
scenario.CW-ORDER-001.storage_orders=identity,reverse,affine-257k-plus-17-mod-1152
scenario.CW-ORDER-001.reference=exact-trajectory-root-equality
case.CW-FLOAT-001=all-startup-probes-pass
case.CW-ROUND-001=bit-decoder-ties-zero-nonfinite-and-i64-boundaries
case.CW-CONV-DENSITY=99999-pass,100000-pass,100001-fail-at-iteration-20
case.CW-CONV-DIVERGENCE=999999-pass,1000000-pass,1000001-fail-at-iteration-20
case.CW-CAP-SAMPLES=49999-pass,50000-pass,50001-WATER_SAMPLE_CAPACITY_EXCEEDED
case.CW-CAP-BOUNDARY=16383-pass,16384-pass,16385-WATER_BOUNDARY_CAPACITY_EXCEEDED
case.CW-CAP-FLUID-ROW=127-pass,128-pass,129-WATER_NEIGHBOR_CAPACITY_EXCEEDED
case.CW-CAP-BOUNDARY-ROW=127-pass,128-pass,129-WATER_BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED
case.CW-CAP-STEPS=7199-pass,7200-pass,7201-WATER_STEP_CAPACITY_EXCEEDED
case.CW-CAP-REPORT=16777215-pass,16777216-pass,16777217-WATER_REPORT_CAPACITY_EXCEEDED
case.CW-CAP-REFERENCE=33554431-pass,33554432-pass,33554433-WATER_REFERENCE_INPUT_CAPACITY_EXCEEDED
CORPUS_MANIFEST_V1_END
```

Corpus-manifest SHA-256:
`cb091e0f3a04f2c052b614aeab72034bddd3c0b32430fba85854ede2d183aa91`.

The aperture is open for particle centres on the declared closed rectangle;
the internal wall owns its complement. No sample is created or removed.
`CW-ORIFICE-001` measures net transfer into the receiver half-space
`x >= 1_000_000 µm`; at every output, left count plus right count must remain
exactly `6_000`. This replaces an implicit sink with a closed-system drain
experiment while preserving the intended orifice curve.

The free-fall golden recurrence applies gravity, publishes velocity, then uses
that published velocity for position. For the lowest initial `y`, mandatory
checkpoints `(step, y_um, vy_um_s)` are:

```text
(1, 1274830, -40875)
(2, 1274489, -81750)
(24, 1223907, -981000)
(48, 1074713, -1962000)
(72, 827419, -2943000)
(96, 482025, -3924000)
```

## Metric formulas and frozen thresholds

Metrics consume accepted canonical frames. Diagnostic density, multipliers,
iterations and boundary impulses are sampled before scratch is discarded, in
the same canonical folds. A report cannot recompute a different state.

### Common metrics

- Exact mass is integer `sample_count * 125_000 mg`. It is `750_000_000 mg`
  for each 6k scenario and `6_000_000_000 mg` for `CW-SEALED-001`.
- Centre of mass is the exact `i128` sum of canonical micrometre coordinates
  divided by sample count and rounded ties-to-even to micrometres.
- Boundary penetration is the maximum clearance penetration defined above,
  evaluated every substep; it must be `<= 2_500 µm`. Any outer escape is a
  typed failure even if an output was not due.
- Solver metrics are the exact ppb branch values. A successful step has
  density `<= 100_000 ppb`, divergence `<= 1_000_000 ppb`, iterations within
  `2..=20` and `1..=20`, and no nonfinite value.
- `K = left_fold(0.5 * mass * dot(v,v))` and
  `U = left_fold(mass * gravity_magnitude * y)`, using decoded accepted state.
  Static boundary and internal-orifice work are zero. The energy residual is
  `abs((K_n + U_n) - (K_0 + U_0)) /
  max(abs(K_0) + abs(U_0), 1 joule)`.
- Fluid momentum residual is
  `norm(P_n - P_0 - J_gravity - J_boundary) /
  max(norm(P_0) + norm(J_gravity) + norm(J_boundary), 1 kg*m/s)`.
  `J_gravity` uses exactly one `mass*dt*gravity` per accepted substep;
  `J_boundary` is the left-folded boundary contribution from the two final
  accepted pressure-acceleration applications. Both normalized residuals are
  converted to ppb and must be `<= 10_000_000` (`1%`) at every metric output.

Gravity is represented by potential energy in the energy equation and by
impulse in the momentum equation; it is not counted twice. Fixed walls do no
work, but their impulse is external to fluid momentum. The orifice transfers
mass internally and has no outlet-energy subtraction.

### Scenario metrics

| Scenario | Additional exact pass rule |
|---|---|
| `CW-HYDRO-001` | For outputs at steps `960..=1200`, `COM_x` and `COM_z` remain within `2_500 µm` of `500_000 µm`, and `COM_y` within `25_000 µm` of `375_000 µm`. Record density p50/p95/p99; common solver/work bounds are blocking. |
| `CW-FREEFALL-001` | Every frame equals the canonical gravity recurrence; all pressure/divergence multipliers are positive zero and iteration counts are exactly `2` and `1`. |
| `CW-DAMBREAK-001` | At each output, front is `min(4_000_000, q99(x)+25_000)/4_000_000`; height is `min(1_000_000, q99(y)+25_000)/1_000_000`. Each curve separately has reference RMSE `<= 50_000_000 ppb` and maximum absolute error `<= 100_000_000 ppb`. |
| `CW-STILL-001` | Count/mass and common bounds pass at every output; lateral COM drift from step zero is `<= 2_500 µm`; two identical-profile runs have the same complete trajectory root. |
| `CW-ORIFICE-001` | Transfer is `count(x >= 1_000_000)/6000`; left/right partition and total mass are exact at every output; the transfer curve has reference RMSE `<= 50_000_000 ppb` and maximum absolute error `<= 100_000_000 ppb`. |
| `CW-SEALED-001` | All `48_000` samples and `6_000 kg` remain; common boundary/work bounds pass; two runs have the same complete trajectory root. |
| `CW-ORDER-001` | Identity, reverse and affine storage permutations produce identical frame and trajectory roots. |

`q99` is the nearest-rank canonical quantile: sort the chosen canonical integer
coordinate ascending and use zero-based index `ceil(99*N/100)-1`, computed with
checked integers. At the same integer output steps, for candidate values `c_k`
and reference values `r_k`, `RMSE = sqrt(left_fold((c_k-r_k)^2)/K)` and maximum
error is the ascending-time maximum of `abs(c_k-r_k)`. Convert each final
normalized result to ppb with the frozen bit-decoder before its integer
threshold comparison. No resampling, time-warp, best-fit scale, discarded
transient or post-run tolerance change is allowed.

## Independent source and comparator manifest

The formulation source is the official
[DFSPH paper](https://animation.rwth-aachen.de/media/papers/2015-SCA-DFSPH.pdf).
The independent implementation source is the official
[SPlisHSPlasH repository](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH).
These inputs are non-normative evidence sources; W0B is the implementation
contract.

| Input | Frozen revision / SHA-256 |
|---|---|
| DFSPH paper PDF | `2f1991ee4b90a0523b60ef684835275bbca516c5867b35f6e97a394e7a1f5d56` |
| SPlisHSPlasH commit | `eccce86155776f6ac52d5080b1f720a52bf29450` |
| `TimeStepDFSPH.cpp` | `5dcc295c24fbf083c200f264e2284516a540b5b1b61747cdea80a56ac74e9889` |
| `TimeStepDFSPH.h` | `759fb7b2ea860cf6383fd8e6e7571bf32428666a28e540ddaa56bfe7fd35c264` |
| `SPHKernels.cpp` | `82092c628d8791c6da221fd3b396ed9c5008285e051b1a7df67b71fd6115f5d2` |
| `SPHKernels.h` | `b4cc0631c171b4d25a298ae320828058fa52445a7c5bf7f4517119727f9c3819` |
| upstream `DamBreakModel.json` provenance example | `5e70ff2a1fbab30e47e676a3996da1397adc3e52076c53b6033d53bb00bbf134` |
| upstream `LICENSE` | `608181acd95c1b672b984157254568d581b6d9eafc3e5785899ce1ac85dc9c29` |

The external comparator is built outside Next Engine and outside Git with
`USE_DOUBLE_PRECISION=ON`, `USE_AVX=OFF`, fixed `1/240 s`, CFL off, DFSPH,
iterations/errors matching this profile, Akinci 2012 boundaries, and every
non-pressure model off. The exact source adaptation comments out
`USE_WARMSTART` and `USE_WARMSTART_V` in the hash-bound header and replaces the
3D `0.8 * diameter³` fluid volume initialization with `diameter³`, yielding
the frozen `0.000125 m³` and `0.125 kg`. The scenario translator emits the
same fluid centres, boundary quadrature coordinates and output steps from the
canonical manifest. It exports little-endian binary64 positions at those
steps; the Next Engine importer rejects nonfinite values and applies the same
exact micrometre conversion before computing reference quantiles and chamber
counts.

The bounded interchange is:

```text
bytes[8] = "CWREFV1\0"
scenario_root[32]
output_count_u32_le
sample_count_u32_le
repeat output_count times in ascending step:
    step_u32_le
    repeat initial SampleId order:
        position_x_f64_bits_le
        position_y_f64_bits_le
        position_z_f64_bits_le
```

Counts and byte products are checked against the manifest before allocation or
payload decode; trailing or missing bytes are invalid. The file SHA-256 is W1
evidence. The two 6k/181-output reference files each fit the exact `32 MiB`
input cap.

There is no published numeric curve for the exact selected dam-break or
two-chamber geometry, so both are explicitly
`REFERENCE_NOT_AVAILABLE` for a published curve. The hash-bound paper supports
the formulation; the hash-bound, independently built SPlisHSPlasH aggregate
plus analytical invariants supplies the numeric comparison. Compiler/CMake,
adaptation diff, generated-scene and output hashes are recorded in the W1
evidence artifact. Output hashes cannot exist before execution and therefore
are evidence, not movable W0B inputs.

## Capacity, report and failure contract

The production-profile corpus uses the `50_000` cap. The `100_000` variant is
a separately labelled `NON_AUTHORITATIVE_STRESS` report profile with the
stress capacities in the manifest; it cannot satisfy a correctness or
promotion gate. Full particle trajectories are not written. The report holds
bounded per-step summaries, output-step aggregates, roots and first failure;
streaming hashing keeps decoded heap and report bytes within the manifest.

Stable terminal strings are:

```text
WATER_PROFILE_MISMATCH
WATER_FLOAT_ENVIRONMENT_MISMATCH
WATER_SCENARIO_INVALID
WATER_DUPLICATE_SAMPLE_ID
WATER_SAMPLE_CAPACITY_EXCEEDED
WATER_BOUNDARY_CAPACITY_EXCEEDED
WATER_NEIGHBOR_CAPACITY_EXCEEDED
WATER_BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED
WATER_STEP_CAPACITY_EXCEEDED
WATER_REPORT_CAPACITY_EXCEEDED
WATER_REFERENCE_INPUT_CAPACITY_EXCEEDED
WATER_DECODED_HEAP_CAPACITY_EXCEEDED
WATER_NONFINITE_VALUE
WATER_NUMERIC_OVERFLOW
WATER_DIVERGENCE_NONCONVERGENCE
WATER_DENSITY_NONCONVERGENCE
WATER_BOUNDARY_PENETRATION_LIMIT
WATER_BOUNDARY_ESCAPE
WATER_INVARIANT_MISMATCH
WATER_REFERENCE_CORPUS_MISMATCH
WATER_NONDETERMINISTIC_RESULT
```

Preflight precedence is profile, float environment, scenario syntax/identity,
then samples, boundary, row/aggregate neighbors, steps, decoded heap, report
and reference-input capacity. During a substep, the first failing named phase
in the fixed schedule wins; within a phase the first ascending
sample/neighbor/component wins.
Nonfinite precedes range conversion, non-convergence precedes integration, and
outer escape precedes penetration. Post-run analytical/reference mismatch
precedes repeat/permutation mismatch. The report includes the last accepted
frame root and never reports success after a terminal value.

## W1 entry checklist

- [x] Exact target/toolchain/features, rounding, subnormal, FMA and math
  primitives are frozen.
- [x] Kernel, boundary method, neighbor/reduction/Jacobi order and convergence
  branches are frozen.
- [x] Scenario geometry, horizon, cadence, metrics and tolerances are frozen.
- [x] Reference source revisions and missing-published-curve handling are
  explicit.
- [x] Canonical roots, capacities, N-1/N/N+1 cases and stable failures are
  frozen.
- [x] No runtime/public-contract work is implied.

W1 may begin only from a clean checkpoint containing the final hashes in this
file and the roadmap. Its first PASS still requires executed corpus evidence;
compilation, unit types or this documentation checkpoint are not PASS.
