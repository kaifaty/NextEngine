# ADR-023: HumanReviewDecisionV2 и offline attestation

| Поле | Значение |
|---|---|
| ID | ADR-023 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Verification & Evidence Team |
| Требуемые согласующие | Architecture Working Group, Verification & Evidence Team, Importer Team, Security & Governance Team |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | отсутствуют |
| Заменяет | [ADR-015](015-evidence-trust-fixture-separation-and-attestation.md) |
| Заменён | не заменён |

## История принятия

ADR заменяет ADR-015 и сохраняет его artifact-first acceptance, split-fixture, cleanup/privacy, offline project trust, revocation и signer-isolation решения. Он исправляет неоднозначность `Approve|Reject` против `Approved|Rejected|NeedsChanges`, отделяет cryptographic verification от admission, подписывает `project_id` и `key_id` как явные поля V2 envelope и закрывает использование V1 для нового admission.

Принятие ADR не создаёт reviewer keys, human decision, legal approval или evidence PASS. Любая такая запись остаётся отдельным hash-bound artifact и проходит соответствующие automatic gates.

## Контекст

В ADR-015 `HumanReviewDecisionV1` объявлял `Approve|Reject`, тогда как SPEC-15 использовал `Approved|Rejected|NeedsChanges`. Кроме того, V1 envelope содержал `key_id`, но не включал его в signed preimage, а `project_id` входил в preimage, не будучи явным envelope field. Формулировка «valid decision accepted» смешивала два разных результата:

1. payload/envelope cryptographically valid и подписан authorized reviewer;
2. exact changeset допускается к mutation, merge или promotion.

`Reject` и `NeedsChanges` обязаны быть полноценными подписанными решениями, но не должны становиться admission token. Простое переименование V1 fields или автоматический upgrade разрушили бы проверяемость historical records.

## Сохранённый artifact-first contract

- Required suites определяет engine-owned `ImpactResolver`; author/agent MAY добавлять проверки, но не уменьшать resolved set.
- Automatic gates всегда предшествуют human admission и не могут быть waived human decision.
- Observable categories `visual`, `ui`, `camera`, `animation`, `physics`, `motor`, `audio` требуют exact hash-bound review.
- Raw normalized frame/audio roots являются canonical evidence; MP4/GIF — review derivatives.
- Missing GPU, encoder или reviewer capability даёт `AwaitingCapability`, а не `PASS`.
- Human reviewer принимает решение только по complete, verified, redacted bundle; dossier, media и diagnostic strings остаются untrusted input до проверки.

Сохранённый `MEDIA-P1` threshold:

| Поле | Значение |
|---|---|
| Сценарий | `next gate MEDIA-P1 --corpus canonical-review-media --targets windows-x86_64,linux-x86_64` |
| Threshold | exact pinned binary/config/SBOM; GIF/MP4 outputs decode successfully; frame count, dimensions, timestamp sequence и audio sample count match CapturePlan на 100%; normalized raw frame/audio roots remain exact; 0 outbound network attempts; all license/config fields classified |
| Evidence | encoder manifest, build/config/license/SBOM report, decoded stream report, frame/audio hashes, packet capture |
| Fallback | engine-owned canonical PNG/WAV set + GIF encoder; MP4-required review остаётся `AwaitingCapability`, пока другой MediaEncoder adapter не пройдёт `MEDIA-P1` |

## Split-fixture vertical model

### `vertical-v1-import-smoke`

Import smoke использует только user-provided Gothic installation во временном isolated root и закрывает VS-01/`LEGAL-IMPORT-01` import→cook→load checks. Runner MUST:

- принимать source root только как untrusted external input вне repository и publishable evidence roots;
- разрешать source-file access только importer process согласно manifest allowlist;
- хранить NIM, cooked bytes и transient projections только в ephemeral root;
- никогда не публиковать NIM, cooked/imported bytes, asset payloads, screenshots, video или audio;
- публиковать только sanitized source provenance/hash metadata, validator result, load projection summary и source-file-access trace;
- не включать source paths, filenames, strings или hashes, если policy классифицирует их как protected disclosure beyond approved metadata.

Screenshot исключён из VS-01. `LEGAL-IMPORT-01` сохраняет самостоятельную blocking legal approval и не заменяется neutral fixture или техническим PASS.

### `vertical-v1-neutral`

VS-02…VS-15, screenshots/video/audio, capture replay и human review используют только project-generated fixture с recorded CC0-1.0 provenance. Neutral fixture не содержит imported structure, names, textures, audio, scripts, cooked output или hashes из Gothic installation. Scenario manifest связывает exact fixture content hash, build/config/schema hashes и replay.

Human-review bundle MUST строиться только из neutral fixture. Смешанный bundle отклоняется как `EVIDENCE_FIXTURE_CLASS_MIXED`.

### Cleanup и privacy ordering

До VS-09 ephemeral import root MUST быть уничтожен либо quarantined вне repository/cache/build/package/final-artifact/publishable-evidence roots. Quarantine root недоступен capture worker и publisher. Cleanup audit фиксирует root identity token, lifecycle result и allowed sanitized hashes без protected bytes.

VS-09/`PRIVACY-02` сканирует:

- parent repository и Git index;
- tool/runtime caches;
- build and staging roots;
- packages/distribution roots;
- final artifact and EvidenceBundle roots;
- captured media metadata и logs.

Любое protected/imported byte finding quarantines run, блокирует publication/promotion и даёт stable location-class diagnostic. Cleanup failure не может быть overridden человеком.

## `HumanReviewDecisionV2`

`HumanReviewDecisionV2` — closed JCS-canonical object:

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
baseline =
  { kind = "None" }
  | { kind = "EvidenceBaseline", sha256 }
verification_policy_sha256
impact_manifest_sha256
fixture = {
  class = "vertical-v1-neutral"
  sha256
}
automatic_gate_summary_sha256
requirement_graph_sha256
gate_descriptor_set_sha256
review_category
decision = "Approve" | "Reject" | "NeedsChanges"
reviewer_role
reason_code
issued_at_unix_seconds
```

Normative encoding rules:

- object, `subject`, `baseline` и `fixture` являются closed schemas: unknown, duplicate, missing, `null` и implicit-default fields rejected;
- exact JCS UTF-8 bytes являются payload bytes; `project_id`, `review_category` и `reviewer_role` MUST быть non-empty valid Unicode NFC strings without control characters, а `reason_code` — non-empty ASCII identifier;
- every `*_sha256` and `sha256` field is exactly 64 lowercase hexadecimal characters;
- `decision_id` is a project-unique 128-bit value encoded as exactly 32 lowercase hexadecimal characters;
- `issued_at_unix_seconds` is a non-negative JSON safe integer; wall-clock time не участвует в simulation;
- enum tokens case-sensitive и ограничены exact values выше;
- mutable display comment, reviewer display name и rendered artifact list не входят в payload.

`requirement_graph_sha256` связывает exact canonical `RequirementGraphV1`, а `gate_descriptor_set_sha256` — canonical ordered set всех применимых `GateDescriptorV1`; оба пересчитываются после automatic validation и делают gate/trace drift изменением reviewed subject. Повторный `decision_id` с другим payload hash rejected как `REVIEW_DECISION_ID_REUSED`; exact duplicate является idempotent input. Любое изменение subject, evidence, baseline, policy, impact, fixture, automatic summary, requirement graph, gate descriptors, category, decision, role, reason или issued time требует нового payload, нового `decision_id` и новой human signature.

## `AttestationEnvelopeV2`

`AttestationEnvelopeV2` — closed JCS-canonical object:

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

`domain` MUST equal exact ASCII literal `nextengine.human-review-decision.v2`. `payload_hash`, `trust_policy_hash`, `trust_manifest_hash` и `revocation_snapshot_hash` are exactly 64 lowercase SHA-256 hexadecimal characters. `payload_hash` is SHA-256 of exact canonical payload bytes; the other three hashes are SHA-256 of exact canonical referenced policy/manifest/snapshot bytes. `signature` is canonical unpadded base64url and MUST decode to exactly 64 bytes. `project_id` is Unicode NFC; `key_id` is a non-empty ASCII identifier from the referenced trust manifest. Unknown, duplicate, missing or `null` fields are rejected.

The Ed25519 signature covers this exact byte preimage:

```text
"nextengine.human-attestation.v2\0"
|| u32_le(schema.len) || schema_ascii
|| u32_le(algorithm.len) || algorithm_ascii
|| u32_le(domain.len) || domain_ascii
|| u32_le(project_id.len) || project_id_nfc_utf8
|| u32_le(key_id.len) || key_id_ascii
|| u32_le(payload_type.len) || payload_type_ascii
|| payload_hash[32]
|| trust_policy_hash[32]
|| trust_manifest_hash[32]
|| revocation_snapshot_hash[32]
```

Lengths count the following byte sequence, not Unicode scalar values. Hex fields are decoded to their 32 raw bytes before preimage assembly. The verifier MUST require exact `domain`, exact equality between envelope and payload `project_id`, recompute `payload_hash` from exact canonical payload bytes, resolve the signed `key_id` only in the hash-bound trust manifest and reject any schema/domain/payload-type substitution.

## Offline project trust и revocation

V2 reuses the following immutable trust structures without changing their schemas:

- project/release policy pins `ProjectTrustAnchorV1` fingerprints outside the bundle under review;
- signed `ReviewerTrustManifestV1` binds project identity, policy version/hash, reviewer public keys and `key_id`, allowed roles/scopes/categories, validity interval, rotation predecessor/successor overlap, trust-root fingerprint and manifest issuance/expiry;
- signed monotonic `ReviewerRevocationSnapshotV1` binds trust-manifest hash, snapshot sequence, issued-at, next-update, revoked/compromised key IDs and effective compromise/revocation instants.

Admission MUST offline verify the exact project, payload, bundle, baseline, verification-policy, impact, automatic-summary, trust-policy, trust-manifest and revocation-snapshot closure; signer role/scope/category; decision time inside key validity; rotation chain; snapshot freshness; and absence of effective revocation/compromise. Bundle-supplied trust roots, stale snapshots, unknown roles, expired keys, missing rotation chain and agent/test keys fail closed.

Later-compromise policy MUST explicitly state which historical decisions are invalidated. Verifier не придумывает local exception и не обращается к network. Concrete JCS, SHA-256 and Ed25519 libraries remain replaceable implementation choices, not public contracts.

## Signer isolation

`next review record` sends canonical unsigned V2 payload and expected hashes to an isolated human signer adapter. Public CLI schema, environment dump, logs and artifacts MUST NOT accept or contain raw private-key path, key bytes, seed phrase or agent credential. Adapter returns only `AttestationEnvelopeV2` and public certificate/trust-chain material.

Agent workspace, runner, capture worker, plugin, AI process and optional MCP adapter have no access to the private key and cannot issue a trusted reviewer role. Test keys use a separate untrusted project ID and never enter a production trust manifest. Agent-issued decision is rejected independently of cryptographic validity.

## Verification и admission

Cryptographic verification and admission are separate deterministic operations:

1. every persisted V2 review decision MUST have an `AttestationEnvelopeV2`; verifier validates canonical V2 payload/envelope, signature, project/hash closure and offline trust;
2. a successful verifier returns exactly one of `VerifiedDecision::Approve`, `VerifiedDecision::Reject` or `VerifiedDecision::NeedsChanges`;
3. admission evaluates the verified result together with required automatic gates, evidence completeness, baseline and redaction results;
4. only `VerifiedDecision::Approve` with every required automatic gate `PASS` and complete valid evidence returns `Admit`;
5. `VerifiedDecision::Reject` and `VerifiedDecision::NeedsChanges` are signed, cryptographically valid, machine-readable non-admitting feedback;
6. `Approve` with any automatic result other than `PASS` is rejected as `AUTO_GATE_NOT_PASS`; human decision cannot waive failure or missing capability.

Success of signature verification alone MUST NOT emit gate `PASS`, mutate project state, merge, promote a baseline/model/package or publish a release.

## V1 historical verification only

`HumanReviewDecisionV1` and `AttestationEnvelopeV1` remain readable only in an explicit read-only `historical-audit` verifier mode. Successful legacy verification returns `HistoricalVerified`, never `VerifiedDecision`, `PASS`, `Admit`, merge, promotion or baseline acceptance.

Current admission policy MUST set `review_contract_major = 2`. Supplying V1 to an admission path fails closed as `REVIEW_SCHEMA_HISTORICAL_ONLY`. Generic verifier API requires an explicit known mode; gate and mutation-capable workflows are hard-wired to `admission`, while `historical-audit` is always explicit. V1 payload/envelope MUST NOT be relabelled, field-renamed, automatically upgraded or automatically re-signed. A current decision requires a fresh V2 payload reviewed and signed by a human.

## Stable diagnostics

At minimum, implementations expose:

- `REVIEW_SCHEMA_HISTORICAL_ONLY`;
- `REVIEW_DECISION_VALUE_INVALID`;
- `REVIEW_DECISION_ID_REUSED`;
- `REVIEW_ENVELOPE_DOMAIN_MISMATCH`;
- `REVIEW_ENVELOPE_PAYLOAD_TYPE_MISMATCH`;
- `REVIEW_ENVELOPE_PROJECT_MISMATCH`;
- `REVIEW_PAYLOAD_HASH_MISMATCH`;
- `REVIEW_ATTESTATION_SIGNATURE_INVALID`;
- `REVIEW_DECISION_NON_ADMITTING`;
- `AUTO_GATE_NOT_PASS`;
- `EVIDENCE_FIXTURE_CLASS_MIXED`.

Diagnostics do not contain private key material or protected/imported bytes.

## Gates

| Gate | Blocking contract | Negative corpus |
|---|---|---|
| `REVIEW-01` | Observable changes require cryptographically verified V2 `Approve` plus all required automatic gates `PASS`; `Reject` and `NeedsChanges` remain verified non-admitting feedback | Missing review, each exact token, invalid token/case, automatic FAIL/AwaitingCapability with `Approve`, hash change after signing, V1 admission attempt |
| `REVIEW-02` | Offline verifier deterministically validates all three V2 decisions and exact envelope/trust closure without network or private key; only `Approve` is admission-eligible | Tampered/non-canonical payload, wrong schema/domain/payload type/project/key/hash/role/scope/time/rotation/revocation, stale snapshot, agent/test key, cross-version substitution |
| `SEC-07` | Dossier/media/reviewer boundary rejects malicious content and exposes no signer secret | HTML/media/quota/redaction attacks, bundle-supplied trust root, secret in CLI/env/log/artifact/capture process |
| `PRIVACY-02` | Split fixtures and cleanup prevent protected bytes in every prohibited root; neutral human bundle has no imported dependency | Protected marker in repository/cache/build/package/final/evidence/media/log roots, mixed fixture bundle, failed cleanup/quarantine, capture from import-smoke |

`PRIVACY-02` PASS не означает `LEGAL-IMPORT-01` PASS. Missing reviewer, GPU, encoder or shipping capability remains `AwaitingCapability`, not `PASS`.

## Последствия и синхронизация

- SPEC-09, SPEC-11 и SPEC-15 use V2 as the only current review/admission contract.
- Accepted documents MUST depend on ADR-023 rather than ADR-015 for current semantics.
- V1 artifacts remain immutable historical evidence but cannot authorize new state.
- Implementations need separate schema registries/result types for legacy audit, V2 verification and admission.
- Any later token, payload, envelope, trust or admission semantic change requires a new superseding ADR and synchronized RFC, evidence register and traceability update.
