# Nonlocal corrected CUDA f64-energy evidence — 2026-08-30

Status:
`AUTHOR_SUPPORTED_CAUSAL / PRESSURE_OPERATOR_PRECISION_REQUIRED / REVIEW_NOT_RUN / REPORT_ONLY`

## Outcome

Deterministic GPU binary64 objective evaluation is causal but insufficient.
With f32 stored state, profile, gradient and Hessian unchanged, it accepts the
next trial that strict-f32 energy rejects in both retained static cases. It
reduces the final state error and residual, but neither case reaches the frozen
`R_x<=1e-7` scale-aware solve gate before minimum-radius termination.

| Case | Strict accepted / final `R_x` | f64-energy accepted / final `R_x` | Residual improvement | Mixed drift |
| --- | ---: | ---: | ---: | ---: |
| compressed pair | `2 / 1.32298e-5` | `3 / 2.77070e-6` | `4.77x` | `0.00917917 um` |
| combined tetrahedron | `11 / 5.73101e-7` | `12 / 3.00034e-7` | `1.91x` | `0.00284800 um` |

Mixed objective differences remain `1.97257e-6` and `3.08388e-6`, outside the
unchanged `1e-6` band. The result is therefore
`PRESSURE_OPERATOR_PRECISION_REQUIRED`, not a successful static solve.

## What the discriminator established

- the f64-energy path and strict path share state/profile/gradient/Hessian;
- at least one mixed trial is accepted exactly where strict f32 reports a
  non-positive actual reduction;
- rounding the f64 energy back to f32 restores a failing path;
- canonical/permuted mixed inputs are bit-identical;
- first-HVP inversion and nonfinite-state controls are rejected; and
- the combined mixed solve still executes multi-HVP residual CG.

The selected next change is therefore not a tolerance or a new controller.
Pressure density/compression, coefficient, gradient and Hessian products need
a bounded binary64 discriminator while state storage remains f32.

## Exact identities and verification

| Artifact | SHA-256/root |
| --- | --- |
| frozen contract | `9050a0e988ec17cc300d660441497a818a0304cd93775595153e6165e01e264b` |
| raw report, both clean runs | `87eeefc6ea53be8c81cb5e3a764f339e4332bf4a518bee5aa2b28b1c7c66fc82` |
| semantic result | `d1d02eef58da3abe01024f25cd8e9e35144c10017a17cc6a92e8f49b85e90ee3` |
| stripped Release binary, both clean builds | `aefe5efe62fe709edecfea2f0beae260ff9f3564a8f6d591d6b3eabc484eb253` |

Implementation checkpoint commit `c3b4e5d99d28b1b2eac96a32311ecdb9e3feb8d2`,
tree `357e64b9f79033614e2937cf2c24a113f259b90a`. CUDA assembly SHA-256 is
`d4ad4ab41a0cb28201cb2b56e9fb31d674db9b97f4cb8ee2d48e0b91e97020ab`;
driver SHA-256 is
`3594e5bdafcfe13adb040d729d94f3507f8a6fb93ba44499b0477cccd19c91b7`.

Two fresh Ninja Release builds produced byte-identical binaries and reports.
Compute Sanitizer `memcheck`, `initcheck` and `synccheck` each reported zero
errors. NCGA0--4 retained exact stdout hashes; the default NCGA5 report remains
exact `fa1e5f...fc03c`, and retained CPU NSR1 remains exact
`520258...678f`.

No performance timing or workspace ProductCheck was run after the failed solve
gate. Independent review is `NOT_RUN`.

## Claim ceiling

NCGA6 proves only that objective precision is one causal part, but not all, of
the two tiny static-solve failures. It grants no trajectory, dynamic-neighbor,
boundary/contact, GPU-resident-solver, 50k, frame-time, runtime or game-water
claim.
