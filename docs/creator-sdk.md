# Next Engine Creator SDK beta

This guide is the canonical source-build workflow for the current creator beta.
It takes a clean checkout from an empty output directory to an independently
namespaced RPG project, a cooked content store and a self-verifying creator
package without changing an engine crate.

The SDK is pre-1.0 and current-only. It is useful for local projects and CI,
but it is not yet a compatibility promise for a shipped v1. Unsupported old
formats fail closed; the tools do not silently migrate or rewrite source.

## Supported boundary

The current public surface is deliberately small:

- Project Authoring V7 is the editable data source.
- `next project create|validate|cook|run|package|inspect|diff` owns the project
  lifecycle.
- Creator Runtime Scenario V1 provides bounded validate/run/minimize.
- production Replay V10 provides validate and one-tick domain inspection.
- Luau Package Manifest V1 and Wasm Plugin Manifest V1 are the current
  extension-host contracts, with checked-in source examples below.

Every `next` invocation emits exactly one versioned JSON report on stdout.
Progress and human diagnostics belong on stderr. Consume `status`, `command`,
stable diagnostic `code` and typed `details`; do not parse rendered messages or
depend on filesystem paths appearing in reports.

There is no current GUI, MCP authoring protocol, live mutable inspector,
project-local replay recorder or automatic alpha migration. These omissions do
not create hidden alternatives: JSON files and the commands in this guide are
the supported creator path.

## Prerequisites

- a clean Next Engine checkout;
- the repository-pinned Rust 1.97.1 toolchain;
- native Linux x86_64 for the active development workflow.

Linux x86_64 is the sole current v1 shipping target. Windows/THOTH execution is
outside the roadmap indefinitely under ADR-090; a Linux creator-beta pass is
still development evidence rather than a release-readiness claim.

Run every command below from the repository root. The source-build spelling is
`cargo run --locked -p next_cli --` followed by the documented `next`
arguments.

## 1. Create the cold starter

Choose an absent output directory and a lowercase dot-namespaced project ID:

```bash
cargo run --locked -p next_cli -- project create \
  --template rpg-starter \
  --project-id org.example.my-rpg \
  --output target/sdk-my-rpg
```

Creation is atomic. It writes `project.authoring.json`, `NOTICE`, `README.md`
and `assets/original-source.txt` only after the generated project loads and
cooks successfully. It never merges into or overwrites an existing path.

The starter is intentionally valid rather than empty. In one cold operation it
adds the minimum accepted RPG closure: one NPC/character, item, ability,
dialogue, quest, relationship and interaction plus three streamed chunks and
the required navigation/population/cognition/activity roots. Project-owned IDs
are rewritten under `org.example.my-rpg`.

## 2. Make a public file-backed edit

Open `target/sdk-my-rpg/project.authoring.json` in a text editor. The following
exercise personalizes each R6 cold-authoring concept while preserving the
starter's dependency closure:

| Record or field | Generated value | Example authored value |
| --- | --- | --- |
| character `nextengine.creator.role` | `org.example.my-rpg.character.outpost-keeper` | `org.example.my-rpg.character.ridge-warden` |
| ability `nextengine.ability.definition-id` | `org.example.my-rpg.ability.signal-strike` | `org.example.my-rpg.ability.ridge-pulse` |
| quest `nextengine.quest.entry-state` | `org.example.my-rpg.quest.available` | `org.example.my-rpg.quest.awaiting-signal` |
| interaction `nextengine.interaction.quest-source` | `org.example.my-rpg.quest.available` | `org.example.my-rpg.quest.awaiting-signal` |
| third partition `chunk_id` | `org.example.my-rpg.chunk.crossing` | `org.example.my-rpg.chunk.signal-tower` |
| same partition entry `region_id` | `org.example.my-rpg.region.crossing` | `org.example.my-rpg.region.signal-tower` |

Change the quest entry state and the interaction quest source together: the
interaction transition must still start at the quest's declared entry state.
The third chunk is safe to rename in this exercise because the starter's
population route references the first two chunks.

Keep project-owned schema/value IDs inside the project namespace. Asset IDs and
persistent record IDs are fixed-width hexadecimal identities; do not duplicate
or casually regenerate them. Keep every `source_span.relative_path` confined
to the project root and retain the CC0 source/NOTICE hash closure unless you
deliberately replace the source and update its provenance.

Project Authoring V7 currently models the starter's RPG definitions as one
coherent minimum set. This exercise customizes that set; it does not claim a
general graphical editor or unrestricted collection merge API.

## 3. Validate, cook and run authoring

Validate without writing output:

```bash
cargo run --locked -p next_cli -- project validate \
  --project target/sdk-my-rpg
```

Cook into an absent or recognizable content-store root and reopen the exact
published generation through production activation:

```bash
cargo run --locked -p next_cli -- project cook \
  --project target/sdk-my-rpg \
  --output target/sdk-my-rpg-cooked
```

Run one project-neutral production headless tick and ordinary final
save-on-close directly from authoring:

```bash
cargo run --locked -p next_cli -- project run \
  --project target/sdk-my-rpg
```

The `project_lock_sha256` reported by validate, cook and run must match for the
same source bytes. A later source edit is expected to change the authoring and
derived composition hashes.

## 4. Inspect and package

Inspect the immutable stable-ID projection:

```bash
cargo run --locked -p next_cli -- project inspect \
  --project target/sdk-my-rpg
```

Publish to an absent creator-package directory:

```bash
cargo run --locked -p next_cli -- project package \
  --project target/sdk-my-rpg \
  --output target/sdk-my-rpg-package
```

The package contains the exact current ContentStore, bounded inventory and a
required root `NOTICE`. It is a content envelope for a compatible `next`
runtime, not a standalone native application bundle.

Run and inspect the packaged bytes, then prove source/package parity:

```bash
cargo run --locked -p next_cli -- project run \
  --package target/sdk-my-rpg-package
cargo run --locked -p next_cli -- project inspect \
  --package target/sdk-my-rpg-package
cargo run --locked -p next_cli -- project diff \
  --base-project target/sdk-my-rpg \
  --candidate-package target/sdk-my-rpg-package
```

For a package built from those exact authoring bytes, diff returns `PASS` with
`different: false`. A valid source-to-source change also returns `PASS`, with
`different: true` and categorized stable-ID changes. Invalid or tampered input
produces no partial projection.

## Scenarios and replay

The tracked scenario is bound to the exact `creator-smoke` project closure. It
is the current example for three-tick assertions and failure-preserving prefix
minimization:

```bash
cargo run --locked -p next_cli -- scenario validate \
  --scenario projects/creator-smoke/scenarios/smoke.scenario.json \
  --project projects/creator-smoke
cargo run --locked -p next_cli -- scenario run \
  --scenario projects/creator-smoke/scenarios/smoke.scenario.json \
  --project projects/creator-smoke
```

Do not copy that file to another project without rebinding every exact project
hash and assertion. Scenario minimize requires a failing scenario and an
absent output file; it preserves the assertion and failure identity.

Replay tooling consumes an existing canonical Replay V10 from the exact
project/build closure:

```text
next replay validate --replay <replay-v10.json> --project <project-directory>
next replay inspect --replay <replay-v10.json> --project <project-directory> \
  --tick <existing-tick> --domain <runtime|world-services|physics|owners>
```

Inspect verifies the complete replay before returning one bounded tick/domain.
No public replay capture/record command is claimed by this beta.

## Data, Luau and Wasm extension examples

The data-first starter and `projects/creator-smoke` use the same cooker,
mechanics capabilities and validators as the reference project. The extension
host examples are ordinary source files rather than Rust string literals:

- `examples/creator-sdk/luau/scripted-melee.lua` — the governed Luau callback;
- `examples/creator-sdk/wasm/increment-component.wat` — the governed Wasm
  Component fixture;
- `crates/plugin-host/wit/v3/nextengine-extension.wit` — current WIT world V3.

`content-package` executes the Luau and Wasm examples through their current
manifest, capability, budget, effect proposal and common command-validation
paths. Ambient filesystem, network, clock/process APIs and Wasm WASI are not
granted. A trap, forged handle, incompatible API, forbidden capability or
quota overrun publishes no partial gameplay change.

The current extension boundary is source/API level. Project Authoring V7 does
not yet ingest an arbitrary project-local Luau/Wasm directory through `next
project`; adding that surface requires a concrete consumer and its own
fail-closed package/provenance design. Do not work around this by adding engine
crate branches or mutable runtime hooks.

## Failure and output safety

- Treat every output operand as caller-owned. `create`, `package` and scenario
  minimize require an absent destination; choose a fresh path for a retry.
- `validate`, `run`, `inspect`, `diff` and replay inspection are read-only with
  respect to project/package source.
- Cook and package stage privately and publish only complete validated output.
- Links, traversal, escaped project references, oversized input, missing
  NOTICE, unknown files and retired versions fail closed.
- Stable diagnostic codes are the automation contract. Preserve the failed
  input and choose a new output path after correcting the cause.

## Repository regression check

Run the governing creator/content check after changing authoring, packaging,
Luau/Wasm examples or this executable workflow:

```bash
cargo run --locked -p xtask -- content-package
```

It generates a fresh namespaced starter in scratch storage, applies the public
JSON edits from this guide, then requires create, validate, cook, run, package,
package-run, inspect and source/package diff to agree. It also executes the
tracked scenario and the governed Luau/Wasm paths. `host-check` is the broader
Linux handoff when a public contract or cross-cutting implementation changes.
