# ADR-089: Governed external creator SDK workflow

| Field | Value |
|---|---|
| ID | ADR-089 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-20 |
| Last verified | 2026-08-20 |
| Normative dependencies | [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-082](082-linux-first-development-and-deferred-windows-host.md), [ADR-083](083-public-creator-project-cli-vertical.md), [ADR-084](084-public-creator-run-and-project-package-vertical.md), [ADR-085](085-public-creator-project-inspect-and-diff-vertical.md), [ADR-086](086-public-creator-rpg-starter-template.md), [ADR-087](087-public-creator-runtime-scenario-and-prefix-minimization.md), [ADR-088](088-public-replay-first-divergence-and-domain-inspection.md) |
| Supersedes | Narrowly supersedes ADR-088 and SPEC-09/12/15 statements that the bounded external SDK workflow/documentation remains open. Existing command, report, authoring, package, scenario, replay, Luau and Wasm contract semantics are unchanged. |
| Superseded by | not superseded |

## Context

R6a–R6f expose the complete bounded public project, scenario and replay
command surface, but a creator still had to assemble an operational workflow
from the repository README, the `creator-smoke` README and architecture
documents. The generated starter listed only part of its own lifecycle.

The governing cold-template check also created a valid project and then called
the production loader/cooker directly before the public run/package path. It
proved the template, but did not preserve a real post-create JSON edit and did
not call public validate and public cook as one external workflow.

Luau and Wasm sources were executable examples, but their source text was
embedded inside Rust modules. This made the examples harder to discover and
copy even though both already ran through the same public capability,
mechanics and command validators as first-party behavior.

## Decision

### Canonical beta guide

`docs/creator-sdk.md` is the canonical human-facing source-build guide for the
current creator beta. It documents:

- active Linux and pinned Rust prerequisites;
- cold `rpg-starter` creation and project-namespace rules;
- one concrete post-create edit spanning the starter NPC role, ability ID,
  quest entry/source and one streamed chunk/region;
- public validate, cook, authoring run, inspect, package, package run and
  authoring-to-package diff;
- exact-report automation rules, output atomicity and license/NOTICE handling;
- current scenario and Replay V10 consumers;
- Luau/Wasm example locations, capability/sandbox boundary and current limits.

The guide is operational documentation, not a new wire format or mutation
authority. Its commands remain the ADR-083–088 surface. The starter's
`README.md`, the repository root README and `creator-smoke` point to the guide.

### Governed cold edit and lifecycle

`content-package` creates a fresh independently namespaced starter in scratch
storage and then edits only public `project.authoring.json` fields using JSON
semantics available to an external creator. It changes the same concepts
listed by the guide while retaining fixed asset/persistent identities and the
valid dependency graph.

The check requires the edit to change the project lock, then calls public
`project validate` and `project cook` and requires their exact lock to match
the production cooker. The edited project must subsequently pass the existing
public authoring run, package, packaged-byte run, source/package inspect parity
and empty diff path. A hidden project constructor or template-specific runtime
branch is not accepted.

This extends the existing `content-package` scenario; it does not add a new
global ProductCheck or another report family.

### External extension sources without identity change

The governed Luau scripted-melee and Wasm Component WAT sources live under
`examples/creator-sdk/`. The production reference constants include those
ordinary files byte-for-byte. Their existing leading/trailing bytes are
preserved, so package content hashes, manifests and runtime semantics do not
change merely because the examples became externally visible.

The current WIT V3 source remains under `crates/plugin-host/wit/v3/` and is
linked by the guide. Project Authoring V7 does not gain arbitrary project-local
Luau/Wasm directory ingestion. That future surface needs a concrete consumer
and an explicit package/provenance/failure decision; it cannot be simulated by
an engine-crate content branch.

## Product impact

A creator now has one discoverable clean-checkout path from an absent
directory through authored JSON, cooked content, a running project and an
exact package, with the same route enforced as a regression rather than only
described in prose. Extension authors can inspect and copy the exact Luau/Wasm
source bytes already executed by the governing package check.

This completes R6g and the bounded R6 Creator beta/B-09 workflow blocker.
Broader inspectors, editor UX, project-local extension ingestion, replay
capture and MCP remain consumer-driven future work rather than hidden R6
requirements.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| focused verification | Fresh starter, documented JSON edit, public validate/cook | Edited ability/quest/NPC/chunk values cook; public and production locks agree | Keep R6g incomplete |
| `content-package` | Complete edited starter lifecycle plus governed Luau/Wasm sources | Create→validate→cook→run→package→package-run→inspect/diff passes; common extension validators remain exact | Fail the governing check |
| `fast` / `host-check` | Public docs/template/example source and cross-crate include paths | Formatting, strict lint, workspace tests, doctests and boundaries pass | Keep R6g incomplete |
| docs links/commands | Root, starter, sample and architecture references | Every changed local target exists and command spelling matches SPEC-09 | Correct before handoff |

`play`, `persistence-replay`, `platform` and `performance` are not added by
this change: no runtime, authoritative state, session, host or hot-path
semantics change. Existing creator run/scenario/replay checks retain their
prior evidence.

## Considered alternatives

- **Add a GUI, MCP server or live mutable inspector.** Rejected: no concrete
  R6g consumer requires another mutation or observation surface.
- **Add a generic project patch/merge schema.** Rejected: ordinary JSON editing
  is sufficient for the bounded exercise, while merge identity, provenance and
  rollback semantics would be a new API.
- **Teach Project Authoring V7 arbitrary project-local Luau/Wasm ingestion.**
  Rejected for this increment: it needs a concrete distribution consumer and
  fail-closed package policy, not a documentation workaround.
- **Leave executable examples embedded in Rust.** Rejected: creators cannot
  discover or reuse the exact source without extracting an implementation
  string.
- **Document commands without governing their sequence.** Rejected: the prior
  check could pass while public validate/cook or post-create edit regressed.

## Consequences and next boundary

- The public creator surface remains twelve operations and seven report
  families; no version changes.
- `content-package` now governs the complete edited cold-start lifecycle.
- R6 and B-09 close on the existing bounded beta surface. Future creator work
  requires a concrete consumer and a new roadmap slot; it does not reopen R6
  by default.
- Linux remains the active development and sole v1 shipping host. Windows/THOTH
  are outside current scope indefinitely under ADR-090.

## Supersession

Changing the canonical workflow's public operations, report semantics,
current-only policy, edit authority, output atomicity or extension package
ingestion boundary requires a successor. Editorial improvements and new links
that do not change those semantics do not.
