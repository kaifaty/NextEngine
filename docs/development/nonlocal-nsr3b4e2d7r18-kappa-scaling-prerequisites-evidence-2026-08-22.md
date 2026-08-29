# NSR3-B4E2D7R18 kappa scaling prerequisites evidence

Date: `2026-08-22`

Status: `PASS_CLASSIFICATION / DT_KAPPA_NONDIMENSIONAL_MISMATCH / D7R19_BLOCKED`

Implementation commit: `0b649563`.

## Result

D7R18 threads explicit finite-positive `kappa` through the complete sparse AL
candidate path and deterministically selects:

```text
DT_KAPPA_NONDIMENSIONAL_MISMATCH
```

This is a successful bounded classification, not confirmation of the scaled
nominal candidate. No nominal substep, outer transaction, trajectory, timing
or state mutation executes.

## Exact coefficient algebra

The derived coefficient controls all pass:

| Quantity | Result |
|---|---:|
| reference `dt` bits | `0x3f71111111111111` |
| reference `kappa` bits | `0x4093290000000000` |
| aligned `dt` bits | `0x3f0c01c01c01c01c` |
| aligned `kappa=7,460,505` bits | `0x415c75a640000000` |
| `dt_ref/dt_sub` | exact `78` |
| squared ratio | exact `6084` |
| common `kappa*dt^2` bits | `0x3f95cccccccccccd` |
| inertia scale | exact `7200 -> 43,804,800` (`6084x`) |

One-ULP mutation of aligned `kappa` changes the dimensionless product and the
active output root. Zero, negative, NaN and positive infinity reject before
static support, workspace/pair, precision, HVP or outer work.

## Propagation and mismatch

Reference and scaled profiles each reproduce dense/sparse evaluation,
gradient, HVP, divided difference, long-double and binary128 outputs exactly.
Workspace coefficient tapes retain their exact `kappa` bits. Density,
constraint and active membership remain exact across profiles.

The frozen normalized comparison nevertheless fails only in HVP assembly:

| Normalized component | Maximum relative error | Frozen limit |
|---|---:|---:|
| support energy | `0` | `1.4210854715202004e-14` |
| total energy | `0` | same |
| active coefficient | `2.0554578840662433e-16` | same |
| gradient | `3.180539420634439e-15` | same |
| HVP | `7.51297647658249e-14` | same |
| divided reduction | `6.920583014038748e-16` | same |
| long-double reduction | `1.5017174901601858e-16` | same |
| binary128 reduction | `0` | same |
| multiplier update | `2.0554578840662433e-16` | same |
| scaled dual update | `1.8357832746658422e-16` | same |

The HVP miss is about `5.29x` the predeclared limit and about `338` binary64
epsilons. The threshold is not weakened after observing it. The coefficient
law is not falsified: every non-HVP control closes, including independent
extended precision. The dimensional HVP accumulation is the isolated next
mechanism.

## Work and lifecycle

| Fact | Value |
|---|---:|
| workspaces built / released | `5 / 5` |
| maximum live workspaces | `2` |
| long-double / binary128 audits | `2 / 2` |
| candidate all-pair calls | `0` |
| invalid-`kappa` cases rejected before work | `4 / 4` |

D7R16 remains byte exact, and both clean binaries reproduce D7R17's complete
`7,549`-byte stdout at SHA-256
`a2a8de930d41d8e49c255a3fcf987a8794b4da067ddadf658732946fca0ffced`.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r18.mPqnVw`.

Two independent clean GCC 15.2 Release builds produce identical
`5,345,848`-byte executables:

```text
SHA-256  2fe4a2e6b4724a5ed481f24f3b309119194c41b64fc77dd479c0cead84f89d0d
Build ID f49880e7fb88e2b43aa765cd6c326114285ce882
```

Both D7R18 processes exit zero with empty stderr and emit the same
`2,073`-byte stdout:

```text
stdout SHA-256  0267e094641aaddd9b604a71b527db976aaa699f51dacd558624512bd3b1b925
semantic result 871e1ca5b84968261aaa33bca78d5deab92562e2279dadbe1d5e311be0d19084
```

Build directories:

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18-a.NuLWgZ
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18-b.vUMXcg
```

## Decision

Preserve the D7R18 mismatch and keep D7R19 blocked. Do not relax the HVP gate
or run a scaled nominal substep.

Research a fully nondimensional sparse AL transaction next. With
`u=lambda/kappa` and `theta=kappa*dt^2/M`, the normalized PHR terms are

```text
Ebar = 1/2 ||y-y_hat||^2
     + theta/2 sum(max(0,u+c)^2-u^2),
u_next = max(0,u+c).
```

The normalized HVP can assemble `I + theta J^T J + theta*active*H_c`
directly instead of constructing large dimensional terms and scaling them
after summation. This also exposes that raw absolute-`lambda` and equivalent-
pressure change gates cannot be inherited unchanged when `kappa` changes by
`6084`; a later nominal contract must use representation-invariant state and
kinematic/impulse admission.

No scaled nominal solve, coefficient selection, macro, trajectory, timing,
parallel/GPU, runtime or production authority is granted.
