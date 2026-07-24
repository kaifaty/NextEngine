# Bootstrap governance

NextEngine остаётся local pre-release bootstrap, который ведёт один человек. `Repository Owner` является владельцем всех SPEC и final maintainer для architecture, security, licensing и release decisions до принятия permanent public governance. Runtime/state ownership подсистем по-прежнему задаётся самими SPEC и не требует имитации отдельных human reviewers.

## Capability status

| Capability/status | Bootstrap authority | Public-release state |
|---|---|---|
| `review.changeset` | Explicit authorized human reviewer only | Reviewer registry + valid attestation required |
| `baseline.promote` | Explicit authorized human promoter only; independent from changeset review | ReviewerTrustManifest capability required |
| `publisher.trust` | Repository owner + Security & Governance | PublisherTrustManifest process required |
| `release.sign` | No configured production key | `AwaitingCapability` |
| `security.triage` | Repository owner via out-of-band bootstrap contact | Permanent monitored private contact required |
| `conduct.enforce` | Repository owner via out-of-band bootstrap contact | Permanent monitored contact required |

Architecture documents use the normal repository review workflow. A direct Repository Owner task is sufficient authorization for the requested documentation change; no separate exact-root capability or promotion record is required. `baseline.promote`, `review.changeset`, publisher/release trust and implementation evidence remain separate capabilities. Coding agents, test runners, capture workers, MCP adapters and training backends cannot own or use those trusted human capabilities.

## Approval matrix

| Change | Required approval |
|---|---|
| Public contract или architecture semantics | Direct Repository Owner task; новый superseding ADR для изменения Accepted semantics |
| Capability/parser/IPC/trust/license boundary | Repository Owner + обязательные automatic security/boundary gates |
| Observable changeset | Automatic gates PASS + authorized `review.changeset` exact hash |
| Baseline Bootstrap/Replace | Automatic evidence PASS + independent `baseline.promote` exact decision |
| Model/physical certification | Owning subsystem + Security/Provenance + required TRAIN/POLICY/PHYS evidence |
| Importer publication | Exact written legal approval |
| Release | All resolved gates + release owner + configured signing capability |

Passing automatic checks не создаёт human approval. Human decision не может отменить automatic failure. Любое изменение changeset/evidence/policy/baseline hash инвалидирует decision.

## Architecture changes

Accepted architecture semantics меняются только новым ADR и синхронизированными SPEC/evidence/traceability updates. Редакционные исправления и согласованные изменения пакета проходят обычный repository review; отдельное exact-root подтверждение не требуется. History Accepted ADR не переписывается.

### Historical promotion records

Records in `docs/reviews/architecture/packet-1.5.md` through `packet-1.8.md` are retained only as historical audit artifacts. They do not define a required workflow for future architecture edits, and `host-check` does not validate candidate roots or human documentation decisions.

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
