# NSR3-B4E2D7R19R49 restoration Hager--Zhang evidence

Date: `2026-08-25`

Status: `PASS / RESTORATION_HZ_PRIMAL_ACCELERATOR_CANDIDATE / ROLLBACK ONLY`.

Implementation commit: `5e5c54d1`.

Frozen identity SHA-256:
`e61ba2c8ca22f43c09ca7ad6a8949af6437b91f13b1f45b5b5c17926dbae62b8`.

## Result

R49 reproduces exact R48, rebuilds the unchanged next-TRQP problem and replaces
only the primal candidate generator. The guarded Hager--Zhang recurrence
completes all 16 frozen accepted steps:

```text
accepted steps                 16
HZ memory steps                15
maximum memory streak          15
restarts                        0
pair passes                    48
terminal psi                   1.0003161482435810e-20
terminal h                     1.4144370952740040e-10
directed maximum row upper     1.9771445876661678e-11
certified-positive rows        366
route                           RESTORATION_HZ_PRIMAL_ACCELERATOR_CANDIDATE
```

Every proposal and exact hinge line stays in the inherited contact-box/normal-
ball geometry. Each accepted step is independently re-evaluated with the R48
directed JVP enclosure. All HZ memory directions pass the raw/projected descent
guards without restart, so the observed gain belongs to the direction-memory
mechanism rather than a hidden steepest-descent fallback.

The directed maximum remains positive. R49 therefore does not certify
compatibility, does not prove infeasibility and does not authorize restoration
exit. No witness, R43 state, filter entry or following outer is committed.

## Comparison with R48

Against the terminal R48 PDAL witness, R49 improves:

```text
h                               36.406327777138593x
psi                           1325.4207022164533x
directed maximum upper          23.735093399820773x
```

R49 uses 48 operator pair passes versus 322 in R48, a `6.708333x` work-count
ratio. This is a deterministic algorithmic-work comparison, not a measured
wall-clock speedup. The R49 witness norm is `5.0385810268018367e-7`, still only
a tiny fraction of the `0.03125` normal radius, confirming that contact/trust
geometry remains inactive in the observed boundary.

Checkpoint and step roots are respectively
`b19e6e64eaf68e9bd8eccf748d89cda661b42850118d355846b92f329975caba`
and `ec1fcee08dd724d98b0ce4e5506fdc75e90d654e122f65dcb2afcb8436aaade7`.
The terminal witness root is
`7716e30c95b802a36f8c7d62dc993c5d912e678ec4fcb1617e364fba7898ded5`.

## Pre-solver control correction

The first implementation run stopped before one R49 generator pass at
`RESTORATION_HZ_WORKSPACE_REJECTED`. A dense control expected its projected
coordinate to reach the box face `0.25`, while the unchanged frozen projector
correctly stopped at the smaller normal-ball radius `0.03125`. The control was
corrected to the already frozen radius; no algorithm, parameter, work cap,
certificate or identity changed. That pre-solver result receives no HZ credit.

## Work, controls and rollback

Exact new work is 16 line JVPs, 16 fresh directed-certificate JVPs and 16 VJPs:
48 pair passes total. Sixteen projections use 976 bounded projection scans.
Dense inherited HZ, ball-box chord, active-switch/restart, exact line, rounding,
source, workspace, geometry, comparison, work, route and rollback controls pass.
Fourteen precedence routes close at route root
`b52c48763b3e1c964ef10e1874cba84a07ebefecaa6d353f9fd35d5ae3ce5350`.

Semantic result SHA-256:
`bbdfae6fbd301b74dbdbfde8bc416f814cf7e06ec74b8b54c8abd7718dac6544`.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r49-a.ahJEsy
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r49-b.SjsTST
binary SHA-256 6d69ec8d9828ef740462d15dedd145e177afeaf6a42636cdb0158d7d47e27047
size           7878552
ELF build-id   61dbb5dfce124ad5a2b0a24c546c590ed4e598ee

/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r49-a.AQUQTl
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r49-b.zeThbD
stdout bytes   1987
stdout SHA-256 7e1dde137b6164f67a448ab704eb713148e08c3a3a6e693614cb2645b01fa656
```

Both independently built binaries and both concurrently executed stdout
payloads are byte-exact. Each run reproduces exact R48 parent stdout SHA
`70a66443d0a278697e3e4d8d4456f80f5f1ccb1269596d3e5ce569e87d9b90b6`.
Concurrent wall time is intentionally not admitted as performance evidence.

## Consequence

Guarded Hager--Zhang is selected as the stronger primal generator for this
local compatibility problem. The remaining boundary is much narrower but still
real: 366 directed-positive rows with maximum upper `1.9771e-11`. Extending the
observed 16-step recurrence would select work after seeing the endpoint.

The next stage should research a distinct active-face closure mechanism over
the exact R49 witness, preserving the same contact/trust geometry and directed
certificate. Candidate mechanisms include a semismooth active-set Newton/
minimum-norm correction or a certified row-covering linear solve. No runtime or
restoration transaction is authorized first.
