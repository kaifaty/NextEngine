# Nonlocal NSR3-B1S acoustic-policy evidence -- 2026-08-20

Status: `FAIL / LINEAR_ACOUSTIC_POLICY_REJECTED / REPORT_ONLY`

## Outcome

The frozen `C<=0.25` policy passes all three stiffness scales at initial
spacing `0.99 dx` and fails all three at `0.98 dx`. Every matrix trajectory is
finite, conservative, capacity-valid and self-convergent; the failure is the
predeclared accuracy limit, not nonlinear-solver behavior.

The linear policy `c=sqrt(kappa/m)` therefore does not cover the tested finite
compression amplitude. `ACOUSTIC_SUBSTEP_POLICY_R0` is rejected.

## Matrix

| Spacing | `kappa` factor | Substeps/frame | `q_x / q_v` | Error / `dx` | Error / `c` | Result |
|---:|---:|---:|---:|---:|---:|---|
| `0.99` | `0.25` | 17 | `1.894 / 1.874` | `0.00467` | `0.000458` | PASS |
| `0.99` | `1` | 34 | `1.883 / 1.874` | `0.01034` | `0.000458` | PASS |
| `0.99` | `4` | 67 | `1.910 / 1.905` | `0.02210` | `0.000467` | PASS |
| `0.98` | `0.25` | 17 | `1.865 / 1.846` | `0.01286` | `0.001244` | FAIL |
| `0.98` | `1` | 34 | `1.854 / 1.846` | `0.02827` | `0.001244` | FAIL |
| `0.98` | `4` | 67 | `1.853 / 1.849` | `0.05975` | `0.001258` | FAIL |

The frozen limits are `0.05 dx` and `0.001 c`. All 2% cases fail the velocity
limit; the high-stiffness case also fails position. Kinetic errors remain
`0.107--0.117`, below `0.15`, and pressure-exit error remains within one policy
step. Ratios near `1.85--1.91` show that refinement is behaving consistently.

Every level ends with zero pressure-active centers, never exceeds its initial
density, uses zero rejected trust trials and remains below `3 / 0 / 6` maximum
outer/reject/HVP work per step. The largest policy cost is the explicitly
published 67 substeps per frame; the 4x reference reaches 268.

## Degenerate controls

At `kappa=0`, the policy returns one substep. One-frame free flight and rigid
translation both pass with zero HVP calls; position error is zero and velocity
error is at most `1.7e-15`. The policy's zero-stiffness branch is valid.

## Interpretation

The failure is amplitude-dependent and almost stiffness-scale invariant after
normalizing velocity by `c`. This is consistent with a missing finite-state
tangent-stiffness factor rather than a bad square-root scaling in `kappa`.

The pressure Hessian already contains both the positive `kappa J^T J` term and
the compression-dependent geometric term. A deterministic estimate of the
largest mass-normalized pressure eigenvalue can measure the actual local
frequency directly. That is preferable to lowering the global Courant target
after seeing one failed amplitude.

## Repeatability and lineage

- B1S semantic result SHA-256:
  `1537a691dfa821c6ccc7ce168c4d4c74868b10db95075a615eaf887f443eabee`;
- two byte-identical raw reports:
  `4d57744c4c7ff393d156d2c86510570783bd41982b421a00a36e3d6e8c6ff62c`;
- B1D1 remains byte-identical PASS at
  `2a9ddd605cc1ec89a8cb71fd24c239d08416ab398d09dfa3c37c80a9913adcc1`.

## Decision

Reject `ACOUSTIC_SUBSTEP_POLICY_R0` without changing its target or thresholds.
Freeze B1S1 to measure pressure-only tangent spectra across the same amplitude
and stiffness matrix, including an independent dense tiny oracle and
deterministic Krylov convergence. B1R/B2, CUDA and runtime remain blocked.

