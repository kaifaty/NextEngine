# NSR3-B4E2D7R19R35 equal-work generalized-Hessian evidence

Date: `2026-08-24`

Status: `PASS / FIRST_ORDER_REFERENCE_RETAINED / REPORT ONLY`

## Outcome

R35 executes four undamped generalized-Hessian outer iterations from exact
R33 `v8` under the same 51-pair-pass continuation budget as R34. All 20
matrix-free CG HVPs have positive curvature; no nullspace/curvature stop fires.

Compared with the R34 first-order endpoint:

| Direct terminal metric | R34 first order | R35 curvature | Ratio R35/R34 |
|---|---:|---:|---:|
| hinge objective | `1.6477454905319591e-16` | `4.6563584124278823e-17` | `0.28258966200688074` |
| violation norm | `1.8153487216135412e-8` | `9.650241875132335e-9` | `0.53159163086610073` |
| projected mapping norm | `1.4988794937300478e-9` | `2.177255315170091e-9` | `1.4525886332275224` |

Curvature is substantially better on feasibility energy and violation, but
worse on terminal stationarity. The frozen rule requires strict improvement
of all three metrics, so standalone curvature is not selected and the R34
first-order reference is retained.

The result motivates a separately frozen hybrid, not relaxation to a
two-of-three vote: preserve the first three curvature outers, replace the
fourth 12-pass curvature block with six 2-pass first-order polishing steps,
and keep the total budget at 51.

## Controls and authority

SPD, singular-consistent, active-switch and trust-projection controls pass.
The first dense implementation failure and formula-preserving finite-precision
repair are retained in the
[first-diagnostic record](nonlocal-nsr3b4e2d7r19r35-first-diagnostic-2026-08-24.md).

Exact R34 parent/baseline roots, `v8` prefix, four CG/globalization records,
fresh terminal operators, 51 passes/20 HVPs, all twelve routes and rollback
pass. Terminal maintained/direct response defect is `5.82400e-16`.

No correction, nonlinear moved-state evaluation, model/trial/outer or state
mutation occurs. No floor, timing, runtime or production authority is claimed.

## Reproducibility

Research/contract commit: `4e0a02d3`.

Implementation commit: `ef63cb57`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r35-a.H3y557`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r35-b.MbKVmL`.

Both binaries are `7,270,728` bytes, have SHA-256
`ff3e9bebc633d651895555738a7eff1f10dc8ddbe53f3936a16f4ba1e0b0483b`
and GNU build ID `f4871b5f3cb6f78ce8f92ddc18f05494c5ad8fab`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r35-a.TaD99Y`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r35-b.tIvKY2`.

Both exit `0`, have empty stderr and reproduce `3,634` stdout bytes exactly:

```text
stdout SHA-256  8f4161e6d4cf9ac24d1ee828dec7019ae0179bd5979c97ea8fec3f2910446fe9
semantic        bacfbc3d6a3a620d83bddac51cde4133169100e7300dd9850ae1851d6997c368
route           FIRST_ORDER_REFERENCE_RETAINED
```

These are correctness/reproducibility runs, not timing measurements.

## Next action

Freeze R36 as the exact equal-work hybrid: three inherited curvature outer
blocks plus six projected exact-line polishing steps from the same `v8`, with
prefix/terminal checks and total cap 51. Require strict dominance over R34 on
objective, violation and projected mapping before selecting the hybrid.
