# Nonlocal NSR2-A1 scale-aware evidence -- 2026-08-20

Status: `FAIL / BLOCK_PRECONDITIONER_REJECTED / UNPRECONDITIONED_SELECTED`

## Outcome

The mass-normalized displacement stop fixes the NSR2-A scale defect: all three
unpreconditioned 8/27/64-particle baselines now converge with no rejected trial
or radius change. The block preconditioner passes 8 and 27 particles but fails
the frozen 64-particle work/quality gate.

| Fixture | Baseline eval/HVP | Block eval/HVP | Candidate result |
|---|---:|---:|---|
| 8 | `8 / 18` | `7 / 15` | PASS |
| 27 | `8 / 16` | `6 / 13` | PASS |
| 64 | `9 / 24` | `22 / 83` | FAIL: 13 rejected trials and trust contraction |

For the 64-particle case the block-scaled trust geometry triggers two boundary
steps and contracts from `0.05` to `7.45e-10 m` despite reaching an objective
within `1.5e-13` of the baseline. The large-case aggregate is consequently
`96` candidate versus `40` baseline HVPs, failing the frozen `<=75%` gate.

This does not repeat FCR3-A's conclusion: the full Hessian remained the Krylov
operator and the candidate is mathematically valid. It is rejected because its
local metric is a poor scaling of the global trust region on the 4x4x4
coupled lattice. No block blend, shift, clamp or switch-point tuning is allowed.

The new stopping rule itself is selected for scaling research. It corresponds
to a maximum inverse-inertial displacement residual of `1e-8` particle spacing
and removes dimension-dependent raw-gradient tail chasing without changing an
accepted objective step.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| raw report, run 1 | `bc3c0d1cfc11efe5b9c4dcbad42bb161344c7dc12b7d2505218cecaff2a3a2f5` |
| raw report, run 2 | `bc3c0d1cfc11efe5b9c4dcbad42bb161344c7dc12b7d2505218cecaff2a3a2f5` |
| semantic result | `ed170fd955109d21b883a7debbccf88402408eec8f9fb6895d1abaae2681f65c` |

The original NSR2-A FAIL remains byte-identical at
`2ea06e5f0d24295ad3730c0285628f0db64bd35885e93bd58e5810c7d3942012`.

## Decision

Select the unpreconditioned full-HVP solver plus the scale-aware stopping
criterion for NSR2-B. Reject `block-gn-metric-v1`. This does not prove that no
useful preconditioner exists; a future candidate must approximate global
coupling without redefining the trust geometry in this failed way.

