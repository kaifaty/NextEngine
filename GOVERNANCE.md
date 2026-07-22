# Bootstrap governance

NextEngine остаётся local pre-release bootstrap. Уполномоченный repository owner является final maintainer для архитектуры, security, licensing и release decisions до принятия permanent public governance. Subsystem ownership задаёт нормативный architecture packet.

## Capability status

| Capability/status | Bootstrap authority | Public-release state |
|---|---|---|
| `architecture.approve` | Repository owner acting as Architecture Working Group | Named reviewer group required |
| `review.changeset` | Explicit authorized human reviewer only | Reviewer registry + valid attestation required |
| `baseline.promote` | Explicit authorized human promoter only; independent from changeset review | ReviewerTrustManifest capability required |
| `publisher.trust` | Repository owner + Security & Governance | PublisherTrustManifest process required |
| `release.sign` | No configured production key | `AwaitingCapability` |
| `security.triage` | Repository owner via out-of-band bootstrap contact | Permanent monitored private contact required |
| `conduct.enforce` | Repository owner via out-of-band bootstrap contact | Permanent monitored contact required |

Один человек MAY владеть несколькими capabilities, но каждая проверяется независимо. Coding agent, test runner, capture worker, MCP adapter и training backend не могут владеть или использовать trusted human capability.

## Approval matrix

| Change | Required approval |
|---|---|
| Public contract или architecture semantics | Subsystem owner + `architecture.approve`; новый superseding ADR |
| Capability/parser/IPC/trust/license boundary | Subsystem owner + Security & Governance + Architecture |
| Observable changeset | Automatic gates PASS + authorized `review.changeset` exact hash |
| Baseline Bootstrap/Replace | Automatic evidence PASS + independent `baseline.promote` exact decision |
| Model/physical certification | Owning subsystem + Security/Provenance + required TRAIN/POLICY/PHYS evidence |
| Importer publication | Exact written legal approval |
| Release | All resolved gates + release owner + configured signing capability |

Passing automatic checks не создаёт human approval. Human decision не может отменить automatic failure. Любое изменение changeset/evidence/policy/baseline hash инвалидирует decision.

## Architecture changes

Accepted architecture меняется только новым ADR и синхронизированными SPEC/evidence/traceability updates. Proposed Packet 1.5 остаётся candidate до human approval exact candidate hash. После approval status/supersession update выполняется атомарно; history Accepted ADR не переписывается.

### Hash-bound promotion records

Bootstrap promotion 1.4→1.5, 1.5→1.6 и 1.6→1.7 использует ненормативные records в `docs/reviews/architecture/packet-1.5.md`, `packet-1.6.md` и `packet-1.7.md`. Record не входит в architecture document counts и не является источником subsystem semantics.

Каждый record MUST содержать точный transition, `Pending` или `Approved` status, candidate file manifest, candidate root, результаты automatic checks, решения всех required owners и отдельные capability decisions `architecture.approve` и `baseline.promote`. Один человек MAY записать обе capability decisions, но строки и decision references MUST быть раздельными. Agent MAY вычислять hashes и проверять структуру, но MUST NOT заполнять human identity/decision или объявлять такой record одобренным от имени reviewer.

Candidate manifest охватывает каждый Markdown-файл в `docs/architecture/`, использует repository-relative UTF-8 paths и lowercase SHA-256 каждого файла. Review records находятся вне этого scope и исключаются из manifest. Entries сортируются по UTF-8 path; для каждой entry в hash input последовательно добавляются `path`, один NUL byte, 64 ASCII bytes lowercase `file_sha256` и один LF byte. SHA-256 всей последовательности является `Candidate root SHA-256`; machine identifier алгоритма — `sha256-path-nul-file-sha256-lf-v1`.

Любое изменение manifest file инвалидирует hash closure и требует нового review. `Approved` record действителен только когда все обязательные automatic checks имеют `PASS` с evidence reference, все required owner decisions имеют `Approved`, обе bootstrap capabilities записаны отдельно и authoritative transition chain непрерывна. `docs-check` проверяет exact file hashes, root, completeness и sequencing, но не является криптографической проверкой личности bootstrap reviewer.

## Release signing policy

- Release signing key MUST находиться вне Git, agent context, build logs и evidence media.
- Release manifest MUST содержать exact source/content/toolchain/SBOM hashes, signing algorithm/key ID и trust-manifest revision.
- Key rotation/revocation MUST быть опубликована project trust manifest и требует повторной подписи непубликованного candidate.
- Unknown/revoked key или missing signing capability блокирует release fail-closed.
- Bootstrap не имеет production signing key: `release_signing_status: AwaitingCapability`.

## Public readiness status

- `public_contact_status: AwaitingCapability`
- `security_disclosure_status: BootstrapOutOfBand`
- `conduct_enforcement_status: BootstrapOutOfBand`
- `release_signing_status: AwaitingCapability`

Public release и permanent remote governance не считаются готовыми, пока permanent monitored disclosure/conduct contact, maintainer succession и release signing trust не настроены и не прошли `GOV-01`.
