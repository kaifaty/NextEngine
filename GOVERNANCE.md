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
