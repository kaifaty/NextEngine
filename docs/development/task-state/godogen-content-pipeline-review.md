# Godogen content-pipeline review — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-26` |
| Task key | `godogen-content-pipeline-review` |
| Scope | Determine which evidence-backed Godogen ideas should inform future NextEngine asset, map and material generation, then promote the approved boundary into Proposed architecture. |
| Definition of done | Godogen evidence is mapped against current authority; SPEC/ADR/routing/traceability/roadmap record the provider-neutral candidate/promotion boundary, rejected borrowings, monitored upstream and smallest follow-up consumer. |
| Authority | Working context only; Accepted SPEC/ADR and `docs/roadmap.md` outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** SPEC-45/ADR-095 now record a `Proposed` provider-neutral, offline pre-cook boundary. Related Accepted SPECs constrain any future consumer without claiming implementation or altering the current runtime/cook baseline.
- **Why:** A new cross-context contract is now authorized, but ADR-046 still requires a concrete consumer before public schemas or commands become current.
- **Next action:** If implementation is explicitly scheduled, write the smallest `GEN-MATERIAL-P1` plan and freeze its concrete formats/bounds before adding public schemas or commands.
- **Current blocker:** None.
- **Do not retry:** Encoding Godogen's former rigid multi-stage prompt workflow, provider-specific APIs, live paid calls or direct runtime-asset writes into engine contracts.
- **Reconsider when:** A concrete creator consumer is approved for implementation, an offline/local model path becomes a current product requirement, or Godogen publishes a new stable artifact/provenance contract.

## Evidence and consequences

| Evidence | Result | Consequence |
| --- | --- | --- |
| Godogen `05cebffc8b10c5817e8a3db495b82e7b6004ab84` (`2026-07-02`) | Current project is a thin coding-agent runtime plus one asset-generation skill and one-page engine guides. | Borrow stable artifacts and proof loops, not a fixed orchestration state machine. |
| Godogen simplification commit `9ac4d847715c8b755ea88298bb034a94d4a7dfd0` | Removed the former planner/decomposer/scene/scaffold pipeline in favor of emergent planning and earlier live-game iteration. | Keep creative workflow in skills/guides; put only durable cross-provider semantics in SPECs. |
| Historical Godogen planner at `51954f9` | Recorded intended size, anchors/derivatives, budget, retry limits, risk ordering and visual proof. | Preserve these as typed authoring intent and reports, without making screenshots authoritative requirements. |
| Current `asset_gen.py` and `tripo3d.py` | JSON results, stderr progress, multi-provider image/video/mesh/rig operations and resumable paid jobs; provider identifiers, costs and output writes are tool-local. | Standardize stable JSON operations, local resumable job receipts and spend controls, but exclude provider job handles and price tables from engine contracts. |
| SPEC-03, SPEC-17, SPEC-24 and ADR-048/051 | Exact source bytes and hashes flow through deterministic cooking, exact locks and atomic publication. | Reproducibility starts at captured candidate bytes; cook, package, replay and runtime must never call a generator. |
| SPEC-09, SPEC-11, SPEC-15 and ADR-046 | Tooling is consumer-driven/current-only, remote integrations are untrusted/optional, and visual evidence is report-only. | A generative-authoring SPEC must remain `Proposed` until a concrete consumer and must keep live-provider checks non-gating. |
| SPEC-25 and SPEC-30 | Partition admission and render/material records already define runtime-facing structure and fallbacks. | Map and material generation should produce candidates for these existing boundaries rather than introduce parallel runtime authority. |
| Current Rust/project authoring schemas | Final PBR fields and provenance/license closure exist; generation briefs, derivation graphs, provider receipts, cost approval and candidate promotion do not. | Add pre-cook authoring contracts instead of widening runtime asset records with prompts or provider state. |

## Decisions

### D-001 — Promote the review only as a Proposed architecture track

- **Observation:** The follow-up request explicitly authorized SPEC updates, while ADR-046 still forbids claiming unconsumed formats as current.
- **Decision:** Add SPEC-45/ADR-095 as `Proposed`, connect future guardrails to the affected Accepted SPECs, routing, traceability and roadmap, and make no implementation/current-format claim.
- **Consequence:** The architecture now preserves the candidate/promotion boundary and Godogen watchpoint; a later consumer-backed Accepted successor decision is still required for public schemas/commands.

### D-002 — Define a future provider-neutral pre-cook boundary

- **Observation:** Generated output is untrusted and nondeterministic, while the existing cooker expects exact, validated inputs.
- **Decision:** A future `Proposed` SPEC should define four authoring artifacts:
  1. `ContentGenerationBriefV1`: asset kind/use, intended scale and units, texel density or tile size, target profile, topology/LOD/collider/rig constraints, reference hashes, acceptance criteria, cost and retry budget.
  2. `GenerationGraphV1`: typed anchor/derivative operations and exact input/output hashes, without provider SDK types.
  3. `GenerationReceiptV1`: adapter/profile/model identity, declared parameters, idempotency key, stage/status, attempts, quoted/actual cost and approval receipt; credentials and opaque provider handles remain developer-local.
  4. `GeneratedContentCandidateV1`: immutable output hashes, media kind, validation state, provenance/license/rights classification and preview references.
- **Rejected alternatives:** Storing only prompts; treating provider task IDs as durable asset IDs; making generated GLB/images directly activatable.
- **Consequence:** Explicit review/promotion converts a candidate into the existing project authoring/neutral import path, after which current deterministic cooking and locks govern it.

### D-003 — Separate structural validation, visual proof and authority

- **Observation:** Godogen's running-game proof loop is useful, but a screenshot cannot prove scale, topology, navigation, licensing or determinism.
- **Decision:** Structural checks are authoritative for admission; standardized preview scenes/cameras/lights and captures are report-only evidence; human or agent approval is an auditable promotion decision, not simulation authority.
- **Consequence:** Material previews use known primitives and lighting; prop previews include a scale reference and turntable; map previews complement connectivity, bounds, collision, navigation and streaming checks.

### D-004 — Start with three bounded consumers

1. `GEN-MATERIAL-P1`: generate a material set, validate color space/channel conventions/tile scale and render it on standard primitives before promotion.
2. `GEN-ASSET-P1`: image-to-static-prop candidate with scale, pivot, orientation, bounds, triangle/material budgets, collider and fallback validation.
3. `GEN-MAP-P1`: one bounded region/three chunks with persistent IDs, entrance-objective-exit connectivity, collider/navigation/render coherence, dependency and streaming-budget validation.

The order is intentional: it exercises existing neutral material/texture records first, then one mesh, then spatial composition. A general world generator or full editor is not the first consumer.

## Promoted architecture changes

- [SPEC-45](../../architecture/45-generative-content-authoring-and-candidate-promotion.md): owns the proposed brief, derivation, receipt, candidate, preview and promotion contracts and the monitored Godogen upstream.
- [ADR-095](../../architecture/adr/095-provider-neutral-generative-content-authoring-boundary.md): records the provider-neutral quarantined-candidate decision and ordinary authored-source fallback.
- SPEC-09: add future `next content generate|resume|inspect|preview|validate|promote` semantics with JSON stdout and progress on stderr; keep editor/MCP breadth consumer-driven.
- SPEC-24: classify generated files as untrusted candidates and bind selected source bytes, derivation and provenance into existing neutral records.
- SPEC-03 and SPEC-17: state that cook/package/replay consume captured exact bytes and never invoke a generator; selection changes the authoring hash/project lock.
- SPEC-11: require explicit remote-provider opt-in, credential isolation, data-disclosure and rights review, protected-source exclusion, spend ceilings and idempotent resume.
- SPEC-15: add recorded-response/captured-output fixtures for deterministic checks; live paid-provider smoke remains report-only.
- SPEC-25: define map-generation structural acceptance without deriving persistent IDs from geometry order.
- SPEC-30/SPEC-04: add authoring intent for real-world scale, texel density, normal/color conventions and standardized material preview, while preserving required renderer fallbacks.
- `docs/architecture/agent-routing.md`, `README.md`, `traceability.md` and `docs/roadmap.md`: route/index the track, define its observable future checks and keep it `PLANNED / NOT_ACTIVE`.

## Rejected borrowings

- Hard-coded Gemini, Grok or Tripo types in public contracts.
- Direct writes from a provider adapter into runtime/project authority.
- Claims that a prompt or seed can reproduce provider output bytes.
- Provider price estimates as durable architecture data.
- Spend confirmation implemented only as prompt instructions.
- A screenshot object inventory as the complete map/asset requirement set.
- Godogen's deleted rigid multi-stage pipeline as normative engine architecture.
- Live network/provider availability as a ProductCheck gate.

## Verification policy for a future implementation

- Recorded provider responses and captured exact output bytes drive repeatable contract/cook tests.
- Tamper, hash mismatch, path escape, missing rights/license, invalid dimensions/topology, timeout/resume and double-charge prevention have negative coverage.
- Repeated cook from the same promoted bytes is byte-identical.
- Live provider smoke tests are explicit, bounded, paid-network report-only checks.
- Map promotion additionally exercises play, save/reload and partition admission on the bounded scenario.

## Handoff

- **Workspace state:** SPEC-45/ADR-095 plus nine affected subsystem SPECs, routing/index/traceability, roadmap and this task-state form one documentation-only architecture change; executable implementation remains unchanged.
- **Checks:** `git diff --check` and direct local-link/ID plus exact Godogen HEAD validation pass; Cargo/ProductChecks are `NotRun(NoExecutableChange)`.
- **Remaining risk:** Godogen is a fast-moving experimental project with no tagged releases at the inspected commit; provider behavior and pricing remain external and unstable.
- **Promotion needed:** A concrete material consumer, frozen bounds/schema/report shapes, `GEN-MATERIAL-P1`/resume/promotion evidence and a later Accepted successor ADR before any public current command or schema.
