# SPEC-15: Headless testing, agent validation и human evidence

| Поле | Значение |
|---|---|
| ID | SPEC-15 |
| Статус | Proposed |
| Версия | 1.1 |
| Владелец | Verification & Evidence Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-10](10-gothic-importer-boundary.md), [SPEC-11](11-security-licensing-and-governance.md), [ADR-014](adr/014-artifact-first-review-baselines-and-attestation-v2.md) |
| Связанные документы | [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-010](adr/010-artifact-first-headless-validation-and-review.md) |
| Заменяет | SPEC-15 v1.0 после human approval exact candidate hash |

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
| Approved baseline | `BaselineIndexManifest` → `EvidenceBaselineManifest` + `BaselinePromotionDecisionV1` | generated candidate files |
| Qualitative decision | Immutable `HumanReviewDecision` + `AttestationEnvelopeV1` от authorized reviewer | agent text, unsigned comment |

Verification & Evidence Team владеет schemas, runner orchestration semantics, impact resolver, artifact roles и gates. Subsystem owner владеет assertions/thresholds своего behavior. Release Engineering принимает root evidence; Security & Governance владеет reviewer trust/redaction policy.

## Public boundary и data flow

Public contracts: `VerificationPolicyManifest`, `TestScenarioManifest`, `ScenarioAction`, `ProbeSpec`, `AssertionSpec`, `ChangeImpactManifest`, `OffscreenPresentationTarget`, `CapturePlan`, `CaptureJobManifest`, `EvidenceBaselineManifest`, `BaselineIndexManifest`, `BaselinePromotionDecisionV1`, `AttestationEnvelopeV1`, `ReviewerTrustManifest`, `EvidenceBundleManifest` и `HumanReviewDecision`.

```text
AgentChangeSet
  → ImpactResolver → ChangeImpactManifest
  → CPU TestScenario run → RunManifest + replay + probes/diagnostics
  → optional CaptureJobManifest → displayless GPU worker
  → raw frame/audio roots → MediaEncoder derivatives
  → EvidenceBundleManifest → automatic admission gate
  → offline review dossier → HumanReviewDecision
  → exact changeset admission or rejection
```

CLI/JSON является normative surface. CI/MCP/GUI/static HTML являются projections. Public contracts не содержат CI vendor, window-system, Vulkan handle, encoder library или artifact-store vendor types.

## VerificationPolicyManifest и profiles

Project-owned manifest MUST фиксировать schema/revision/hash, reference CPU/GPU/platform profiles, suite/tag dependency rules, observable-category mapping, time/resource budgets, long-gate classes, reviewer roles, baseline policy, artifact quotas/retention requirements и redaction policy.

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
- exact content/MechanicsLock/model/build/toolchain inputs;
- initial cooked fixture либо SaveManifest hash;
- named RNG streams/seeds и fixed gameplay/physics/motor rates;
- ordered `ScenarioAction` timeline;
- declared adapter fault injections;
- `ProbeSpec`, `AssertionSpec` и optional CapturePlan references;
- required profiles, time/resource limits и expected diagnostic policy;
- positive, negative/failure и save/replay roles;
- provenance/license/redaction classification.

`MechanicTestScenario` из SPEC-13, physical suites SPEC-05/14 и vertical scenario SPEC-12 являются specializations одного runner/schema registry.

Scenario initialization выполняется cooker/fixture loader до first tick. После старта разрешены только timestamped virtual production input, validated WorldCommand/MechanicCommand proposals, process/device/adaptor fault controls и lifecycle commands. Fault adapter заменяется только в composition root и не открывает arbitrary state mutation.

## ScenarioAction, probes и assertions

`ScenarioAction` поддерживает:

- timestamped `VirtualInputEvent`, проходящий production input mapping;
- canonical command proposal с normal issuer/capability/preconditions;
- lifecycle `start`, `save`, `load`, `restart`, `capture-marker`, `stop`;
- named fault activation/deactivation для declared backend/process boundary.

`ProbeSpec` выбирает только documented read-only state projection, DomainEvent/diagnostic stream, normalized physics/motor/audio/render metric либо resource counter. Selector использует PersistentId/AssetId/stable fact/event ID; raw ECS ID/vendor handle запрещён.

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

Observable categories: `visual`, `ui`, `camera`, `animation`, `physics`, `motor`, `audio`. Dependency edge к любой из них устанавливает `HumanReviewRequired`. Pure RPG/mechanics/schema change не требует qualitative review только если complete graph не достигает observable output. Unknown owner/schema/file/output добавляет maximal affected suite и HumanReviewRequired.

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
| `next review record <bundle> --decision <value> --attest <key-id>` | human-only capability; produces HumanReviewDecision |
| `next baseline candidate --bundle <evidence> --key <baseline-key>` | immutable candidate; approved index не меняется |
| `next baseline record --mode bootstrap\|replace --candidate <manifest> --attest <key-id>` | human-only decision с independent `baseline.promote` capability |
| `next baseline promote --decision <manifest>` | verify trust/signature/current-index precondition и atomic CAS publish |

Every failed run MUST emit stable diagnostic code, owner/subsystem, first divergent tick/event/schema, expected/actual typed values, causal command/event IDs, remediation key and replay/minimization status. Text rendering is secondary to JSON. Minimization MUST preserve content/build/schema/model hashes and exact failure code.

## Displayless capture

`OffscreenPresentationTarget` является engine-owned render target backed by Vulkan images/readback, not PlatformHost window/surface/swapchain. Capture worker MUST NOT connect X11/Wayland, enumerate monitor/DPI/input, create hidden windows or depend on interactive frame pacing.

`CaptureJobManifest` MUST содержать base/candidate engine/content/replay hashes; CapturePlan hash; platform/GPU/backend/toolchain requirements; camera/view/overlays; semantic start/end markers; resolution/FPS/color/audio specification; artifact roles; output quota; encoder requirement и expected gameplay hash.

Worker является short-lived process. Он валидирует inputs before GPU mutation, replays exact scenario через common simulation/runtime contracts, renders declared PresentationSnapshots, verifies gameplay hash parity и atomically publishes RunManifest/artifacts. Worker crash/device loss сохраняет CPU replay/evidence и не оставляет partial published bundle.

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

`EvidenceBaselineManifest` immutable и связывает scenario/profile/camera/toolchain/content/renderer/audio parameters с approved semantic/media roots. Agent MAY создать baseline candidate, но command не имеет auto-promote mode. Missing/stale/incompatible baseline blocks qualitative approval. Base и candidate MUST использовать один scenario revision/profile.

`BaselineIndexManifest` является immutable project index baseline key → approved manifest hash + promotion decision hash. `BaselinePromotionDecisionV1` использует `Bootstrap` только при отсутствии key и `Replace` только с exact current previous hash. Publication валидирует independent `baseline.promote` capability и выполняет atomic compare-and-swap; mismatch оставляет current index неизменным. Intended baseline change не может переиспользовать changeset review decision.

## Human review

`next review bundle` создаёт self-contained offline dossier с root `index.html`: no external network/resources, exact changeset summary, impact reasons, automatic gate status, metrics, synchronized media, failure/minimized replay links и baseline diff. Untrusted diagnostic/content strings escaped; scripts/assets embedded с declared hashes.

`HumanReviewDecision` содержит schema/version, `Approved|Rejected|NeedsChanges`, reviewer identity/role, base/candidate/AgentChangeSet/EvidenceBundle/Baseline hashes, reviewed artifact list, reason codes/comment, UTC metadata и `AttestationEnvelopeV1`.

Attestation input — domain-separated RFC 8785 JCS object без поля `attestation`; Ed25519 RFC 8032 обязателен в v1. `ReviewerTrustManifest` фиксирует key/capabilities/validity/revocation. Unknown algorithm/key/role и revoked key fail closed; current revocation требует повторного review всех решений key.

Decision rules:

1. every automatic required gate MUST be PASS;
2. bundle/hash/redaction verification MUST pass;
3. reviewer role MUST be granted by VerificationPolicyManifest;
4. reviewer attestation credential MUST be inaccessible to agent workspace;
5. only `Approved` admits exact changeset;
6. any input/artifact/policy/baseline hash change invalidates decision;
7. human cannot waive automatic failure or missing required evidence;
8. `Rejected`/`NeedsChanges` becomes stable machine-readable feedback to agent.

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
- Bootstrap existing key, Replace wrong previous hash, wrong role или CAS race → reject decision/publish; current baseline index unchanged.
- Missing/tampered/inaccessible evidence → reject bundle/decision.
- Agent/unsigned/unauthorized/stale review → reject decision; changeset unchanged.
- Automatic gate failure with human approval attempt → reject approval as `AUTO_GATE_NOT_PASS`.
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
| MEDIA-P1 | pinned MediaEncoder canonical corpus Win/Linux | ADR-010 threshold; valid required GIF/MP4 and exact decoded stream structure; 0 network | encoder/SBOM/license/decoded-stream reports, hashes | PNG/WAV + engine GIF; MP4-required review awaits adapter |
| EVIDENCE-01 | complete + missing/tampered/oversized/redaction corpus | 100% valid bundles verify; 100% invalid cases rejected/quarantined; atomic manifest publication | bundle validator/fault/privacy report | retain previous valid bundle; regenerate |
| REVIEW-01 | observable/non-observable changes, approve/reject/needs-changes, stale/tampered/agent decisions | 100% observable changes blocked without valid human approval; automatic FAIL never approved; valid decision admits exact hash only; any hash change invalidates | policy/decision/audit/tamper report | keep changeset blocked; new evidence/review |
| BASELINE-01 | first baseline, replacement, stale previous hash, concurrent promotion, wrong capability and revoked key | only absent-key Bootstrap and exact-current Replace publish; invalid cases preserve exact prior index | baseline candidate/index/decision manifests, CAS fault audit | retain prior index; AwaitingCapability without promoter |
| ATTEST-01 | RFC 8785 JCS + RFC 8032 Ed25519 vectors, tamper/order/number/domain/key/role/revocation cases | canonical bytes and signatures exact across targets; invalid cases rejected 100% | golden vectors, trust/revocation audit | reject decision; obtain new authorized review |

Verification & Evidence Team владеет TEST/HEADLESS/IMPACT/DIAG/CAPTURE/EVIDENCE gates; Developer Experience co-owns AGENT-03, Rendering/Audio co-own CAPTURE/MEDIA, Security & Governance co-owns MEDIA/EVIDENCE/REVIEW. Release Engineering принимает root artifacts.
