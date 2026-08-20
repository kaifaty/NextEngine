# Nonlocal FCR3-B corrected SISSM v1 evidence — 2026-08-20

Status: `FAIL / TERM_LOCAL_DISCRIMINATOR_NEXT / PROFILE_BLOCKED`

## Outcome

`corrected-sissm-armijo-v1` fails twice byte-identically, first at combined
fixed-budget gradient quality. It passes every isolated term case, but the
combined tetrahedron ends at gradient `3.68e-3` versus FCR2 `3.31e-6`.
Aggregate stiff-case objective evaluations are `2880` versus reference
`4028`, missing the required `4x` reduction.

The exact failure authorizes one term-localized combination discriminator. It
does not authorize coefficient tuning, Chebyshev acceleration or profile
reclosure.

## Results

| Case | Reference final gradient | SISSM final gradient | Iterations / backtracks | Result |
|---|---:|---:|---:|---|
| compression | `5.86e-6` | `2.14e-7` | `80 / 1402` | quality PASS, cost poor |
| normal viscosity | `8.08e-8` | `3.94e-11` | `20 / 0` | PASS |
| shear viscosity | `7.48e-11` | `8.51e-11` | `33 / 18` | PASS |
| surface repulsion | `1.76e-8` | `1.76e-8` | `80 / 1238` | PASS, no gain |
| surface attraction | `8.65e-11` | `4.52e-10` | `80 / 966` | PASS, slower |
| combined tetrahedron | `3.31e-6` | `3.68e-3` | `80 / 0` | FAIL |

Every accepted path is finite, monotonic, momentum-closed and directionally
correct. The failure is convergence quality/cost, not an energy increase.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| failing raw report, run 1 | `dee2744a9741a1f8fac40d32ca0b4201bb354f83274ab403750ccd38706cdc5b` |
| failing raw report, run 2 | `dee2744a9741a1f8fac40d32ca0b4201bb354f83274ab403750ccd38706cdc5b` |
| result root in report | `aad92c4e377d535db3b93d27fb98bc540e5597b7ea4653a8b159b4d4dd3e8fc4` |
| executable | `307346b5075fcb592ccbb1e51ce4cfe471b4d4a795bbc4618dd79250af33ab3b` |
| unchanged FCR2 report | `10b98cb4d29935062d25a649994112c09598ece552023d88b22110340a92a2a8` |

Implementation commit: `db27ad7`.

## Next discriminator

Run the identical tetrahedron under pressure-only, viscosity-only,
surface-only and all three pairwise term combinations. Compare each corrected
SISSM result to FCR2 with the same 80-iteration quality gate. The first
failing minimal combination identifies the only admissible remediation scope.
