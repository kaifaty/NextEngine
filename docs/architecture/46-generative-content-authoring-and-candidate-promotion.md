# SPEC-46: Proposed generative content authoring and candidate promotion

| Field | Value |
|---|---|
| ID | SPEC-46 |
| Status | Proposed |
| Version | 1.0 |
| Last verified | 2026-08-26 |
| Normative dependencies | [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-083](adr/083-public-creator-project-cli-vertical.md), [ADR-089](adr/089-governed-external-creator-sdk-workflow.md), [ADR-095](adr/095-provider-neutral-generative-content-authoring-boundary.md) |
| Promotion state | No current command, public schema, provider, runtime dependency or ProductCheck PASS; ordinary authored exact files remain the complete fallback |

## Status and purpose

This SPEC proposes an offline creator workflow for generating assets,
materials and bounded maps without giving a model, provider or agent runtime
authority. It defines the stable boundary from typed intent through untrusted
candidate bytes and explicit promotion into the existing project-authoring
path. The current cooker, neutral records, `ProjectLockV3`, package closure and
runtime activation remain unchanged and authoritative.

The proposal does not select an image, mesh, material, map or model provider.
It does not claim that prompts reproduce bytes, add generation to the game or
headless runtime, create a generic job scheduler, or make a visual comparison a
correctness oracle. Under ADR-046, exact public wire shapes and commands land
only with the first bounded production consumer.

The required fallback is ordinary creator-authored source files passing the
existing validate/cook/package workflow. Provider or network absence cannot
block that path, project activation, gameplay, save, replay or packaging.

## Authority and data classes

Generative authoring has four one-way data classes:

1. **Developer-only request state.** Prompt text, local references, provider
   session, credentials, quoted cost and opaque remote job handles live in a
   confined ignored workspace. They are never content or runtime authority.
2. **Quarantined candidate.** Downloaded or locally generated exact bytes plus
   bounded metadata are untrusted. Candidate bytes cannot be read by runtime,
   referenced by `ProjectLockV3` or written directly into an active project.
3. **Promoted project source.** An explicit atomic promotion copies selected,
   validated bytes under a new exact project-source identity and records the
   provenance/license decision. This is the first point at which generated
   bytes may enter normal authoring.
4. **Cooked content.** Existing SPEC-03/17/24 cooking, locking, packaging and
   activation own all subsequent identity and validation. They never call a
   generator or resolve a provider receipt.

Movement is only `request → candidate → promoted source → cooked content`.
Rejecting, deleting or losing developer-only state cannot mutate a promoted
source or active project. Changing the selected candidate bytes produces a
different project-source hash and therefore a different cook and project lock.

## Proposed authoring records

The following are logical V1 records. Until a concrete consumer freezes their
bounds and codecs, they remain tooling-local candidate contracts rather than
current `crates/contracts` API.

### `ContentGenerationBriefV1`

The brief binds intent that can be validated independently of one provider:

```text
ContentGenerationBriefV1 {
  brief_id,
  purpose,
  output_kind,
  target_profile_ids[],
  intended_units,
  intended_extent,
  texel_density_or_tile_scale,
  topology_lod_collider_or_rig_constraints,
  reference_content_hashes[],
  structural_acceptance_rules[],
  preview_profile_id,
  maximum_billable_cost,
  maximum_attempts,
  brief_hash,
}
```

`purpose` explains the in-project use, not only the visual subject. Scale,
axis, pivot/origin, texture color-space/channel convention, material tiling,
mesh/LOD/collider/rig expectations and map topology constraints are explicit
when applicable. Reference images or source assets are addressed by exact
hash; ambient clipboard, current viewport and mutable filesystem selection are
not request identity.

Billable limits use an exact non-negative integer amount plus an explicit
currency or provider-credit unit; binary floating-point money is forbidden.
Quotes and price-table revision/expiry are diagnostics, not request or content
identity, and exceeding the approved ceiling stops before submission.

### `GenerationGraphV1`

The derivation graph records anchor and derivative relationships between
bounded operations such as text-to-image, image-to-image, image-to-mesh,
material-map derivation, rigging, retargeting and typed map drafting. Nodes use
stable operation/candidate IDs and exact input/output hashes. Edges form a
closed directed acyclic graph; duplicate IDs, missing inputs and cycles reject
the graph.

A bounded adjacency list plus stable-ID map is the required representation.
Validation/topological ordering is `O(V + E)` time and `O(V + E)` memory. The
graph is lineage, not a generic scheduler: provider queues, worker handles,
retry trees and cache eviction stay private to the adapter/tool.

### `GenerationReceiptV1`

One receipt captures a billable or local attempt without becoming asset
identity:

```text
GenerationReceiptV1 {
  attempt_id,
  brief_hash,
  operation_id,
  adapter_profile_id,
  declared_model_identity,
  sanitized_parameter_hash,
  exact_input_hashes[],
  idempotency_key,
  stage,
  terminal_status,
  attempt_ordinal,
  approved_cost_ceiling,
  optional_quoted_cost,
  optional_actual_cost,
  exact_output_hashes[],
  diagnostic_code,
  receipt_hash,
}
```

Credentials, raw authorization headers and provider response bodies are
excluded. Opaque remote task handles MAY be stored in a separate encrypted or
permission-confined local resume sidecar, but cannot enter a project, package,
default log or public report. Provider names and price estimates are adapter
data, not engine enums or durable pricing authority.

### `GeneratedContentCandidateV1`

Each candidate binds one immutable exact output set, media/content kind,
declared units, structural-validation results, provenance/license/rights
classification, derivation root and optional preview evidence hashes.
Candidate status is closed and monotonic. The successful path is:

```text
Produced → StructurallyValid → Reviewed → Promoted
```

Any pre-promotion state may become terminal `Rejected`; rejection cannot be
relabelled as promotion without creating a new candidate identity.

`Promoted` requires `StructurallyValid`, a complete rights decision and one
explicit destination whose prior state is absent or whose exact base hash was
declared. Visual approval cannot skip structural validation. Promotion writes
through sibling staging, reopens the staged bytes, then performs one atomic
publication; fault leaves the project and candidate unchanged.

### `ContentPreviewPlanV1`

A preview plan fixes presentation-only evidence inputs: preview profile,
cameras, lighting/environment, background, scale reference, primitive or
turntable subject, extent, frame count and output quota. Captures have exact
hashes for comparison convenience but remain developer evidence. Pixels,
model scores and screenshot inventories never determine runtime content
identity or replace structural checks.

## Reproducibility and resume

The prompt, declared model and seed are useful lineage but do not guarantee
that an external or evolving generator will reproduce output bytes.
Reproducibility begins at the captured candidate bytes:

- every input and output is hashed before validation or promotion;
- repeated cook of the same promoted bytes must remain byte-identical under
  the existing cooker contract;
- cook, package, replay, runtime activation and save/load never submit or
  resume generation work;
- a provider retry always creates a new attempt/receipt even when the request
  text is unchanged;
- a promoted candidate is immutable; iteration creates a new candidate.

Before a billable submission, the tool must enforce an explicit positive cost
ceiling and attempt budget. Prompt wording such as “ask before spending” is not
enforcement. The adapter uses a provider idempotency key when available. After
an ambiguous timeout without an idempotent status query, automatic resubmission
is forbidden: the receipt becomes `UnknownRemoteState` and requires explicit
operator resolution so the tool cannot double-charge by guessing.

Resume reconstructs only developer tooling state from the local receipt and
private sidecar. It never reconstructs candidate bytes that were not
downloaded and hash-verified, and never treats a remote “complete” status as a
promoted result.

## Asset and material admission

The first proposed consumers are deliberately narrow.

### `GEN-MATERIAL-P1`

One material-set brief declares physical tile scale or texel density, expected
map channels, color spaces, alpha semantics, normal convention and target B0
fallback. Candidate validation checks exact dimensions, channel/encoding
compatibility, finite values, declared relationships and license/provenance.
Preview renders the candidate on a standard sphere, cube and plane under a
pinned B0 light/environment profile. Promotion produces ordinary texture and
material source records for SPEC-24/30; the generator receipt is not a runtime
material field.

### `GEN-ASSET-P1`

One static-prop brief declares real-world extent, up/forward axes, pivot,
triangle/material/texture budgets, required LODs and collider policy.
Validation checks bounds, indices, attributes, finite transforms, supported
materials, orientation, pivot, dependency closure and selected collider
relationship. Preview includes a scale reference and turntable. Promotion
produces ordinary mesh/material/texture/collider authoring input.

Rigging, retargeting, skinned characters and animation are later independent
consumers because their skeleton, skinning, BodySchema and fallback closure is
larger than the static-prop vertical.

## Bounded map admission

### `GEN-MAP-P1`

The first map consumer is one region with exactly three bounded chunks and a
declared `entrance → objective → exit` route. A map candidate is typed project
source data plus referenced asset candidates, not a screenshot, heightmap
alone or directly activatable scene.

Persistent object/chunk IDs are allocated from the authored brief and remain
stable across regeneration; they are never derived from triangle order,
provider node indices or filesystem paths. Before promotion, validation checks:

- unique declared region/chunk/object IDs and exact dependency closure;
- finite bounds and intended world units;
- required route connectivity and reachability;
- render/collider coherence and the declared navigation input relation;
- no cross-chunk reference to an absent or undeclared target;
- bounded chunk size, asset counts and streaming/admission limits;
- exact reload of the promoted source through current cook/activation.

The current `WorldPartitionManifestV1` and World Services contracts remain the
only runtime partition/navigation authority. A generated map cannot add a
parallel spatial database, infer durable IDs from geometry, or make visual
similarity a traversal oracle.

## Proposed command and report surface

The future public shape, if promoted by its first consumer, is a narrow
extension of `next content`:

```text
next content generate
next content resume
next content inspect
next content preview
next content validate
next content promote
```

Each command emits exactly one versioned JSON result/report on stdout;
progress and human diagnostics go to stderr. Commands are non-interactive once
launched, accept explicit roots/IDs/budgets and never depend on current editor
selection. Unknown provider, unavailable network, exhausted cost, occupied
destination, stale base hash or invalid candidate returns a stable typed
failure and publishes no partial promoted source.

This command family is not current. The material consumer should first prove
the smallest local/private equivalent; public schemas land only after its
bounds, reports and negative cases are demonstrated.

## Security, provenance and distribution

Remote generation is explicit opt-in. Before submission, tooling presents or
records the exact endpoint/profile, content classes leaving the machine,
reference hashes, declared retention/privacy mode and maximum billable cost.
Protected/imported Gothic data, secrets, unlicensed references and content
without redistribution rights are excluded.

Promotion requires a positive rights classification for every selected output
and reference chain. The chosen source hash and normalized provenance/license/
NOTICE facts feed the existing content closure. Raw prompt text, credentials,
remote session, private provider task handle, discarded candidates and price
quotes remain developer-only unless a project explicitly elects to preserve a
sanitized recipe as non-runtime source documentation.

Live provider availability and changing provider policy cannot be a release or
ProductCheck gate. Recorded responses and captured exact candidate bytes drive
repeatable contract tests; provider smoke is bounded, explicitly paid/networked
and report-only.

## Evidence and promotion gates

| Check | Required result |
|---|---|
| `GEN-MATERIAL-P1` | One material set passes structural checks, pinned standard preview, explicit review/promotion and repeated byte-identical cook from captured exact bytes; malformed material maps, rights failure and occupied destination publish nothing. |
| `GEN-ASSET-P1` | One static prop passes scale/pivot/orientation/mesh/material/collider budgets and preview, then follows normal neutral cook/package with required B0 fallback; corrupt topology and stale promotion base fail all-or-none. |
| `GEN-MAP-P1` | One three-chunk route passes identity, connectivity, bounds, dependency, collider/navigation and streaming admission, then play/save/reload from promoted source; regeneration cannot silently renumber durable IDs. |
| `GEN-RESUME-P1` | Recorded timeout/resume and provider-status permutations never duplicate a billable attempt, never promote an unverified remote result and preserve monotonic receipts. |
| `GEN-PROMOTION-P1` | Tamper, hash/license/path/base-version mismatch and publication fault retain candidate plus prior project; selected source bytes bind a changed authoring hash/project lock. |

No check is current. The promotion order is material → static prop → bounded
map. Passing a live provider smoke gives no gate credit. A full editor, generic
world generator, multi-provider scheduler and runtime generation remain out of
scope until separate production consumers justify them.

## Upstream watchpoint

[Godogen](https://github.com/htdt/godogen) is a monitored external reference,
not an authority, dependency or compatibility target. This proposal was
derived from its thin agent runtime, asset-generation/resume tooling and its
earlier typed asset-planning ideas, inspected at revision
[`05cebffc8b10c5817e8a3db495b82e7b6004ab84`](https://github.com/htdt/godogen/tree/05cebffc8b10c5817e8a3db495b82e7b6004ab84).

Revisit the upstream at the start of each bounded `GEN-*` consumer, or when it
publishes a stable release/contract affecting one of these questions:

- provider-neutral recipes, receipts or candidate lineage;
- idempotent paid-job resume and spend enforcement;
- provenance, license and rights closure;
- structural validation/promotion for materials, meshes or maps;
- standardized engine-running preview evidence.

Prompt-library changes, provider additions, price changes and workflow
reordering alone do not change Next Engine architecture. Any adopted upstream
idea still requires an explicit SPEC/ADR update and the affected product check.
