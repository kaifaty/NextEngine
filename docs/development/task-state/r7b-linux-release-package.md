# R7b Linux release package — task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / IMPLEMENTATION_GREEN` |
| Updated | 2026-08-21 |
| Task key | `r7b-linux-release-package` |
| Scope | Freeze the representative Linux release input, publish a reproducible native package, validate its ELF/runtime prerequisites and run copied `game`, `headless` and tools in an isolated clean-install environment |
| Definition of done | One exact clean Linux commit passes the versioned package manifest, reproducibility, ABI/dependency and isolated copied-root smoke matrix without relying on Windows or R7c timing authority |
| Authority | Working context only; ADR-090, SPEC-04, SPEC-12 and `docs/roadmap.md` remain normative |

## Resume in 60 seconds

- **Current conclusion:** `PackageManifestV5` is implemented through the
  existing production packager. It now copies/audits `next_game`,
  `next_headless` and public `next`, freezes the exact authoring source, cooks
  from that copy and requires an isolated one-tick `next project run` receipt.
- **Why:** Focused package tests exercise copied-root success, ABI-before-smoke,
  exact source bytes/layout, environment stripping, V5 strict decoding and
  tool receipt failures; all currently pass.
- **Next action:** Commit the coherent implementation, then run two clean
  release package invocations and compare every emitted byte before the exact
  clean Linux native-gate evidence run.
- **Current blocker:** None.
- **Do not retry:** Do not add a Windows package slot, inherit THOTH timing, or
  treat the R7a package smoke as proof of the broader R7b clean-install matrix.
- **Reconsider when:** A source audit proves one of these requirements already
  exists with strict positive/failure coverage and exact evidence.

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
- **Uncertainty:** The smallest schema/version change is pending source audit.
- **Reconsider when:** Existing code cannot express the accepted V4 package
  boundary without a separate production consumer.

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
  needs a directly inspectable tool hash/status/state/ledger/final-save tuple.
- **Decision:** Current Linux `v1-package` emits strict
  `CommandReportV2<PackageDetailsV2>`; legacy schema 1 remains only in the
  dormant historical target decoder.
- **Rejected alternatives:** Keep emitting an incomplete current report or add
  optional fields to the strict schema-1 wire.
- **Consequences:** Linux native-gate parsing requires schema 2 and binds its
  existing target summary to the V5 manifest hash.
- **Uncertainty:** None.
- **Reconsider when:** A future package-report schema replaces V2 explicitly.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: current native manifest predates `PackageManifestV4` | R7b was still open after R7a | Rejected: strict V4 implementation and tests exist | Closed; advance only because tools/source are wire-incompatible |
| H2: current native bundle omits a copied tools root | Confirmed in binary sources, inventory, report and smoke | None | Implement and prove V5 tool/source binding |
| H3: current smoke environment is not fully clean-install isolated | R7b requires cleared loader/process state | Rejected: smoke uses `env_clear`, isolated HOME/state/temp and a narrow display allowlist | Reuse the same launcher policy for `next project run` |

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, ADR-090
2. SPEC-00, SPEC-04, SPEC-09, SPEC-12 and SPEC-29
3. ADR-001, ADR-003, ADR-028, ADR-030, ADR-035, ADR-036 and ADR-045
4. `docs/roadmap.md` R7/R7b and the completed R7a task-state
5. Current `v1-package`, native-gate package validation and package manifest implementation

## Next action

1. Update SPEC-04 to describe V5, the frozen source and public-tool receipt.
2. Commit the coherent implementation so exact checks can reject dirty state.
3. Build two Linux packages from that exact commit, compare their complete
   trees, then run the affected fast/platform checks and exact native gate.

## Do not retry

- Synthetic Windows/paired package evidence — outside current v1 scope and
  explicitly rejected by ADR-090.
- Reusing R7a `PASS` as R7b completion — that run predates the R7b package
  contract and proves only the prior bundle.

## Handoff

- **Workspace state:** V5 implementation, tests and task-state are modified but
  not committed; no unrelated pre-existing changes were present.
- **Checks:** Focused package `22/22`, xtask command `44/44`, package-summary
  validation `3/3`, boundary scan and Linux desktop platform pass. The first
  broad host check ran every workspace test successfully but failed its final
  source-size scan; the split fix passes the direct boundary scan.
- **Remaining risk:** Real release ELF/tool execution, complete workspace gates,
  byte-for-byte two-build reproducibility and exact clean native-gate evidence
  have not run yet.
- **Promotion needed:** Roadmap/SPEC updates only after executable R7b evidence
  passes; no new ADR is expected unless the accepted semantics must change.
