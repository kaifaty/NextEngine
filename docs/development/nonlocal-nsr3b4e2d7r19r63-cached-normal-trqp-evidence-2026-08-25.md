# NSR3-B4E2D7R19R63 cached-normal projection TRQP evidence

Date: `2026-08-25`

Status: `PASS / TANGENTIAL_MASTER_EXPANSION_REQUIRED / ROLLBACK ONLY`.

Implementation commit: `bd5ebe35`.

Frozen identity SHA-256:
`7cc6eb487c099cce6c08c4ef52c7474f304fdf4e8a127cc49c2926a22b4c38d8`.

## Result

R63 validates the pure-inertia projection TRQP and rejects reuse of the fixed
494-row restoration master for tangential optimization.

The R62 payload is consumed exactly once. Its cached normal remains the
feasible anchor and model reference; it is never applied alone.

```text
payload root  5995a2cca7c24b99b328d4c66af7f9c7a710cfe4e684d0af305f81f1e162b41c
normal root   040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd
consume count 1
```

## Projection model validation

The exact target is `(predicted-R43)/SPACING`:

```text
target root       6b4bf4a7c8556e7c494ec49b39a2f78cfa0a3364cc96acbd26da54cc4b2c0540
normal direct f   8.2188466601660054e-13
projection f      8.2188466600327973e-13
identity error    1.3320818841532357e-23
forward bound     7.0101460858069737e-23
```

The identity error is contained by the frozen floating-point forward bound.
No HVP or augmented-Lagrangian objective curvature is needed.

The 64-cycle Hildreth/Dykstra reference remains finite, contact/box/trust safe
and strongly improves pure inertia, but violations appear far outside the old
master:

| Cycle | Raw positive | Candidate positive | Outside master | Inertia | Reduction from normal |
|---:|---:|---:|---:|---:|---:|
| 8 | 3469 | 3470 | 2995 | `2.6969324156773668e-13` | `5.5219142443554304e-13` |
| 16 | 3280 | 3281 | 2807 | `2.9016759035815649e-13` | `5.3171707564512324e-13` |
| 32 | 3268 | 3268 | 2796 | `2.9195491858725135e-13` | `5.2992974741602832e-13` |
| 64 | 3078 | 3080 | 2796 | `2.91966213187725e-13` | `5.2991845281555473e-13` |

The fixed master residual itself contracts from about `9.95e-9` at cycle 8
to `9.03e-14` at cycle 64, while 2,796 outside-master rows remain positive.
More sweeps over the same 494 rows therefore target the wrong subproblem.

This separates two ownership domains:

```text
restoration master   -> certifies one feasible normal
tangential active set -> follows objective-driven composite movement
```

They cannot be represented by one fixed active row set.

## Terminal geometry and work

```text
composite root   cc4975dae7387f8cca3e970644fb83984ab4c22c09587b65dda28fd97ce2c26f
endpoint root    53062a52d93683348bdd7f05f493bed59f502d1ffc617685b16bd62a84cd539f
tangential root  d9a4c3fec4f10840f08fd01ebb31afe8838baf00d9e101d13875bfae367bd98a
contact tests    36000
new / worsened   0 / 0
```

The endpoint is diagnostic only and is not a linearly feasible candidate.

```text
density cycles       64
ball-box projections 64
pair-once JVP        64
directed audits       4
new row VJP           0
new Gram JVP          0
new HVP               0
state root  216c12aeeb47686e823a9911f0aa9dd14ddee53db1701874488dc325a292cea5
route root  d0433f118a39ffd3fca4abbfb1204a542068090a71f2a4e203f4ac1d900c621c
semantic    9f456232956a0e080063123acf8a4a0b2dac7a6fd8f2615b270f0958e66a8440
```

Rollback is exact. No nonlinear density trial, switching, state mutation or
runtime policy executes.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r63-final-a.njlhe5
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r63-final-b.h1lmth
binary SHA-256 75cd888654abb6b96db2a199db1a8e2df0f88441756854354a77a5d70802a30b
size           8526072
ELF build-id   32d5c88e159dee40b5d8de36ad5a830c62b4b34a
stdout bytes   3806
stdout SHA-256 38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62
```

Both clean binaries and outputs are byte-exact; both processes exit zero.
Wall time is not performance evidence.

## Consequence

Preserve the projection model and R63 as its full-vector reference. Do not add
depth to the fixed 494-row master. Research R64 as sparse all-row operator
ownership: derive row gradients from flat topology, build particle-to-row
incidence/overlap, validate values/diagonals/updates against captured R51 rows
and only then authorize dynamic tangential active-set expansion.

