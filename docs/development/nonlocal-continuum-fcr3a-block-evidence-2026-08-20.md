# Nonlocal FCR3-A block-conditioning evidence — 2026-08-20

Status: `FAIL / BLOCK_V1_REJECTED / ONE_BOUNDED_REMEDIATION`

## Outcome

`block-jacobi-gn-armijo-v1` is rejected as the sole FCR3 preconditioner. It
almost eliminates line-search backtracking but does not reach the FCR2 final
gradient quality within the same 80-iteration cap on the compression and
combined cases. Two runs fail byte-identically at
`FCR3A_BLOCK_QUALITY_FAILED:compressed_pair`.

This is not a formula or objective failure. The unchanged FCR2 report still
passes. The exact negative result suggests one bounded hybrid remediation:
use block curvature for coarse descent, then switch to the inertial reference
direction for final polishing while warm-starting the accepted line-search
step. Pure block v1 must not be retried or have its gradient gate weakened.

## Results

Aggregate backtracks fall from `3788` to `7`. Minimum accepted alpha improves
by `33,554,432x` for compression and `1,073,741,824x` for the combined case.
Those performance signals are real but insufficient to override the frozen
quality gate.

| Case | Baseline final objective / gradient | Block final objective / gradient | Backtracks baseline → block | Result |
|---|---|---|---:|---|
| compressed pair | `0.2000222821 / 5.86e-6` | `0.2000222977 / 7.31e-2` | `1735 → 5` | FAIL quality |
| surface repulsion | `-2.0386964432 / 1.76e-8` | `-2.0386964432 / 1.81e-11` | `1238 → 1` | PASS |
| combined tetrahedron | `0.7567934969 / 3.31e-6` | `0.7567934970 / 1.15e-3` | `815 → 1` | FAIL gradient |

Every path remains monotonic, finite, momentum-closed and preserves the
declared physical direction. The failure is specifically fixed-budget final
quality.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| failing raw report, run 1 | `f1f17fb37413596bfb2ce1fde8a78146c26273faa233a5b385b58d9363a32d19` |
| failing raw report, run 2 | `f1f17fb37413596bfb2ce1fde8a78146c26273faa233a5b385b58d9363a32d19` |
| result root in report | `9ea22058f62ae685d9b21bb161b6465b3b6e2c594b4bc7b23b1cb4521b237c86` |
| executable | `c7b176574eb4275011b3690e76d665b80b9420928a6c1a354d7483509134f531` |
| unchanged FCR2 report | `10b98cb4d29935062d25a649994112c09598ece552023d88b22110340a92a2a8` |

Implementation commit: `8771917`.

## Disposition

- Reject pure block v1.
- Allow exactly one hybrid v2 discriminator with the same total iteration
  cap and unchanged objective/gradient gates.
- If hybrid v2 fails, stop this conditioning branch and evaluate corrected
  SISSM or another separately specified solver directly against FCR2.
