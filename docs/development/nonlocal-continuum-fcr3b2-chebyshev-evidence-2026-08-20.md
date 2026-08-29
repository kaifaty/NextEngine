# Nonlocal FCR3-B2 pressure Chebyshev evidence — 2026-08-20

Status: `FAIL / NON_DESCENT / FORMULA_RECLOSURE_STOP`

## Outcome

The single frozen pressure remediation fails reproducibly. The published
SISPH Chebyshev recurrence with `spectral_radius=0.9` creates a non-descent
direction after only a few accepted steps in every pressure-bearing case.
The exact FCR2 objective safeguard rejects the direction before it can become
state.

| Case | Accepted iterations | Backtracks | Final gradient | First failure |
|---|---:|---:|---:|---|
| compressed pair | 2 | 3 | `3.5003717287868646e1` | `NON_DESCENT_SISSM_DIRECTION` |
| combined tetrahedron | 7 | 6 | `2.5926589891297183` | `NON_DESCENT_SISSM_DIRECTION` |
| P mask | 3 | 4 | `2.2939431123591838e1` | `NON_DESCENT_SISSM_DIRECTION` |
| PV mask | 7 | 5 | `2.7387341468116548` | `NON_DESCENT_SISSM_DIRECTION` |
| PS mask | 3 | 4 | `1.8775149310691784e1` | `NON_DESCENT_SISSM_DIRECTION` |
| PVS mask | 7 | 6 | `2.5926589891297183` | `NON_DESCENT_SISSM_DIRECTION` |

The non-pressure `V`, `S` and `VS` paths remain exactly unchanged and pass.
This isolates the rejection to applying the fixed SISPH acceleration to the
pressure-bearing corrected objective; it is not a regression in the
underlying v1 maps.

The aggregate stiff-case objective-evaluation count is report-only here:
`1336` versus FCR2's `4028`. It cannot promote the candidate because the
mandatory quality gate fails first. The isolated repulsive-surface path also
retains its known `1238` backtracks, so a pressure-only change could not have
closed the original aggregate cost gate in any event.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| failing raw report, run 1 | `95c51f978953a784bdd9a7895fa825bd22c282253fd70403ade726393597cd06` |
| failing raw report, run 2 | `95c51f978953a784bdd9a7895fa825bd22c282253fd70403ade726393597cd06` |
| semantic result SHA-256 | `fae20ebac0baec8da5804d0bd016da9505e2a75afda169f9df0ff624fff142b3` |
| implementation commit | `2b99afbb9740cc08a19f46df3ff48d6f288d1764` |

Frozen non-regression report hashes remain:

- FCR0: `996eff3d61126491a3c1c92b6147d1c1f3eec0487d4b9dee45caadc6588b4345`;
- FCR1: `ead18de38f7e5fa68602c99f69cd891d11034502fe935fd473978040f2672140`;
- FCR2: `10b98cb4d29935062d25a649994112c09598ece552023d88b22110340a92a2a8`;
- FCR3-B v1 expected failure:
  `dee2744a9741a1f8fac40d32ca0b4201bb354f83274ab403750ccd38706cdc5b`;
- FCR3-B1 localization:
  `2ef351755844b658c05e89b19a4063282cd964246711bef6377c9d2d576b79dd`.

## Decision

Select `FORMULA_RECLOSURE_STOP`. FCR3-C through FCR7, profile selection,
CUDA correspondence, performance reclosure and runtime promotion are not
executed. The slow FCR2 binary64 optimizer remains a valid bounded objective
oracle, not a production solver.

Reopening requires a separately specified nonlinear method with its own
globalization argument or artifact, such as the authors' pairwise-descent
work when its paper and code become public. A spectral-radius sweep, silent
fallback, higher iteration cap or coefficient relaxation cannot continue
this lineage.
