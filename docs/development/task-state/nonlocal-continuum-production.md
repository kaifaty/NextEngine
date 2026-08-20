# Nonlocal continuum productionization — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NPR1_B_TERM_CONTROLS / REPORT_ONLY` |
| Updated | `2026-08-20` |
| Task key | `nonlocal-continuum-production` |
| Scope | Evidence-gated path from the retained Nonlocal GPU lab to one bounded basin production candidate |
| Definition of done | NPR8 selects production candidate or research stop from physical, authority, coupling, persistence and integrated-budget evidence |
| Authority | Working context only; SPEC-38, ADR-076/081, accepted runtime/physics ADRs and the main roadmap outrank this file |

## Resume in 60 seconds

- **Current conclusion:** NPR0 selects one exact report-only h3/16 basin
  profile; it is eligible for NPR1 falsification, not runtime integration.
- **Why:** v4 passes audit, repeated full-basin exact P1/P2 checks and all four
  h3-consistent tiny cases with declared 38,856 boundary capacity.
- **Next action:** implement NPR1-B independent energy/force directional
  derivatives, invariance and viscosity/surface controls.
- **Current blocker:** broader physical validity and canonical authority are
  unproven; the one-sided NPR0 hydro metric is insufficient for production.
- **Do not retry:** runtime/public contract integration from the old 50k
  benchmark; it is not product-profile evidence.
- **Reconsider when:** NPR0 selects one hash-bound basin-scale profile through
  independent small physical controls.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Fixed-work decision](../nonlocal-continuum-performance-decision-2026-08-20.md) | `NONLOCAL_50K_FIXED_WORK_RECLOSURE_CANDIDATE` | performance research may advance to a separate profile reclosure; no runtime credit |
| [Productionization roadmap](../../plans/nonlocal-continuum-production/README.md) | `ACTIVE / NPR0_PROFILE_BRIDGE` | profile/correctness/authority precede integration |
| [NPR0 contract](../../plans/nonlocal-continuum-production/00-profile-reclosure-contract.md) | `SPECIFIED` | three one-axis bridge identities precede boundary work |
| [Roadmap research](../nonlocal-continuum-production-roadmap-research-2026-08-20.md) | `PROFILE_MISMATCH_CONFIRMED` | current 3.2 ms result cannot be relabelled as basin performance |
| [NPR0 bridge evidence](../nonlocal-continuum-npr0-profile-bridge-evidence-2026-08-20.md) | `NPR0_A_B_SCALE_LAW_COMPLETE` | scale/cadence/support execution and derived scale algebra close; boundary/physics remain open |
| [NPR0 boundary contract](../../plans/nonlocal-continuum-production/01-static-boundary-discriminator.md) | `SPLIT_STATIC_BOUNDARY_SELECTED` | two-layer support and swept contact are separate mandatory operations |
| [NPR0 boundary research](../nonlocal-continuum-npr0-boundary-research-2026-08-20.md) | `GHOST_SUPPORT_NOT_CONTACT` | full support forces u32 CSR fallback; upstream boundary is not production sealing |
| [NPR0 boundary evidence](../nonlocal-continuum-npr0-static-boundary-evidence-2026-08-20.md) | `TINY_CORPUS_AUTHORIZED` | both support profiles execute exactly; negative contact discriminator selects split schedule |
| [NPR0 tiny corpus](../../plans/nonlocal-continuum-production/02-tiny-physical-corpus.md) | `EXECUTED` | immutable selection rule emits remediation and no selected profile |
| [NPR0 tiny-corpus evidence](../nonlocal-continuum-npr0-tiny-corpus-evidence-2026-08-20.md) | `PROFILE_RECLOSURE_REMEDIATION_1` | other tiny cases pass; hydro compression rejects both coefficient profiles |
| [NPR0 hydro remediation](../../plans/nonlocal-continuum-production/03-hydro-remediation-1.md) | `EXECUTED` | h2 fails through 50 iterations; h3 first passes at 16 |
| [NPR0 hydro-remediation evidence](../nonlocal-continuum-npr0-hydro-remediation-evidence-2026-08-20.md) | `H3_SUPPORT_REMEDIATION_CANDIDATE` | h3/16 requires explicit capacity/profile reclosure and full-corpus rerun |
| [NPR0 v4 discriminator](../../plans/nonlocal-continuum-production/04-h3-support-profile-discriminator.md) | `EXECUTED` | audit, repeated exact preflights and complete h3 tiny corpus pass |
| [NPR0 v4 selection evidence](../nonlocal-continuum-npr0-v4-selection-evidence-2026-08-20.md) | `NONLOCAL_PRODUCT_PROFILE_CANDIDATE` | NPR0 complete; exact h3/16 identity enters NPR1 only |
| [NPR1 research](../nonlocal-continuum-npr1-correctness-research-2026-08-20.md) | `NPR1_CONTRACT_INPUT` | reuse hash-bound water curves; paper visuals are not a numeric oracle |
| [NPR1 contract](../../plans/nonlocal-continuum-production/05-npr1-physical-correctness-contract.md) | `ACTIVE / NPR1-A_PASS` | term, smoke and nominal corpus work remain ordered |
| [NPR1-A evidence](../nonlocal-continuum-npr1a-canonical-evidence-2026-08-20.md) | `NPR1_A_PASS` | isolated half-even publication and frame/trajectory roots close exactly |
| Later `CONTINUUM-*` ProductChecks | `NOT_RUN` | no production claim |

## Decisions that still constrain the work

### D-NPR-001 — Profile bridge before integration

- **Observation:** old and product profiles share count but differ in physical
  scale, orientation, support, cadence and boundary.
- **Evidence:** tracked v1 profile JSON, SPEC-38 and the dated research report.
- **Decision:** isolate scale, cadence and support in separate v2 identities;
  specify the boundary only after those preflights.
- **Rejected alternatives:** one combined retuned basin profile or visual
  coefficient fitting.
- **Consequences:** NPR1 may execute only from the selected v4 identity;
  PhysX and public schemas remain blocked through NPR2.
- **Uncertainty:** whether unchanged or dimensionally derived coefficients can
  satisfy product-scale physical controls.
- **Reconsider when:** a scale-law derivation and independent tiny corpus pass.

### D-NPR-002 — Authority is an evidence decision

- **Observation:** the fast implementation is strict `f32` CUDA, while current
  SPEC-38 names CPU `f64` DFSPH as the sole canonical candidate.
- **Evidence:** ADR-076/081 and the fixed-work decision.
- **Decision:** NPR2 explicitly selects CPU, deterministic GPU or research
  stop after physical reclosure; no implicit GPU promotion.
- **Rejected alternatives:** quantize the final GPU state and infer exact
  history, or run CPU only as an occasional validator.
- **Consequences:** runtime architecture does not start before a credible
  authority route exists.
- **Uncertainty:** cross-device/cross-target exactness and CPU cost.
- **Reconsider when:** NPR1 has a selected profile and canonical output corpus.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: geometric/mass scaling with `h/dx=3` remains numerically finite | mass follows `rho*dx^3`; the paper varies resolution with fixed coefficients | paper range is narrow and does not test `10×` spacing | `nuv-basin-48k-source-scale.v2` exact P1/P2 preflight |
| H2: product cadence is stable with the control coefficients | SISSM is semi-implicit | `dt` enters terms differently and grows by `4.1667×` | `nuv-basin-48k-cadence.v2` after H1 |
| H3: product `h/dx=2` retains adequate support | fewer pairs may reduce cost | paper resolution study uses `h/dx=3`; free surfaces/boundaries may be under-supported | `nuv-basin-48k-spec-support.v2`, then tiny sealed hydro control |

The H1/H2/H3 executable preflights pass finite exact P1/P2 correspondence.
They remain open as physical hypotheses until the boundary-aware tiny corpus
and NPR1 reference corpus pass.

## Required context

1. [Continuum routing](../../architecture/agent-routing.md),
   [SPEC-38](../../architecture/38-continuum-material-physics.md),
   [ADR-076](../../architecture/adr/076-continuum-material-physics-track.md)
   and [ADR-081](../../architecture/adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md).
2. [Main roadmap](../../roadmap.md), R8 continuum row.
3. [Productionization roadmap](../../plans/nonlocal-continuum-production/README.md)
   and [NPR0 contract](../../plans/nonlocal-continuum-production/00-profile-reclosure-contract.md).
4. [Fixed-work decision](../nonlocal-continuum-performance-decision-2026-08-20.md)
   and [dated roadmap research](../nonlocal-continuum-production-roadmap-research-2026-08-20.md).

## Next action

1. Implement independent NPR1-B term/derivative and invariance controls.
2. Add the deterministic cell-grid trajectory runner and canonical decode-next
   schedule.
3. Execute smoke and reference attestation before any nominal corpus.
   before any nominal corpus.

## Do not retry

- P3 dynamic locality on coupled profiles — retained regression already closed.
- P4 Verlet cache with anchor CSR order — reproduced active-order mismatch.
- Pairwise Descent from title alone — no audited public formula/code yet.
- PhysX/public state before NPR0/NPR1/NPR2 — profile, physics and authority are
  unresolved.

## Handoff

- **Workspace state:** dedicated `codex/nonlocal-continuum-n0` worktree; new
  roadmap work is isolated from current runtime.
- **Checks:** NPR0 audit, scale law, split boundary, remediation matrix,
  repeated v4 P1/P2 exact checks and complete h3 tiny corpus pass.
- **Remaining risk:** broader physical validity, canonical authority, h3
  integrated cost, coupling, persistence and world-step budget are open.
- **Promotion needed:** later consumer-backed Accepted ADR only after NPR7.
