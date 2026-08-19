# R6d creator project template — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-19 |
| Task key | `r6d-creator-project-template` |
| Scope | Add one reusable current-only RPG starter and a public cold-authoring command that creates a new independent project without engine-crate edits |
| Definition of done | A clean Linux checkout can create an absent project directory from the built-in starter, validate/cook/run/package/inspect it through the existing public toolchain, prove deterministic path-free reports and empty authoring/package diff, and reject unsafe or existing destinations without changing them |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in contracts and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R6d is complete on the active Linux host.
  `next project create --template rpg-starter --project-id <namespaced-id>
  --output <absent-directory>` atomically creates and production-validates a
  complete independent Project Authoring V7 source. The starter is project
  source, not a new engine content model.
- **Starter closure:** one NPC/character, item, ability, dialogue, quest,
  relationship, interaction, three streamed chunks, minimal render content and
  the current navigation/population/cognition/activity catalogs. The current
  authoring contract cannot represent a useful empty project and requires at
  least two world chunks.
- **Publication:** render into a private sibling staging directory, validate and
  cook it with the production Project Authoring V7 path, then atomically rename
  to the still-absent destination. Failure removes staging and never overwrites
  caller data.
- **Host boundary:** Linux is active under ADR-082. Windows/THOTH remain
  deferred.
- **Next boundary:** R6 and B-09 remain open; R6e is public scenario
  validate/run/minimize. Replay/domain inspectors and broader SDK documentation
  remain later increments.

## Locked acceptance matrix

| Case | Expected result |
| --- | --- |
| Fresh output and valid namespaced project ID | `PASS`, one path-free Creator Command Report V1, complete project source appears atomically |
| Same project ID in two independent roots | Byte-identical generated files and deterministic validate/inspect results |
| Generated project lifecycle | Existing validate, cook, one-tick run, package, package run and inspect/diff all pass without engine edits |
| Starter authored content | Exactly one ability, one quest/NPC closure and three streamed chunks are present and independently namespaced |
| Existing file/directory/symlink destination | Stable typed failure; destination and target remain untouched |
| Invalid/unknown template or invalid project ID | Argument failure and no output directory |
| Existing creator-smoke workflow | Reports and behavior unchanged |

## Explicit non-goals

- Do not add a second authoring schema, tolerant decode or migration.
- Do not implement arbitrary filesystem templates, remote registries, plugins,
  editor mutation, GUI or MCP.
- Do not add scenario/minimization, replay/domain inspectors or native release
  distribution in R6d.
- Do not allow create to overwrite, merge into or rewrite an existing path.
- Do not close R6 or B-09 from this increment alone.
- Do not run deferred Windows/THOTH checks.

## Verification closure

- `cargo test --locked -p next_cli` — `PASS`: 2 unit and 15 integration tests.
- `cargo clippy --locked -p next_cli -p next_verification --all-targets -- -D
  warnings` — `PASS`.
- `cargo run --locked -q -p xtask -- content-package` — `PASS`: reference
  123 records/64 chunks; creator 18 records/3 chunks; creator composition lock
  `008ca7acbee5a934aca9f228b1fb41038843f29dee7af26dcb5c142670b1c339`;
  the generated namespaced project completes run/package/inspect parity.
- `cargo run --locked -p xtask -- host-check` — `PASS` on
  `x86_64-unknown-linux-gnu`, Rust `1.97.1`; full workspace tests, doctests and
  repository boundary scan are green.
- `cargo fmt --all -- --check`, strict focused clippy,
  `cargo run --locked -q -p xtask -- boundary-scan`, `git diff --check` and
  direct validation of changed links/paths/IDs — `PASS`.
- Manual cold create/validate/run — `PASS`: four generated files, ten neutral
  records, 16 roots, 18 entries and three chunks under the requested namespace.
- Performance — `NOT_RUN / NoEstablishedHotPathChanged`: R6d is bounded local
  tooling and project-source generation.
- Play/persistence/platform — `NOT_RUN / ScopeCoveredByGeneratedLifecycle`:
  generated authoring and package cross the existing one-tick Application
  Session/final-save path in focused and governing checks; no gameplay,
  save/replay schema, host or renderer behavior changed.
- Windows/THOTH — `NOT_RUN / WindowsHostDeferred`: ADR-082 keeps Linux as the
  active development host until explicit pre-R7 Windows bring-up.

## Decisions

### D-001 — A valid starter instead of an empty skeleton

- **Decision:** ship the smallest currently valid RPG project closure and
  namespace its authored IDs from the requested project ID.
- **Reason:** Project Authoring V7 and the direct cooker require related RPG
  definitions and at least two navigation chunks; an empty skeleton would fail
  its first public validation.
- **Consequence:** a new creator immediately has editable examples of the exact
  ability, quest/NPC and streamed-world concepts named by the R6 acceptance
  criterion.

### D-002 — Fresh-output atomic creation

- **Decision:** create only an absent leaf below an existing real directory,
  validate/cook in a private sibling staging directory, and publish with one
  rename.
- **Reason:** template generation must not become a broad filesystem mutation
  or leave a half-created project after validation/storage failure.
- **Consequence:** retries use another absent destination or explicitly remove
  the prior generated project outside this command.
