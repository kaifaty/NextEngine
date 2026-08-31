# NCGP12 Nonlocal pressure-state discriminator evidence

Date: `2026-08-31`

Author result: `NONLOCAL_SUPPORT_REDESIGN_REQUIRED / REVIEW_PENDING`

Claim ceiling: `512-SAMPLE CPU LINEARIZED PRESSURE ORACLE ONLY`

## Question and answer

NCGP12 tested whether the corrected Nonlocal density Jacobian can generate a
nonnegative constraint pressure before another trajectory or GPU run.

The numerical pressure solve succeeds. It closes derivative, convex-QP, KKT,
complementarity, exact-density and stationarity gates with all five negative
controls. The complete frozen route nevertheless rejects because the pressure
field alone does not satisfy the independent wall-contact/profile gates:

- only `60/512` multipliers are positive and both frozen bottom/top medians are
  zero;
- the lowest trial position crosses the exact particle-centre inset by
  `0.178781 mm`.

The exact route is therefore `NONLOCAL_SUPPORT_REDESIGN_REQUIRED`, not a PASS.
It says the pressure-state idea is numerically viable, but the current ghost
density support cannot replace the analytical wall constraint. A successor
must couple pressure projection with analytical contact before attempting a
trajectory.

## Frozen identity

- revision-3 contract commit: `6036192efef33107b44bbf28f935c5dd44593ff2`;
- implementation commit: `4a788ea15d2af0444e28f11eb332329144d14e7c`;
- implementation tree: `fb23ee3fc1aec4a0c304a753cca9cc113e03134d`;
- aggregate contract root:
  `27df304703efc7e51d9509ea337a82f2146412bdbfc72944d0ceef7e6866a256`;
- source root:
  `0e97ff43d1eab8912abde3da05f60488a4679c3aecdf312f511b466da4571424`;
- input root:
  `82cf83cbfc5839026c8dff77c55b81be2e5597982c31493a7016a9f5fca2a8fa`;
- clean Release binary A/B SHA-256:
  `008799c4577517b7ee13e2d45c7a8b21cdaacdcec58cf999cdf177231d1d2185`;
- byte-identical stdout A/B SHA-256:
  `be63239a385e1a9263acc99eb9f22f251548e331ffa2d502e748b7a475e92655`;
- final result root:
  `df013dd766e09cb50fccdd6b9ee502bcaaaeeb19da13b4fda680dfce94744698`.

Clean build directories:

- `/tmp/nextengine-ncgp12-a.nOMpye`;
- `/tmp/nextengine-ncgp12-b.pF1OAn`.

Raw reports remain outside Git at `/tmp/ncgp12-a.json` and
`/tmp/ncgp12-b.json`.

## Exact command

```text
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B <fresh> -DCMAKE_BUILD_TYPE=Release
cmake --build <fresh> \
  --target nonlocal-corrected-cpu-pressure-state -j2
<fresh>/nonlocal-corrected-cpu-pressure-state \
  --pressure-state-discriminator
```

Both processes exit zero because they reach a valid scientific classification;
`APPARATUS_INCONCLUSIVE` would exit nonzero.

## Numerical result

The fixture contains `512` dynamic samples and `43,056` immutable three-layer
basin ghosts. Surface and viscosity are disabled only for causal isolation.

| Observable | Frozen limit | Result |
| --- | ---: | ---: |
| directional Jacobian relative L2 | `<=2e-7` | `2.8023155e-11` |
| matrix symmetry relative L2 | `<=2e-12` | `2.9436821e-20` |
| pressure primal residual | `<=1e-8` | `9.9027351e-9` |
| projected KKT residual | `<=1e-8` | `1.5590102e-11` |
| complementarity | `<=1e-10 J` | `5.9493921e-12 J` |
| linearized identity relative L2 | `<=2e-12` | `1.9226988e-19` |
| exact maximum positive strain | `<=1e-3` | `3.5742406e-6` |
| exact RMS positive strain | `<=2.5e-4` | `5.8010394e-7` |
| normalized stationarity | `<=1e-8` | `2.0784084e-20` |
| exact inset penetration | `0` | `1.7878107e-4 m` — FAIL |
| bottom median pressure > top | strict | `0 Pa / 0 Pa` — FAIL |

The cyclic projected coordinate solve uses `726` complete sweeps and `43,612`
coordinate updates. Candidate and stable-ID-permuted multiplier, trial, work and
result roots are exact. The candidate roots are:

- multiplier:
  `e3a62beac5ce4ebb2d39e9baf8ce8a28d95f43ab9291c11f415ba20cb6975b6a`;
- trial:
  `00f5d0ff3bc77716e4a3da4a209bef8ab1382de6c317710058495e9ecd27b0c5`;
- work:
  `7e569aef9998a03f38762a3a2e7fb068c593240b272c7f7d7362daad9a8df596`;
- metric result:
  `3d43636f76dbe6cdeeebe8aac85941ee235cc646052044c251d088cbbd6e19d1`.

## Controls

All frozen controls pass:

- zero multiplier retains `100%` of the positive predicted-density violation;
- negated pressure reaches maximum/RMS strain
  `0.00284574 / 0.000552138`, rejecting the wrong sign;
- removing ghost derivatives reaches maximum/RMS strain
  `0.00142270 / 0.000256906` and penetrates `0.170348 mm`;
- stable-ID permutation is root-exact;
- one published binary64 `nextafter` multiplier mutation changes its root.

The exact work receipt includes `22,306,816` density candidates,
`39,453` admitted pairs, `38,941` derivative pairs and `4,770,216` matrix
products for each candidate assembly. Finite-difference and nonlinear controls
are separately counted in the work root.

## Interpretation and next action

This result rules out two simplistic conclusions:

1. the corrected Nonlocal density Jacobian is not intrinsically unable to
   produce a converged nonnegative pressure correction;
2. that correction alone is not a complete basin boundary treatment.

It does not distinguish whether surface tension was the other cause of NCGP10
fragmentation, because surface was intentionally absent. It also does not
prove a nonlinear multi-step pressure state, correct water or GPU feasibility.

After independent review, the smallest successor is a newly frozen CPU test
that composes the same pressure projection with swept analytical contact,
relinearizes after projection/contact, and runs a small 240-step hydrostatic
trajectory. Surface remains a separate controlled reintroduction after the
pressure/contact trajectory holds.

Relevant primary literature remains the 2026
[Nonlocal framework](https://doi.org/10.1145/3799902.3811196) and the
[DFSPH incompressibility analysis](https://doi.org/10.1109/TVCG.2016.2578335).
They motivate the experiment but do not certify this implementation.
