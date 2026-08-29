# NSR3-B4DR1C external trajectory-preflight research -- 2026-08-21

Status: `COMPLETE / CONTRACT_FREEZE_RECOMMENDED / NO_TRAJECTORY_EXECUTED`

## Question

What is the smallest external DFSPH experiment that can distinguish a broken
adapter/profile from an actual trajectory mismatch before the expensive full
hydro, dam-break and orifice generation?

## Findings

The pinned SPlisHSPlasH revision is not directly observable enough for the
fail-closed R1C gate:

- `USE_WARMSTART` and `USE_WARMSTART_V` are enabled in
  `TimeStepDFSPH.h`, while the parent B4DR1 profile requires cold solves;
- the public iteration getter reports only pressure iterations;
- the last pressure/divergence residual and the reason the loops stopped are
  not exposed;
- the equations themselves do not need to change.

Therefore the minimum source adaptation is a tracked patch which disables the
two warm-start macros and exposes pressure/divergence iteration counts, last
errors and convergence booleans. It changes no force, density, pressure,
divergence or integration formula. The patch SHA-256 is
`e89cf9befc2a08a9c15bd6b290a97b3f6815da1ea699508c41c15645f86c33bc`.

The upstream defaults also require explicit adapter configuration. The
adapter must set sample volume to `0.000125 m^3`, mass to `0.125 kg`, disable
CFL and every non-pressure force, select the precomputed cubic kernel and the
Akinci-2012 gradient, and run with one static OpenMP thread. Z-sort remains
enabled with period 500; stable sample IDs, not storage slots, own output.

## Frozen scenario construction

All scenarios contain a `20 x 15 x 20` fluid lattice. Stable ID is
`((iy * 20) + iz) * 20 + ix`, so `ix` varies fastest. The first position is
`(0.025, 0.025, 0.025) m`, spacing is `0.05 m` and initial velocity is zero.
The canonical text projection uses integer micrometres and LF only.

The fluid projection is:

```text
B4DR1C_FLUID_V1_BEGIN
scenario=<scenario-id>
count=6000
<id>=<x_um>,<y_um>,<z_um>;<vx_um_s>,<vy_um_s>,<vz_um_s>
...
B4DR1C_FLUID_V1_END
```

The frozen fluid roots are:

| Scenario | SHA-256 |
|---|---|
| `CW-HYDRO-001` | `7d4e661d08de08b18d43a76342329b51f6ae98bca9baee3d850e0f403eae5606` |
| `CW-DAM-001` | `9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76` |
| `CW-ORIFICE-001` | `21307ab2d1655ab5da33f3b5d4e887152601ed531679723b8452023df1f48425` |

Hydro and dam boundaries are the two-cell-thick lattice complement around
their respective `20 x 20 x 20` and `80 x 20 x 20` source-cell domains. The
orifice preflight deliberately owns only source-chamber support: it starts
from the hydro complement and removes the two positive-x layers for y-cell
indices `4..7` and z-cell indices `8..11`. The independent R1B swept contact
owns the analytical wall/aperture transition. This is a comparator-specific
source-side construction, not a general two-sided orifice boundary.

The boundary projection is:

```text
B4DR1C_BOUNDARY_V1_BEGIN
scenario=<scenario-id>
<id>=<x_um>,<y_um>,<z_um>
...
count=<count>
B4DR1C_BOUNDARY_V1_END
```

Generation order is x index, then y index, then z index; IDs are assigned
after omissions. Frozen counts and roots are:

| Scenario | Count | SHA-256 |
|---|---:|---|
| hydro | 5,824 | `25de85b5eeec041c12bbb5de10e00b8374457b4cf09cb61d99dfc4d5511d8d62` |
| dam | 16,384 | `1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d` |
| orifice | 5,792 | `5d23bd8c407c1b1d6bb86fc6dda2250c36db4cde1227d9f48fc6239c728a2cb3` |

## Step ownership

R1C captures frame zero and then exactly 24 steps at `dt = 1/240 s`.
For every nonzero frame:

1. save pre-step positions by stable ID;
2. execute one upstream cold DFSPH step;
3. map upstream storage slots back to stable IDs;
4. apply the independent R1B contact projection to the upstream velocity;
5. publish `x[n+1] = x[n] + dt * v_projected` to the upstream model;
6. validate finite state, convergence, outer clearance and the safe-aperture
   chord before serialization.

The post-step position replacement is intentional: upstream owns fluid
acceleration/pressure, while R1B owns the already-attested analytical contact
semantics. No R1B fixture result is treated as trajectory credit.

## Serialization decision

`CWREFV2` is a research interchange container only. It is not a runtime,
plugin or persistence schema. The file begins with magic `CWREFV2\0`, a
little-endian manifest length and exact LF profile/scenario manifest bytes,
then sample and frame counts. Frames are ascending `0..24`; samples are
ascending stable ID and store raw binary64 position/velocity bits plus solver,
density, speed and 25-feature contact diagnostics.

At 6,000 samples a frame is 312,156 bytes. The R1C file is about 7.8 MB; the
later densest R1D schedule remains below the frozen 64 MiB reader limit.

## Cost-aware execution ladder

The first executable mode is manifest-only. It creates no Simulation, Fluid
Model, Boundary Model or TimeStep and must reproduce all scenario/fluid/
boundary roots plus at least one rejected or root-changing mutation. Only its
PASS authorizes trajectory mode.

Trajectory mode then runs twice per scenario in fresh processes. Files and
canonical reports must be byte-identical per scenario. A failure stops the
ladder immediately; it cannot be averaged away or relabelled as physical
variance. Full R1D schedules remain forbidden until all three R1C scenarios
pass.

## Recommendation

Freeze the companion R1C contract and tracked upstream patch now. Implement
manifest-only preflight first, attest it, and only then permit the six short
trajectory processes. Keep B4E, CUDA, runtime integration and production
claims blocked.
