# ADR-083: Public creator project CLI vertical

| Field | Value |
|---|---|
| ID | ADR-083 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-18 |
| Last verified | 2026-08-18 |
| Normative dependencies | [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-025](025-schema-content-and-migration-authority.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](048-direct-exact-project-lock.md), [ADR-082](082-linux-first-development-and-deferred-windows-host.md) |
| Supersedes | Narrowly supersedes SPEC-09 and ADR-046 statements that no public creator CLI has a current consumer. Only the two commands and Creator Command Report V1 defined here become current; the broader agent-authoring/MCP/SDK design remains consumer-driven and unpromoted. |
| Superseded by | Narrowly [ADR-084](084-public-creator-run-and-project-package-vertical.md) for the later creator run/package consumer; the two R6a commands and Creator Command Report V1 remain unchanged |

## Context

R3 established a direct, file-backed Project Authoring V7 cooker and exact
content-addressed publication path, but only repository code and the reference
fixture exercised it. R6 now has a concrete external consumer: an author must
be able to validate and cook a second project without editing an engine crate
or calling a hidden first-party bootstrap.

Promoting the whole planned R6 surface at once would invent contracts for
inspectors, scenarios, packaging and migrations before their first workflows
exist. Keeping all creator operations behind `xtask` would fail the opposite
way: repository maintenance commands are not a public SDK entry point and do
not prove that a project can stand outside reference-game Rust source.

## Decision

### Current public command surface

The workspace provides a binary named `next` with exactly these current
creator operations:

```text
next project validate --project <project-directory>
next project cook --project <project-directory> --output <content-store-directory>
```

Both commands are non-interactive and use the project ID declared in
`project.authoring.json`. A CLI project-ID override, implicit defaults,
repository-specific fixture constructor or direct runtime mutation path is
forbidden.

`project validate` performs the production file-backed V7 load, complete cook
validation and publication assembly without changing caller output.
`project cook` performs the same work, atomically publishes the assembled
generation through `ContentStore`, then reopens and validates that generation
through the production activation path before reporting success.

### Creator Command Report V1

Every invocation writes exactly one JSON object and one trailing newline to
stdout. Success has `schema_version = 1`, `status = "PASS"`, the stable command
name and a `details` object containing project identity, exact composition
hashes, bounded counts and publication state. Failure has the same version,
`status = "FAIL"`, the best-known command name and one `diagnostic` containing
only stable `code`, `subsystem` and `message_key` fields.

Progress and optional human guidance belong on stderr. Raw machine-local
paths, parser/vendor text, source bytes and unordered logs are not report
fields. Exit status is zero only for `PASS`.

Report V1 and Project Authoring V7 remain current-only pre-v1 formats. A
recognizable retired authoring version returns the existing typed unsupported
diagnostic before nested schema processing; no migration or rewrite occurs.

### Filesystem and publication safety

Every referenced authoring/source/notice span must be a safe relative path
whose resolved target remains under the explicitly selected project root.
Textual traversal and symbolic-link escape fail with
`CONTENT_SOURCE_PATH_INVALID` before escaped bytes are consumed.
Each individual authoring/source/notice file is read under the current 16 MiB
bound; an oversized file fails with `CONTENT_SOURCE_LIMIT_EXCEEDED` before JSON
or nested content decode.

Cook output may be absent, empty or contain only the recognized ContentStore
root layout. A symbolic-link output, ordinary project directory or unrelated
nonempty directory fails with `CREATOR_OUTPUT_INVALID`. Invalid input is fully
loaded/cooked before output publication starts. ContentStore staging and the
atomic `CURRENT` switch remain the only publication mechanism, so a failed
operation cannot replace the prior active generation.

### First external fixture

`projects/creator-smoke` is a small independent Project Authoring V7 project.
It declares its own IDs/provenance and contains one character definition, item,
ability, dialogue, quest, relationship, interaction and three streamed world
chunks plus the minimal current render/world-service closure. It is data only:
no engine or reference-game crate contains a constructor for it.

The fixture must validate, cook and activate through the public command. The
normal content regression contour also opens it through the same production
loader/cooker/activation contracts so later engine changes cannot silently
return to a single-project assumption.

## Product impact

An external author or CI job gains the first stable machine-readable workflow:
edit a bounded project directory, validate it without side effects, then cook
an exact atomic generation that the engine itself can reopen. The second
project turns authoring extensibility from a type-level claim into an
observable product path and gives subsequent R6 inspectors/templates/run and
package commands a real consumer to extend.

This increment does not yet make the second project playable through the game
composition root or distributable as a v1 package. It therefore starts R6 and
reduces B-09 but does not close either.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| focused creator CLI | Validate/cook the independent project twice | Exact report V1, identical hashes, active generation reopens through production activation | Stable failure object; no active generation replacement |
| creator CLI negative matrix | Retired format, malformed content, path/symlink escape and unsafe output | Typed path-free code and nonzero exit; prior output remains unchanged | Reject complete operation; never guess/migrate |
| `content-package` | Load/cook/publish/activate reference and creator fixtures | Both independent projects close without fixture-only project construction | Fail the product check |
| `fast` / `host-check` | Parsing, report schema, cross-platform compilation and boundaries | Public binary has no runtime authority or reference-game dependency | Keep R6a incomplete |

## Considered alternatives

- **Expose the existing `xtask content-package` command.** Rejected: it is a
  repository conformance scenario with a hard-coded reference consumer, not an
  author-selected project workflow.
- **Add all planned R6 commands now.** Rejected: diff/inspect/run/package and
  scenario semantics need their own production consumers and acceptance data.
- **Let callers override project identity.** Rejected: it would make the lock
  identity differ from the reviewed authoring source and create a hidden
  first-party-style composition path.
- **Cook directly into a loose directory tree.** Rejected: it would bypass the
  existing content-addressed staging, verified index and atomic current switch.
- **Return rendered internal errors as the JSON contract.** Rejected: local
  paths and parser wording are unstable and may disclose host information.

## Consequences and next boundary

- SPEC-09 now distinguishes public creator commands from repository `xtask`.
- R6a owns only project validate/cook and report V1 compatibility within the
  current pre-v1 policy.
- ADR-084 uses this fixture for run/package with separate report contracts;
  read-only diff/inspect remains the next bounded creator surface.
- Windows/THOTH remains deferred under ADR-082; Linux `host-check` is the active
  implementation handoff.

## Supersession

Changing report V1 fields, accepting additional authoring versions, allowing
identity override, changing atomic output semantics or granting creator tools
gameplay mutation authority requires an explicit successor decision. Adding a
new bounded creator command with its own real consumer may be accepted by a
later ADR without superseding the two commands defined here.
