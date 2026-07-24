# SPEC-15: Headless testing, agent validation и human evidence

| Поле | Значение |
|---|---|
| ID | SPEC-15 |
| Статус | Accepted |
| Версия | 1.8 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md), [ADR-024](adr/024-requirement-gate-evidence-and-profile-closure.md) |
| Заменяет | отсутствует |

## Назначение и invariants

SPEC-15 задаёт единый public verification contract для engine, first-party content, mods, physical policies и coding agents. Mandatory acceptance MUST выполняться без interactive game launch, monitor, keyboard/mouse или display server. Human reviewer оценивает только hash-bound artifacts после успешных automatic gates.

Testability является свойством production architecture: scenario runner использует production input/WorldCommand paths, probes читают immutable projections, capture воспроизводит exact replay. Test-only mutable ECS/component/backend access запрещён.

## Source of truth и ownership

| Состояние | Единственный owner/source of truth | Не является source |
|---|---|---|
| Verification policy, profiles и review categories | Project `VerificationPolicyManifest` | agent prompt, CI YAML |
| Scenario definition | Immutable `TestScenarioManifest` revision/hash | test runner flags outside manifest |
| Required affected suites | Generated `ChangeImpactManifest` от engine `ImpactResolver` | author/LLM estimate |
| Gameplay outcome | Runtime WorldCommand/DomainEvent/state/replay owners | screenshots, human opinion |
| Capture request | Immutable `CaptureJobManifest` | worker-local defaults |
| Evidence completeness | `EvidenceBundleManifest` + verified artifact hashes | CI green label |
| Approved baseline | `EvidenceBaselineManifest` + human attestation | generated candidate files |
| Qualitative decision | JCS-canonical `HumanReviewDecisionV2` + cryptographically valid `AttestationEnvelopeV2` от authorized reviewer; admission отдельно требует exact `Approve` + automatic `PASS` | agent text, unsigned comment, bundle-supplied trust root, valid `Reject`/`NeedsChanges` |

Verification & Evidence Team владеет schemas, runner orchestration semantics, impact resolver, artifact roles и gates. Subsystem owner владеет assertions/thresholds своего behavior. Release Engineering принимает root evidence; Security & Governance владеет reviewer trust/redaction policy.

## Public boundary и data flow

Public contracts: `VerificationPolicyManifest`, `TestScenarioManifest`, `ScenarioAction`, `ProbeSpec`, `AssertionSpec`, `ChangeImpactManifest`, `OffscreenPresentationTarget`, `CapturePlan`, `CaptureJobManifest`, `EvidenceBaselineManifest`, `EvidenceBundleManifest`, `HumanReviewDecisionV2`, `AttestationEnvelopeV2`, `ProjectTrustAnchorV1`, `ReviewerTrustManifestV1` и `ReviewerRevocationSnapshotV1`. Scenario inputs reference the exact `ProjectCompositionLock`, `ApplicationSessionManifestV1`, canonical `PlayerActionFrame`, revisioned RPG/World Services state, physics/motor/animation projections and `PresentationSnapshotV2` through their owning engine contracts; the verification layer does not redefine them. V1 review payload/envelope остаются только historical-audit schemas и не входят в current admission surface.

```text
AgentChangeSet
  → ImpactResolver → ChangeImpactManifest
  → CPU TestScenario run → RunManifest + replay + probes/diagnostics
  → optional CaptureJobManifest → displayless GPU worker
  → raw frame/audio roots → MediaEncoder derivatives
  → EvidenceBundleManifest → automatic admission gate
  → offline review dossier → HumanReviewDecisionV2 + AttestationEnvelopeV2
  → cryptographic verification → verified Approve | Reject | NeedsChanges
  → automatic-gate-aware admission → exact changeset admit or non-admitting result
```

CLI/JSON является normative surface. CI/MCP/GUI/static HTML являются projections. Public contracts не содержат CI vendor, window-system, Vulkan handle, encoder library или artifact-store vendor types.

## VerificationPolicyManifest и profiles

Project-owned manifest MUST фиксировать schema/revision/hash, `review_contract_major = 2`, reference CPU/GPU/platform profiles, suite/tag dependency rules, observable-category mapping, time/resource budgets, long-gate classes, reviewer roles, baseline policy, artifact quotas/retention requirements и redaction policy. Current gate/admission path MUST reject any V1 review artifact as `REVIEW_SCHEMA_HISTORICAL_ONLY`.

| Profile | Обязательное назначение | Admission semantics |
|---|---|---|
| `agent-fast` | affected schema/unit/property/small deterministic scenarios на CPU Linux | target ≤5 min; feedback only, не заменяет changeset gate |
| `changeset` | все resolved ordinary suites, replay/fault checks, capture jobs и review requirements | target compute ≤30 min excluding queue/human; required before merge |
| `nightly` | full seeds, fuzz, mutation, stress и cross-package suites | обнаруженный regression блокирует subsequent admission/release |
| `milestone` | Windows/Linux/reference GPU, packaging и full vertical-v1 | release/tag gate |

Physical certification, training correspondence, long fuzz и performance endurance MUST быть marked `long`. Они MAY работать asynchronous и не входят в 30-minute target, но promotion связанного model/package запрещён до PASS.

## TestScenarioManifest

Manifest MUST содержать:

- ScenarioId/version/hash, owner, required features и tags;
- exact `ProjectCompositionLock` plus content/package/schema/trust/budget/config/model/build/toolchain hashes;
- initial cooked fixture либо SaveManifest hash;
- named RNG streams/seeds и fixed gameplay/physics/motor rates;
- exact action-map/input-context, RPG aggregate schema/revision and world calendar/population/tier profile hashes used by the scenario;
- ordered `ScenarioAction` timeline;
- declared adapter fault injections;
- `ProbeSpec`, `AssertionSpec` и optional CapturePlan references;
- required profiles, time/resource limits и expected diagnostic policy;
- positive, negative/failure и save/replay roles;
- provenance/license/redaction classification.

`MechanicTestScenario` из SPEC-13, physical suites SPEC-05/14 и vertical scenario SPEC-12 являются specializations одного runner/schema registry.

Scenario initialization выполняется composition resolver/cooker/fixture loader до first tick from one exact `ProjectCompositionLock`. После старта разрешены только timestamped semantic controls through the production action resolver and `PlayerActionFrame` ingress cutoff, validated WorldCommand/MechanicCommand proposals, process/device/adaptor fault controls и lifecycle commands. RPG/calendar/population/tier changes use their production command/transaction paths. Fault adapter заменяется только в composition root и не открывает arbitrary state mutation.

### Split-fixture profiles

`vertical-v1-import-smoke` использует user-provided installation только во временном isolated root и публикует sanitized provenance/hash metadata, validator/load projection и source-access trace. Imported/NIM/cooked bytes, source paths/strings и capture media не могут попасть в publishable evidence. Cleanup/quarantine MUST завершиться до VS-09 и исключить root из repository/cache/build/package/final/evidence/capture access.

`vertical-v1-neutral` — единственный fixture class для VS-02…VS-15, screenshots/video/audio и human review. Он имеет recorded CC0-1.0 provenance и не зависит от imported structure/bytes/hashes. Смешанный bundle rejected как `EVIDENCE_FIXTURE_CLASS_MIXED`; import smoke и neutral suite имеют отдельные RunManifest roots.

## ScenarioAction, probes и assertions

`ScenarioAction` поддерживает:

- timestamped engine-owned semantic control event, проходящий production action mapping into canonical `PlayerActionFrame` and persisted current/next ingress assignment;
- canonical command proposal с normal issuer/capability/preconditions;
- lifecycle `start`, `save`, `load`, `restart`, `capture-marker`, `stop`;
- named fault activation/deactivation для declared backend/process boundary.

`ProbeSpec` выбирает только documented read-only state projection, including revisioned RPG aggregate and World Services calendar/population/tier views, DomainEvent/diagnostic stream, normalized physics/motor/audio/render metric либо resource counter. Selector использует PersistentId/AssetId/stable fact/event ID; raw ECS ID/vendor handle запрещён.

Каждый `AssertionSpec` MUST объявить owner, oracle kind, sample/window, pass/fail threshold, evidence role и stable failure code. Oracle kinds:

- `Exact` — canonical value/order/hash;
- `Tolerance` — units, absolute/relative bounds и reference profile;
- `Distribution` — fixed seeds/sample count/statistic/confidence rule;
- `Invariant` — forbidden/required property over interval;
- `PresentationMetric` — image/audio metric, не gameplay authority;
- `Qualitative` — только HumanReviewRequired и никогда не заменяет automatic oracle.

Implicit epsilon, wall-clock sleeps, unordered log text и retry-to-green запрещены. Divergent repeat deterministic gate emits `NONDETERMINISTIC_RESULT`; retries используются только для diagnosis. Statistical gates следуют declared distribution, а не retry count.

## Change impact resolution

`ImpactResolver` является pure deterministic application service:

```text
(AgentChangeSet, base/candidate hashes, ownership map, package DAG,
 schema registry, exported output categories, VerificationPolicyManifest)
    → ChangeImpactManifest
```

Manifest MUST перечислять changed definitions/files/packages, dependency edges/reasons, required scenario suites/profiles/long gates, observable categories, capture jobs, human-review flag и resolver/version hashes. Agent/author MAY добавить suites/captures, но MUST NOT удалить resolved requirement.

Observable categories: `visual`, `ui`, `camera`, `animation`, `physics`, `motor`, `vfx`, `audio`. Dependency edge к любой из них устанавливает `HumanReviewRequired`. Pure RPG/mechanics/schema change не требует qualitative review только если complete graph не достигает observable output. Unknown owner/schema/file/output добавляет maximal affected suite и HumanReviewRequired.

## Agent feedback loop и diagnostics

Normative CLI:

| Команда | Contract |
|---|---|
| `next test impact --changeset <manifest>` | generate/validate ChangeImpactManifest и explanations |
| `next test run --impact <manifest> --profile <profile> --artifact-root <dir>` | execute exact resolved plan; no implicit omission |
| `next scenario validate|run <scenario>` | schema/reference validation и deterministic run |
| `next scenario minimize <replay-or-run>` | produce smallest reproducing action/fault prefix/subset without changing failure signature |
| `next test explain --run <manifest> --format json` | stable owner/code/first-divergence/assertion/causal report |
| `next capture render --job <manifest> --artifact-root <dir>` | displayless capture worker entrypoint |
| `next evidence build|verify|diff <input>` | content-addressed evidence operations |
| `next review bundle <evidence> --out <dir>` | self-contained offline human dossier |
| `next review record <bundle> --decision <Approve\|Reject\|NeedsChanges> --attest <key-id>` | human-only capability; produces canonical `HumanReviewDecisionV2` + `AttestationEnvelopeV2` through isolated signer; all tokens are signable, only `Approve` can become admission-eligible |

Every failed run MUST emit stable diagnostic code, owner/subsystem, first divergent tick/event/schema, expected/actual typed values, causal command/event IDs, remediation key and replay/minimization status. Text rendering is secondary to JSON. Minimization MUST preserve content/build/schema/model hashes and exact failure code.

## Displayless capture

`OffscreenPresentationTarget` является engine-owned displayless render target behind a private graphics adapter, not interactive PlatformHost window/surface/swapchain. Capture worker `ApplicationSessionManifestV1` MUST declare `DisplaylessOffscreen`; worker MUST NOT connect a display server, enumerate monitor/DPI/input, create hidden windows or depend on interactive frame pacing.

`CaptureJobManifest` MUST содержать base/candidate engine/content/replay hashes; CapturePlan hash; platform/GPU/backend/toolchain requirements; camera/view/overlays; semantic start/end markers; resolution/FPS/color/audio specification; artifact roles; output quota; encoder requirement и expected gameplay hash.

Worker является short-lived process. Он валидирует inputs before presentation-device mutation, replays exact scenario через common simulation/runtime/session contracts, renders declared `PresentationSnapshotV2` sequence, verifies gameplay hash parity and pinned-profile SDR/frame roots, и atomically publishes RunManifest/artifacts. Worker crash/device/cache loss сохраняет CPU replay/evidence, simulation/session/snapshot roots и не оставляет partial published bundle.

Worker MAY быть локальным процессом либо tagged CI worker. Scheduling, queue и network protocol не входят в engine contract: обе формы принимают один CaptureJobManifest и публикуют один portable result contract.

Generic HumanReviewRequired bundle MUST содержать:

- `before.mp4` и `after.mp4` с одинаковыми scenario/camera/timeline parameters;
- synchronized labelled `comparison.mp4`;
- bounded `preview.gif` для быстрого review;
- `failures.gif` или `failures.mp4` для каждого unique failure signature и declared worst-N episode;
- canonical normalized frame hashes, lossless PNG fallback set, PCM WAV/hash when audio exists;
- metrics/overlay metadata и exact encoder/decoder manifests.

Media generation follows simulation; capture/encoding timing MUST NOT enter state hash. MP4/GIF bytes имеют собственные SHA-256; canonical frame/audio Merkle/root hash позволяет отделить encoder drift от rendered-content drift.

Physical existing names `baseline.gif`, `selected-policy.gif`, `failures.gif`, `comparison.gif` сохраняются как specialization roles. Audio change MUST иметь automatic waveform/loudness/event assertions, canonical WAV/hash и audible track в review MP4.

## Evidence bundles и baselines

`EvidenceBundleManifest` MUST ссылаться на exact ChangeImpactManifest, AgentChangeSet/base/candidate hashes, every required RunManifest/gate result, diagnostics, replay/minimized replay, reports, raw frame/audio roots, media files, baseline, review policy и redaction scan. Artifact entries содержат relative path, media/schema type, bytes, SHA-256, producer и required/optional role.

Artifacts хранятся под explicit `--artifact-root` content-addressed layout и не коммитятся в source repository. Missing referenced required artifact, hash mismatch, quota overflow или inaccessible approved artifact makes bundle invalid. Atomic publish exposes manifest last.

`EvidenceBaselineManifest` immutable и связывает scenario/profile/camera/toolchain/content/renderer/audio parameters с approved semantic/media roots. Agent MAY создать baseline candidate, но command не имеет auto-promote mode. Missing/stale/incompatible baseline blocks qualitative approval. Base и candidate MUST использовать один scenario revision/profile; intended baseline change имеет отдельный rationale и `HumanReviewDecisionV2`.

## Human review

`next review bundle` создаёт self-contained offline dossier с root `index.html`: no external network/resources, exact changeset summary, impact reasons, automatic gate status, metrics, synchronized media, failure/minimized replay links и baseline diff. Untrusted diagnostic/content strings escaped; scripts/assets embedded с declared hashes.

`HumanReviewDecisionV2` — closed JCS-canonical payload:

```text
schema = "nextengine.human-review-decision.v2"
project_id
decision_id
subject = {
  changeset_sha256
  revision_kind = "GitCommit" | "Diff"
  revision_sha256
}
evidence_bundle_sha256
baseline = { kind = "None" } | { kind = "EvidenceBaseline", sha256 }
verification_policy_sha256
impact_manifest_sha256
fixture = { class = "vertical-v1-neutral", sha256 }
automatic_gate_summary_sha256
requirement_graph_sha256
gate_descriptor_set_sha256
review_category
decision = "Approve" | "Reject" | "NeedsChanges"
reviewer_role
reason_code
issued_at_unix_seconds
```

Unknown/duplicate/missing/`null` fields rejected. Hashes are exact 64-character lowercase SHA-256 hex; `decision_id` is project-unique 128-bit lowercase hex; `project_id`, `review_category` и `reviewer_role` are non-empty Unicode NFC without control characters; `reason_code` is a non-empty ASCII identifier; enum tokens exact and case-sensitive. `requirement_graph_sha256` and `gate_descriptor_set_sha256` bind the exact validated `RequirementGraphV1` and canonical applicable `GateDescriptorV1` set, so trace/gate drift invalidates review. Reviewed artifact list, display comment и reviewer display name являются projections и не меняют canonical payload. Reusing `decision_id` with another payload rejected; any input change requires a fresh V2 payload and human signature.

`AttestationEnvelopeV2` is a closed JCS object:

```text
schema = "nextengine.attestation-envelope.v2"
algorithm = "ed25519"
domain = "nextengine.human-review-decision.v2"
project_id
key_id
payload_type = "HumanReviewDecisionV2"
payload_hash
trust_policy_hash
trust_manifest_hash
revocation_snapshot_hash
signature
```

`domain` MUST equal exact ASCII literal `nextengine.human-review-decision.v2`; signature is unpadded base64url. ADR-023 domain-separated preimage signs length + bytes for schema, algorithm, domain, `project_id`, `key_id` and payload type, followed by all four decoded 32-byte hashes. Envelope and payload project identity MUST match. Offline verifier получает `ProjectTrustAnchorV1` из project/release policy, а не из bundle, recomputes exact canonical payload hash и проверяет exact domain, role/scope/category, decision time, key validity/rotation, snapshot freshness и revocation/compromise. Конкретная crypto library не является public contract или Accepted technology row.

Decision rules:

1. V2 payload/envelope canonicalization, signature, project/hash closure and offline trust MUST verify before admission evaluation;
2. every persisted V2 review decision MUST be signed, and cryptographic verification may successfully return any exact token: `Approve`, `Reject` or `NeedsChanges`;
3. every automatic required gate, bundle/hash/redaction verification and required baseline MUST be `PASS`/valid before admission;
4. reviewer role MUST быть разрешён одновременно VerificationPolicyManifest и valid offline ReviewerTrustManifest;
5. reviewer attestation credential MUST быть inaccessible to agent workspace; signer получает canonical unsigned payload и возвращает only envelope/public chain;
6. only cryptographically verified `Approve` together with every required automatic gate `PASS` admits exact changeset;
7. cryptographically verified `Reject` and `NeedsChanges` are stable signed non-admitting feedback;
8. any input/artifact/policy/impact/automatic-summary/baseline/trust hash change invalidates decision;
9. human cannot waive automatic failure, `AwaitingCapability` or missing required evidence;
10. V1 payload/envelope may be verified only in explicit read-only `historical-audit` mode, which returns `HistoricalVerified` and never `PASS`, admission, merge, promotion or baseline acceptance.

## Failure semantics

- Unknown impact/owner/output → maximal affected suites + HumanReviewRequired, not guessed omission.
- Deterministic repeat mismatch → `NONDETERMINISTIC_RESULT`, gate fails; retry cannot turn it green.
- Scenario/probe/assertion schema mismatch → reject before world mutation.
- Failed minimization → original replay remains evidence; gate retains failure and reports minimizer diagnostic.
- Missing CPU capability → profile fails; no substitution by unrecorded host behavior.
- Missing GPU/encoder/worker → `AwaitingCapability`; CPU results preserved, required observable changeset cannot merge.
- Capture replay/gameplay hash mismatch → discard capture bundle and fail CAPTURE-01.
- GPU/encoder crash or quota overflow → no atomic publish; preserve prior valid generation.
- Missing/stale baseline → generate candidate only; review remains blocked.
- Missing/tampered/inaccessible evidence → reject bundle/decision.
- Agent/unsigned/unauthorized/stale/revoked/wrong-scope review → reject decision; changeset unchanged.
- Invalid V2 decision token/case → reject before signer as `REVIEW_DECISION_VALUE_INVALID`.
- V1 review on admission path → reject as `REVIEW_SCHEMA_HISTORICAL_ONLY`; no relabel, automatic upgrade or automatic re-sign.
- Envelope/payload schema, domain, payload type, `project_id`, signed `key_id` or hash mismatch → reject before admission and project mutation.
- Cryptographically valid V2 `Reject`/`NeedsChanges` → preserve signed feedback, return `REVIEW_DECISION_NON_ADMITTING`, leave changeset unchanged.
- Automatic gate failure or `AwaitingCapability` with signed `Approve` → reject admission as `AUTO_GATE_NOT_PASS`.
- Redaction/protected-data/HTML/media validation failure → quarantine bundle before human viewing.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback/rollback |
|---|---|---|---|---|
| TEST-01 | 100 reference scenarios ×10 repeats, execution-order permutations + curated invalid mutations | exact command/event/final-state hashes for deterministic cases; 100% curated mutations detected; 0 retry-to-green | scenario/seed/oracle manifests, state/replay diffs | fix scenario/oracle/runtime determinism; release block |
| HEADLESS-01 | clean CPU Linux container without GPU/display sockets/env | agent-fast reference workflow PASS ≤5 min; 0 renderer link/load, 0 display/window connection attempt | link/file/socket trace, RunManifest, timing report | fix composition boundary; no interactive fallback |
| IMPACT-01 | mutation corpus across every owner/schema/package/output category | 100% required suites/categories/review flags included; 0 silent omission; unknown 100% selects maximal+review | resolver graph/report, golden manifests | maximal suite + review; fix resolver |
| DIAG-01 | 100 injected command/state/process/backend failures | stable owner/code and first divergence for 100%; minimized replay reproduces same code 100%; original preserved | diagnostics, original/minimized replays, causal report | original replay/manual code diagnosis; gate fails until contract fixed |
| AGENT-03 | cold agent fixes seeded mechanic and visual regression | uses only AuthoringContextBundle/CLI/JSON; agent-fast target ≤5 min, ordinary changeset compute target ≤30 min; no private source/runtime UI/test omission | changesets, command transcript, impact/diagnostic/evidence manifests | fix context/diagnostics; reviewed manual CLI workflow |
| CAPTURE-01 | exact replay null vs displayless Vulkan worker, two repeated captures | gameplay hashes exact; normalized raw frame/audio root exact on pinned worker; 0 X11/Wayland/window/swapchain calls; atomic output after injected device crash | replay/state hashes, frame/audio roots, API/socket trace, crash matrix | retain CPU evidence; AwaitingCapability/alternate passing worker |
| MEDIA-P1 | pinned MediaEncoder canonical corpus Win/Linux | ADR-023 threshold; valid required GIF/MP4 and exact decoded stream structure; 0 network | encoder/SBOM/license/decoded-stream reports, hashes | PNG/WAV + engine GIF; MP4-required review awaits adapter |
| EVIDENCE-01 | complete + missing/tampered/oversized/redaction corpus | 100% valid bundles verify; 100% invalid cases rejected/quarantined; atomic manifest publication | bundle validator/fault/privacy report | retain previous valid bundle; regenerate |
| REVIEW-01 | observable/non-observable changes; exact V2 `Approve`/`Reject`/`NeedsChanges`; stale/tampered/agent/V1 decisions | 100% observable changes blocked without verified V2 `Approve` plus automatic `PASS`; `Reject`/`NeedsChanges` verify but never admit; automatic FAIL/AwaitingCapability never admitted; any bound hash change invalidates; V1 admission rejected | policy/decision/admission/audit/tamper report | keep changeset blocked; new V2 evidence/review |
| REVIEW-02 | V2 canonical payload/envelope/offline trust + V1 historical corpus | all three authorized V2 decisions cryptographically verify; only `Approve` is admission-eligible; exact domain is `nextengine.human-review-decision.v2`; 100% invalid token and wrong schema/domain/domain length/payload type/project/key/hash/role/scope/time/rotation/revocation/canonicalization/agent/test-key cases rejected; V1 can return only `HistoricalVerified` in explicit audit mode | V2 payload/envelope/trust manifests, V1 historical fixtures, domain/preimage/tamper/rotation/revocation report | keep changeset blocked; issue fresh authorized V2 decision |
| PRIVACY-02 | import-smoke cleanup + neutral evidence/mixed fixture corpus | 0 protected bytes in prohibited roots; 100% mixed fixture/failed cleanup/quarantine cases rejected before bundle publication | fixture provenance, source-access/cleanup/scanner audit | quarantine run; retain no publishable bundle |

Verification & Evidence Team владеет TEST/HEADLESS/IMPACT/DIAG/CAPTURE/EVIDENCE gates; Developer Experience co-owns AGENT-03, Rendering/Audio co-own CAPTURE/MEDIA, Security & Governance co-owns MEDIA/EVIDENCE/REVIEW. Release Engineering принимает root artifacts.
