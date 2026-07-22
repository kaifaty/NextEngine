# SPEC-11: Security, licensing и governance

| Поле | Значение |
|---|---|
| ID | SPEC-11 |
| Статус | Proposed |
| Версия | 1.4 |
| Владелец | Security & Governance Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-014](adr/014-artifact-first-review-baselines-and-attestation-v2.md), [ADR-015](adr/015-luau-wasm-package-trust-v2.md), [ADR-016](adr/016-rust-first-audited-ffi-boundary-v2.md), [ADR-017](adr/017-artifact-first-ai-content-generation.md) |
| Связанные документы | [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-16](16-ai-assisted-world-and-asset-generation.md), [ADR-006](adr/006-scripting-and-plugin-model.md), [ADR-010](adr/010-artifact-first-headless-validation-and-review.md) |
| Заменяет | SPEC-11 v1.3 после human approval exact candidate hash |

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
- Review/baseline attestations v1 используют RFC 8785 JCS и RFC 8032 Ed25519 с domain separation, project `ReviewerTrustManifest`, independent capability checks и fail-closed revocation.
- Publisher trust хранится отдельно от reviewer trust; package signature не отменяет sandbox/capability/command validation.
- Coding agents и optional MCP adapters не получают ambient shell/filesystem/network/runtime authority; mutations проходят scoped AgentChangeSet preconditions/review.
- Model weights immutable/content-addressed; runtime learning, self-promotion и self-certification запрещены.
- Reviewer credentials/attestation capability MUST быть недоступны agent workspace, MCP adapters, test runner и capture worker.
- Evidence/media/static dossier считаются untrusted до hash/schema/quota/redaction validation; human viewing происходит только после quarantine gate.
- Generative prompts, reference images, provider responses, meshes/materials и external worker outputs считаются untrusted; remote upload и network adapter capability default-denied и выдаются отдельно от filesystem/project mutation.

## Threat model

| Boundary/угроза | Required controls | Failure outcome | Проверка |
|---|---|---|---|
| Luau escape/DoS | isolated env, allowlist libs, instruction/wall/allocation quotas, no native/fs/network | abort callback, discard candidates, circuit-break | SCRIPT-P1/P2 |
| Wasm plugin escape/DoS/confused deputy | WIT only, capability intersection, fuel/memory/table limits, no ambient authority | terminate instance, audit, optional disable | PLUGIN-P1/P2/P4 |
| Mod dependency/patch/hook ambiguity или malicious package | exact lock/hash, DAG, preconditioned patches, capability ceiling, sandbox/provenance | pre-world reject/quarantine | MECH-03, MOD-01/02 |
| Agent prompt injection/path escape/stale edit | bounded context, project-root allowlist, base hashes, changeset dry-run/review/atomic apply | reject changeset, no mutation | AGENT-01/02 |
| Generation prompt/reference exfiltration, provider drift или malicious/oversized output | explicit upload consent, protected/private-data deny policy, provider-neutral schema, sandbox/quotas, atomic quarantine, hashes/provenance/normalization | deny upload или quarantine result; no source/cook mutation | GEN-02, IMAGEGEN-P1, IMG3D-P1 |
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
| Reviewer-key exposure или agent self-approval | credential isolation, reviewer role check, hash-bound attestation, audit | reject/revoke/re-review exact changeset | REVIEW-01/SEC-07 |
| Baseline bootstrap/replace race или forged promotion | independent `baseline.promote`, exact previous hash, atomic compare-and-swap, attested decision | reject; approved index unchanged | BASELINE-01/ATTEST-01 |
| Unsafe или lint opt-out вне audited FFI crate | default workspace forbid, exact Cargo allowlist, SAFETY/API/boundary scans | build/admission failure | FFI-01 |
| Protected data in replay/frame/audio/evidence | source classification, default redaction, protected-data scanner before publish | quarantine/incident/no human bundle | PRIVACY-01/EVIDENCE-01 |

Threat model MUST be reviewed for every new public parser, capability, IPC method, WIT import, network endpoint, model operator set или native dependency.

## Licensing policy

Engine source license is Apache-2.0. Default dependency allowlist: `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `MIT`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `BSL-1.0`, `CC0-1.0`, `Unicode-3.0` and verified Public Domain dedications. Data/model/font licenses имеют отдельную classification и не наследуют code allowlist.

Distributed physical-policy weights являются content assets и MUST иметь publisher, source/training provenance, license/redistribution classification, immutable hash и model-card/evaluation linkage. Raw datasets и intermediate checkpoints не коммитятся в engine repository. Isaac Lab может использоваться только в status `Proposed`; BSD-3-Clause framework record не освобождает release owner от отдельной проверки Isaac Sim, bundled assets и transitive terms для exact training environment.

Generated content имеет отдельные records для generator code, transitive dependencies, checkpoint/model, input/reference data, service/account terms и output usage/redistribution terms. Open-source license generator code не доказывает license checkpoints, training data или output; generated output не доказывает ownership source references. Unknown element quarantines candidate and blocks changeset/cook publication.

FFmpeg CLI является external `Proposed` development tool, не engine/runtime library. Exact binary, configure flags, enabled codecs/external libraries, license expression, source URL/checksum и SBOM MUST быть записаны в encoder manifest. GPL/nonfree-enabled build нельзя распространять или использовать в official pipeline без separate license decision. Media artifact itself не считается доказательством compliant toolchain.

Copyleft, source-available, field-of-use, non-commercial, custom redistribution и unknown licenses default-rejected. Exact dependency MAY получить time-bounded written exception только после legal review, documented distribution obligations, closed-game compatibility assessment, owner и fallback. Steam Audio относится к custom license и не принимается без AUDIO-L1.

Каждый shipped package MUST иметь generated SBOM (SPDX or CycloneDX), license notices, exact versions/checksums/source URLs, vulnerability scan timestamp и mapping binary→dependencies. Dev-only dependency не попадает в runtime SBOM, но остаётся в source SBOM.

## Contribution governance

- Contributions use DCO sign-off; inbound license equals outbound Apache-2.0, если path не имеет явной отдельной compatible license.
- CLA не требуется для baseline; его добавление требует ADR и community review.
- Repository MUST иметь `CONTRIBUTING`, Code of Conduct, security disclosure channel, maintainer/reviewer roles и release signing policy до public bootstrap release.
- Bootstrap MAY использовать documented out-of-band conduct/security process, но public release остаётся `AwaitingCapability`, пока `public_contact_status` не содержит permanent monitored contact.
- `review.changeset`, `baseline.promote`, architecture approval, release signing и publisher trust являются независимыми capabilities. Один человек MAY иметь несколько capabilities, но каждая проверяется отдельно.
- Accepted architecture меняется superseding ADR, не silent edit. Один review packet принимает весь initial specification set.
- Code owner профильного subsystem и Architecture owner должны одобрить public contract change; Security owner также обязателен для trust boundary/capability/parser/license changes.
- Generated/AI-assisted contribution проходит те же provenance, license, tests и human review; tool output не считается доказательством авторских прав на входные assets.
- HumanReviewDecision не заменяет code-owner/security/legal approvals и не может быть создан trusted identity агента. Any evidence or changeset hash change требует нового review.

## Vulnerability и disclosure

Private disclosure channel MUST acknowledge report ≤3 business days, initial triage ≤7 days, severity/embargo/credit фиксируются с reporter. Critical exploitable issue в released parser/VM/plugin/model boundary MUST block new release, получить advisory и supported-version fix/mitigation. Secrets/keys никогда не коммитятся; release signing credentials живут вне CI logs/artifacts и имеют rotation/revocation runbook.

## Importer legal boundary

Перед публичным release Gothic importer отдельный legal review MUST письменно оценить code provenance, trademarks/naming, local-install workflow, generated/derived outputs, bundled fixtures, notices и distribution territories. Отсутствие review означает `not approved for distribution`, а не подразумеваемое разрешение. Engine Apache-2.0 release MAY продолжаться без importer.

## Data/privacy disclosure

Project/game MUST раскрывать optional network AI/generation, telemetry, voice recording/storage и third-party services до их включения. Consent granular и revocable; offline mode не посылает network traffic. Upload private images, protected/imported assets или unredacted prompts default-denied; разрешение требует exact source classification, endpoint class, purpose и retention disclosure. Default local logs use retention/size caps и redaction classes from SPEC-09.

## Failure semantics

- License/vulnerability/provenance unknown для shipped required artifact → release fail-closed.
- Signature unknown у optional plugin → policy prompt/deny; unsigned не получает trusted capabilities.
- Security scanner unavailable → release gate не считается пройденным.
- Discovered protected data → artifact quarantine, distribution stop, cache/registry incident assessment; удаление фиксируется audit record.
- Security exception expiry → dependency снова rejected до renewal/replacement.
- Invalid package lock/patch/signature/provenance или AgentChangeSet precondition → no install/apply; current project revision remains.
- Invalid model compatibility, missing evaluation/media evidence или forged certification → no `PhysicalCertified` promotion; prototype or previous certified package remains usable by explicit project policy.
- Runtime attempt to mutate weights or bypass owner review → request denied, policy quarantined on repeated violation, authoritative state unchanged.
- Evidence/media/dossier validation failure → bundle quarantined before viewing; prior valid evidence/decision remains immutable.
- Missing/invalid/stale/agent-issued reviewer attestation → no changeset admission; reviewer key incident follows revoke/re-review policy.
- Revoked reviewer key → все связанные решения invalid независимо от недоверенного timestamp; требуется новый review non-revoked key.
- Invalid baseline Bootstrap/Replace precondition → atomic index unchanged; automatic retry не создаёт approval.
- Unsafe token или crate lint opt-out вне exact FFI allowlist → boundary gate fail; unsafe не переносится в общий workspace policy.
- Encoder license/config unknown → MEDIA-P1 not passed; retain canonical PNG/WAV/GIF fallback, MP4-required review AwaitingCapability.
- Missing/inconsistent generation provenance, service/output terms, source consent or protected-data classification → candidate stays quarantined; no normalization changeset, cook or human dossier.
- Generation worker sandbox escape, output/quota overrun or unauthorized network attempt → terminate worker, quarantine staging, revoke job capability and audit; project/source revision remains exact.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback/rollback |
|---|---|---|---|---|
| SEC-01 | locked source/runtime dependency scan | 0 unreviewed critical/high exploitable advisories; checksums/sources 100% | signed scan report | update/remove dependency; block release |
| LIC-01 | license/SBOM policy scan every package | 100% artifacts classified; 0 non-allowlisted without valid exception; notices complete | SBOM, policy report, exceptions | remove/replace artifact |
| SEC-02 | parser/VM/IPC fuzz aggregate | required CPU-hour suites completed, 0 escape/UB/host crash | fuzz manifests | disable affected surface |
| SEC-03 | inspector/network/offline test | default offline produces 0 outbound connection attempts; unauthorized live access 100% denied | packet capture/audit log | disable adapter/endpoint |
| SEC-04 | malicious/incompatible model corpus | 100% oversized/hash/schema/opset/non-finite cases rejected before actuation | validator report | heuristic controller |
| GOV-01 | release governance packet | DCO on every commit; CONTRIBUTING, Contributor Covenant 2.1, SECURITY disclosure state, role/capability registry и release-signing policy present/consistent; required approvals and ADR/traceability consistency 100%; public contact not configured → AwaitingCapability, never PASS | governance validation report, signed review record | release block; configure permanent monitored contact |
| LEGAL-IMPORT-01 | importer legal review | exact release has written `approved` decision | legal record | importer not distributed |
| SEC-05 | mechanic package + agent authoring aggregate | MOD-02, AGENT-02 и MCP-P1 security cases pass; 0 ambient authority/out-of-root mutation | package/changeset/MCP audit packet | disable package/adapter; reviewed manual workflow |
| SEC-06 | physical model/provenance/certification corpus | 100% hash/schema/license/provenance/certification mismatches rejected before activation; 0 runtime weight writes; owner review present for every promoted model | validator/fault report, provenance graph, review records | quarantine model; PrototypeFallback/previous certified revision |
| SEC-07 | evidence/dossier/reviewer trust corpus | 100% malicious HTML/media/quota/redaction/tamper cases rejected before viewing; 100% agent/unauthorized/stale attestations rejected; 0 reviewer secret exposed to agent/capture process | sandbox/privacy/tamper/credential audit packet | quarantine bundle, revoke key, regenerate/re-review |
| BASELINE-01 | empty/existing baseline key, Bootstrap/Replace/CAS/role/revocation fault corpus | valid Bootstrap only on absent key; valid Replace only on exact current hash; all stale/wrong-role/revoked cases rejected with index unchanged | baseline index/decision manifests, atomic fault audit | retain current baseline; candidate stays unpromoted |
| ATTEST-01 | RFC 8785/8032 golden and tamper vectors across supported platforms | canonical bytes/signatures exact; wrong domain/field/role/key/algorithm/revocation rejected 100% | canonical vectors, trust/revocation report | reject decision; obtain new authorized attestation |
| FFI-01 | Cargo allowlist, unsafe/public-API and applicable Miri/sanitizer suite | exact allowlist; 0 unsafe outside it; 0 vendor/raw FFI type in public contracts; all required safety evidence PASS | metadata/boundary/API/SAFETY/Miri/sanitizer reports | safe adapter or process boundary; backend unavailable |
