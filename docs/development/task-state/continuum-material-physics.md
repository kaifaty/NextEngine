# Continuum material physics — current task state

| Field | Value |
| --- | --- |
| Status | `PAUSED_AFTER_RESEARCH` |
| Updated | `2026-08-16` |
| Task key | `continuum-material-physics` |
| Scope | Research and specification series for local water and deformable materials |
| Definition of done | Proposed architecture, implementation work packages, source-backed report, routing and roadmap are coherent and documentation checks pass |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact future ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** use one Physical Embodiment ownership umbrella with CPU DFSPH for the water oracle, GPU correspondence second and APIC/MLS-MPM for dry terrain; wet mud is later.
- **Why:** DFSPH evidence fits incompressible free surfaces, while APIC/MLS-MPM evidence fits elastoplastic/history-dependent solids; current ADR-058 and ADR-046 forbid silently promoting another production backend or speculative contracts.
- **Next action:** implement work package 01 only after selecting a bounded player-visible or tool consumer and freezing its particle/error budget.
- **Current blocker:** no selected production consumer or measured Next Engine solver evidence; every `CONTINUUM-*` check is `NOT_RUN`.
- **Do not retry:** one SPH solver for every material, GPU-first authority, or broad public schemas before a consumer.
- **Reconsider when:** a measured corpus shows a different method strictly dominates on the same correctness, conservation and budget criteria.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Research report](../continuum-material-physics-research-2026-08-16.md) | `REPORT_ONLY` | Supports the multi-lane Proposed architecture; proves no implementation |
| [SPEC-36](../../architecture/36-continuum-material-physics.md) and [ADR-072](../../architecture/adr/072-continuum-material-physics-track.md) | `Proposed` | Candidate ownership/failure/promotion semantics only |
| [Implementation series](../../plans/continuum-material-physics/README.md) | `Proposed` | Orders falsifiable work packages and stop conditions |
| `CONTINUUM-*` ProductChecks | `NOT_RUN` | No solver, performance, persistence or production claim is admissible |

## Decisions that still constrain the work

### D-001 — Solver family, not solver monoculture

- **Observation:** free-surface liquids and elastoplastic terrain need materially different state and discretization behavior.
- **Evidence:** linked primary DFSPH, APIC/MLS-MPM, sand and snow sources in the research report.
- **Decision:** CPU DFSPH water lane; APIC/MLS-MPM dry-terrain lane; separate material-specific SoA.
- **Rejected alternatives:** one generic particle struct and SPH formulation for every material.
- **Consequences:** shared infrastructure is limited to identity, bounds, scheduling, coupling and evidence, not numerical state layout.
- **Uncertainty:** measured Next Engine costs and quality thresholds do not exist.
- **Reconsider when:** equal-corpus measurements falsify the separation.

### D-002 — GPU is mirror-first

- **Observation:** parallel floating-point order and GPU execution profiles are not inherently cross-target exact.
- **Evidence:** Vulkan/SPIR-V specifications and CUDA floating-point guidance linked in the report.
- **Decision:** the first GPU water path is non-authoritative correspondence evidence.
- **Rejected alternatives:** final-output quantization as proof of deterministic trajectory equivalence.
- **Consequences:** production promotion must choose an explicit authority/replay model.
- **Uncertainty:** acceptable gameplay correspondence and target profiles remain unmeasured.
- **Reconsider when:** pinned target evidence and a production scenario support a narrower Accepted decision.

### D-003 — Persistence is an explicit transition

- **Observation:** active particle state to a compact sleeping field is generally approximate and can lose constitutive history.
- **Evidence:** architecture persistence invariants plus adaptive/conversion stability concerns in the report.
- **Decision:** require conservation/error receipts and repeated wake-cycle tests; otherwise pin active or disable persistent deformation.
- **Rejected alternatives:** call the conversion lossless or reconstruct dirty state from immutable base content.
- **Consequences:** sleeping format follows implementation evidence, not the other way around.
- **Uncertainty:** no candidate sleep representation has been tested.
- **Reconsider when:** a bounded conversion corpus passes repeated cycles.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: CPU DFSPH is affordable for one local gameplay region | Mature formulation and libraries | Published examples do not prove Next Engine real-time budgets | Package 01 at 10k/50k/100k particles |
| H2: Drucker-Prager MLS-MPM covers the first terrain scenario | Established sand model and contact behavior | May miss rate dependence, cohesive clay or saturated behavior | Package 05 curve corpus |
| H3: simplified saturation coupling is enough for mud | Smallest product-driven model | May not fit drainage and shear together | Package 06 contradictory-fit test |

## Required context

1. [Agent routing](../../architecture/agent-routing.md), SPEC-26 and current physics ADRs.
2. [Roadmap](../../roadmap.md), especially R8 and the permanent B-10 scope gate.
3. [SPEC-36](../../architecture/36-continuum-material-physics.md) and [implementation series](../../plans/continuum-material-physics/README.md).
4. [Research report](../continuum-material-physics-research-2026-08-16.md).

## Next action

1. Select one bounded water consumer and freeze scenario/error/particle limits.
2. Implement package 01 without public contracts or runtime integration.
3. Require repeat/conservation checks; remove the lab cleanly if it cannot meet them.

## Do not retry

- General `ContinuumMaterialSystem` public API first — ADR-046 requires a production consumer.
- GPU-first authoritative solver — cross-target semantics are unresolved.
- Adaptivity before fixed resolution — split/merge conservation and stability are unproven.
- Wet mud before dry terrain — too many unconstrained variables.

## Handoff

- **Workspace state:** documentation-only Proposed architecture/research/plan changes; no runtime or schema changes.
- **Checks:** `git diff --check` PASS; changed local Markdown link/path validation PASS; Cargo/host-check/ProductChecks NOT_RUN because the change is documentation-only.
- **Remaining risk:** numerical accuracy, performance, target correspondence and sleep conversion are all unmeasured.
- **Promotion needed:** later consumer-backed Accepted ADR/SPEC update; none is authorized by this state.
