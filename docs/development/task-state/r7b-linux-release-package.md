# R7b Linux release package — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE / EXACT_LINUX_PASS` |
| Updated | 2026-08-21 |
| Task key | `r7b-linux-release-package` |
| Scope | Freeze the representative Linux release input, publish a reproducible native package, validate its ELF/runtime prerequisites and run copied `game`, `headless` and tools in an isolated clean-install environment |
| Definition of done | One exact clean Linux commit passes the versioned package manifest, reproducibility, ABI/dependency and isolated copied-root smoke matrix without relying on Windows or R7c timing authority |
| Authority | Working context only; ADR-090, SPEC-04, SPEC-12 and `docs/roadmap.md` remain normative |

## Resume in 60 seconds

- **Current conclusion:** R7b is complete on exact clean Linux commit
  `a5f896e94736c8d2c4aa70a81592a8e7e4507fdb`. Two independent 157-file V5
  package trees are byte/mode-identical, and the exact native gate records all
  eight checks `PASS` with `release_ready=true`.
- **Why:** Copied game/headless prove isolated runtime launch, copied public
  `next` proves the exact frozen authoring/cook boundary, and the manifest binds
  all three ELF files, source/project roots, runtime prerequisites and complete
  inventory without machine-local operational state.
- **Next action:** Start R7c as a new bounded task: accept the Linux release
  performance fingerprint and numeric policy before collecting baselines or
  hard-gate samples.
- **Current blocker:** None.
- **Do not retry:** Do not add a Windows package slot, inherit THOTH timing, or
  treat the R7a package smoke as proof of the broader R7b clean-install matrix.
- **Reconsider when:** A future Accepted contract changes the active Linux
  target, package contents or public creator validation boundary.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| R7a executable commit `1e88934e0d811d90dcf46f6b6f0f27a348a0d9b5` | `PASS / release_ready=true`, all eight checks and existing package/runtime/desktop smoke | Valid prerequisite and regression baseline; not sufficient by itself for R7b |
| ADR-090 R7b boundary and SPEC-04 `PACKAGE-01` | Native Linux only; reproducible package, versioned manifest, ELF/glibc/dependency audit and isolated copied-root smoke are required | Implementation must fail closed before publication and must not schedule Windows |
| `tools/xtask/src/package.rs`, `runtime.rs`, `smoke.rs` source audit | V4 exact inventory, direct ELF dependency/glibc audit and cleared HOME/state/temp smoke already cover copied game/headless | Preserve these controls and extend the same authority to the public tool |
| `PackageBinarySources` and `PackageBinariesV2` | Only `game` and `headless` are copied and bound | V5 must add `bin/next`, a frozen source root and an exact tool-run receipt |
| `cargo test -p xtask --lib package::tests::` | `22 passed; 0 failed` | V5 package/source/tool implementation and failure controls are focused-green |
| `cargo test -p xtask --bin xtask` | `44 passed; 0 failed` | Current command/native-gate scheduler integration remains green |
| Full `cargo test -p xtask --lib` before the top-level source fixture update | `107 passed; 1 expected fixture regression`; the missing mandatory `source` root was then repaired and its focused test passed | Not final evidence; retain as an implementation transition, not a release claim |
| First broad `cargo run -p xtask -- host-check` | All compile/clippy/workspace tests passed, then boundary scan rejected `tools/xtask/src/native_gate/tests.rs` at `1003 > 1000` lines | Negative evidence retained; split the package-manifest fixture into its own test module before retrying the complete gate |
| `cargo run -p xtask -- boundary-scan` after fixture split | `PASS`; affected files are now `936` and `70` lines | The broad-gate failure condition is resolved; exact native gate will rerun the complete host check |
| `cargo run -p xtask --features desktop-sdl-ash -- platform` | `PASS`, SDL/ash candidate `PASS`, 4 normalized events, 9 rendered objects | Active Linux desktop/platform prerequisite is healthy; no Windows execution attempted |
| Clean `22459f9…` first `v1-package` attempt | `FAIL`, no package published; copied `next project run` returned `PROJECT_RUNTIME_BOOTSTRAP_FAILED` | Exact negative evidence: do not retry this command/commit; narrow the tool proof to the public validation/cook boundary |
| Release `next project validate --project projects/reference-alpha` | `PASS`, exact authoring/project/content/schema/world/mechanics roots and bounded counts | Valid discriminator: copied public tool can prove the frozen final source without expanding application runtime semantics |
| Corrected focused package/schema/CLI tests and boundary scan | `22/22`, `1/1`, `44/44`, all `PASS` | Old `project.run` receipt is removed from executable package authority; proceed to a real copied-tool release probe |
| Dirty real `v1-package` correction probe | `PASS`, schema 2; copied game/headless/tool all `PASS`; authoring hash `bcc23ec1…`, 76 neutral records, 141 publication files | Corrected public-tool boundary works with the real release ELF in the isolated package; safe to freeze a clean evidence commit |
| Exact package A/B from `a5f896e…` | Both `PASS`; 157 files each; full-tree bytes and modes identical; manifest SHA-256 `4ece6d7c454869ebf6af3e58b532edb7c3aa572d28a9ef6fd3b77f54389c5097` | Reproducibility requirement is closed on one clean commit and target/toolchain |
| Exact native gate from `a5f896e…` | Schema 2, all eight checks `PASS`, `release_ready=true`; target-report SHA-256 `2a92866141e70a970c1dea29d8581eba98c273127bffa04571d8edcb9658fdc7` | R7b package/runtime/desktop evidence is accepted on the active Linux target |
| Exact native-gate package report/manifest | Report SHA-256 `7daefc1c…`; V5 manifest `4ece6d7c…`; copied tool `project.validate` PASS with authoring `bcc23ec1…`, 76 neutral records and 141 publication files | Bundle binds the same reproducible package and meaningful public-tool receipt inspected outside the aggregate |

## Decisions that still constrain the work

### D-001 — Extend the existing native package path

- **Observation:** R7a already binds the native package descriptor into the
  current Linux release report.
- **Evidence:** R7a task-state and schema-v2 target report at `1e88934…`.
- **Decision:** Prefer evolving `v1-package` and its native-gate consumer over
  creating a parallel release packager or a new global ProductCheck category.
- **Rejected alternatives:** A second package authority, a creator-package
  relabel, or an external shell-only acceptance procedure.
- **Consequences:** The final package descriptor and release report must bind
  the new manifest and copied-root results exactly.
- **Uncertainty:** None at the completed R7b boundary.
- **Reconsider when:** A future package authority cannot evolve through the
  same strict versioned native path.

### D-002 — Use an honest V5 manifest for the tools/source boundary

- **Observation:** Adding a mandatory public tool, its source input and its
  execution receipt changes the strict package wire format.
- **Evidence:** V4 denies unknown fields and requires only two binaries; silently
  adding optional fields would let old consumers accept an incomplete package.
- **Decision:** Replace the emitted/accepted native manifest with
  `PackageManifestV5`; retain old report structures only where dormant historic
  evidence still needs to decode.
- **Rejected alternatives:** An unversioned sidecar, optional V4 fields, or a
  shell-only tools smoke outside the package authority.
- **Consequences:** SPEC-04, current `v1-package` report parsing and Linux native
  gate bindings must advance together.
- **Uncertainty:** None at the manifest boundary.
- **Reconsider when:** A normative consumer requires V4 compatibility for a
  shipped artifact; none exists in the current Linux-only development scope.

### D-003 — Advance the current package command report to schema 2

- **Observation:** The schema-1 `PackageDetailsV1` report can name only
  game/headless and would hide the new mandatory tools receipt.
- **Evidence:** The V5 manifest hash binds all bytes, but release evidence also
  needs a directly inspectable tool hash/status/authoring/root/count tuple.
- **Decision:** Current Linux `v1-package` emits strict
  `CommandReportV2<PackageDetailsV2>`; legacy schema 1 remains only in the
  dormant historical target decoder.
- **Rejected alternatives:** Keep emitting an incomplete current report or add
  optional fields to the strict schema-1 wire.
- **Consequences:** Linux native-gate parsing requires schema 2 and binds its
  existing target summary to the V5 manifest hash.
- **Uncertainty:** None.
- **Reconsider when:** A future package-report schema replaces V2 explicitly.

### D-004 — Tool clean-install proof is public project validation

- **Observation:** The final product reference project launches through the
  dedicated game/headless bootstrap; current creator `project run` intentionally
  covers independent generic projects and rejects this product-specific source.
- **Evidence:** Exact package attempt and direct debug/release invocations all
  return `PROJECT_RUNTIME_BOOTSTRAP_FAILED`, while the same release tool returns
  a strict `project.validate` PASS with the exact frozen roots.
- **Decision:** The copied tool smoke executes isolated `next project validate`
  and binds authoring hash, all neutral roots and bounded publication counts.
  Game/headless remain the runtime clean-install receipts.
- **Rejected alternatives:** Broaden generic application bootstrap inside R7b,
  add a second synthetic project to the shipping package, or accept a mere
  `--help`/version process launch.
- **Consequences:** V5 retains a mandatory meaningful tool receipt without
  conflating creator-generic runtime behavior with the final game bootstrap.
- **Uncertainty:** None at the R7b boundary.
- **Reconsider when:** A later accepted creator contract makes the final product
  project runnable through the generic runtime path.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: current native manifest predates `PackageManifestV4` | R7b was still open after R7a | Rejected: strict V4 implementation and tests exist | Closed; advance only because tools/source are wire-incompatible |
| H2: current native bundle omits a copied tools root | Confirmed in the predecessor binary sources, inventory, report and smoke | Rejected by exact V5 bundle `a5f896e…`: copied `bin/next` and frozen source are mandatory and bound | Closed; future format changes require a new strict manifest version |
| H3: current smoke environment is not fully clean-install isolated | R7b requires cleared loader/process state | Rejected by exact gate: `env_clear`, isolated HOME/state/temp and a narrow display allowlist cover all copied launches | Closed; retain the launcher policy for future package versions |

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, ADR-090
2. SPEC-00, SPEC-04, SPEC-09, SPEC-12 and SPEC-29
3. ADR-001, ADR-003, ADR-028, ADR-030, ADR-035, ADR-036 and ADR-045
4. `docs/roadmap.md` R7/R7b and the completed R7a task-state
5. Current `v1-package`, native-gate package validation and package manifest implementation

## Next action

R7b has no remaining implementation action. Open a separate R7c task-state
before changing performance authority, fingerprints, budgets or evidence.

## Do not retry

- Synthetic Windows/paired package evidence — outside current v1 scope and
  explicitly rejected by ADR-090.
- Reusing R7a `PASS` as R7b completion — that run predates the R7b package
  contract and proves only the prior bundle.
- Retrying `next project run` on `reference-alpha` at `22459f9…` — exact direct
  and packaged evidence shows the product-specific bootstrap is incompatible
  with that generic creator-runtime command.

## Handoff

- **Workspace state:** Exact executable commit is `a5f896e…`; package A/B and
  native-gate evidence remain under ignored `artifacts/r7b/`. Only the closing
  roadmap/task-state promotion is dirty and should be committed separately.
- **Checks:** Corrected focused package `22/22`, current package report `1/1`,
  xtask command `44/44`, full xtask library `109/109`, clippy and boundary scan
  pass. Exact package A/B and the complete eight-check native gate pass on the
  same clean commit.
- **Remaining risk:** No open R7b implementation or evidence gap. R7c Linux
  performance authority, R7d hardening and R7e distribution closure remain
  separate roadmap work.
- **Promotion needed:** SPEC-04 and roadmap are aligned; commit this closing
  documentation separately. No new ADR is expected unless accepted semantics
  change.
