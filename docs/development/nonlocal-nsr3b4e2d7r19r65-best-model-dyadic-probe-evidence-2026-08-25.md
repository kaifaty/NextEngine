# NSR3-B4E2D7R19R65 best-model dyadic probe evidence

Date: `2026-08-25`

Status: `REFUTED / PROFILE-BOUNDED NUMERICAL EVIDENCE / NO COMPOSED DESCENT`.

Implementation commit: `cf6819ed`.

## Strongest honest result

The frozen claim that exhaustive selection of the best composed-inertia
candidate from the 16-value dyadic projected path strictly dominates the
FISTA reference at lower sparse work is `REFUTED` on the one frozen 6,000-row
binary64 fixture. This does not refute projected Newton, proportioning,
Nonlocal continuum mechanics or any production-scale claim.

Evidence classes: `NUMERICAL`, `CORRESPONDENCE`. The candidate and reference
share the R64 operator lineage, so this is not an independent mathematical
oracle for the continuum model.

The v6 harness returns `PASS`: the experiment executed exactly. Its scientific
route is `BEST_MODEL_DYADIC_NO_COMPOSED_DESCENT`.

## Decisive evidence

| outer | dual candidates | normal rejects | composed rejects | admissible | selected alpha | applied zeros | maximum raw | projected gradient |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 16 | 0 | 2 | 14 | `1/4` | 65 | `2.7675576999617465e-7` | `3.1641307482550777e-6` |
| 2 | 16 | 1 | 15 | 0 | none | 0 | `1.9993571421741890e-7` | `4.0196543874845109e-6` |
| 4 | 15 | 1 | 14 | 0 | none | 0 | `1.1868079415009041e-7` | `2.4065257072344697e-6` |
| 8 | 14 | 0 | 14 | 0 | none | 0 | `3.7743253335493082e-8` | `4.3249334051358442e-7` |
| 16 | 15 | 0 | 15 | 0 | none | 0 | `1.9801268253383156e-8` | `1.5057040456012030e-7` |

Across all 16 outers, all 14 admissible candidates occur in outer 1. Of 239
dual-decreasing candidates, 4 fail the positive normal-reference gate and 221
fail strict improvement over the post-Hildreth composed baseline. Only one
line is accepted.

Candidate and fresh commit model reductions agree bit-exactly when a line is
accepted. Parent stdout, joint reprojection, recurrence, stationarity, work and
rollback gates pass. Corrected semantic result SHA-256:
`bd568e0f367d34ef75f5ebeeca085f6cd6c36bb9fd56cb6966965a629ae8f0e2`.

## Work

```text
face PCG products                    240
dyadic candidate transposes          256
baseline / candidate projections 16 / 239
A^T / A calls                   528 / 272
structural terms              468,968,743
FISTA structural terms        596,971,680
dense Gram storage                      0
```

Sparse structural work remains `21.44%` below FISTA, but final maximum raw and
projected gradient are `16.75x` and `15.79x` higher. Exhaustive search is not a
viable acceleration policy on this fixture.

## Independent checks and controls

A classification-only replay changed route precedence from the incorrect
normal-first route to the predeclared composed-descent-first route. Every
numeric and structural field remained identical; only route and semantic hash
changed. This supports the claim that the result is not caused by reporting
logic. It is not an independent solver oracle.

## Claim ledger

| Claim | Status | Evidence | Ceiling / gap |
|---|---|---|---|
| v6 dominates FISTA terminal KKT pair at lower sparse work | `REFUTED` | deterministic binary64 fixture and exact work ledger | one 6,000-row fixture; shared operator lineage |
| candidate/commit composed model handoff is coherent | `SUPPORTED_BOUNDED` | zero model gap, exact reprojection and rollback | accepted candidates on this fixture only |
| monotone composed-inertia selection is a suitable global R65 policy | `REFUTED` | 221 composed rejects and only one accepted outer | this fixed path/direction/budget only |
| active-face PCG is being invoked in the wrong phase | `NOT_TESTED` | free/chopped gradient split absent | next discriminator |

## Next discriminator

Do not tune alpha, PCG depth or model tolerances. Measure the free and chopped
gradient norms at every post-Hildreth state, preserve the v6 trajectory and
work identity, and determine whether failed blocks are non-proportional active
faces or a genuine density/inertia merit conflict.
