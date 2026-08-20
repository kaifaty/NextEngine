# ADR-087: Public creator runtime scenario and prefix minimization

| Field | Value |
|---|---|
| ID | ADR-087 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-20 |
| Last verified | 2026-08-20 |
| Normative dependencies | [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-29](../29-platform-host-and-application-session.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-047](047-simple-application-session-and-save-on-close.md), [ADR-048](048-direct-exact-project-lock.md), [ADR-082](082-linux-first-development-and-deferred-windows-host.md), [ADR-083](083-public-creator-project-cli-vertical.md), [ADR-084](084-public-creator-run-and-project-package-vertical.md), [ADR-085](085-public-creator-project-inspect-and-diff-vertical.md), [ADR-086](086-public-creator-rpg-starter-template.md) |
| Supersedes | Narrowly supersedes ADR-086 and SPEC-09/15 statements that public creator scenario validate/run/minimize is unimplemented. Existing project commands, reports, one-tick run, package, projection and template semantics remain unchanged. |
| Superseded by | not superseded |

## Context

R6a–R6d let a creator build, run, package and inspect an independent project,
but a regression reproducer still had to live in Rust tests or an informal
shell sequence. The existing one-tick project run proves startup and close; it
cannot name a bounded ordered runtime exercise, assert exact public results or
reduce a failing action sequence for another developer.

The full SPEC-15 scenario intent includes commands, faults, replay and capture.
Promoting that breadth before a consumer would violate ADR-046. The current
`creator-smoke` world-service transition provides a smaller real consumer:
three ordinary ticks produce an exact population-command/event/ledger/state
and final-save proof without a test-only mutation path.

## Decision

### Public commands and current-only manifest

R6e adds exactly:

```text
next scenario validate --scenario <file> (--project <directory> | --package <directory>)
next scenario run --scenario <file> (--project <directory> | --package <directory>)
next scenario minimize --scenario <file> (--project <directory> | --package <directory>) --output <absent-file>
```

`nextengine.creator-runtime-scenario.v1` is a current-only pre-v1 manifest. It
binds the complete public `CreatorProjectIdentityV1`, a lowercase namespaced
scenario ID, an explicit tick budget, one to 256 ordered uniquely identified
`tick` actions and one to 32 sorted uniquely probed exact assertions. Current
probes are tick/event/RPG-event/revision counts and authoritative-state,
command-archive, command-identity-index, command-ledger and final-save hashes.

The manifest is not the full future `TestScenarioManifest`: arbitrary command
payloads, input actions, fault adapters, capture and replay artifacts remain
unpromoted. Unknown fields, duplicate IDs/probes, unsorted assertions, invalid
hashes, over-budget actions and every non-current format fail before scenario
world creation.

### Validation and execution

Validate fully decodes the scenario first, then fully loads/cooks the selected
authoring project or revalidates/reruns the exact creator package. Its bound
project identity must match byte-for-byte. Validate executes no scenario tick;
the normal package proof rerun remains part of untrusted-package validation.

Run starts an isolated headless Application Session. Each action advances one
ordinary Runtime plus World Routine/Population atomic transaction with the
engine-owned principal, capability and command-stream routes required by the
selected project. The current generic creator bootstrap has no fabricated RPG
aggregate, so activity/cognition owner snapshots remain validated and
unchanged rather than inventing outcomes. After the final tick, the complete
owner closure is materialized and the session uses the existing save-on-close
journal.

`ApplicationCoordinator::run_project_headless_scenario` accepts only 1–256
tick actions. Existing `run_project_headless` and `next project run` remain
exactly one tick and retain their prior proof bytes. Headless scenario execution
creates no window, platform device, presentation target or resume slot.

Assertions observe only the final path-free runtime proof. The first mismatch
returns stable assertion ID, last executed action ID, observation tick, probe
and typed canonical expected/actual strings plus the complete project and
scenario identities. Assertions never alter schedule, owner state or hashes.

### Prefix minimization and publication

Minimize first requires the original scenario to reproduce an assertion
failure. It searches prefixes from shortest to longest and accepts the first
candidate preserving the same failure code, category, assertion ID, probe and
exact project closure. It does not edit or remove assertions, substitute a
different failure, modify project/package bytes or directly mutate world
state.

The selected candidate is serialized into a private sibling file, decoded and
rerun again, then atomically linked to a still-absent destination. Existing
file/directory/link output, publication race, validation failure or changed
failure identity publishes no replacement. A passing original returns
`CREATOR_SCENARIO_NOT_REPRODUCED`; a failure that cannot be preserved returns
`CREATOR_SCENARIO_NOT_MINIMIZABLE`.

### Reports and safety

Creator Scenario Report V1 is independent of the five existing report
families. Every command emits exactly one strict JSON object and trailing
newline; exit status is zero only for `PASS`. Success carries path-free
scenario/project identities and validate, runtime or minimization details.
Failure carries stable code/subsystem/message key, optional loaded identities,
optional assertion failure identity and explicit minimization status.

Scenario input is a regular non-link file bounded to 1 MiB. Project/package
validation retains all prior confinement, NOTICE, inventory and current-format
rules. Reports contain no source/output path, source bytes, private store
layout, ECS object or native/backend value.

## Product impact

A creator or CI job can check in one small JSON reproducer, validate that it is
bound to the exact intended project, execute it identically from source or a
distributed package and reduce a failing multi-action run to its earliest
reproducing prefix. The tracked three-tick creator scenario proves real
world-service commands/events and final-save identity beyond the one-tick
startup smoke while preserving production authority boundaries.

This completes R6e and further reduces B-09. It does not complete R6: replay
first-divergence/domain inspectors and the remaining SDK workflow remain R6f+
work.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| focused creator scenario | Validate and repeat the tracked three-tick scenario from authoring and package | Same path-free scenario/project identities and exact runtime/ledger/final-save proof; all nine assertions pass | Stable failure; inputs unchanged |
| minimization matrix | Fail one exact tick assertion across three actions | One-action output reruns with the same assertion failure identity and unchanged assertion | Retain original scenario; publish nothing |
| negative matrix | Retired/malformed/link/over-budget input, project mismatch, passing minimization and occupied/link output | Typed nonzero result before partial publication or hidden migration | Preserve source and destination |
| `content-package` | Run validate/run/minimize through the public CLI over `creator-smoke` and its package | Governing creator workflow proves source/package runtime parity and shortest prefix | Fail the ProductCheck |
| `play` / `persistence-replay` | Shared Runtime, command-ledger, owner transaction or save regression | Existing gameplay and persistence/replay roots remain valid | Keep R6e incomplete |
| Linux `platform` | New shared Application Session scenario entry point | Existing Linux headless/interactive platform smoke remains valid | Report Linux failure; Windows stays deferred |

## Considered alternatives

- **Minimize assertions instead of actions.** Rejected: it weakens the oracle
  and does not meet SPEC-15 failure-preserving action minimization.
- **Repeat isolated one-tick project runs.** Rejected: it would test a shell
  workflow, not an ordered runtime scenario or world-service transition.
- **Expose arbitrary `WorldCommand` JSON now.** Rejected: no current generic
  creator aggregate/input consumer justifies that mutation surface.
- **Emit a replay artifact as the scenario result.** Rejected for R6e: replay
  inspection and first-divergence reporting are the next bounded consumer.
- **Overwrite minimized output.** Rejected: the tool cannot infer ownership of
  arbitrary caller data and a race must not replace it.

## Consequences and next boundary

- The public creator surface now has ten operations and six separately
  versioned report families.
- The first public scenario is deliberately a tick-prefix contract, but every
  action crosses production Runtime/World Services commit and final-save paths.
- Existing one-tick project run/package evidence remains byte-compatible.
- R6 and B-09 stay open. R6f should add read-only replay first-divergence and
  bounded domain inspection before the remaining SDK documentation closure.
- Linux is the active implementation and sole v1 shipping host. Windows/THOTH
  are outside current scope indefinitely under ADR-090.

## Supersession

Changing manifest/report fields, action/probe semantics, project binding,
failure identity, prefix selection, output publication or scenario authority
requires an explicit successor. Additional action/fault/capture/replay kinds
require a concrete consumer and equivalent positive/failure evidence.
