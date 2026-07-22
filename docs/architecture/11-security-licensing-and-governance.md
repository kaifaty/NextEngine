# SPEC-11: Security, licensing и governance

| Поле | Значение |
|---|---|
| ID | SPEC-11 |
| Статус | Accepted |
| Версия | 1.4 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-23 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-015](adr/015-evidence-trust-fixture-separation-and-attestation.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

Security & Governance Team владеет threat model, secure defaults, vulnerability process, dependency/license policy, contribution policy, release attestations и disclosure rules. Subsystem teams владеют исправлениями своих boundaries. SPDX/SBOM + reviewed exception records являются source of truth для shipped dependency licensing; informal README/website claims не заменяют package-level evidence.

## Public boundary и security data flow

Public governance boundary включает contribution/DCO rules, vulnerability disclosure channel, capability/security policies, SBOM/notices, signed release attestations и time-bounded exception records. Security scanners/registries/CI vendor formats остаются internal adapters. Поток `locked inputs → provenance/license/vulnerability/security validation → signed review decision → atomic release`; unknown или failed required evidence не может быть преобразован в warning.

## Security principles

- Все external files, NIM/bundles/saves/models, Luau/Wasm packages и `ai-host` responses считаются untrusted.
- Default capability — deny; least privilege выдаётся конкретному package/process/project.
- Parser/VM/model runtime isolation и resource limits обязательны до обработки content.
- Authoritative mutation проходит common WorldCommand validation и atomic transaction.
- Network и telemetry default-off; offline play complete.
- Secrets, user paths, prompts/voice и imported bytes default-redacted.
- Cryptographic integrity использует SHA-256 для artifact identity; подпись определяет publisher identity/trust policy, но не заменяет validation.
- Coding agents и optional MCP adapters не получают ambient shell/filesystem/network/runtime authority; mutations проходят scoped AgentChangeSet preconditions/review.
- Model weights immutable/content-addressed; runtime learning, self-promotion и self-certification запрещены.
- Reviewer credentials/attestation capability MUST быть недоступны agent workspace, MCP adapters, test runner и capture worker.
- Evidence/media/static dossier считаются untrusted до hash/schema/quota/redaction validation; human viewing происходит только после quarantine gate.

## Threat model

| Boundary/угроза | Required controls | Failure outcome | Проверка |
|---|---|---|---|
| Luau escape/DoS | isolated env, allowlist libs, exact instruction/allocation/host-call quotas, non-authoritative wall watchdog, no native/fs/network | abort callback, discard candidates, watchdog marks run NonConforming, circuit-break | SCRIPT-P1/P2/P5 |
| Wasm plugin escape/DoS/confused deputy | WIT only, capability intersection, fuel/memory/table limits, no ambient authority | terminate instance, audit, optional disable | PLUGIN-P1/P2/P4 |
| Mod dependency/patch/hook ambiguity, forged signature или malicious package | exact lock/hash, DAG, preconditioned patches, `PackageTrustManifestV1`, signer scope + consent + capability ceiling, sandbox/provenance | pre-world reject/quarantine; invalid signature never downgrades | MECH-03, MOD-01/02, MOD-P2 |
| Agent prompt injection/path escape/stale edit | bounded context, project-root allowlist, base hashes, changeset dry-run/review/atomic apply | reject changeset, no mutation | AGENT-01/02 |
| Local MCP tool abuse/confused deputy | stable pinned protocol, stdio/local default, per-tool capability, no generic shell, audit/user control | deny tool/disable adapter | MCP-P1, SEC-03 |
| `ai-host` malformed/prompt injection/crash | separate process, schema/size/deadline, no mutable tools, idempotency, optional network grant | reject/fallback/restart | AI-02/AI-03 |
| Imported malformed/archive bomb/protected data | sandbox process, bounds/ratio quotas, provenance, protected-data scan | abort/quarantine/no publish | IMPORT-P3/P5 |
| Cooked bundle/save corruption | version/checksum/dependency validation, atomic generations, fail-closed | no activation/load, retain prior | ASSET-02/SAVE-01/SAVE-02 |
| Model file malicious/incompatible/NaN | size/opset/schema/hash allowlist, safe runtime config, golden inference, finite checks | quarantine/recovery controller | MOTOR-P1/SEC-04 |
| Forged/incomplete physical certification | signed/hash-bound report, exact suite/threshold/artifact closure, owner review | reject claim; retain PrototypeFallback/previous revision | EMB-01/SEC-06 |
| Poisoned/unlicensed training input или policy artifact | dataset/model provenance, license classification, config/environment hashes, holdout + correspondence review | quarantine/no promotion | TRAIN-P1/AUTHOR-P1/SEC-06 |
| Runtime learning/model mutation request | immutable content registry, read-only inference mapping, denied write/network capabilities | reject request; audit/circuit-break caller | SEC-06 |
| Shader/GPU input/device loss | offline compilation, reflection/layout validation, bounds-safe descriptors, validation layers | reject asset/lower tier/clean recovery | SHADER-P1/RENDER-03 |
| Dependency/supply chain | lockfile/checksums, source registry policy, SBOM, vulnerability/license scan, reproducible builds | block merge/release | LIC-01/SEC-01 |
| Live inspector abuse | development-only local authenticated read-only default, audited mutate capability | deny/disconnect/non-conforming run | SEC-03 |
| Impact/test omission или forged PASS | generated immutable impact plan, required-artifact closure, automatic gate precedence | maximal suite/reject admission | IMPACT-01/EVIDENCE-01/REVIEW-01 |
| Malicious media/encoder/decoder/artifact bomb | sandbox process, byte/frame/duration/decode quotas, exact tool manifest, no network | abort/quarantine/no review | MEDIA-P1/EVIDENCE-01/SEC-07 |
| Static dossier HTML/script injection | escaped untrusted strings, self-contained hashed resources, CSP/no external resources | reject bundle before viewing | EVIDENCE-01/SEC-07 |
| Reviewer-key exposure, stale trust state или agent self-approval | isolated signer, offline trust anchor/manifest/revocation validation, reviewer role/scope check, hash-bound attestation | reject/revoke/re-review exact changeset | REVIEW-01/02, SEC-07 |
| Protected data in replay/frame/audio/evidence | split import-smoke/neutral fixtures, cleanup, source classification, default redaction, protected-data scanner before publish | quarantine/incident/no human bundle | PRIVACY-01/02, EVIDENCE-01 |

Threat model MUST be reviewed for every new public parser, capability, IPC method, WIT import, network endpoint, model operator set или native dependency.

## Licensing policy

Engine source license is Apache-2.0. Default dependency allowlist: `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `MIT`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `BSL-1.0`, `CC0-1.0`, `Unicode-3.0` and verified Public Domain dedications. Data/model/font licenses имеют отдельную classification и не наследуют code allowlist.

Distributed physical-policy weights являются content assets и MUST иметь publisher, source/training provenance, license/redistribution classification, immutable hash и model-card/evaluation linkage. Raw datasets и intermediate checkpoints не коммитятся в engine repository. Isaac Lab может использоваться только в status `Proposed`; BSD-3-Clause framework record не освобождает release owner от отдельной проверки Isaac Sim, bundled assets и transitive terms для exact training environment.

FFmpeg CLI является external `Proposed` development tool, не engine/runtime library. Exact binary, configure flags, enabled codecs/external libraries, license expression, source URL/checksum и SBOM MUST быть записаны в encoder manifest. GPL/nonfree-enabled build нельзя распространять или использовать в official pipeline без separate license decision. Media artifact itself не считается доказательством compliant toolchain.

Copyleft, source-available, field-of-use, non-commercial, custom redistribution и unknown licenses default-rejected. Exact dependency MAY получить time-bounded written exception только после legal review, documented distribution obligations, closed-game compatibility assessment, owner и fallback. Steam Audio относится к custom license и не принимается без AUDIO-L1.

Каждый shipped package MUST иметь generated SBOM (SPDX or CycloneDX), license notices, exact versions/checksums/source URLs, vulnerability scan timestamp и mapping binary→dependencies. Dev-only dependency не попадает в runtime SBOM, но остаётся в source SBOM.

## Contribution governance

- Contributions use DCO sign-off; inbound license equals outbound Apache-2.0, если path не имеет явной отдельной compatible license.
- CLA не требуется для baseline; его добавление требует ADR и community review.
- Repository MUST иметь `CONTRIBUTING`, Code of Conduct, security disclosure channel, maintainer/reviewer roles и release signing policy до public bootstrap release.
- Accepted architecture меняется superseding ADR, не silent edit. Один review packet принимает весь initial specification set.
- Code owner профильного subsystem и Architecture owner должны одобрить public contract change; Security owner также обязателен для trust boundary/capability/parser/license changes.
- Generated/AI-assisted contribution проходит те же provenance, license, tests и human review; tool output не считается доказательством авторских прав на входные assets.
- `HumanReviewDecisionV1` не заменяет code-owner/security/legal approvals и не может быть создан trusted identity агента. Any changeset/evidence/baseline/policy/trust hash change требует нового canonical decision и `AttestationEnvelopeV1`.

## Offline reviewer trust

Project release policy pins immutable `ProjectTrustAnchorV1` fingerprints. Signed `ReviewerTrustManifestV1` назначает reviewer keys, roles/scopes/categories и validity; signed monotonic `ReviewerRevocationSnapshotV1` фиксирует freshness, revocation и compromise instants. Admission MUST offline verify exact project/payload/bundle/baseline/policy/trust/revocation hashes, decision time, role/scope, key validity/rotation и non-revocation. Bundle-supplied trust root, stale snapshot, unknown/agent/test key или invalid rotation fail-closed.

`AttestationEnvelopeV1` подписывает domain-separated ADR-015 preimage над canonical `HumanReviewDecisionV1` payload hash и trust hashes. Private key/path/seed MUST быть недоступен workspace, environment dump, runner, agent, MCP, capture worker и artifacts; signer adapter возвращает только envelope/public chain.

## Vulnerability и disclosure

Private disclosure channel MUST acknowledge report ≤3 business days, initial triage ≤7 days, severity/embargo/credit фиксируются с reporter. Critical exploitable issue в released parser/VM/plugin/model boundary MUST block new release, получить advisory и supported-version fix/mitigation. Secrets/keys никогда не коммитятся; release signing credentials живут вне CI logs/artifacts и имеют rotation/revocation runbook.

## Importer legal boundary

Перед публичным release Gothic importer отдельный legal review MUST письменно оценить code provenance, trademarks/naming, local-install workflow, generated/derived outputs, bundled fixtures, notices и distribution territories. Отсутствие review означает `not approved for distribution`, а не подразумеваемое разрешение. Engine Apache-2.0 release MAY продолжаться без importer.

## Data/privacy disclosure

Project/game MUST раскрывать optional network AI, telemetry, voice recording/storage и third-party services до их включения. Consent granular и revocable; offline mode не посылает network traffic. Default local logs use retention/size caps и redaction classes from SPEC-09.

## Failure semantics

- License/vulnerability/provenance unknown для shipped required artifact → release fail-closed.
- Invalid/unknown/expired/revoked package signature → deny/quarantine; automatic downgrade в unsigned запрещён. Explicit unsigned local install является новой consent-bound transaction и не получает trusted capabilities.
- Security scanner unavailable → release gate не считается пройденным.
- Discovered protected data → artifact quarantine, distribution stop, cache/registry incident assessment; удаление фиксируется audit record.
- Security exception expiry → dependency снова rejected до renewal/replacement.
- Invalid package lock/patch/signature/provenance или AgentChangeSet precondition → no install/apply; current project revision remains.
- Invalid model compatibility, missing evaluation/media evidence или forged certification → no `PhysicalCertified` promotion; prototype or previous certified package remains usable by explicit project policy.
- Runtime attempt to mutate weights or bypass owner review → request denied, policy quarantined on repeated violation, authoritative state unchanged.
- Evidence/media/dossier validation failure → bundle quarantined before viewing; prior valid evidence/decision remains immutable.
- Missing/invalid/stale/agent-issued reviewer attestation → no changeset admission; reviewer key incident follows revoke/re-review policy.
- Encoder license/config unknown → MEDIA-P1 not passed; retain canonical PNG/WAV/GIF fallback, MP4-required review AwaitingCapability.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback/rollback |
|---|---|---|---|---|
| SEC-01 | locked source/runtime dependency scan | 0 unreviewed critical/high exploitable advisories; checksums/sources 100% | signed scan report | update/remove dependency; block release |
| LIC-01 | license/SBOM policy scan every package | 100% artifacts classified; 0 non-allowlisted without valid exception; notices complete | SBOM, policy report, exceptions | remove/replace artifact |
| SEC-02 | parser/VM/IPC fuzz aggregate | required CPU-hour suites completed, 0 escape/UB/host crash | fuzz manifests | disable affected surface |
| SEC-03 | inspector/network/offline test | default offline produces 0 outbound connection attempts; unauthorized live access 100% denied | packet capture/audit log | disable adapter/endpoint |
| SEC-04 | malicious/incompatible model corpus | 100% oversized/hash/schema/opset/non-finite cases rejected before actuation | validator report | heuristic controller |
| GOV-01 | release governance packet | DCO on every commit, required approvals, ADR/traceability consistency 100% | signed review record | release block |
| LEGAL-IMPORT-01 | importer legal review | exact release has written `approved` decision | legal record | importer not distributed |
| SEC-05 | mechanic package + agent authoring aggregate | MOD-02, AGENT-02 и MCP-P1 security cases pass; 0 ambient authority/out-of-root mutation | package/changeset/MCP audit packet | disable package/adapter; reviewed manual workflow |
| SEC-06 | physical model/provenance/certification corpus | 100% hash/schema/license/provenance/certification mismatches rejected before activation; 0 runtime weight writes; owner review present for every promoted model | validator/fault report, provenance graph, review records | quarantine model; PrototypeFallback/previous certified revision |
| SEC-07 | evidence/dossier/reviewer trust corpus | 100% malicious HTML/media/quota/redaction/tamper cases rejected before viewing; 100% agent/unauthorized/stale/revoked attestations rejected; 0 reviewer secret exposed to agent/capture process | sandbox/privacy/tamper/credential audit packet | quarantine bundle, revoke key, regenerate/re-review |
| REVIEW-02 | offline attestation/trust/rotation/revocation corpus | exact valid decision accepted; 100% wrong hash/project/role/time/key/rotation/revocation/canonicalization cases rejected without network/private key | payload/envelope/trust manifests, tamper/rotation/revocation report | keep changeset blocked; issue fresh authorized decision |
| PRIVACY-02 | split-fixture and cleanup corpus | neutral bundle has 0 imported dependency; all protected markers in prohibited roots and mixed/failed cleanup cases rejected before publication | fixture provenance, cleanup/source-access/scanner audit | quarantine run; no human bundle/publication |
