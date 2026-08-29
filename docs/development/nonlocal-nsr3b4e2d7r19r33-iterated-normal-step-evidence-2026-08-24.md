# NSR3-B4E2D7R19R33 iterated all-inequality normal-step evidence

Date: `2026-08-24`

Status: `PASS / ITERATED_ALL_INEQUALITY_NORMAL_STEP_CANDIDATE / REPORT ONLY`

## Outcome

R33 proves that the R32 all-row Cauchy step is not an isolated improvement.
Eight projected exact-line iterations preserve the fixed global-L2 trust ball
and continue reducing the frozen linearized hinge objective.

| Accepted iterations | Hinge objective | Violation norm | Active rows | Step norm |
|---:|---:|---:|---:|---:|
| 1 | `2.6701495047730572e-15` | `7.3077349497269772e-8` | 1,450 | `1.7940547251618376e-7` |
| 2 | `2.2890059816305388e-15` | `6.7661007702081097e-8` | 1,378 | `8.1580254114545246e-8` |
| 4 | `1.8619103802752764e-15` | `6.1023116607975315e-8` | 1,252 | `5.7424044301597985e-8` |
| 8 | `1.2977364014902319e-15` | `5.0945782975438346e-8` | 1,154 | `4.6484639503287796e-8` |

Relative to the original R32 state, the violation norm falls to
`0.6278754454x`, a `37.21%` reduction, and the hinge objective falls to
`0.3942275749x`. Relative to the one-step R32 result, the final violation norm
is another `0.6971487516x` smaller.

The progression remains strict at every frozen doubling checkpoint. The
eighth step is smaller than the first, but it is not a numerical stall. The
terminal iterate norm is only `4.3836492729214171e-7` against trust radius
`0.25`, so this result is not caused by reaching the trust boundary.

The terminal projected-gradient mapping norm remains
`4.3315949134275429e-9`. R33 therefore proves continued first-order progress,
not solution of the normal subproblem.

## Controls and authority

All four independent dense controls pass. Iteration one reproduces the exact
R32 gradient, direction, response, line root, alpha and objective. Every
accepted nominal line has strict reduction and passes its direct line KKT and
JVP/VJP adjoint checks.

A fresh terminal JVP agrees with the maintained response at relative
`6.8649018658521866e-16`; a fresh terminal VJP and projected mapping are
finite. All eleven route cases, exact R32 parent bytes, source roots, the
18-pair-pass cap and rollback pass.

No correction, nonlinear moved-state evaluation, HVP, model, trial, outer or
state mutation occurs. No floor, timing, runtime or production authority is
claimed.

## Reproducibility

Research/contract commit: `4a8bbdfa`.

Implementation commit: `1693f02b`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r33-a.wQJ5xp`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r33-b.P3nYLg`.

Both binaries are `7,151,704` bytes, have SHA-256
`023ba963f672e68217727b40df3e5e6dbd9eff78f2f52c5c9baa9fd2bb61d281`
and GNU build ID `c2950b1fecdae0c13172b9ad6f528a3884c29337`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r33-a.dPy6wL`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r33-b.jO57rR`.

Both exit `0`, have empty stderr and reproduce `3,417` stdout bytes exactly:

```text
stdout SHA-256  6bab6bfb7e203ca45b387bbad04eefbf62aa03fdf187761c70f46d5fd7bc5464
semantic        61f21b041257c92222d61b4c454e44aa69c30b3da3e45d8ecc70f952db3e9ec0
route           ITERATED_ALL_INEQUALITY_NORMAL_STEP_CANDIDATE
```

These are correctness/reproducibility runs, not timing or performance
measurements.

## Next action

Freeze R34 as a continuation of the exact R33 prefix to checkpoints 16 and
32 under a declared operator budget. This creates a deeper first-order
reference. If it is not projected-stationary, compare a later
generalized-Hessian TRON/Newton-CG candidate against that reference at equal
operator work before choosing solver complexity.
