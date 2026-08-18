# R6b creator run/package — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-18 |
| Task key | `r6b-creator-run-package` |
| Scope | Promote the smallest public creator runtime/distribution vertical: generic one-tick external-project startup, ordinary Application Session final save, exact current project package and packaged-byte rerun |
| Definition of done | A clean Linux checkout can deterministically run `projects/creator-smoke` from authoring and from an independently published package through the public `next` binary; both paths produce the same exact runtime/final-save proof without reference-game bootstrap; package output is reproducible, bounded, inventory/NOTICE-complete, current-only and fail-closed; affected product checks pass |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in contracts and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R6b is complete on the active Linux host. It closes
  only the run/package increment. R6 and B-09 remain open for diff/inspect,
  templates/scenarios and broader SDK documentation.
- **Runtime path:** `ApplicationCoordinator::run_project_headless` consumes the
  exact already activated project, derives deterministic bootstrap identity
  from its lock, activates project-authored world/RPG/service closure, executes
  exactly one production tick and closes through the ordinary session journal
  and SaveStore. It creates no reference-game aggregate or presentation.
- **Public commands:** `next project run --project <dir>`, `next project run
  --package <dir>` and `next project package --project <dir> --output
  <new-dir>`. R6a validate/cook commands and Creator Command Report V1 remain
  unchanged.
- **Output contracts:** run and package use separately versioned Creator Run
  Report V1 and Creator Package Report V1. Each invocation emits exactly one
  path-free JSON object. Runtime proof binds the exact project lock, state,
  archive/index/ledger, close receipt and final save generation.
- **Package boundary:** Creator Project Package V1 is a current-only content
  envelope for a compatible `next` runtime, not a standalone OS package. Its
  canonical manifest binds sorted exact payload inventory, project identity,
  run proof and nonempty root `NOTICE`.
- **Publication/safety:** package output must not exist. A private sibling
  staging directory is built, validated and rerun before one final rename.
  Links, traversal, unknown/tampered bytes, missing/linked NOTICE, unsupported
  format/version and activation/run mismatch reject before success.
- **Host boundary:** Linux is active under ADR-082. Windows/THOTH and paired
  native shipping evidence remain deferred to explicit pre-R7 bring-up.

## Locked acceptance matrix

| Case | Expected result |
| --- | --- |
| Repeat authoring run | Same Creator Run Report V1; `PASS`, one tick, exact final-save proof |
| Build package twice in fresh destinations | Same Creator Package Report V1 and canonical manifest bytes |
| Run either package | Exact project and runtime/final-save proof equals authoring/package report |
| Existing or symlink package destination | `CREATOR_PACKAGE_OUTPUT_INVALID`; caller content untouched |
| Changed/unknown inventory byte | `CREATOR_PACKAGE_INVALID`; no runtime launch |
| Missing, empty or linked root NOTICE | `CREATOR_PACKAGE_NOTICE_INVALID` during build or invalid inventory during run; no published output |
| Unsupported package format/version | `UNSUPPORTED_CREATOR_PACKAGE_FORMAT` before nested activation |
| Malformed manifest, link, traversal, size/count overflow | Stable package failure before launch/publication |
| Existing reference game/session/persistence behavior | `play` and `persistence-replay` remain `PASS` |
| Active Linux shared launch/session path | `platform` remains `PASS`; Windows stays deferred |

## Explicit non-goals

- Do not claim complete R6 or close B-09 from run/package alone.
- Do not make one tick an interactive gameplay or scenario-authoring promise.
- Do not expose temporary creator state as a resumable public session.
- Do not add a creator-only Runtime, reference-role shim or privileged mutation.
- Do not call Creator Project Package V1 a native Linux/Windows game bundle.
- Do not add tolerant decode, implicit migration, overwrite or merge behavior.
- Do not run deferred Windows/THOTH checks.

## Verification closure

| Check | Result | Boundary proved |
| --- | --- | --- |
| `cargo run --locked -q -p next_cli -- project package --project projects/creator-smoke --output <fresh>` plus `project run --package <fresh>` | `PASS`: format `nextengine.creator-project-package.v1`; manifest `17a1aa6820a692b6b93b5853900796b675f6fdfbd966a654416d82798b82b9c1`; 27 payload files; 93,188 total bytes; required `NOTICE`; packaged run exactly equals recorded proof | A recipient can validate and run the exact packaged ContentStore without authoring sources or reference-game bootstrap. |
| `cargo test --locked -p next_cli` | `PASS`: 2 unit and 11 integration tests | Four-operation parser/report contract, deterministic run/package, two-package reproducibility, actual packaged-byte launch, tamper/unsupported-version rejection, bounded inputs and Linux symlink/output/NOTICE confinement pass. |
| `cargo test --locked -p next_application` | `PASS`: 25 tests | Generic external-project one-tick startup/final close works and existing interactive worker/session/save/load behavior remains intact. |
| strict Clippy for `next_application`, `next_cli`, `next_verification` with all targets | `PASS`, `-D warnings` | All affected production/tool/verification surfaces are warning-free. |
| `cargo run --locked -q -p xtask -- content-package` | `PASS`: reference 123 entries/64 chunks; creator 18 entries/3 chunks; creator lock `008ca7acbee5a934aca9f228b1fb41038843f29dee7af26dcb5c142670b1c339` | The governing content check runs authoring, builds/reopens the package and matches exact creator runtime proofs. |
| `cargo run --locked -q -p xtask -- play` | `PASS`: 32 ticks, 52 events, 23 RPG events, final save published | Existing reference gameplay and full physical-animation save route remain unchanged. |
| `cargo run --locked -q -p xtask -- persistence-replay` | `PASS`: 20 ticks, 2 generations, final state `62013d24b7f4fba5463416e288ee764666fb5232c7378e354f76ce70ea601da6` | Existing save/load/replay and owner closure remain valid after the optional creator physical-animation segment. |
| `cargo run --locked -p xtask --features desktop-sdl-ash -- platform` | `PASS`: portable contract and real `sdl_ash_candidate` both `PASS`; 9 rendered objects | Shared launch/session changes remain valid on the active Linux SDL3/Vulkan host with the connected display. |
| `cargo run --locked -p xtask -- host-check` | `PASS` on `x86_64-unknown-linux-gnu`, Rust `1.97.1` | Workspace formatting/build, strict Clippy, all unit/integration/doc tests and repository boundary scan remain green. |

The creator runtime/package exact proof is:

- project lock: `008ca7acbee5a934aca9f228b1fb41038843f29dee7af26dcb5c142670b1c339`;
- session: `04c4743942da9fbf98e90d9d32649279`;
- close receipt: `dabe3e148c354ba213121107499cef77942e083b33bd8caae260080a7afcc409`;
- final save generation: `36dc39814ae66188486c772a338ee22dd175ed4b9d6b37078b3950b9c0b974c5`;
- authoritative state: `bb8f7423ff05449ee0b4bbc85dceb82ec0841262447de75d22738bc5d9e838ff`;
- command archive: `5044cac0e14290192e8f9038b2684b155442bbd4a556c056284006f43355549e`;
- command identity index: `dfdf00c3d1956df39f4ee92415ca2e56ef75e0f97ff79ee531a3a87d1b72dd62`;
- command ledger: `89b765ddbc577af23d558c4c4419a5a2f0334d8e2de79555f317d033fda66638`;
- one tick, zero commands/events/RPG events, authoritative revision 1.

Performance was `NOT_RUN (NoEstablishedHotPathChanged)`: the bounded CLI
startup/package path is not an admitted measured hot path. Windows/THOTH and
paired native evidence are `NOT_RUN (WindowsHostDeferred)` under ADR-082.

## Decisions

### D-001 — Add a project-neutral application cut, not a second game driver

- **Decision:** The generic path bootstraps only owner data available from the
  activated project and executes exactly one production tick with null
  presentation.
- **Reason:** Requiring creator data to mimic reference-alpha roles would keep
  the hidden fixture coupling that R6b exists to remove.
- **Consequence:** Runtime construction, schedule, owner closure and final save
  are proven; multi-tick scenarios and interactive player composition remain
  later consumers.

### D-002 — Reuse ordinary close with an optional physical-animation segment

- **Decision:** A prepared application run may omit the reference-only
  physical-animation save segment while retaining the same cognition-only
  authoritative save constructor and close journal.
- **Reason:** The external project has no reference physical-animation driver,
  but still must cross the production persistence boundary rather than report a
  synthetic run success.
- **Consequence:** Existing reference runs preserve their full save shape;
  generic creator runs publish an owner-complete final save for their actual
  runtime closure.

### D-003 — Publish exact content, not authoring source or a native binary

- **Decision:** Package one immutable ContentStore generation plus canonical
  outer inventory, root NOTICE and run proof into a fresh directory.
- **Reason:** Recipients need the exact tested bytes without committing R6b to
  target ABI/renderer/launcher distribution or recooking source locally.
- **Consequence:** A compatible `next` can validate and run the package;
  standalone target packaging remains R7.

## Do not retry

- Do not broaden R6b into diff/inspect, templates, scenarios/minimization,
  replay inspection or native distribution.
- Do not restore the `ReferenceGameDriverV2` path for an external creator
  project or fabricate required reference IDs in authoring.
- Do not accept existing package destinations, symbolic links, extra files or
  an unverified recorded run proof.
- Do not schedule Windows/THOTH evidence while ADR-082 keeps Linux as the
  active development host.
