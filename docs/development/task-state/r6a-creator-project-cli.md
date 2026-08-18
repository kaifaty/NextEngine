# R6a creator project CLI — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-18 |
| Task key | `r6a-creator-project-cli` |
| Scope | Promote the smallest public creator vertical: stable non-interactive project validate/cook JSON commands plus one independent project authored without engine-crate edits |
| Definition of done | A clean checkout can validate and atomically cook `projects/creator-smoke` through the public `next` binary, reopen the published generation through the production loader, reject malformed/escaped input with stable path-free diagnostics, preserve an existing valid generation on failure, and run the second project in the normal regression contour |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in contracts and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Successor checkpoint:** [R6b creator run/package](r6b-creator-run-package.md)
  is now complete. The remaining statements in this file preserve the narrower
  R6a acceptance boundary and its historical handoff.
- **Current conclusion:** R6a is complete as the first bounded R6 increment. It promotes
  only `next project validate` and `next project cook`; project diff,
  inspectors, run/package/replay commands, templates and the wider SDK remain
  later R6 increments.
- **Production path:** both commands open only the authoritative
  `project.authoring.json` project ID and use the existing file-backed V7
  loader and V7 cooker. `cook` additionally publishes through `ContentStore`
  and reopens through `activate_project`; no creator-only bootstrap exists.
- **Output contract:** stdout is exactly one Creator Command Report V1 JSON
  object. Success carries only stable IDs, hashes and bounded counts. Failure
  carries a stable code/subsystem/message key and never a raw local path.
- **Safety boundary:** relative source paths must remain beneath the selected
  project root even through symbolic links. Cook output must be new, empty or
  recognizably ContentStore-owned; arbitrary directories and symlinks fail
  closed.
- **Format policy:** Project Authoring V7 and report V1 remain current-only
  pre-v1 formats under ADR-046. This increment introduces no migration promise.
- **Host boundary:** Linux is active under ADR-082. Windows/THOTH evidence is
  deferred and is not part of R6a completion.

## Locked acceptance matrix

| Case | Expected result |
| --- | --- |
| Validate `projects/creator-smoke` | `PASS`, deterministic hashes/counts, no output publication |
| Cook `projects/creator-smoke` to a fresh directory | `PASS`, atomic `CURRENT`, production activation equals reported lock |
| Repeat the same cook | Same JSON and generation; no duplicate or mutable hidden state |
| Retired authoring format | Nonzero plus `UNSUPPORTED_PROJECT_AUTHORING_FORMAT`; no output change |
| Escaped relative source through `..` or symlink | Nonzero plus `CONTENT_SOURCE_PATH_INVALID`; no source bytes/path leak |
| Individual source larger than 16 MiB | Nonzero plus `CONTENT_SOURCE_LIMIT_EXCEEDED` before nested decode |
| Output symlink or unrelated nonempty directory | Nonzero plus `CREATOR_OUTPUT_INVALID`; unrelated content untouched |
| Existing valid generation followed by invalid input | Prior `CURRENT` and complete generation remain byte-identical |

## Explicit non-goals

- Do not claim complete R6 or close B-09 from validate/cook alone.
- Do not add a graphical editor, MCP surface or mutable inspector.
- Do not add project-ID override, implicit migration or alpha legacy decoder.
- Do not make `next` a privileged gameplay mutation route.
- Do not couple creator tooling to the reference-game crate.
- Do not run deferred Windows/THOTH checks.

## Verification closure

| Check | Result | Boundary proved |
| --- | --- | --- |
| `cargo run --locked -q -p next_cli -- project validate --project projects/creator-smoke` | `PASS`: project `org.nextengine.creator-smoke@1`; 10 neutral records, 18 content entries, 16 roots, 3 render assets, 3 chunks and 24 publication files; no bytes written | The documented public loader/cooker path is deterministic and side-effect-free for validation. |
| `cargo run --locked -q -p next_cli -- project cook --project projects/creator-smoke --output target/creator-smoke-r6a-check` | `PASS`: the same counts and hashes; `published-and-activated`; repeated cook retains one immutable generation | Public cook publishes atomically through `ContentStore` and the production loader reopens the exact reported project lock. |
| `cargo test --locked -p next_cli` | `PASS`: 2 unit tests and 6 integration tests | Exact argument/report shape, deterministic validate/cook, idempotence, production activation, retired-format preservation, bounded reads, source symlink escape and output confinement are covered. |
| `cargo run --locked -q -p xtask -- content-package` | `PASS`: reference project remains 123 entries/64 chunks with lock `2c5b466d95ed6e6cb636a7850e984a98d4e444627f6a04b9f2b71be670f689b4`; creator project activates as 18 entries/3 chunks with lock `008ca7acbee5a934aca9f228b1fb41038843f29dee7af26dcb5c142670b1c339` | The independent data-only project is in the normal production content regression contour without a reference-game constructor. |
| Scoped format and strict Clippy | `PASS` for `next_cli`, `next_project`, `next_verification` and all affected targets | The new public workspace member and authoring hardening are formatted and warning-free. |
| Full `cargo run --locked -p xtask -- host-check` | `PASS` on `x86_64-unknown-linux-gnu`, Rust `1.97.1` | Workspace build, strict Clippy, all Rust/doc tests and repository boundary checks remain green. |

The creator project's exact roots are:

- authoring: `b80e7d162b5f195c94ef9e62b831b4f9f4bfcbae3783587c81ce96b495df720b`;
- project lock: `008ca7acbee5a934aca9f228b1fb41038843f29dee7af26dcb5c142670b1c339`;
- schema registry: `73d81f227c760b2372a5f69f85624f89d8ca5c01c674a530c7aa8ed0d87214e1`;
- content manifest: `f63c7b0353c172135356c8f46b9970924ec9eac7be40c87db9c3d022f1f02e9d`;
- world partition: `811a6c66f41066b16f9e639826cf82ffb85a5ef2d2f2b3a46cbb1ad864b99dc7`;
- mechanics lock: `e2ef7d2e3142630aafcabd240769a9822140415e093c3652614b83852d1fb1df`.

`play`, persistence/replay, Linux platform/render evidence and Performance V5
were not rerun independently because R6a changes creator tooling, file-backed
authoring input and content-package regression coverage, not runtime authority,
save state, a backend or an established measured hot path. Their shared build,
test and boundary surfaces remain covered by `host-check`. Windows and THOTH
are intentionally `NOT_RUN (WindowsHostDeferred)` under ADR-082.

## Decisions

### D-001 — Promote only the first two creator commands

- **Decision:** The public binary accepts exactly `project validate` and
  `project cook`, with explicit project/output paths and no project-ID
  override.
- **Reason:** These commands close a useful external authoring loop over the
  existing production cooker without prematurely freezing run/package,
  inspector, template or SDK contracts.
- **Consequence:** R6 remains `IN_PROGRESS`, B-09 remains open and R6b owns the
  next creator run/package vertical.

### D-002 — Prove activation before switching the requested output

- **Decision:** Cook assembles and activates the complete publication in an
  isolated temporary `ContentStore` before publishing the same immutable files
  to the caller's confined output store.
- **Reason:** A semantically invalid generation must not become `CURRENT` in an
  already valid creator output merely because its file hashes are internally
  consistent.
- **Consequence:** Authoring, cook and activation failures leave the prior
  generation unchanged; only a post-preflight external filesystem race can
  interrupt target publication, which still follows ContentStore's atomic
  switch protocol.

### D-003 — Make the acceptance project independent and data-only

- **Decision:** `projects/creator-smoke` declares its own authoring manifest,
  source provenance and three-chunk world entirely as checked-in data.
- **Reason:** Reusing the reference-game constructor would prove an internal
  fixture path, not the creator workflow a product user receives.
- **Consequence:** `content-package` now continuously activates two independent
  projects through the same loader/cooker/publication contracts.

## Do not retry

- Do not broaden R6a into project run/package, diff, inspectors, templates or
  a general SDK surface; those require their own consumer-backed increments.
- Do not add a creator-only project constructor, ID override or hidden legacy
  decoder to make malformed authoring data pass.
- Do not write into a symlink or unrelated nonempty directory, or expose raw
  host paths through Creator Command Report diagnostics.
- Do not schedule Windows/THOTH evidence while ADR-082 keeps Linux as the
  active development host.
