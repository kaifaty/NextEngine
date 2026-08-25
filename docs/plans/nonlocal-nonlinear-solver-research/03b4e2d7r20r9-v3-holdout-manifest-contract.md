# NSR3-B4E2D7R20R9 v3 holdout manifest contract

Status: `FROZEN / SOURCE CONSTRUCTION AUTHORIZED`.

## Parent

- R20R8 implementation `301b3a85`, semantic
  `afca1c777053172169e227bcf3625ad7c828892520340db5eb45c98d53b4374a`;
- route `VERIFIED_INVERSE_DEVELOPMENT_CERTIFIED`;
- v2 cases are development data and provide no blind credit.

## Frozen sources

Create exactly four `BLIND_HOLDOUT_V3` fixtures:

```text
v3-filled-lower-x-upper-z-edge-6x3x4
v3-filled-lower-x-lower-y-upper-z-corner-5x5x3
v3-filled-upper-y-shear-layer-7x2x5
v3-filled-radial-compression-5x3x4
```

Each uses `make_lattice_fluid` with the named dimensions and a matching
`make_box_owned_shell(...,2)`. The closed box has low corner `(RADIUS,... )`
and high corner `(count_axis*SPACING-RADIUS,...)`. Velocities are deterministic
index/lattice-coordinate formulas recorded directly in source; no randomness
or observed solver state is allowed.

Require finite owned geometry, 72/75/70/60 fluid samples, distinct geometry and
source roots, exactly four v3 roles and parent R8 identity. The manifest
semantic includes all source roots and the literals

```text
operator-executed=0
preflight-executed=0
solver-executed=0
timing-admitted=0
```

Route:

```text
V3_HOLDOUT_MANIFEST_REJECTED
V3_HOLDOUT_MANIFEST_FROZEN
```

No projection, operator, excitation, KKT, solver, inverse audit, timing,
runtime/GPU integration, production or generalization authority. Commit the
manifest result before authorizing preflight.
