# R6f replay/domain inspection — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-20 |
| Task key | `r6f-replay-domain-inspection` |
| Scope | Add current-only public Replay V10 validation, production verification with first-divergence reporting, and one-tick bounded domain inspection |
| Definition of done | A creator can validate an exact-project Replay V10, replay it through the production application/runtime path, inspect one declared tick without private snapshot disclosure, and receive the first divergent tick/stage/owner for a deterministic mismatch |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in contracts and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R6f is complete on the active Linux host. The public
  read-only consumer uses the existing production Replay V10 decoder and
  runner; verification remains a caller, never a production dependency.
- **Public commands:** `next replay validate` and `next replay inspect`, each
  with one bounded regular Replay V10 file and exactly one authoring project,
  creator package or cooked content store. Inspect additionally requires one
  exact tick and one of `runtime`, `world-services`, `physics` or `owners`.
- **Failure identity:** deterministic mismatch is
  `NONDETERMINISTIC_RESULT` plus first tick, stable stage and owner. Invalid,
  retired or project-incompatible data fails before replay mutation.
- **Projection:** reports contain hashes, stable IDs and bounded counts from
  the recorded current Replay V10; no canonical owner bytes, paths, ECS state,
  backend handles or private store layout cross the tooling boundary.
- **Host boundary:** Linux is active under ADR-082. Windows/THOTH remain
  deferred.
- **Next boundary:** R6 and B-09 remain open. R6g closes the bounded SDK
  workflow and documentation surface without widening replay capture or
  inspection semantics.

## Locked boundary

- Replay input is canonical current `ReplayManifestV10`; no tool-specific
  replay schema, V9 projection, migration or tolerant reader is added.
- `validate` performs bounded source decode, current contract validation,
  exact project activation and compatibility binding, but executes no replay
  tick.
- `inspect` first completes the same validation, then runs the complete replay
  through `next_application::replay::run_replay_manifest_v10`. A report is
  emitted only after every recorded compare point passes.
- A successful inspection projects exactly one requested existing tick and
  one domain. This keeps report size independent of replay duration.
- Authoring input is cooked into an isolated temporary content store; creator
  packages retain their full revalidation/rerun rules; `--content-store`
  activates the immutable output of `next project cook` directly.

## Explicit non-goals

- No replay recording/editing, branching, counterfactual execution,
  minimization, capture, live inspector, GUI or MCP surface.
- No raw canonical bytes, full owner snapshots, arbitrary JSON queries or
  mutable domain probes.
- No completion claim for all R6 SDK/documentation breadth.
- No Windows/THOTH execution while ADR-082 keeps Linux as the active host.

## Acceptance matrix

| Case | Expected result |
| --- | --- |
| Validate generated Replay V10 against its exact cooked project | Current contract and full project compatibility pass without executing a replay tick |
| Inspect each declared domain at an existing tick | Full production replay passes, then one bounded path-free domain projection is emitted |
| Change the first compare-point state root | `NONDETERMINISTIC_RESULT` reports the first tick, `application-state-root` stage and `application` owner |
| Request an absent tick | Typed missing-tick failure; no partial report |
| Supply Replay V9 | Typed unsupported-format failure before project access |
| Bind Replay V10 to another project identity | Typed project-compatibility failure before replay execution |
| Supply a replay source link | Typed source failure without following the link |

## Verification closure

- `cargo test --locked -p next_cli -p next_application -p next_verification
  --all-targets` — `PASS`: 26 application tests, six CLI unit tests, 15
  creator-project integration tests, four creator-scenario integration tests,
  74 verification library tests (one report-only long-history test ignored),
  three headless-replay tests, one physics-collision test and five RPG-slice
  tests.
- Governed generated Replay V10 matrix in `persistence-replay` — `PASS`:
  validation, all four domain projections, absent tick, retired V9, exact
  first-divergence identity and foreign-project rejection run from the same
  scratch production content generation; no private canonical owner bytes are
  checked into source.
- `cargo run --locked -q -p xtask -- persistence-replay` — `PASS`: 20 ticks,
  two generations, final state root
  `62013d24b7f4fba5463416e288ee764666fb5232c7378e354f76ce70ea601da6`
  and final ledger hash
  `ad6234b7fbbb4d7fde733ac7c448bd2440297fe5aef46b039325b9c44084508b`.
- `cargo run --locked -q -p xtask -- content-package` — `PASS`: reference
  123 records/64 chunks and creator 18 records/3 chunks; creator composition
  lock
  `008ca7acbee5a934aca9f228b1fb41038843f29dee7af26dcb5c142670b1c339`.
- `cargo clippy --locked -p next_application -p next_cli -p
  next_verification --all-targets -- -D warnings` — `PASS`.
- `cargo run --locked -p xtask -- host-check` — `PASS` on the active Linux
  host; workspace strict clippy, tests, doctests and repository boundary scan
  are green.
- `cargo fmt --all -- --check`, `cargo run --locked -q -p xtask --
  boundary-scan`, `git diff --check` and direct validation of changed local
  Markdown links — `PASS`.
- Performance — `NOT_RUN / NoEstablishedHotPathChanged`: R6f adds bounded
  tooling projections and compatibility/error identity around the existing
  replay runner, not a new shipping hot path.
- Platform — `NOT_RUN / ScopeUnaffected`: no platform session, renderer or
  hardware-backend boundary changed.
- Windows/THOTH — `NOT_RUN / WindowsHostDeferred`: no Windows execution is
  scheduled while ADR-082 keeps Linux as the active development host.

## Decisions

### D-001 — One production Replay V10 authority

- **Decision:** accept only the existing current `ReplayManifestV10` and call
  the production application replay runner.
- **Reason:** a tooling-only replay format or verifier would create a second
  determinism authority and could disagree with runtime behavior.
- **Consequence:** retired Replay V9 is rejected explicitly; migration and
  tolerant readers remain outside R6f.

### D-002 — Replay completely before projecting once

- **Decision:** `inspect` verifies the complete manifest, then emits exactly
  one requested tick/domain projection.
- **Reason:** a locally valid tick must not conceal a later deterministic
  mismatch, while bounded output must not grow with replay duration.
- **Consequence:** successful reports prove the whole replay and disclose only
  the selected bounded view.

### D-003 — Bind replay to the activated project closure

- **Decision:** validate runtime, build, project, schema, content, mechanics
  and tick compatibility against the exact activated project before restore.
- **Reason:** internally self-consistent replay bytes are insufficient when
  evaluated against a different cooked project.
- **Consequence:** foreign project closure fails before replay mutation, and
  the governed persistence fixture now exercises the same exact check.

### D-004 — Generate the governed replay fixture in scratch storage

- **Decision:** construct the current production Replay V10 from the governed
  persistence run instead of checking canonical owner snapshots into source.
- **Reason:** the acceptance matrix needs real restore bytes but the public
  repository must not establish those private bytes as an inspection API.
- **Consequence:** ProductCheck retains exact positive and negative coverage
  without widening the public report or fixture surface.
