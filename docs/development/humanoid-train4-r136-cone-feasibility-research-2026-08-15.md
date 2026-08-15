# TRAIN-4 R136 projected fixed-PD cone-feasibility research — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Optimizer-free diagnosis after the sole R136 projected inverse-dynamics execution |
| Status | `R136_VALID_INFEASIBLE_RESEARCH_REQUIRED` |
| Confirmed boundary | `The exact projected q/v plus frozen fixed-PD effort lineage has no admissible pointwise contact-force witness at 2418/3200 collocations` |
| Gate decision | `STOP_VALID_PROJECTED_FIXED_PD_CONE_INFEASIBILITY_FOR_RESEARCH` |
| Claim ceiling | Pointwise CPU inverse dynamics only; no qdot/integration, release impulse, candidate, PhysX, corpus admission or training |

## Immutable R136 result

The sole R136 process ran from clean commit `3766e6c`, passed all six frozen
validations and completed without an invalid condition. It reconstructed R133
exactly once, verified the four real R133 arrays before the first inverse-
dynamics assembly, then classified all `3200` frozen systems exactly once.

The report is `COMPLETE`, the solver result is `VALID_COMPLETE`,
`invalid_reason` is null and the final feasibility is `INFEASIBLE`:

| Result | Count |
| --- | ---: |
| Numerically valid collocations | `3200/3200` |
| Cone-feasible collocations | `782/3200` |
| Cone-infeasible collocations | `2418/3200` |
| First infeasible collocation | `2` (`interval 0 / substep 2`) |

The execution closes its exact inventory: `3200` state lifts, `2640`
mass-metric projections/factorizations/solves, one controller schedule,
`3200` inverse-dynamics assemblies/SVDs/particular solutions, `2316`
flat-foot gauge classifications, `107668` equality rows, independent rank
`105352` and `4956` active point cones. It performs zero kinodynamic solves,
candidate builds, PhysX scenes, optimizer steps and training runs.

All R133 identities reproduce before the first ID system:

- projected generalized velocity:
  `e57bcb75629d403e04cc6d3197e8e2dada19ac0c0ef4c920083f2276c33bfb17`;
- projection delta velocity:
  `6308386172a7341885acb46b51ef68189fcd5be1a152db7a72f1dab8d0e86760`;
- applied target:
  `195810f7e9c20b15ec6ddf165c8873f905a96414fe055a0dae7a2b45382be1a3`;
- applied effort:
  `5ad7ac66c4d394ef5d06d111d86f58120069d42e61d1adf1ac82ca1d1d42a1ff`.

The frozen R135 synthetic target-shape defect is bounded as declared: R135's
synthetic metadata says `3200x23`, the real provenance-only target is
`800x23`, and R136 independently verifies the real hash before dynamics. The
applied effort remains the exact `3200x23` dynamics input.

Canonical/file/profile SHA-256 is
`b522dc92062d3f760536669cc30a053f6c11d845dfac9e09f2865de597daf6d5` /
`3721ce6acde8f3ef943f33a9811f550611b1b1cbe47364aa75b44b61c9e5e070` /
`6775f58d9e8cedac6f4cb574953e1a42abb881e2e026587fc15daf1421aada2e`.
The canonical hash independently recomputes exactly. The solver-private cache
is transient and non-authoritative; its SHA-256 is
`26b2ca90348db4a0b7105c43cff50d12f217d49f17c0e2390198b2658b2286c6`.

The numerical section used one process and one observed OS thread,
`87.883 s`, `126308352` bytes peak RSS, seed zero, no restart and no manual
intervention. Ruff check/format, the single-thread import, `369/369` lab tests,
`56/56` motor tests and full `host-check` all pass.

## Numerical validity is not the blocker

Every equality system is numerically valid and remains far from its frozen
rank and residual ambiguity bands:

| Guard | Frozen bound | Observed worst value |
| --- | ---: | ---: |
| Largest null relative singular value | `<= 1e-12` | `8.8975e-17` |
| Smallest retained relative singular value | `>= 1e-10` | `3.6909e-5` |
| Analytic gauge scaled residual | `<= 1e-10` | `1.5301e-15` |
| Analytic/SVD null-projector spectral error | `<= 1e-8` | `3.4436e-12` |
| Selected scaled equality residual | `<= 1e-9` | `7.5999e-12` |
| Selected backward error | `<= 1e-10` | `4.5134e-12` |
| Selected dynamics residual | `<= 1e-7` | `3.3411e-9` |
| Selected contact-closure residual | `<= 1e-7` | `8.3381e-11` |

The closest infeasible friction margin is `-0.127798 N`, versus the frozen
`1e-7 N` cone tolerance. The result is therefore not a boundary ambiguity,
rank-selection defect, residual failure or tolerance-tuning opportunity.

## Failure topology

Flight is always feasible. Contact modes fail broadly:

| Active points | Total | Feasible | Infeasible |
| --- | ---: | ---: | ---: |
| Flight (`0`) | `560` | `560` | `0` |
| Single point (`1`) | `324` | `12` | `312` |
| Flat foot (`2`) | `2316` | `210` | `2106` |
| **Total** | **`3200`** | **`782`** | **`2418`** |

The exact mode split is asymmetric but not localized to one transition:

| Contact mode `(left,right)` | Feasible | Infeasible |
| --- | ---: | ---: |
| `(0,0)` flight | `560` | `0` |
| `(2,0)` left forefoot | `12` | `140` |
| `(0,2)` right forefoot | `0` | `172` |
| `(3,0)` left flat | `189` | `1019` |
| `(0,3)` right flat | `21` | `1087` |

Failures are nearly uniform across substeps: infeasible counts are
`602/600/607/609` for substeps `0/1/2/3`. Of `800` motor intervals, `590`
are infeasible at all four substeps, `29` are mixed and only `181` are fully
feasible. The nine contact-exit intervals contain `32/36` infeasible rows, but
non-exit intervals also contain `2386/3164`. R131–R133 therefore closed the
real contact-exit actuator defect without making it the sole cause of R136.

All `2418` infeasible rows have a negative friction margin beyond tolerance;
`1345` also have a negative normal margin, while `1073` retain nonnegative
normal force and fail friction alone. The infeasible friction margin has
median `-312.326 N` and worst value `-1004.841 N` at collocation `2947`; the
worst normal margin is `-658.032 N` at collocation `1052`. By contrast, the
`222` feasible contact rows have minimum normal margin `31.569 N` and minimum
friction margin `-7.105e-15 N`, inside numerical tolerance.

Gauge handling is working as designed, not causing the stop. All `2316`
flat-foot gauges classify numerically; only `210` have a feasible interval and
`2106` are empty. Particular forces violate a cone at `2425` collocations,
and the exact gauge freedom rescues seven of them. It cannot rescue the broad
physical mismatch.

## Causal conclusion and decision

R136 rejects hypothesis H29: a controller-safe projected fixed-PD schedule is
not necessarily a dynamically admissible contact-force schedule. Equality,
rank, projection and force-gauge mechanics are closed within their exact
claims; the remaining failure is the frozen physical combination of projected
q/v, fixed-PD effort and rigid unilateral friction cones.

R136 cannot distinguish which stronger model should replace that combination.
The smallest remaining research choice is between:

1. a trajectory-level formulation that jointly chooses q/v/a, bounded effort
   and contact force while enforcing integration and contact transitions; and
2. a separately justified contact-semantics change, such as compliant,
   sliding or impact/release treatment, with renewed CPU/native correspondence.

Neither branch is authorized by this result. Raising friction, weakening the
cone tolerance, changing actuator limits, substituting a force witness, retrying
R136 or proceeding directly to a kinodynamic solve would change or bypass the
frozen contract and is rejected.

The exact transition is `R136_VALID_INFEASIBLE_RESEARCH_REQUIRED`. Freeze R136
and its cache, keep TRAIN-4 reopened and every TRAIN-5 checkpoint rejected,
and require a separate evidence-backed research/roadmap decision before any
new formulation or execution. This report grants no execution authority and
does not advance TRAIN-4, R5, Stage 0 or any learned-policy ProductCheck.
