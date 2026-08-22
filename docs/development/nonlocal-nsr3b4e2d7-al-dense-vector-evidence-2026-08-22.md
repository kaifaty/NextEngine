# NSR3-B4E2D7 augmented-Lagrangian dense-vector evidence

Date: `2026-08-22`

Status: `FAIL / PRESSURE_STATE_UNSTABLE_AT_COMMIT / NO_TRAJECTORY`

## Reproducibility and stop boundary

Two clean Release builds produce byte-identical 4,642,024-byte executables at
SHA `c595feb0c0254e976a955a38a3646a951b88f05de4bac2e2a0fdf7a6d671761c`
and Build ID `80f7bb7159c565a909c74f0c2fbbb4cfaa822b0d`.

Fresh process A exits one with empty stderr and a 4,570-byte report at SHA
`e0542abc4e0c7ff0b38acc0fe38095e270dec030617ad5183665414ce9b3db11`.
Its semantic result is
`0453794038d8788e4ae8e6a77a691619a6e0769aa910b82291031c7d042cbab4`.
The frozen cost ladder correctly stops after the hard control failure, so
process B is `NOT_RUN(FIRST_PROCESS_HARD_FAIL)`. Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7.17bosW`.

## Passing numerical controls

The full 24-coordinate/eight-multiplier oracle closes every derivative and
inner-solver boundary:

| Control | Result | Frozen limit |
|---|---:|---:|
| directional gradient error | `2.732411132287143e-10` | `1e-7` |
| HVP error | `8.057510362644277e-10` | `2e-6` |
| dense symmetry error | `4.199775755776337e-15` | `2e-12` |
| dense product error | `9.540934821681501e-16` | `2e-12` |
| pressure reaction closure | `1.7298958462588385e-16` | `1e-12` |

Every accepted trust-region trial has positive actual and model reduction;
all cold inner solves pass without a rejected trial. Inactive state, zero-
multiplier reset, forced rollback and multiplier-mutation controls also pass.

Cold primal violation decreases monotonically over all eight outer updates:

```text
3.9107638798374644e-4
6.6541920197504822e-5
1.1374740289671337e-5
1.9459519333686615e-6
3.3298364554568138e-7
5.6943943782528095e-8
9.7424577294447090e-9
1.6669525759738235e-9
```

The cold state root is
`04a9c03308662d6102b165d8109b23c1b60a3145314603909b7dbfda8385d95e`.

## Failure and diagnosis

The eighth cold record passes the frozen dimensionless condition
`max(delta_lambda)/beta <= 1e-8`, but one additional warm update moves the
multipliers by `3.497549225794927e-7 J`, exceeding the independent absolute
warm-state limit `1e-8 J`. Its normalized position change is only
`5.123111573784287e-9 dx` and the warm primal falls further to
`2.852231784089554e-10`.

The two gates therefore answer different dimensional questions. With
`beta=1226.25 J`, the cold criterion can admit an absolute update as large as
`1.22625e-5 J`; it cannot certify pressure-state stability at `1e-8 J`.
Observed primal contraction remains regular at approximately `0.171x`, so the
result does not support changing the AL formula, tuning `beta` or selecting the
semismooth fallback.

## Decision

Preserve B4E2D7 as a hard FAIL. The next discriminator must retain the same
fixture, derivatives, `beta`, inner solver and first eight cold records, but
use one dimensionally explicit absolute multiplier-update admission followed
by a private confirmation update before state commit. A larger outer cap under
a new identity is convergence certification, not a performance claim. Nominal
Dam, runtime pressure persistence, GPU/PhysX and production remain blocked.
