# R6e creator runtime scenario — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-20 |
| Task key | `r6e-creator-runtime-scenario` |
| Scope | Add one bounded current-only public creator scenario manifest and `validate`/`run`/`minimize` lifecycle over an exact authoring project or creator package |
| Definition of done | A path-free public scenario validates against one exact project closure, runs an ordered bounded tick-action prefix through the production headless Application Session and final save, minimizes a reproducible assertion failure without weakening the assertion, and rejects malformed/retired/unsafe input or output without partial publication |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in contracts and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R6e is complete on the active Linux host. Public
  `next scenario validate|run|minimize` accepts one bounded current-only
  scenario bound to an exact authoring project or creator package.
- **Execution:** the tracked scenario performs three ordinary Runtime + World
  Routine/Population transactions through an isolated Application Session and
  ordinary final save. It produces three ticks, two events, revision five and
  exact state/ledger/save roots identically from authoring and packaged bytes.
- **Minimization:** an intentionally wrong three-action tick assertion reduces
  to the shortest one-action reproducing prefix. The assertion list and exact
  project closure are unchanged; the staged result is decoded and rerun before
  fresh-file publication.
- **Compatibility:** existing `next project run` remains one tick with its prior
  report contract and proof. No arbitrary commands, faults, capture, replay
  artifact, reference-game fixture constructor or mutable scenario hook was
  introduced.
- **Host boundary:** native Linux and the hardware-enabled SDL/ash platform
  check pass. Windows/THOTH remain deferred under ADR-082.
- **Next boundary:** R6 and B-09 remain open. R6f is read-only replay
  first-divergence and bounded domain inspection; broader scenario actions and
  the remaining SDK workflow stay later consumer-driven increments.

## Locked boundary

- Commands are `next scenario validate|run|minimize --scenario <file>` with
  exactly one explicit `--project` or `--package`; minimize additionally
  requires an absent `--output` file.
- `nextengine.creator-runtime-scenario.v1` binds the full exact public project
  identity, one to 256 ordered `tick` actions, a declared tick budget and one
  to 32 sorted exact read-only runtime assertions.
- Existing `next project run` remains exactly one tick. R6e adds a separately
  bounded multi-tick application entry point used only by the scenario
  consumer; it uses the same bootstrap, runtime tick, owner validation and
  save-on-close path.
- Minimization searches the shortest action prefix preserving the same stable
  assertion code, category, assertion identity, probe and exact project
  closure. It does not edit assertions, world state or project/package bytes.
- Scenario input is a bounded regular non-link file. Minimized output is
  validated and rerun in private sibling staging, then published only to an
  absent destination.
- Reports are current-only, strict, path-free and independently versioned from
  existing creator report families.

## Acceptance matrix

| Case | Expected result |
| --- | --- |
| Validate tracked creator scenario against authoring and package | Same exact scenario/project identities; no scenario runtime execution |
| Run tracked scenario repeatedly from authoring and package | Exact runtime/final-save proof and all assertions pass |
| Run scenario with a wrong exact assertion | Stable typed failure with assertion/action/tick, expected/actual and complete project identity |
| Minimize a multi-action failing scenario | Shortest reproducing prefix is rerun, assertion is unchanged and fresh output is published atomically |
| Passing scenario sent to minimize | `CREATOR_SCENARIO_NOT_REPRODUCED`; no output |
| Malformed, retired, duplicate/unsorted or over-budget scenario | Reject before scenario world creation |
| Scenario link or existing/link output | Typed failure; source/destination remains untouched |
| Existing creator commands and one-tick run | Reports and semantics unchanged |

## Explicit non-goals

- No arbitrary command payloads, fault adapters, capture, GUI, MCP or editor
  mutation in this first scenario consumer.
- No assertion minimization, assertion weakening or substitution of a
  different failure.
- No public replay artifact or replay/domain inspector; those remain R6f.
- No authoring/package migration or tolerant pre-v1 reader.
- No Windows/THOTH execution while ADR-082 keeps Linux as active host.

## Verification closure

- `cargo test --locked -p next_cli -p next_application --all-targets` — `PASS`:
  26 application tests, three CLI unit tests, 15 creator-project integration
  tests and four creator-scenario integration tests.
- `cargo test --locked -p next_cli -p next_application -p next_verification
  --all-targets` — `PASS`: focused application/CLI plus the complete
  verification library and its headless-replay, physics-collision and RPG-slice
  integration suites.
- `cargo clippy --locked -p next_cli -p next_application -p next_verification
  --all-targets -- -D warnings` — `PASS`.
- `cargo run --locked -q -p xtask -- content-package` — `PASS`: reference
  123 records/64 chunks; creator 18 records/3 chunks; creator composition lock
  `008ca7acbee5a934aca9f228b1fb41038843f29dee7af26dcb5c142670b1c339`;
  tracked scenario validates/runs from authoring/package with identical proof
  and minimizes the governed failure from three actions to one.
- `cargo run --locked -p xtask -- host-check` — `PASS` on
  `x86_64-unknown-linux-gnu`, Rust `1.97.1`; workspace strict clippy, tests,
  doctests and repository boundary scan are green.
- `cargo run --locked -q -p xtask --features desktop-sdl-ash -- platform` —
  `PASS`: portable contract and hardware-enabled SDL/ash candidate both pass,
  with nine rendered objects and the unchanged platform state/ledger roots.
- `cargo run --locked -q -p xtask -- play` and `cargo run --locked -q -p
  xtask -- persistence-replay` — `PASS`: existing 32-tick gameplay close/save
  proof and 20-tick/two-generation replay closure remain valid.
- Direct `next scenario validate/run` — `PASS`: path-free Scenario Report V1,
  scenario hash
  `b2c0c7c0c2b981c51659bb00f5b1dc5ce4c13bf068eb9311a44bc618f3276d85`,
  state root
  `0cdb74d69029dfac963be0da11a44c3388332f12878d4a4c6974bc6bb4c3f9b6`,
  ledger hash
  `34727b83c31ff6edbdd72074ee349ff56deb633a10d9bd033ab00d97bc043ca2`
  and final-save hash
  `d1fbfdc1cadddfbd00d1bd00074498966421735a1791ee4fcc8b9fed3a2f0315`.
- `cargo fmt --all -- --check`, `cargo run --locked -q -p xtask --
  boundary-scan`, `git diff --check` and direct validation of changed local
  Markdown links — `PASS`.
- Performance — `NOT_RUN / NoEstablishedHotPathChanged`: R6e adds a bounded
  tooling-only scenario consumer over existing per-tick transactions, not a
  live shipping loop or new performance budget.
- Windows/THOTH — `NOT_RUN / WindowsHostDeferred`: no Windows execution is
  scheduled while ADR-082 keeps Linux as the active development host.

## Decisions

### D-001 — Separate bounded multi-tick consumer

- **Decision:** keep the public one-tick project run unchanged and add a
  scenario-only 1–256 tick application entry point.
- **Reason:** a regression scenario needs ordered state evolution, while
  changing the established project-startup proof would silently rewrite R6b.
- **Consequence:** old commands and reports retain their bytes; the new route
  installs only the engine-owned world-service authority needed by its ticks.

### D-002 — Exact final-proof assertions

- **Decision:** V1 assertions observe only public final tick/event/revision,
  state, command-ledger and final-save values.
- **Reason:** these values are deterministic, path-free and already cross the
  production authority boundaries without exposing private owner storage.
- **Consequence:** command/fault/capture/domain-probe breadth requires a later
  concrete consumer rather than speculative fields in this format.

### D-003 — Minimize actions, never the oracle

- **Decision:** search action prefixes shortest-first while preserving the same
  assertion code/category/ID/probe and exact project closure.
- **Reason:** removing or weakening assertions can manufacture a smaller file
  that no longer represents the original failure.
- **Consequence:** the current minimizer finds the earliest reproducing tick
  prefix; arbitrary subset/delta minimization waits for future action kinds.

### D-004 — Validate, rerun and publish only to an absent file

- **Decision:** serialize the candidate into a private sibling file, load and
  rerun it, then publish with a no-overwrite hard-link operation.
- **Reason:** a minimizer must not replace caller data or report success for an
  unvalidated artifact; a publication race must fail closed.
- **Consequence:** retries require another absent output or explicit caller
  cleanup outside the command.
