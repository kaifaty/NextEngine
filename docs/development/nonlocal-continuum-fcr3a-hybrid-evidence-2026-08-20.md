# Nonlocal FCR3-A hybrid-conditioning evidence — 2026-08-20

Status: `FAIL / CONDITIONING_BRANCH_CLOSED / FCR3B_SISSM_NEXT`

## Outcome

The sole allowed `block16-inertial64-warm-armijo-v2` remediation fails the
unchanged FCR3-A gate twice byte-identically. It restores compression quality
and reduces aggregate backtracks, but misses combined-case final gradient
quality by more than three orders of magnitude. The block/preconditioned
conditioning branch is closed; switch points or tolerances must not be tuned
further.

The next allowed experiment is a separately frozen corrected SISSM iteration
compared directly with the FCR2 objective/reference. This failure does not
invalidate the corrected energy identity.

## Results

Aggregate backtracks improve `3788 → 164` (`23.1x`).

| Case | Baseline final gradient | Hybrid final gradient | Backtracks baseline → hybrid | Minimum-alpha improvement | Result |
|---|---:|---:|---:|---:|---|
| compressed pair | `5.86e-6` | `3.34e-6` | `1735 → 96` | `2x` | quality PASS, alpha gate FAIL |
| surface repulsion | `1.76e-8` | `1.81e-11` | `1238 → 1` | `524288x` | PASS |
| combined tetrahedron | `3.31e-6` | `7.19e-3` | `815 → 67` | `134217728x` | FAIL quality |

The hybrid remains finite, monotonic, momentum-closed and directionally
correct. Its combined final objective `0.7567934974` is close to the reference
`0.7567934969`, but the frozen gradient gate correctly prevents treating an
under-converged state as equivalent.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| failing raw report, run 1 | `8b395d9139b215606ed5f2ba8975f7200eeb4fd7e57c43ebf26d5bcdc24166df` |
| failing raw report, run 2 | `8b395d9139b215606ed5f2ba8975f7200eeb4fd7e57c43ebf26d5bcdc24166df` |
| result root in report | `04144456f1f7dbdd9322e08f959495d3f2a453da5348cae71035159adebf60ff` |
| executable | `941b80615477e510d1f85950de1d5e0754d321939bd334d682adaa09631c2126` |
| unchanged FCR2 report | `10b98cb4d29935062d25a649994112c09598ece552023d88b22110340a92a2a8` |

Implementation commit: `6f92c1b`.

## Disposition

- Do not retry pure block v1, hybrid v2, alternate block-phase lengths or
  weaker gradient gates.
- FCR3-B may implement corrected SISSM with explicit objective/residual
  observation and an overshoot safeguard.
- If corrected SISSM fails its own bounded remediation, the formula-reclosure
  roadmap must choose another independently specified nonlinear solver or
  stop before profile/CUDA work.
