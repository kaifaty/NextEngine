# Nonlocal NSR2-A block-preconditioner evidence -- 2026-08-20

Status: `FAIL / SCALE_DEPENDENT_STOPPING_CRITERION / PRECONDITIONER_NOT_SELECTED`

## Outcome

The frozen NSR2-A gate fails before it can select the block preconditioner.
Both paths reach the same objective to approximately binary64 precision, but
the inherited absolute `||g|| <= 1e-10` stop becomes unattainable or
meaningless as particle count grows. The solvers then reject near-zero-progress
trials until the minimum trust radius is crossed.

| Fixture | Unpreconditioned | Block candidate | HVP baseline -> candidate |
|---|---|---|---:|
| 8 | PASS | minimum-radius after objective agreement | `23 -> 116` |
| 27 | minimum-radius at `||g||=1.92e-7` | PASS at `1.84e-12` | `112 -> 18` |
| 64 | minimum-radius at `1.31e-9` | minimum-radius at `1.60e-6` | `241 -> 117` |

For 27/64 together the block path passes the non-gating HVP reduction
observation (`353 -> 135`, `38.2%`), but the mandatory per-case success gates
fail. The candidate is therefore not selected and no blended/switching variant
is attempted.

The terminal gradients correspond to extremely small inertial displacement
residuals because `mass/dt^2 = 7200`: even the largest `1.60e-6` aggregate
gradient is far below a nanometre-scale per-particle correction after inverse
mass/time scaling. A dimensionful raw vector norm also grows with particle
count. This identifies the frozen stopping rule, not objective disagreement,
as the first boundary.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| raw failing report | `2ea06e5f0d24295ad3730c0285628f0db64bd35885e93bd58e5810c7d3942012` |
| semantic result | `2183d667d1311c5a4b4f12b494ea79ae84c4b3e41484657ddd66a65c371d7f90` |

## Decision

Retain this FAIL. Open NSR2-A1 solely to replace the non-scalable termination
oracle with a predeclared mass-normalized maximum displacement residual. Keep
the objective, HVP, block metric, trust policy, fixtures and `75%` HVP gate
unchanged. NSR2-A1 is one criterion remediation, not a preconditioner tune.

