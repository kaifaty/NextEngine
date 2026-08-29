# NSR3-B4E2D7R20R9 v3 manifest evidence

Status: `PASS / SOURCE HASHES FROZEN`.

## Frozen result

Implementation `49ea40b8` ran twice with stable semantic:

```text
4125a52d71a2002d1188b948b71a5d2c0173dc8e25dab30ace59fbd9ba43e9c3
```

Exactly four `BLIND_HOLDOUT_V3` sources contain 277 fluid samples:

| source | particles | source root |
|---|---:|---|
| lower-x/upper-z edge `6x3x4` | 72 | `43b8cd31...6b11` |
| lower-x/lower-y/upper-z corner `5x5x3` | 75 | `c2956c2d...e60d` |
| upper-y shear layer `7x2x5` | 70 | `fa4dd6f9...1e83` |
| radial compression `5x3x4` | 60 | `7e6aa37c...7bf2` |

Geometry and source roots are pairwise distinct. All ownership, finite-state
and exact count gates pass. Operator, preflight, solver and timing counters are
zero.

## Pre-freeze construction rejection

The first source-only execution returned semantic `fdbc0adf...df80` and
`V3_HOLDOUT_MANIFEST_REJECTED`: high box faces used the algebraically
equivalent expression `count*SPACING-RADIUS`, while lattice positions use
`RADIUS+(count-1)*SPACING`. Binary64 evaluation differed by one ULP on some
axes, violating exact ownership.

No operator or physics result had been observed. The high-face expression was
changed to the lattice-identical form; velocities, topologies and counts were
not modified. The successful source roots above are now immutable.

## Next boundary

An input-only v3 operator preflight may now build sparse operators, joint
box-ball projected targets and excitation metrics. It must not call the R8
solver. A quiet source is an admission failure, not permission to edit R9.

No generalization, runtime, GPU or production authority exists.
