# ADR-015: Evidence trust, fixture separation и offline attestation

| Поле | Значение |
|---|---|
| ID | ADR-015 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | Verification & Evidence Team + Security & Governance Team |
| Требуемые согласующие | Architecture Working Group, Verification & Evidence Team, Importer Team, Security & Governance Team |
| Дата предложения | 2026-07-22 |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-10](../10-gothic-importer-boundary.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [ADR-010](010-artifact-first-headless-validation-and-review.md), [ADR-012](012-deterministic-command-identity-and-replay.md) |
| Заменяет | [ADR-010](010-artifact-first-headless-validation-and-review.md) после принятия packet 1.5 |
| Заменён | не заменён |

## Статус предложения

Этот ADR сохраняет artifact-first acceptance, engine-owned `ImpactResolver` и automatic-gate precedence ADR-010. Он становится authoritative только вместе с ADR-012 canonical encoding и полным packet 1.5 approval. Candidate не создаёт reviewer keys, human decision, legal approval или evidence PASS.

## Контекст

Один vertical fixture не может одновременно быть publishable capture source и производным пользовательской Gothic installation. Кроме того, hash-bound review без offline trust root, key scope, validity и revocation не определяет, кто вправе подписать решение и что именно verifier принимает.

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

## Canonical human decision

`HumanReviewDecisionV1` — JCS-canonical payload со следующими required fields:

- schema/version и project identity;
- changeset hash и exact commit/diff hash;
- `EvidenceBundleManifest` hash;
- baseline hash или explicit no-baseline marker;
- `VerificationPolicyManifest` hash и `ImpactResolver` output hash;
- fixture class/hash, automatic-gate summary hash и review category;
- decision `Approve` или `Reject`, reviewer role, reason code, issued-at UTC instant и unique decision ID.

Любое изменение любого input требует нового decision; mutable comment не изменяет payload.

## AttestationEnvelopeV1

Envelope является JCS-canonical object:

```text
schema = "nextengine.attestation-envelope.v1"
algorithm = "ed25519"
key_id
payload_type = "HumanReviewDecisionV1"
payload_hash = lowercase sha256 hex
trust_policy_hash = lowercase sha256 hex
trust_manifest_hash = lowercase sha256 hex
revocation_snapshot_hash = lowercase sha256 hex
signature = base64url without padding
```

Signed preimage:

```text
"nextengine.human-attestation.v1\0"
|| u32_le(project_id.len) || project_id_nfc_utf8
|| u32_le(payload_type.len) || payload_type_ascii
|| payload_hash[32]
|| trust_policy_hash[32]
|| trust_manifest_hash[32]
|| revocation_snapshot_hash[32]
```

Verifier rejects unknown field semantics/version, non-canonical encoding, wrong algorithm, padded/non-canonical base64url, invalid key/signature or hash mismatch.

## Offline project trust

Offline verifier получает immutable `ProjectTrustAnchorV1` fingerprints из release/project policy, а не из bundle under review. Anchor authorizes trust-manifest signing keys, not reviewers directly.

Signed JCS `ReviewerTrustManifestV1` содержит project identity, policy version/hash, reviewer public keys, `key_id`, allowed roles/scopes/categories, validity interval, rotation predecessor/successor overlap, trust-root fingerprint и manifest issuance/expiry. Signed `ReviewerRevocationSnapshotV1` содержит trust-manifest hash, monotonic snapshot sequence, issued-at, next-update, revoked/compromised key IDs и effective compromise/revocation instants.

Admission проверяет exact project/payload/bundle/baseline/policy/manifest/revocation hashes, signer role/scope, decision time внутри validity, snapshot freshness и отсутствие revocation/compromise на decision time. Stale snapshot, unknown role, expired key или missing rotation chain fail-closed. Later compromise policy MUST явно определять, какие historical decisions invalidated; verifier не применяет local guess.

## Signer isolation

Команда `next review record` передаёт canonical unsigned payload и expected hashes в isolated human signer adapter. Public CLI schema, environment dump, logs и artifacts MUST NOT принимать или содержать raw private-key path, key bytes, seed phrase или agent credential. Adapter возвращает только envelope/public certificate chain. Agent workspace, runner, capture worker, plugin и AI process не имеют доступа к private key и не могут выдавать trusted role.

Test keys маркируются отдельным untrusted project ID и никогда не входят в production trust manifest. Agent-issued decision всегда отклоняется независимо от cryptographic validity.

## Gates

| Gate | Blocking contract | Negative corpus |
|---|---|---|
| `REVIEW-02` | Offline verifier однозначно принимает exact valid decision и отвергает все trust/hash violations | Tampered payload, wrong bundle/baseline/policy, wrong role, expired/revoked/compromised key, stale snapshot, invalid rotation, agent/test key, non-canonical envelope |
| `PRIVACY-02` | Split fixtures and cleanup prevent protected bytes in every prohibited root; neutral human bundle has no imported dependency | Protected marker in repository/cache/build/package/final/evidence/media/log roots, mixed fixture bundle, failed cleanup/quarantine, capture from import-smoke |

`REVIEW-02` PASS требует offline verification without network or private key. `PRIVACY-02` PASS не означает `LEGAL-IMPORT-01` PASS.

## Последствия и синхронизация при принятии

После approval ADR-010 получает `Superseded` и backlink. SPEC-09/10/11/12/15, VS-01/VS-09 and human-review rows, glossary, evidence register и traceability обновляются атомарно. Конкретные JCS/Ed25519 libraries не добавляются в technology register без отдельного decision; contract replaceable. До transition packet 1.4 остаётся authoritative, а candidate — `Proposed/AwaitingReview`.
