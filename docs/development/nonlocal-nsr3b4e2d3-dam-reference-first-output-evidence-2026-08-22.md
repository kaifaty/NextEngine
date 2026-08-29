# NSR3-B4E2D3 repaired Dam first-output evidence

Date: `2026-08-22`

Status: `FAIL / STEP_2_STRAIN / PREFIX_PRESERVED`

## Result

Two clean Release builds produce byte-identical 4,510,536-byte executables,
SHA `7ecca16aff217884de022b43f0e271b18deb5797f2a3afe97e65b597d7d65467`
and Build ID `a7849b211046a1ac82439bedd8f547f151d23706`. Process A exits one,
emits empty stderr and a 4,189-byte report at SHA
`8619688fd9b810c5e46cc74a7e1e9603d0571641013289baadd76fe68356b410`.
The semantic result is
`a126e8a12f2473f2ff8b8329d6453b504ced47481018c3fe63d9a5e64c749039`.
The frozen stop-first rule correctly prevents process B.

All frame-zero binary64, decoded topology, static-index and scenario alignment
gates pass. Step one passes and commits:

- 39 initial and 78 accepted substeps;
- 117 attempted substeps, 522 outer trials, 874 nonlinear HVPs and 48
  spectral HVPs;
- maximum positive density strain `0.0004576940766030102`;
- maximum KKT residual `4.9626518502059436e-10` and support closure
  `3.9211761635471543e-15`;
- frame root `c9afea4e49863db57b4099e3dee5d56d8d3b6ca9bcb03ba100e5870c46e29477`;
- aggregate root
  `f1598838ff82272113fd9746683d1d857d81f692997a57a22b82151216efdda4`.

Step two's adaptive transaction also solves, publishes and passes its
nonlinear/work gates at 80 accepted substeps. It fails only the separately
frozen physical strain gate:

| Observable | Result | Limit |
|---|---:|---:|
| maximum positive density strain | `0.0011747197409319732` | `0.001` |
| maximum penetration | `0` | `0.0025 m` |
| maximum KKT residual | `4.5875800090895006e-10` | `1e-9` |
| strict residual | `4.2782630883080969e-10` | finite |
| support closure | `3.8994373699290189e-15` | `1e-10` |

Its non-authoritative failed-step frame root is
`59f17359b06f9d428409399aa7a7acf03f0be56dc2fbdf67ad5f98a8a3e7f2b6`
and aggregate root is
`c045627c6abe5ab8bb003fca68796bb06f2b84f14ad8e6be33c4ccdf6ea74c48`.
The authoritative committed prefix remains step one only.

Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d3.lXSI73`.
No timing or performance claim is admitted.

## Interpretation and next discriminator

This is no longer a harness, binary64 or topology-alignment failure. Two
hypotheses remain:

1. the embedded adjacent-output gate accepts 80 substeps before the physical
   strain observable is temporally resolved; or
2. the converged finite-`KAPPA` penalty trajectory itself exceeds the material
   strain cap, so more temporal refinement cannot repair it.

Freeze a new B4E2D4 diagnostic that starts from the exact committed step-one
state, preserves all physics and SIRDI work choices, and compares fixed 80,
160 and 320-substep step-two lanes. Do not tune or rerun B4E2D3.
