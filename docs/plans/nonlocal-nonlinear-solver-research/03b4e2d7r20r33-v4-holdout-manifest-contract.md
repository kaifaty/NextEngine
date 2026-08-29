# NSR3-B4E2D7R20R33 v4 holdout manifest contract

Status: `FROZEN / SOURCE CONSTRUCTION AUTHORIZED`.

## Parent

- R32 implementation `86515ea9`, semantic
  `e7b9acafd3d7aee53b23bff4c3a286f7cf4578e5e566cd69ab04d11c866f0f73`;
- route `TERMINAL_CERTIFICATE_TRAJECTORY_CANDIDATE` with exact 12/12
  certification;
- all v2/v3 cases are development data and provide no new blind credit.

## Frozen sources

Create exactly five `BLIND_HOLDOUT_V4` fixtures:

```text
v4-filled-upper-x-lower-z-torsion-8x3x3
v4-filled-upper-x-lower-y-lower-z-corner-4x6x4
v4-filled-biaxial-counterflow-6x4x3
v4-filled-helical-compression-4x4x5
v4-filled-alternating-y-layer-3x7x4
```

Each uses `make_lattice_fluid` with dimensions respectively `8x3x3`, `4x6x4`,
`6x4x3`, `4x4x5`, `3x7x4`, plus `make_box_owned_shell(...,2)`. Closed-box
low/high corners use the exact R9 lattice-identical expressions.

For zero-based lattice coordinates `x,y,z`, velocities are frozen as:

```text
torsion:
  (0.63+0.008y-0.006z,
   0.075(z-1)-0.014(x-3.5),
  -0.57-0.005x+0.009y)

corner:
  (0.52+0.010y+0.004z,
  -0.49-0.007x-0.003z,
  -0.61-0.006y+0.005x)

counterflow:
  (0.21(y-1.5)+0.018(z-1),
  -0.19(x-2.5)+0.012z,
   0.43+0.011x-0.009y)

helical compression, with radial physical position r from lattice average:
  (0.08-2.0r.x-0.75r.z,
  -0.05-2.6r.y+0.55r.x,
   0.04-2.3r.z+0.65r.y)

alternating layer, s=-1 for even y and +1 for odd y:
  (0.16(y-3)+0.015z,
   0.58+0.010x-0.006z,
   0.22s+0.020(x-1))
```

Require finite owned geometry, `72/96/72/80/84` fluid samples, 404 total,
pairwise-distinct geometry/source roots, exactly five v4 roles and exact R32
parent identity. The manifest semantic binds every source root and literal
zero counters for operator, preflight, solver and timing.

Routes are `V4_HOLDOUT_MANIFEST_REJECTED` and
`V4_HOLDOUT_MANIFEST_FROZEN`.

No projection, operator, excitation, KKT, solver, tuning, timing, runtime/GPU,
generalization or production authority. Commit the source manifest before
authorizing preflight; do not edit a frozen source after observation.
