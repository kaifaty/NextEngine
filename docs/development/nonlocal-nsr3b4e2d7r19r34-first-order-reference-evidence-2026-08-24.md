# NSR3-B4E2D7R19R34 32-step first-order reference evidence

Date: `2026-08-24`

Status: `PASS / EXTENDED_FIRST_ORDER_REFERENCE_CANDIDATE / REPORT ONLY`

## Outcome

R34 extends the exact R33 prefix to a deterministic 32-step all-inequality
first-order reference.

| Iteration | Hinge objective | Violation norm | Active rows | Step norm |
|---:|---:|---:|---:|---:|
| 8 | `1.2977364014902321e-15` | `5.0945782975438353e-8` | 1,154 | inherited |
| 16 | `6.5228220370984118e-16` | `3.6118754234049691e-8` | 1,036 | `3.2910927575165715e-8` |
| 32 | `1.6477454905319594e-16` | `1.8153487216135412e-8` | 928 | `1.7073707009799305e-8` |

The block contractions are:

```text
phi16 / phi8                    0.5026307368436338
phi32 / phi16                  0.25261236335445636
violation16 / violation8       0.70896455260022262
violation32 / violation16      0.50260557433683162
phi geometric factor, 8 steps  0.91760575856265014
phi geometric factor, 16 steps 0.91760001633352328
```

The nearly identical per-step factors show stable linear convergence across
both continuation blocks; no conditioning slowdown is observed through step
32. Relative to the original R32 state, violation norm falls to
`0.2237305662x` (`77.63%` reduction) and hinge objective to
`0.0500553663x` (`94.99%` reduction).

The terminal projected-mapping norm remains
`1.4988794937300478e-9`, so R34 is a high-quality fixed work reference, not a
stationarity certificate. The iterate norm is `9.4181499341243399e-7`, still
far inside the fixed radius `0.25`.

## Controls and authority

The ill-conditioned long-horizon dense control strictly improves from 8 to
32 steps; early and boundary stationarity controls pass. A fresh prefix JVP
reproduces the exact R33 terminal response root and agrees with its maintained
response at relative `6.86490e-16`.

Every nominal continuation step passes strict decrease, direct line KKT,
trust feasibility and JVP/VJP adjoint checks. Fresh terminal response agrees
with maintained recurrence at relative `6.39297e-16`. All eleven route cases,
the exact 51-pair-pass ledger and rollback pass.

No correction, moved nonlinear evaluation, HVP, model, trial, outer or state
mutation occurs. No floor, timing, runtime or production authority is claimed.

## Reproducibility

Research/contract commit: `a7ec6ca0`.

Implementation commit: `31402e68`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r34-a.12orlJ`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r34-b.6hOCAg`.

Both binaries are `7,207,080` bytes, have SHA-256
`f0ac86296b1577e69a566953e64daebb419a616d2ee83da6fc64d04af51a7481`
and GNU build ID `bc3e6345bfe3513865f4209044fa50dcf2d7b5ce`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r34-a.AeUrap`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r34-b.YzDaPZ`.

Both exit `0`, have empty stderr and reproduce `2,743` stdout bytes exactly:

```text
stdout SHA-256  5150f1f697072a21a81ce7d6fc7aa4548ce603e7e27796143483fa0ad796f345
semantic        00dcf602a9266d8170a8c84af1f338903650f4c767a24d7cf00249a9837583b7
route           EXTENDED_FIRST_ORDER_REFERENCE_CANDIDATE
```

These are correctness/reproducibility runs, not timing measurements.

## Next action

Research R35 as a separately frozen generalized-Hessian curvature
discriminator over the same all-row squared-hinge objective and trust ball.
Its total matrix-free operator work may not exceed the R34 reference budget.
Compare direct terminal objective, violation and projected mapping; do not
apply either iterate or evaluate a moved nonlinear state yet.
