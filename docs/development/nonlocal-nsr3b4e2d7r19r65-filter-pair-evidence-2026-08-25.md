# NSR3-B4E2D7R19R65 projected filter-pair evidence

Date: `2026-08-25`

Status: `PASS / MARGIN_SENSITIVE_FILTER_PATH_CANDIDATE`.

Implementation commit: `d8a3a03b`.

## Strongest honest result

Every one of the 15 v6-blocked outers contains at least one normal-safe
projected candidate that strictly improves the full-row linearized density
violation enough to pass the weakest frozen filter envelope. None of the 235
normal-safe candidates reduces violation by the predeclared strong factor of
two. Strict intermediate inertia monotonicity therefore hides a real but
margin-sensitive feasibility path on this fixture.

This is bounded numerical/correspondence evidence. It does not choose a filter
margin, apply a step, establish a persistent filter lifecycle or inherit the
filter-SQP convergence theorem.

Route: `MARGIN_SENSITIVE_FILTER_PATH_CANDIDATE`.

## Result atlas

| outer | blocked | baseline `h` | best safe `h` | best relative reduction | best exponent | safe / envelope candidates |
|---:|:---:|---:|---:|---:|---:|---:|
| 1 | no | `5.5510537e-6` | `5.3795517e-6` | `3.09%` | 2 | 16 / 15 |
| 2 | yes | `4.2436627e-6` | `3.7537076e-6` | `11.55%` | 2 | 15 / 15 |
| 3 | yes | `2.8043070e-6` | `2.4032322e-6` | `14.30%` | 2 | 15 / 15 |
| 4 | yes | `1.9227453e-6` | `1.3140151e-6` | `31.66%` | 2 | 14 / 14 |
| 5 | yes | `1.4732995e-6` | `1.2814291e-6` | `13.02%` | 2 | 15 / 14 |
| 6 | yes | `8.1487047e-7` | `7.8359909e-7` | `3.84%` | 4 | 14 / 13 |
| 7 | yes | `5.1740652e-7` | `4.9120352e-7` | `5.06%` | 4 | 14 / 12 |
| 8 | yes | `3.5930573e-7` | `3.4846424e-7` | `3.02%` | 5 | 14 / 12 |
| 9 | yes | `3.2742885e-7` | `3.1537888e-7` | `3.68%` | 4 | 14 / 12 |
| 10 | yes | `2.6540012e-7` | `2.5624093e-7` | `3.45%` | 4 | 14 / 12 |
| 11 | yes | `2.3243758e-7` | `2.2276145e-7` | `4.16%` | 4 | 15 / 12 |
| 12 | yes | `2.0479337e-7` | `1.9624482e-7` | `4.17%` | 4 | 15 / 12 |
| 13 | yes | `1.8045463e-7` | `1.7161096e-7` | `4.90%` | 4 | 15 / 12 |
| 14 | yes | `1.6084080e-7` | `1.5170914e-7` | `5.68%` | 4 | 15 / 12 |
| 15 | yes | `1.4389717e-7` | `1.3557580e-7` | `5.78%` | 4 | 15 / 13 |
| 16 | yes | `1.2928944e-7` | `1.2194399e-7` | `5.68%` | 4 | 15 / 13 |

Across all outers, 208 of 235 normal-safe candidates pass the weakest
`gamma=2^-24` envelope; zero pass the strong `gamma=1/2` feasibility branch.
The best reduction varies from about 3% to 32%, so selecting the observed
minimum or best exponent would be outcome fitting.

## Integrity

- v6 solver semantic remains
  `bd568e0f367d34ef75f5ebeeca085f6cd6c36bb9fd56cb6966965a629ae8f0e2`;
- v7 proportioning semantic remains
  `9203252f9f330c4ce92bc2bcbe6ed091c361aadb915e28daf6a7ade26620ea03`;
- v8 semantic is
  `bdeeab4bd4d5db6b4f269de61fe84db020726690c22b464434935d8fcf218ab2`;
- all 255 fresh directed audits pass: 16 baselines plus 239 existing
  dual-decreasing candidates, exactly 154,311,720 directed slots;
- solver work remains 528 `A^T`, 272 `A`, 255 existing joint projections and
  468,968,743 structural terms;
- feasibility/objective admissions, dominated rejection, positive-part sign,
  source self-rejection, route precedence and rollback controls pass;
- no candidate, runtime filter, trust state or public state is mutated.

## Claim ledger

| Claim | Status | Evidence | Gap |
|---|---|---|---|
| F1: uniform factor-two feasibility path | `REFUTED_BOUNDED` | zero strong candidates | intentionally severe `gamma=1/2`; one fixture |
| F2: PCG path has no bicriteria value | `REFUTED_BOUNDED` | all 15 blocked outers have weakly acceptable safe candidates | weakest frozen envelope |
| F3: admission is margin-sensitive | `SUPPORTED_BOUNDED` | 15/15 weak, 0/15 strong; reduction atlas varies | no selected production margin |
| F4: v6 rejection is an audit artifact | `FALSIFIED_BOUNDED` | exact old roots and 255 fresh directed audits | shared continuum/operator lineage |

Evidence classes: `NUMERICAL`, `CORRESPONDENCE`.

## Next mathematical question

Do not fit `gamma` or immediately implement a persistent filter. R65 solves a
convex best-approximation TRQP. Before importing outer nonlinear filter
machinery into that inner layer, derive and measure its exact composed
Lagrange dual after the joint projection. If the same candidates ascend that
dual, the strict primal-inertia gate is simply the wrong inner merit; if they
do not, the filter path remains the justified next controller family.
