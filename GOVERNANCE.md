# Bootstrap governance

NextEngine остаётся local pre-release bootstrap, который ведёт один человек. `Repository Owner` является владельцем всех SPEC и final maintainer для architecture, security, licensing и release decisions до принятия permanent public governance. Runtime/state ownership подсистем по-прежнему задаётся самими SPEC и не требует имитации отдельных human reviewers.

## Capability status

| Capability/status | Bootstrap authority | Public-release state |
|---|---|---|
| `architecture.promote` | Repository Owner после automatic checks и проверки exact candidate root | Named reviewer group required |
| `review.changeset` | Explicit authorized human reviewer only | Reviewer registry + valid attestation required |
| `baseline.promote` | Explicit authorized human promoter only; independent from changeset review | ReviewerTrustManifest capability required |
| `publisher.trust` | Repository owner + Security & Governance | PublisherTrustManifest process required |
| `release.sign` | No configured production key | `AwaitingCapability` |
| `security.triage` | Repository owner via out-of-band bootstrap contact | Permanent monitored private contact required |
| `conduct.enforce` | Repository owner via out-of-band bootstrap contact | Permanent monitored contact required |

`architecture.promote` относится только к документационному packet. `baseline.promote`, `review.changeset`, publisher/release trust и implementation evidence сохраняют отдельный смысл и не объединяются с ним. Coding agent, test runner, capture worker, MCP adapter и training backend не могут владеть или использовать trusted human capability.

## Approval matrix

| Change | Required approval |
|---|---|
| Public contract или architecture semantics | Repository Owner + `architecture.promote`; новый superseding ADR для изменения Accepted semantics |
| Capability/parser/IPC/trust/license boundary | Repository Owner + обязательные automatic security/boundary gates |
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

Каждый record MUST содержать точный transition, `Pending` или `Approved` status, candidate file manifest, candidate root, результаты pre-approval automatic checks и единственную human decision `architecture.promote`, которой владеет `Repository Owner`. Agent MAY вычислять hashes и проверять структуру, но MUST NOT заполнять identity/decision или объявлять record одобренным от имени owner.

Candidate manifest охватывает каждый Markdown-файл в `docs/architecture/`, использует repository-relative UTF-8 paths и lowercase SHA-256 каждого файла. Review records находятся вне этого scope и исключаются из manifest. Entries сортируются по UTF-8 path; для каждой entry в hash input последовательно добавляются `path`, один NUL byte, 64 ASCII bytes lowercase `file_sha256` и один LF byte. SHA-256 всей последовательности является `Candidate root SHA-256`; machine identifier алгоритма — `sha256-path-nul-file-sha256-lf-v1`.

Promotion использует две fail-closed фазы, чтобы aggregate `host-check` не зависел циклически от собственного результата:

1. `architecture-review-preflight <target>` проверяет полный candidate packet и `Pending` record. Format, clippy, workspace tests, boundary scan и diff check MUST уже иметь `PASS`; собственная preflight row MAY быть `Pending` только во время первого запуска.
2. После повторного preflight с полностью заполненной PASS-таблицей Repository Owner проверяет exact candidate root и отдельно принимает или отклоняет `architecture.promote`.
3. Только после `Approved` decision обычные `docs-check` и `host-check` выполняют final authoritative admission. Их результат входит в handoff/commit evidence, но не в pre-approval table, поскольку оба зависят от approved record.

Любое изменение manifest file инвалидирует hash closure и требует нового promotion decision. `Approved` record действителен только когда все обязательные pre-approval checks имеют `PASS` с evidence reference, `architecture.promote` имеет `Approved`, а authoritative transition chain непрерывна. `docs-check` проверяет exact file hashes, root, completeness и sequencing, но не является криптографической проверкой личности owner.

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
