# R6c creator project inspect/diff — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-18 |
| Task key | `r6c-creator-project-inspect-diff` |
| Scope | Promote the smallest read-only creator observability vertical: one source-neutral immutable project projection, public authoring/package inspect and directional base→candidate diff |
| Definition of done | A clean Linux checkout can deterministically inspect `projects/creator-smoke` from authoring and a fully verified Creator Project Package V1, prove both projections equal, report one controlled record edit as a localized stable-ID change, reject invalid/tampered input path-free, and pass the affected fast/content-package checks without caller writes or gameplay mutation |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in contracts and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R6c is complete on the active Linux host. Public
  authoring/package inspect and directional diff, source-neutral projection,
  package revalidation, focused tests, governing content verification and the
  full workspace host gate are green. R6/B-09 remain open for templates and
  cold authoring, scenarios/minimization, replay/domain inspectors and the
  broader SDK.
- **Public commands:** `next project inspect --project|--package` and directional
  `next project diff` with exactly one `--base-project|--base-package` and one
  `--candidate-project|--candidate-package`.
- **Projection:** exact project/profile roots; current schemas; root assets;
  assets with stable identity/hash/schema/provenance/license facts; dependencies;
  world partition/chunks; mechanics packages and granted capabilities. It is
  sorted, path-free and contains no content properties/private storage/runtime
  objects.
- **Diff semantics:** compares source-neutral projections. A valid difference is
  `PASS`, exit zero and `different = true`; invalid input is `FAIL`, nonzero and
  produces no partial comparison.
- **Package boundary:** inspect/diff does not trust recorded package metadata.
  Every package operand passes exact inventory/NOTICE/activation validation and
  the ADR-084 run-proof rerun before projection.
- **Host boundary:** Linux is active under ADR-082. Windows/THOTH and paired
  release evidence remain deferred.

## Locked acceptance matrix

| Case | Expected result |
| --- | --- |
| Repeat authoring inspect | Byte-identical Creator Inspect Report V1 with 17 schemas, 16 roots, 18 assets, 11 dependencies, 10 neutral records, three chunks, two packages and three granted capabilities |
| Identical authoring bytes in another root | Exact same report; no input path appears |
| Same-input diff | `PASS`, `different = false`, every change count zero |
| Authoring versus its built package | Equal project/projection and empty diff after full package verification/rerun |
| One item-property edit | Deterministic `PASS`, one keyed asset change plus derived root/mechanics changes; schemas/dependencies/world chunks unchanged |
| Valid non-empty diff | Exit zero; downstream policy reads the `different` field |
| Tampered/unsupported package or malformed authoring | Stable typed nonzero failure, no paths or partial diff |
| Caller inputs | Byte-for-byte untouched; temporary package-run state removed |
| Existing validate/cook/run/package | Reports and behavior unchanged |

## Explicit non-goals

- Do not expose neutral property payloads, source spans/paths or private package
  inventory as the project projection.
- Do not add live ECS/gameplay/replay inspection, scenario actions, editor
  mutation, template generation, GUI or MCP.
- Do not turn diff presence into a CLI failure.
- Do not relax package revalidation or accept tolerant/legacy formats.
- Do not claim complete R6 or close B-09 from inspect/diff alone.
- Do not run deferred Windows/THOTH checks.

## Verification closure

- `cargo test --locked -p next_cli` — `PASS`: 2 unit and 13 integration tests.
- `cargo clippy --locked -p next_cli -p next_verification --all-targets -- -D
  warnings` — `PASS`.
- `cargo run --locked -q -p xtask -- content-package` — `PASS`: reference
  123 records/64 chunks; creator 18 records/3 chunks; creator composition lock
  `008ca7acbee5a934aca9f228b1fb41038843f29dee7af26dcb5c142670b1c339`;
  authoring/package projections and their empty diff agree.
- `cargo run --locked -p xtask -- host-check` — `PASS` on
  `x86_64-unknown-linux-gnu`, Rust `1.97.1`; the complete workspace, doctests and
  repository boundary scan are green.
- `cargo fmt --all -- --check`, `git diff --check` and direct validation of
  changed documentation links, paths and identifiers — `PASS`.
- Manual authoring inspect — `PASS`: exact bounded counts above.
- Manual same-input diff — `PASS`: `different = false`, all counts zero.
- Performance — `NOT_RUN / NoEstablishedHotPathChanged`: R6c is read-only
  tooling over an already validated immutable closure.
- Play, persistence and platform gates — `NOT_RUN / ScopeUnaffectedReadOnlyTooling`:
  no gameplay, save/replay or host/runtime behavior changed.
- Windows/THOTH — `NOT_RUN / WindowsHostDeferred`: ADR-082 keeps Linux as the
  active development host until explicit pre-R7 Windows bring-up.

## Decisions

### D-001 — Project composition projection, not storage dump

- **Decision:** Build one tool-owned immutable projection from the validated
  cooked/activated public project closure.
- **Reason:** ContentStore files are private and source/package layouts differ;
  stable IDs and exact roots are the actual creator-facing composition facts.
- **Consequence:** The package and its authoring source compare directly without
  exposing paths or freezing storage layout.

### D-002 — Directional keyed diff with success-on-difference

- **Decision:** Compare `base → candidate`, key collections by public identity,
  and reserve command failure for invalid operands or report construction.
- **Reason:** Added/removed semantics need direction, and a valid difference is
  useful requested data rather than a malformed command.
- **Consequence:** CI can choose its own policy from `different` and category
  counts while deterministic diagnostics retain ordinary failure semantics.

### D-003 — Verify packages before observing them

- **Decision:** Reuse full package validation, activation and recorded-run rerun
  before creating a package projection.
- **Reason:** A read-only tool must not make tampered bytes appear trustworthy by
  printing their self-declared manifest.
- **Consequence:** Package inspect is intentionally stronger than a cheap file
  listing and remains consistent with ADR-084 safety.

## Do not retry

- Do not broaden R6c into templates/cold-authoring, scenarios/minimization,
  replay inspectors or native distribution.
- Do not use paths, array indices or unordered iteration as diff identity.
- Do not add a second project model under `crates/contracts`; this projection is
  a bounded tool report over existing public immutable contracts.
- Do not schedule Windows/THOTH evidence while ADR-082 keeps Linux active.
