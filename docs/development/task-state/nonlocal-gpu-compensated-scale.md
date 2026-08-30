# Nonlocal GPU compensated scale — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGP3_PRIMARY_WORK_BUDGET_REFUTED_PENDING_REVIEW` |
| Updated | `2026-08-30` |
| Task key | `nonlocal-gpu-compensated-scale` |
| Scope | Extend verified NCGP2 device `(hi, lo)` state to dynamic graphs, analytic boundaries, real rollback and 4k multi-step correspondence |
| Definition of done | NCGP3 passes the frozen tiny/graph/boundary/transaction and 240-step 4k gates, or records the first typed refutation without timing |
| Authority | Working context only; SPEC-38, ADR-076/081, FCR0 and frozen NCGP1–NCGP3 contracts/results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** NCGP2 independently verified H1. Canonical device
  `(hi, lo)` state closes the retained translated pair; surface-only f64 does
  not.
- **Current task:** close reproducibility and independent review of the first
  corrected-FCR2 240-step work-budget failure.
- **Completed apparatus:** pair-aware graph membership, pair-aware swept
  contacts and executable high/low rollback controls pass deterministically.
- **Material correction:** the first 4k density failure used the historical raw
  FCR1/NCGP1 kernel and exactly reproduced its known `~1/8` normalization
  defect. It is a negative control, not a refutation of corrected FCR2.
- **Current blocker:** corrected 16-step trajectories pass, but 240-step hydro
  reaches `WorkBudgetExceeded` at step 39 with the maximum admitted budget.
  50k and timing remain forbidden pending exact reproduction/review.
- **Do not retry:** the raw profile as physical evidence, host coordinate
  localization, high-only graph membership, tolerance widening or graph-only
  timing as a full-solver result.

## Evidence and decisions

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/development/nonlocal-gpu-full-step-evidence-2026-08-30.md` | global-f32 full step `VERIFIED_PHYSICS_REFUTED` | ordinary global binary32 state cannot proceed |
| `docs/development/nonlocal-gpu-surface-translation-evidence-2026-08-30.md` | `VERIFIED H1 SUPPORTED / GO` | retain both state parts through accepted updates |
| NCGP2 reviewer limitations | high-only graph and non-injected transaction controls were explicitly deferred | they are load-bearing in NCGP3 |
| NCGP3 revision-1 contract | SHA-256 `4bc9f67b77457b28e5259ea2d4efb6aeb0019edab43270909f70ef0aa6827c5c` | implementation may not change corpus, gates or claim ceiling |
| NCGP3 revision-2 graph-control corrigendum | SHA-256 `f026e33b7c5e9476d0af40f55acb59a95123713a90f198b060b930368b2e84f0` | physical corpus remains shared binary32; only the graph apparatus control admits direct pair bytes |
| NCGP3 revision-3 boundary/transaction addendum | SHA-256 `5f487d3dc097b20becb2cfffd41b6886210ce213a0773a810496e54df2273558` | high-only boundary and root-only rollback controls are no longer admissible |
| Raw NCGP3 hydrostatic density run | lattice ratio `0.12522433816880058`, density mean `114.988 kg/m^3` | reproduces NSR3B0 raw-profile ineligibility; retain as negative control |
| NSR3B0R selected FCR2 profile | `kernel_scale=7.985668078772472`, corrected material anchors | use this profile in every physical NCGP3 path |
| NCGP3 revision-4 corrected-FCR corrigendum | SHA-256 `5ba46c6043a8942ad522f7a9299cc6a69b1cf06d34e19cd8cccea00d55b4bb18` | resolves profile lineage and one-sided compression-density semantics before code |
| Corrected FCR2 profile/lattice gate | corrected density `999.9999999999986 kg/m^3`; raw `125.2243381688004 kg/m^3` | normalization and profile lineage closed |
| Corrected one-step 4k hydro, budget 128 | PASS; position max `0.230 um`, compression max `0.0240%`, active/permutation exact; 81 GPU HVP | smallest successful one-step budget is 128 |
| Corrected 16-step 4k corpus | hydro/dam/orifice PASS; position max `10.3/9.4/9.1 um`; compression max `0.132/0.124/0.132%`; permutation exact | short trajectory correspondence supported |
| Corrected 240-step hydro | deterministic failure after 38 completed steps; corrected/permuted `WorkBudgetExceeded`, equal work roots, `126/128` HVP consumed before a 3-HVP Jacobi outer boundary | frozen complete-corpus work ceiling is not met; later correctness/performance blocked |

### D-001 — Reconstruct only graph addresses in binary64

- **Observation:** integer-micrometre graph membership currently reads only
  the high part, so a valid low part can be ignored at a support boundary.
- **Decision:** derive graph quantization from `double(hi)+double(lo)` and keep
  physical formulas on the verified NCGP2 binary32 profile.
- **Rejected alternatives:** rounding `hi+lo` back to one float, local host
  anchors or promoting the complete solver state to binary64.
- **Consequence:** graph identity consumes both parts without selecting a new
  physical arithmetic profile.
- **Reconsider when:** an exact boundary fixture or independent review shows
  that integer-micrometre quantization is inconsistent across CPU/GPU.

### D-002 — Require executable rollback evidence

- **Observation:** NCGP2 root mutation controls were sufficient only for its
  finite boundary-free claim.
- **Decision:** NCGP3 must inject a post-finalize failure, corrupt both state
  parts internally, restore every transaction buffer and prove exact recovery
  by state root plus successful retry.
- **Rejected alternatives:** semantic-root sensitivity alone or checking only
  the published high vector.
- **Consequence:** transaction work and recovery become part of the frozen
  scalable claim.

### D-003 — Do not select the pressure-f64 discriminator for a work failure

- **Observation:** primary f32 reaches the frozen maximum work budget while all
  observed state/density correspondence metrics remain well inside their
  gates. No isolated pressure-operator error has been established.
- **Decision:** retain the implemented pressure-f64 route as an unselected
  discriminator; do not use it to bypass the primary work-ceiling result.
- **Rejected alternatives:** raising HVP beyond 128, weakening convergence,
  classifying a near-threshold multi-step active switch as a force error, or
  proceeding to timing after the first failed correctness gate.
- **Consequence:** formal next work is reproduction and independent review,
  not dam/orifice 240-step runs or 50k performance.

## Next action

1. Freeze the expanded failure receipt and commit the exact candidate.
2. Produce two clean Release builds/runs for the 240-step hydro request and
   require identical numerical/result/work bytes apart from binary identity.
3. Run the retained controls and proportional sanitizer routes.
4. Request one read-only independent NCGP3 review. If it confirms the work
   failure, record `VERIFIED_PHYSICS_REFUTED` and leave performance `NOT_RUN`.

## Handoff

- **Workspace:** branch `codex/water-research`, clean before the NCGP3 docs
  change; NCGP2 final documentation commit `41bca030`.
- **Performance status:** `NOT_RUN`; prior ~1.0–1.18 ms p95 result is only the
  neighbor stage and is not a full-water timing claim.
- **Promotion:** none. This remains Proposed report-only research.
