# ADR-085: Public creator project inspect and diff vertical

| Field | Value |
|---|---|
| ID | ADR-085 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-18 |
| Last verified | 2026-08-18 |
| Normative dependencies | [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-018](018-authoritative-project-composition-and-configuration.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](048-direct-exact-project-lock.md), [ADR-082](082-linux-first-development-and-deferred-windows-host.md), [ADR-083](083-public-creator-project-cli-vertical.md), [ADR-084](084-public-creator-run-and-project-package-vertical.md) |
| Supersedes | Narrowly supersedes ADR-046/084 statements that public creator inspect/diff has no current consumer or remains the next unimplemented creator boundary. Existing validate/cook/run/package commands and their three report contracts are unchanged. |
| Superseded by | not superseded |

## Context

R6a/R6b let an author validate, cook, run and distribute an independent project,
but answering “what exact project will run?” or “what changed between these two
closures?” still required opening private ContentStore files or writing Rust.
Raw manifest dumps are not a suitable public contract: they expose storage
layout, make source and package views differ, and leave callers to reconstruct
identity, ordering and change causality themselves.

The existing `creator-smoke` authoring/package pair is a concrete consumer for
one bounded read-only surface. It can prove that the editable source and the
distributed package describe the same immutable project without introducing a
scenario runner, editor mutation API, migration system, GUI or MCP protocol.

## Decision

### Current public commands

R6c adds exactly these operations:

```text
next project inspect --project <project-directory>
next project inspect --package <creator-package-directory>

next project diff \
  (--base-project <project-directory> | --base-package <creator-package-directory>) \
  (--candidate-project <project-directory> | --candidate-package <creator-package-directory>)
```

Each inspect invocation selects exactly one source. Each diff invocation selects
exactly one base and one candidate; the direction defines `added` and `removed`.
The tool never guesses source kind from directory contents and accepts no
project-identity override.

Authoring inspection performs the current production file-backed V7 load and
complete cook validation without publishing caller output. Package inspection
performs the complete ADR-084 inventory/NOTICE/activation validation and reruns
the recorded one-tick proof before exposing the activated closure. Temporary
state is private and removed before success; neither command writes into either
operand or mutates gameplay authority.

### Creator Project Projection V1

Both sources produce the same tool-owned immutable projection. It contains:

- exact project identity and composition/profile roots;
- bounded counts and the sorted current schema inventory;
- sorted root assets and content asset entries with stable IDs, exact hashes,
  schema identity, semantic class, provenance/license roots and generic neutral
  record identity where applicable;
- sorted required/optional content dependency edges;
- exact world partition identity, root regions and chunk bindings;
- exact mechanics registry root, locked package identities and granted
  capabilities.

The projection contains no filesystem path, source span, source bytes, neutral
property value, ContentStore generation path, ECS/runtime object, vendor type or
mutable domain reference. Collections are ordered by their stable public keys.
Moving identical authoring bytes to another directory therefore preserves the
projection, while changing exact authoring bytes still changes the declared
authoring/project roots.

### Creator Inspect and Diff Reports V1

Inspect and diff each receive a separate current-only report family with
`schema_version = 1`. They do not add fields to Creator Command, Run or Package
Report V1. Every invocation emits exactly one path-free JSON object and one
trailing newline.

Diff compares source-neutral projections, not their source labels. Equal
authoring and packaged closures therefore return `different = false`. Changes
are sorted and partitioned into project fields, exact roots, presentation
targets, schemas, root assets, assets, dependency edges, world fields/regions/
chunks and mechanic packages. Keyed collection records use `added`, `removed`
or `changed` and carry typed base/candidate values; the summary contains bounded
counts for every category.

A valid non-empty diff is a successful observation: it returns `status =
"PASS"` and exit status zero with `different = true`. Malformed, unsupported,
tampered or unsafe input returns `status = "FAIL"`, nonzero exit status and the
same stable path-free diagnostic boundary as earlier creator commands. Diff
never hides an invalid operand behind a partial comparison.

### Current-only and authority boundary

Projection/Inspect/Diff V1 are pre-v1 current-only contracts under ADR-046.
They do not promise tolerant readers or migration. Reports are diagnostic
immutable views and never become project, package, save, replay or gameplay
authority. No creator command receives a direct mutation sink from this ADR.

## Product impact

An author or CI job can now review exact runnable composition, compare a local
edit to a known package, verify that packaging introduced no domain drift and
identify a changed asset/package/root by stable ID without understanding private
storage. This completes the bounded R6c inspection increment and further reduces
B-09, but templates, cold-authoring documentation, scenarios/minimization,
replay inspection and the broader SDK remain open.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| focused creator CLI | Inspect repeated authoring bytes from two locations | Byte-identical path-free report; exact 17-schema/16-root/18-asset/three-chunk/two-package projection | Stable failure; no caller output |
| source/package parity | Inspect authoring and its published package, then diff them | Equal project/projection and `different = false` with every change count zero | Fail R6c; retain both inputs |
| controlled drift | Change one neutral item record and repeat directional diff | Deterministic `PASS`, one keyed asset change plus derived exact roots/package changes; unrelated schema/dependency/chunk collections remain empty | Report the valid diff; never treat difference as command failure |
| negative matrix | Tampered/unsupported package, malformed authoring or invalid operand flags | Typed path-free failure before a partial projection/diff | Reject complete operation |
| `content-package` | Run public inspect/diff over the existing creator package path | Authoring/package projections match after full packaged-byte verification | Fail the product check |
| `fast` / `host-check` | Formatting, strict lint, tests and boundaries | Existing report/command behavior remains compatible; no private/runtime authority leaks | Keep R6c incomplete |

## Considered alternatives

- **Dump private manifests or the package directory.** Rejected: storage layout
  is not the public project model and authoring/package output would drift.
- **Use source paths or array indices as diff identity.** Rejected: relocation
  and harmless ordering would create unstable change keys.
- **Return nonzero when valid projects differ.** Rejected: difference is the
  requested result, not invalid input; CI can decide policy from `different`.
- **Trust the package's recorded projection/run proof.** Rejected: package bytes
  remain untrusted and must pass complete current validation and rerun.
- **Expose properties or mutable live objects for richer inspection.** Rejected:
  this increment needs composition provenance, not a gameplay/editor backdoor.
- **Add scenarios, templates and replay inspection together.** Rejected: those
  are separate consumers with mutation/lifecycle and documentation semantics.

## Consequences and next boundary

- The public creator surface now has six operations and five separately
  versioned report families.
- Authoring and package share one exact source-neutral project projection.
- Valid drift is machine-readable success; invalid input remains fail-closed.
- R6 and B-09 remain open. The next bounded R6 increment should provide a
  reusable project template plus a documented cold-authoring exercise before
  scenario/minimization broadens lifecycle and mutation.
- Linux is the active implementation host. Windows/THOTH and paired shipping
  evidence remain `NotRun(WindowsHostDeferred)` under ADR-082.

## Supersession

Changing projection/report fields, collection identity/order, difference exit
semantics, package revalidation, source-kind selection or read-only authority
requires an explicit successor decision. A later consumer-backed template or
scenario ADR may extend the creator surface without changing this boundary.
