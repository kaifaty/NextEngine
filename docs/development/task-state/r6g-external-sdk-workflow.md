# R6g external SDK workflow — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-20 |
| Task key | `r6g-external-sdk-workflow` |
| Scope | Close the bounded external creator SDK workflow and its governed documentation without adding a speculative editor, MCP surface or second runtime authority |
| Definition of done | From a clean checkout, a creator can follow one documented public workflow to create and customize the starter NPC, ability, quest and streamed chunk, then validate, cook, run, package and inspect it; the same cold path is enforced by `content-package` and stable failures remain non-mutating |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in schemas/templates and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R6g is complete. The canonical beta guide, ordinary
  external Luau/Wasm sources and governed post-create public JSON edit pass the
  complete public project lifecycle without changing command/report/schema
  semantics. R6 and B-09 are closed on this bounded surface.
- **Why:** ADR-086 already defines starter creation as the bounded cold addition
  of one NPC/ability/quest and three chunks. Source audit showed that ordinary
  public JSON edits plus existing commands close the remaining workflow gap;
  a new editor/patch API would be speculative.
- **Next action:** Plan R7 only after an explicit pre-R7 Windows bring-up
  decision under ADR-082; no further R6 implementation remains.
- **Current blocker:** None.
- **Do not retry:** Do not add a graphical editor, MCP contract, hidden Rust
  authoring helper or second project/replay schema; none has a concrete R6g
  consumer and each would widen the accepted boundary.
- **Reconsider when:** An existing public file-backed edit cannot express or
  validate the required cold-authoring additions.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/roadmap.md`, R6 and B-09 | `R6G_SDK_WORKFLOW / CLOSED` | The bounded creator beta workflow is complete; future breadth needs a concrete consumer |
| `docs/development/task-state/r6f-replay-domain-inspection.md` | `COMPLETE` | Replay inspection is an existing downstream consumer and remains unchanged |
| Workspace at `81ebca3` | `CLEAN` | R6g starts from the verified R6f checkpoint on the active Linux host |
| `docs/architecture/adr/089-governed-external-creator-sdk-workflow.md` | `ACCEPTED / IMPLEMENTED` | R6g changes workflow governance and documentation, not public wire semantics |
| External source byte audit | `PASS` | Luau SHA-256 `19908fe5a9e8974aae378150ff815082f6f9cd052f537ca80646bcfa077a8e67` and Wasm SHA-256 `759c7c5e0de2c6bfab5e7bf846c0f960915d4e2d20c2394557fea8ea68921508` are unchanged from the former embedded literals |
| `cargo clippy --locked -p next_cli -p next_verification -p next_script_luau -p next_plugin_host --all-targets -- -D warnings` | `PASS` | Changed Rust crates are warning-free |
| `cargo test --locked -p next_cli -p next_verification -p next_script_luau -p next_plugin_host --all-targets` | `PASS` | CLI/template/scenario, plugin, Luau and all 74 active verification tests pass; one report-only verification test remains intentionally ignored |
| `cargo run --locked -q -p xtask -- content-package` | `PASS` | Fresh edited starter completed create/validate/cook/run/package/package-run/inspect/diff; 18 creator records, 3 creator chunks and creator composition `008ca7acbee5a934aca9f228b1fb41038843f29dee7af26dcb5c142670b1c339` remained exact |
| Linux `host-check` component closure | `PASS / COMPONENT-COMPLETE` | Exact Rust `1.97.1` on `x86_64-unknown-linux-gnu`, fmt, workspace clippy and workspace tests passed; the final aggregate JSON was transport-truncated after completion, so the lost `xtask` tail was rerun separately: 148/148 tests plus all workspace doc-tests passed |
| `cargo run --locked -q -p xtask -- boundary-scan` | `PASS` | All six architectural boundary checks pass |
| Final `cargo fmt --all -- --check`, `git diff --check` and linked-target inventory | `PASS` | Source formatting, patch whitespace and all newly documented local targets are valid |

## Decisions that still constrain the work

### D-001 — Close a workflow, not a new authoring subsystem

- **Observation:** The roadmap explicitly limits R6 editor scope to CLI/JSON
  plus schemas and leaves broader inspectors consumer-driven.
- **Evidence:** `docs/roadmap.md` R6 scope and B-09; ADR-088 consequences.
- **Decision:** Reuse current public commands and file-backed authoring formats;
  add only the executable orchestration/documentation needed to prove them as
  one external workflow.
- **Rejected alternatives:** GUI/MCP/live inspector or a tooling-only project
  representation, because R6g has no concrete consumer that justifies them.
- **Consequences:** Any implementation helper must invoke the public CLI and
  edit public files as an external creator would.
- **Uncertainty:** Resolved: existing public Project Authoring V7 properties and
  chunk IDs express every required edit without a new authoring contract.
- **Reconsider when:** Source audit finds that a required addition cannot be
  represented by an accepted public schema.

## Open hypotheses

None for R6g. The edited starter and unchanged extension examples have exact
governing evidence across the complete public lifecycle.

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, `docs/architecture/00-product-contract.md`, `docs/architecture/01-system-architecture.md`
2. `docs/roadmap.md`, especially R6 and B-09
3. `docs/architecture/09-tooling-sdk-and-observability.md`, `docs/architecture/11-security-licensing-and-governance.md`, ADR-083 through ADR-088
4. `docs/architecture/12-vertical-slice-conformance.md`, `docs/architecture/15-headless-testing-agent-validation-and-human-evidence.md`, ADR-089 and the checked-in starter/ProductCheck implementation

## Explicit non-runs

- `play` and `persistence-replay`: `NOT_RUN / ScopeUnaffected`; R6g changes no
  runtime, authority, session, save or replay semantics, and the existing paths
  are exercised by the creator lifecycle plus focused verification tests.
- `platform`: `NOT_RUN / ScopeUnaffected`; no host, renderer or target contract
  changed.
- `performance`: `NOT_RUN / NoEstablishedHotPathChanged`; the change is cold
  tooling, documentation and reference-source placement.
- Windows/THOTH: `NOT_RUN / WindowsHostDeferred`; ADR-082 keeps the active
  development host on Linux until an explicit pre-R7 bring-up decision.

## Next action

1. Keep R6/B-09 closed unless a concrete future consumer proves a missing
   creator contract.
2. Before R7 implementation, make the explicit Windows bring-up decision
   already recorded in ADR-082 and the roadmap.

## Do not retry

- A new GUI/MCP/live inspector contract — no concrete R6g consumer; reconsider
  only when an accepted downstream workflow requires it.
- A hidden Rust-only fixture constructor — it would not prove the external SDK
  path; reconsider only if the public schema itself is shown incomplete.

## Handoff

- **Workspace state:** Coherent R6g implementation is committed; unrelated user
  changes were not observed.
- **Checks:** focused clippy/tests, governing `content-package`, Linux host-check
  components, 148 `xtask` tests, workspace doc-tests, boundary scan, fmt, diff
  and linked-target inventory all pass.
- **Remaining risk:** Broader GUI/MCP/live inspectors, arbitrary project-local
  extension ingestion and replay capture remain deliberate non-goals, not R6g
  defects.
- **Promotion needed:** none. Stable decisions are already in ADR-089,
  SPEC-09/12/15, traceability and the roadmap; this file remains resumption
  context only.
