# NSR3-B4E2D7R20R57 v5 source-manifest contract

Status: `FROZEN / SOURCE CONSTRUCTION AUTHORIZED`.

## Parent

- R56 implementation `6809fc5f`, semantic `daf7c2ce...20b7`;
- v4 is now development data and provides no further blind generalization
  credit for the composed policy.

## Frozen sources

Create exactly six `BLIND_HOLDOUT_V5` fixtures with provenance
`r20r57-source-only-after-v4-composition`:

```text
v5-filled-triaxial-saddle-5x5x4
v5-filled-layered-xz-shear-9x3x4
v5-filled-checkerboard-compression-4x8x3
v5-filled-oblique-jet-twist-3x5x7
v5-filled-radial-expansion-drift-6x6x3
v5-filled-three-face-coupling-5x4x6
```

Each uses `make_lattice_fluid(dimensions)`,
`make_box_owned_shell(dimensions,2)`, the existing closed-box low/high
expressions and pair capacity `160 * fluid_samples`.

For zero-based lattice coordinates `x,y,z`, freeze velocities as:

```text
triaxial saddle:
  (0.34 + 0.09(x-2) - 0.07(y-2) + 0.013z,
  -0.28 + 0.08(y-2) + 0.05(z-1.5) - 0.011x,
   0.22 - 0.06(z-1.5) + 0.04(x-2) - 0.009y)

layered xz shear, s=((x+2z) mod 3)-1:
  (0.47 + 0.12s + 0.007y,
   0.18(z-1.5) - 0.025(x-4),
  -0.36 + 0.015x - 0.009y)

checkerboard compression, q=-1 for even x+y+z and +1 for odd:
  (-0.11(x-1.5) + 0.17q,
    0.39 - 0.08(y-3.5),
   -0.13(z-1) + 0.10q + 0.006y)

oblique jet/twist:
  (-0.22(y-2) + 0.018(z-3),
    0.20(x-1) - 0.014(z-3),
    0.62 - 0.035|z-3| + 0.008y)

radial expansion/drift, r from physical lattice average:
  (0.12 + 1.6r.x - 0.4r.y,
  -0.09 + 1.4r.y + 0.35r.z,
   0.07 + 1.2r.z - 0.3r.x)

three-face coupling:
  (0.55 + 0.04(y-1.5) - 0.03(z-2.5),
  -0.48 - 0.025x + 0.02z,
   0.44 + 0.03x - 0.035y)
```

Require fluid counts `100/108/96/105/108/120`, 637 total, finite owned
geometry, pairwise-distinct source and geometry roots, exact role/provenance and
R56 parent identity. Bind literal zero operator, projection, preflight, solver
and timing counters into the manifest semantic.

Routes are `V5_HOLDOUT_MANIFEST_REJECTED` and
`V5_HOLDOUT_MANIFEST_FROZEN`.

No operator, excitation or solver observation is authorized. Commit the source
manifest before designing v5 preflight; never edit a frozen fixture after
observation.
