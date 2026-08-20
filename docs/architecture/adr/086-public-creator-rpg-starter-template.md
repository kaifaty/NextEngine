# ADR-086: Public creator RPG starter template

| Field | Value |
|---|---|
| ID | ADR-086 |
| Status | Accepted |
| Version | 1.1 |
| Decision date | 2026-08-19 |
| Last verified | 2026-08-20 |
| Normative dependencies | [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-018](018-authoritative-project-composition-and-configuration.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](048-direct-exact-project-lock.md), [ADR-082](082-linux-first-development-and-deferred-windows-host.md), [ADR-083](083-public-creator-project-cli-vertical.md), [ADR-084](084-public-creator-run-and-project-package-vertical.md), [ADR-085](085-public-creator-project-inspect-and-diff-vertical.md) |
| Supersedes | Narrowly supersedes ADR-085's statement that a reusable template and cold-authoring exercise are unimplemented. Existing creator commands, reports, authoring/package formats and inspect/diff semantics are unchanged. |
| Superseded by | Narrowly [ADR-087](087-public-creator-runtime-scenario-and-prefix-minimization.md) for the later public scenario validate/run/minimize boundary; template and cold-authoring semantics remain current. |

## Context

R6a–R6c prove that an already-authored independent project can validate, cook,
run, package and be inspected. A new creator still had to copy a repository
fixture and manually repair project identity before reaching that workflow.
That is not a cold start and makes the first successful project dependent on
undocumented repository knowledge.

Project Authoring V7 also does not admit a useful empty skeleton. Its current
direct cooker expects a coherent RPG definition set and navigation requires at
least two chunks. The smallest honest template must therefore be a complete
valid project rather than an output that fails its first validation.

## Decision

### Current command

R6d adds exactly one operation:

```text
next project create \
  --template rpg-starter \
  --project-id <lowercase-namespaced-id> \
  --output <absent-project-directory>
```

`rpg-starter` is the only current template. The project ID is explicit,
lowercase ASCII, 3–128 bytes, dot-namespaced, and contains only letters,
digits, dots and hyphens. Create accepts no existing output, merge mode,
identity override for existing authoring, arbitrary template path or remote
source.

Success uses the unchanged Creator Command Report V1 with command
`project.create`, exact cooked project hashes/counts and publication state
`created-and-validated`. Failure is one path-free report. Unknown templates or
invalid IDs use `CREATOR_CLI_ARGUMENT_INVALID`; an occupied/unsafe destination
uses `CREATOR_TEMPLATE_OUTPUT_INVALID`; an embedded-template/storage failure
uses `CREATOR_TEMPLATE_INVALID`.

### RPG Starter V1

The built-in current-only starter contains one character/NPC, item, ability,
dialogue, quest, relationship and interaction, three world chunks, minimal
render content and the current navigation/population/cognition/activity roots.
Its project-owned schema/value IDs are deterministically rewritten into the
requested namespace. Shared CC0 source identity, source bytes, hash, NOTICE and
license-manifest closure remain the identity of the common template source.

The resulting directory contains exactly:

```text
project.authoring.json
NOTICE
README.md
assets/original-source.txt
```

This is ordinary Project Authoring V7 input. Validate/cook/run/package/inspect
contain no template-specific branch after creation, and a creator may edit the
generated source with the documented current workflow.

### Atomicity and authority

Create resolves an existing real parent, requires an absent leaf, renders only
the fixed file set into a private sibling staging directory, then invokes the
production file-backed loader and cooker. Only a completely valid cooked
candidate is renamed to the destination. Failure or a publication race removes
staging and leaves caller paths untouched.

The template is a tooling input, not runtime/save/replay authority and not a
new authoring schema. It cannot overwrite a project, migrate an old format,
scan ambient files, mutate gameplay or grant capabilities.

## Product impact

A creator can go from a clean checkout and an empty parent directory to a
runnable independent RPG project with one command. The generated examples map
directly to the R6 cold-authoring concepts—ability, quest/NPC and streamed
chunk—and every subsequent operation uses the already-public production path.
This completes R6d and further reduces B-09. ADR-087 subsequently closes the
bounded scenario validate/run/prefix-minimize increment; replay/domain
inspectors and the broader SDK remain open.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| focused creator CLI | Create the same project ID in two isolated roots | Byte-identical four-file trees and deterministic path-free reports | Fail creation and remove staging |
| cold lifecycle | Validate, one-tick run, package, package-run and authoring→package diff | All pass through ordinary public paths; diff is empty | Keep R6d incomplete |
| authored closure | Inspect generated source | One character, ability and quest plus three chunks; ten neutral records | Reject embedded template during create |
| negative matrix | Unknown template, invalid ID, existing file/directory/symlink output | Stable failure and no changed caller bytes | Reject complete operation |
| `content-package` | Generate an independent project, then run/package/inspect it | Exact source/package identity, runtime proof and projection parity | Fail the governing check |
| `fast` / `host-check` | Formatting, strict lint, tests and boundaries | Existing creator operations remain compatible | Keep R6d incomplete |

## Considered alternatives

- **Document copying `creator-smoke`.** Rejected: the creator must understand
  fixture identity and manually perform unsafe substitutions.
- **Generate an empty manifest.** Rejected: it cannot pass current cooker and
  world-navigation invariants.
- **Accept arbitrary local/remote template paths.** Rejected: this would add an
  unbounded filesystem/network and supply-chain surface without an R6d need.
- **Merge into an existing directory.** Rejected: partial writes, collision
  semantics and rollback would turn a starter into a broad mutation API.
- **Add a new report family.** Rejected: create has the same exact cooked
  project facts as validate/cook, and Creator Command Report V1 already carries
  them without changing its shape.
- **Bundle scenarios, editors or replay inspection.** Rejected: each needs its
  own concrete lifecycle consumer and failure semantics.

## Consequences and next boundary

- The public creator surface has seven operations and still five report
  families.
- The repository's CC0 starter source is reused as one embedded deterministic
  template, avoiding a second drifting content fixture.
- R6 and B-09 remain open. ADR-087 subsequently adds public scenario
  validate/run/prefix-minimize; read-only replay first-divergence and bounded
  domain inspection are now the next increment.
- Linux is the active implementation host. Windows/THOTH remain
  `NotRun(WindowsHostDeferred)` under ADR-082.

## Supersession

Changing template identity/file set, project-ID grammar, overwrite policy,
validation-before-publication rule, diagnostics or report shape requires an
explicit successor. Additional templates require a concrete creator consumer
and equivalent cold-lifecycle evidence.
