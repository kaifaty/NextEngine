# NSR3-B4DR1C -- external 24-step trajectory-preflight contract

Status: `FROZEN / MANIFEST_PREFLIGHT_IMPLEMENTATION_AUTHORIZED / TRAJECTORY_CONDITIONAL`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4dr1c-24step|v1|parent=ade621f889a08fd26ca713592625c50316b893af8c4f1797d25f4e4d4c96b86a|r1b=c65346ec7b215a7a173eabfdd6c91e20869d4a0679b9d014a1e897369cb71f84|upstream=eccce86155776f6ac52d5080b1f720a52bf29450|patch=e89cf9befc2a08a9c15bd6b290a97b3f6815da1ea699508c41c15645f86c33bc|fluid=6000;id=iy-iz-ix;spacing=.05;first=.025;volume=.000125;mass=.125|boundary=hydro:5824:25de85b5,dam:16384:1cf0fd17,orifice:5792:5d23bd8c|solver=dfsph;dt:0x3f71111111111111;gravity:-9.81;cfl-off;cold;pressure:2..100@.01pct;divergence:1..100@.1pct;nonpressure-off;precomputed-cubic;akinci;zsort:on/500;omp1|contact=r1b-v2;post-step-xn+dt-vprojected;8hits|frames=0..24/every1|format=CWREFV2;lf-manifest;explicit-id;posvel-f64;diagnostics|runs=2-fresh-byte-exact|trajectory=after-manifest-preflight|credit=none
```

Identity SHA-256:
`a061f43ea3bc60fc3ff3af03aa242c298ad059e094e2215bab71642902f199d0`.

## Authority and exclusions

This contract refines only B4DR1 R1C. It authorizes research tooling and
external artifacts; it creates no historical W0I/W1 credit, production
claim, public schema, save format or runtime/plugin authority. Generated
sources, libraries, logs and payloads remain outside Git. B4D's exact
missing-artifact failure remains valid negative evidence.

Manifest-only preflight is authorized after this contract is committed.
Trajectory execution is conditional on that preflight passing. R1D, R1E,
B4E, CUDA and product integration remain forbidden until their parent gates.

## Frozen source adaptation

The only allowed upstream source change is tracked patch
`crates/continuum-water/tools/nonlocal-reference-adapter/patches/
splishsplash-eccce861-cold-start.patch`, SHA-256
`e89cf9befc2a08a9c15bd6b290a97b3f6815da1ea699508c41c15645f86c33bc`.
It must apply cleanly to upstream commit
`eccce86155776f6ac52d5080b1f720a52bf29450` and may modify only:

- `SPlisHSPlasH/DFSPH/TimeStepDFSPH.cpp`;
- `SPlisHSPlasH/DFSPH/TimeStepDFSPH.h`.

The patch disables pressure/divergence warm starts and exposes iteration,
last-error and converged diagnostics. It must not change a DFSPH equation.
After application, the exact Git diff SHA-256 and dirty-path list become
build evidence. Any extra modified path stops the run.

Every build starts from an ordinary clean full clone, not a linked worktree,
and preserves the R1A GCC 15.2/CMake/Ninja, Release, binary64, AVX/FMA/
fast-math-off profile. The clean R1A clone is never mutated.

## Exact scenario manifests

Each block below includes its final LF. Scenario-root definition is:

```text
SHA256("nextengine.nonlocal.nsr3b4dr1c-scenario.v1\0" || exact_block_bytes)
```

### Hydro

```text
B4DR1C_SCENARIO_V1_BEGIN
scenario_id=CW-HYDRO-001
kind=hydrostatic-cube
box_um=0,0,0;1000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=7d4e661d08de08b18d43a76342329b51f6ae98bca9baee3d850e0f403eae5606
boundary=two-layer-outer-complement
boundary_count=5824
boundary_root=25de85b5eeec041c12bbb5de10e00b8374457b4cf09cb61d99dfc4d5511d8d62
steps=24
outputs=0..24/every=1
B4DR1C_SCENARIO_V1_END
```

Exact block length is 444 bytes; scenario root is
`88d7b5ee86876d05b8cdef4f5c308fa61ffeb9d559374674db7dda2757e611b7`.

### Dam break

```text
B4DR1C_SCENARIO_V1_BEGIN
scenario_id=CW-DAM-001
kind=dam-break
box_um=0,0,0;4000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76
boundary=two-layer-outer-complement
boundary_count=16384
boundary_root=1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d
steps=24
outputs=0..24/every=1
B4DR1C_SCENARIO_V1_END
```

Exact block length is 436 bytes; scenario root is
`45b586513a4b6ade4635c7af4ed4bdc05c9f2e8b2825a81cde6136e38692ea28`.

### Orifice

```text
B4DR1C_SCENARIO_V1_BEGIN
scenario_id=CW-ORIFICE-001
kind=orifice-release
box_um=0,0,0;2000000,1000000,1000000
wall_um=x=1000000;opening_y=200000..400000;opening_z=400000..600000;radius=25000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=21307ab2d1655ab5da33f3b5d4e887152601ed531679723b8452023df1f48425
boundary=source-chamber-two-layer-minus-safe-opening
boundary_count=5792
boundary_root=5d23bd8c407c1b1d6bb86fc6dda2250c36db4cde1227d9f48fc6239c728a2cb3
steps=24
outputs=0..24/every=1
B4DR1C_SCENARIO_V1_END
```

Exact block length is 543 bytes; scenario root is
`5bb0a197f40ec5e5d3f8536c5db7840da4228978fcb1c788b9de7d233482d42e`.

## Fluid and boundary projections

Every fluid uses 6,000 samples. Stable ID is
`((iy * 20) + iz) * 20 + ix`; `ix` varies fastest. Positions are the exact
binary64 results of integer-micrometre coordinates divided by `1,000,000`.
Initial velocity is positive zero in all components.

Fluid text projection is LF-only, ascending stable ID:

```text
B4DR1C_FLUID_V1_BEGIN
scenario=<scenario-id>
count=6000
<id>=<x_um>,<y_um>,<z_um>;0,0,0
...
B4DR1C_FLUID_V1_END
```

Its hydro/dam/orifice roots are respectively
`7d4e661d08de08b18d43a76342329b51f6ae98bca9baee3d850e0f403eae5606`,
`9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76`
and
`21307ab2d1655ab5da33f3b5d4e887152601ed531679723b8452023df1f48425`.

Boundary points lie on the same 50 mm centre lattice. The two-layer
complement enumerates x index outermost, then y, then z, and keeps cells for
which at least one index is outside the fluid-domain range by one or two
cells. IDs are assigned after omission. The orifice removes positive-x
indices `20,21` only when y is `4..7` and z is `8..11`.

Boundary text projection is LF-only:

```text
B4DR1C_BOUNDARY_V1_BEGIN
scenario=<scenario-id>
<id>=<x_um>,<y_um>,<z_um>
...
count=<count>
B4DR1C_BOUNDARY_V1_END
```

Hydro/dam/orifice counts are `5824/16384/5792`; roots are respectively
`25de85b5eeec041c12bbb5de10e00b8374457b4cf09cb61d99dfc4d5511d8d62`,
`1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d`
and
`5d23bd8c407c1b1d6bb86fc6dda2250c36db4cde1227d9f48fc6239c728a2cb3`.

## Solver profile

The adapter explicitly sets:

- binary64, `dt` bits `0x3f71111111111111`, gravity `(0,-9.81,0)`;
- one fluid model; volume `0.000125 m^3`, density `1000 kg/m^3`, mass
  `0.125 kg` for every sample;
- DFSPH pressure min/max `2/100`, max error `0.01%`;
- divergence min/max `1/100`, max error `0.1%`;
- CFL off, cold pressure/divergence, viscosity/surface tension/vorticity and
  every other non-pressure method off;
- precomputed cubic kernel, Akinci-2012 gradient, z-sort on with period 500;
- Akinci-2012 static boundary particles and pseudo-volume update;
- locale `C`, round-to-nearest, FTZ/DAZ off, `OMP_NUM_THREADS=1`,
  `OMP_DYNAMIC=FALSE`.

Any unsupported setter, changed time step, profile mismatch or diagnostic
which cannot be read exactly is a hard failure.

## Manifest-only preflight gate

`--r1c-manifest-preflight` must execute before any SPlisHSPlasH singleton,
model, boundary or time-step object is created. It independently regenerates
all three scenario blocks, fluid projections and boundary projections and
checks every frozen count/root. Its canonical LF report runs twice in fresh
processes and must be byte-identical.

The preflight must also prove that swapping two stable fluid IDs and changing
one boundary coordinate by one micrometre changes the relevant root, and that
a manifest root mismatch rejects before `trajectory_started=true`.

Only a dated PASS evidence record for this gate authorizes
`--r1c-trajectory`. A preflight failure leaves trajectory forbidden.

## `CWREFV2` exact binary layout

All integers are unsigned little-endian. All floating values are their raw
IEC-559 binary64 bits in little-endian order. No padding is serialized.

```text
magic[8] = "CWREFV2\0"
manifest_length_u32
manifest_bytes[manifest_length]
sample_count_u32 = 6000
frame_count_u32 = 25

for step = 0..24:
  step_u32
  pressure_iterations_u32
  divergence_iterations_u32
  receiver_count_u32
  pressure_error_f64
  divergence_error_f64
  density_min_f64
  density_max_f64
  max_speed_f64
  feature_counts_u32[25]
  for stable_id = 0..5999:
    id_u32
    position_f64[3]
    velocity_f64[3]
```

`manifest_bytes` are the UTF-8 identity projection followed by LF and then
the exact scenario block. Frame size is 312,156 bytes. Step zero has zero
iterations/errors/speed/contact counts; density min/max are zero because no
solver state has yet been constructed for the serialized initial manifest.

Output uses a caller-supplied absolute fresh directory outside Git. Existing
files, symlinks and non-directories reject. Each process writes a fresh
partial file, fsyncs and atomically renames it; failed partial files receive
no credit. The hard file-size cap is 64 MiB.

## Trajectory algorithm and validators

For each step, capture pre-step positions by stable ID, execute one upstream
DFSPH step, remap by upstream particle index, pass velocity through the exact
R1B v2 swept-contact projection, and replace the upstream position with
`x_n + dt * v_projected` before validation/serialization.

Every nonzero frame must satisfy:

- both upstream convergence booleans true;
- pressure/divergence iterations within `2..100` and `1..100`;
- finite errors, density, positions and velocities;
- unchanged time-step bits;
- density min not greater than density max;
- raw-binary64 25 mm outer clearance for every particle;
- no outside-safe-opening chord through the orifice wall;
- stable IDs form exactly `0..5999`, irrespective of z-sort storage order.

The trajectory mode runs hydro, dam and orifice separately. Each scenario is
executed twice in fresh processes with fresh output paths. Complete files and
canonical LF reports must be byte-identical per scenario. On the first
nonzero exit, mismatch or validator failure, stop without running the next
scenario.

## Exit

R1C passes only if manifest preflight and all six trajectory processes pass,
three same-scenario payload pairs are byte-identical, and source/build/
binary/report/payload identities are recorded in dated evidence. R1C PASS
authorizes only R1D full-generation contract execution. It does not authorize
R1E, B4E, runtime integration or production claims.
