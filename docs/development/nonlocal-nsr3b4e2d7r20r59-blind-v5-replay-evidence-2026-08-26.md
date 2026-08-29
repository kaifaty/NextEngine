# NSR3-B4E2D7R20R59 blind v5 replay evidence

Status: `PASS / BLIND_V5_BOUNDARY_IDENTIFIED`.

## Valid result

- semantic `a0881eaa583ed3ed05e9a743957d008db4a45c0c9100f0dd94e458ff9051faa9`;
- aggregate case root
  `469087b01833fa6946361fbc0f1da63711da3dc666bd37535db6e320f8f3bda0`;
- 5/6 immutable v5 problems certify without a centered or slope callback;
- the remaining oblique-jet/twist case stops at
  `VERIFIED_INVERSE_AUDIT_REJECTED` after 1,092 principal transitions;
- its sole R50 request is rejected at the frozen dimension-policy boundary;
- all parent, observation, policy, work and lifecycle accounting is exact;
- no retry, source change, parameter change, new mechanism, factorization,
  centered arithmetic, slope audit or timing is admitted.

| case | route | accepted | transitions | centered calls/dimension rejects | slope calls |
|---|---|---:|---:|---:|---:|
| triaxial saddle | `ORDINARY_CERTIFIED` | 10 | 972 | `0/0` | 0 |
| layered xz shear | `TERMINAL_CERTIFIED` | 8 | 1,006 | `0/0` | 0 |
| checkerboard compression | `ORDINARY_CERTIFIED` | 8 | 790 | `0/0` | 0 |
| oblique jet/twist | `ACTIVE_SET_REJECTED` | 8 | 1,092 | `1/1` | 0 |
| radial expansion/drift | `ORDINARY_CERTIFIED` | 12 | 1,383 | `0/0` | 0 |
| three-face coupling | `TERMINAL_CERTIFIED` | 10 | 1,457 | `0/0` | 0 |

## Invalid first execution

The first execution returned
`FAIL / BLIND_V5_POLICY_ACCOUNTING_REJECTED` and receives no scientific
credit. The slope hook was armed around every NNQP solve, while the
`slope_refinement_attempted` flag was set before requiring an exact,
terminated, KKT-certified solve. On the oblique case the active-set solve had
already failed, so no callback ran although the accounting expected one.

The apparatus correction moved the existing exact/KKT eligibility predicates
before setting the attempted flag. It changes no formula, solution, cap,
source or default-null path. The corrected run classifies the unsupported R50
dimension exactly as the frozen contract requires.

## Reproducibility and regressions

Two corrected independent executions are byte-identical at stdout SHA-256
`461c49623727f8bcfd7ee609a81e4a5706002c611eaa50c8e089e65bc2ce44ac`.
The following parent semantic hashes remain exact:

- R56 `daf7c2ceb1c1e803feb992abc5975a3dff980487a583fb4d203d165b605d20b7`;
- R57 `2e5b935306919c0797a4f3b6f77edd8dfc4dcead3281ed274fc5b7b885523b9a`;
- R58 `966c72648900e7b551091b4a7025312cbce7b4f71b8e3b7f48a5126c90ddbfad`.

## Claim boundary

R59 provides blind generalization evidence for the composed research policy,
not production readiness. It identifies one exact unsupported active-set
dimension and freezes that tuple as the next research target. R60 may audit a
dimension-generic centered certificate report-only; it may not simply remove
the dimension guard, apply a correction, time the solver or alter runtime/GPU
paths.
