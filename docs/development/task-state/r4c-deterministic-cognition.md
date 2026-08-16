# R4c deterministic cognition — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE` |
| Updated | 2026-08-16 |
| Task key | `r4c-deterministic-cognition` |
| Scope | One production Strategic Agent cognition consumer over the accepted R4b population/navigation substrate, with separate Agent and Memory owners, deterministic planning, save and replay |
| Definition of done | The bounded R4c consumer passes mapped fast/play/persistence-replay/content-package checks; its exact current-only schemas and promoting ADR are accepted without claiming R4d, B-12 or full R4 |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in schemas and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R4c must add a specialized cognition vertical, not widen R4b or begin the work/currency/trade/food scenario.
- **Why:** Roadmap R4c and ADR-056 require beliefs, derived drives, fixed-point Utility, bounded GOAP, private executive, Decision Trace and separate owner persistence; ADR-046 permits exact schemas only with this production consumer.
- **Next action:** Add the smallest typed cognition/content boundary for one existing population subject, then thread separate Agent and Memory owners through the existing fixed-stage/joint-publication path.
- **Current blocker:** None.
- **Do not retry:** A test-only planner, world-truth snapshot, monolithic NPC aggregate, public task trait, generic scheduler or social/economy expansion; each violates the consumer, ownership or R4c scope boundary.
- **Reconsider when:** Production evidence shows the bounded synchronous planner cannot satisfy correctness/resource limits, or R4d demonstrates a missing owner projection that cannot be added without changing Accepted semantics.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/roadmap.md` R4 sequence | `R4c NEXT` | R4a/R4b prerequisites are complete and the cognition slot is eligible. |
| `docs/development/task-state/r4b-population-navigation.md` | `COMPLETE` | Reuse immutable calendar/population/route projections; do not change physical traversal or create a scheduler. |
| `crates/agent` and `crates/contracts/src/agent.rs` | typed observation | The current combat-affordance planner is reusable substrate but has no beliefs, drive/goal lifecycle, GOAP or durable owner state. |

## Decisions that still constrain the work

### D-001 — Bounded R4c cut

- **Observation:** SPEC-32 separates R4c cognition from R4d structured social/economy execution.
- **Evidence:** `docs/architecture/32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md` and ADR-056.
- **Decision:** R4c will implement one authored cognition profile bound to an existing R4b population subject, immutable epistemic/owner views, beliefs/retrieval, fixed-point goal selection, bounded GOAP, a private executive, Decision Trace and separate Agent/Memory snapshots.
- **Rejected alternatives:** Work, currency, trade, food, commitments, speech acts and physical route following belong to R4d; learned policies remain optional R8.
- **Consequences:** The R4c executive may emit only a typed intent/proposal; it cannot fabricate traversal or mutate RPG/World state.
- **Uncertainty:** Exact current-only version names and count/root changes will be frozen after the production path compiles and focused vectors pass.
- **Reconsider when:** The production consumer cannot demonstrate the full R4c lifecycle without one narrowly scoped additional owner fact.

### D-002 — Separate persistence owners

- **Observation:** ADR-056 assigns plans/tasks/hysteresis to Agent Runtime and beliefs/recollections to Memory Service.
- **Evidence:** ADR-056 ownership table and SPEC-32 persistence section.
- **Decision:** Persist Agent and Memory as separate full-tuple owner segments and publish them atomically with the existing application tick/save/replay closure.
- **Rejected alternatives:** Embedding both in Runtime or one combined NPC blob would create the wrong owner or a parallel RPG/World authority.
- **Consequences:** Planner caches and Decision Trace stay reconstructible/non-authoritative; only future-affecting owner state enters save roots.
- **Uncertainty:** None at the ownership level.
- **Reconsider when:** A future Accepted ADR supersedes the owner split.

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, architecture README/SPEC-00/SPEC-01/glossary.
2. ADR-056, ADR-005, ADR-046, ADR-020, ADR-021 and ADR-022.
3. SPEC-06, SPEC-32, SPEC-08, SPEC-13, SPEC-15, SPEC-19, SPEC-20 and SPEC-21.
4. `docs/roadmap.md` R4c/R4d sequence and `docs/development/task-state/r4b-population-navigation.md`.

## Next action

1. Freeze exact content, Agent and Memory V1 contracts around one reference subject.
2. Prove hidden-fact isolation, canonical goal/plan selection and bounded failure locally before cross-owner integration.
3. Preserve the R4b roots/behavior except for explicit successor format/profile changes.

## Do not retry

- Generic scheduler or async planner framework — the bounded planner is the only new consumer; reconsider only after a second accepted workload requires shared infrastructure.
- World-truth planner input — violates the epistemic boundary; reconsider only under a superseding ADR.
- Social/economy or physical-transfer execution in R4c — belongs to R4d and would expand the increment beyond its roadmap guard.
- Learned strategic/tactical policy — optional R8; deterministic cognition is the complete R4/v1 fallback.

## Handoff

- **Workspace state:** Clean R4b checkpoint before this task-state activation.
- **Checks:** Not run yet; no implementation exists in this task checkpoint.
- **Remaining risk:** The cross-owner save/replay successor is broad and must be kept specialized rather than becoming a generic transaction framework.
- **Promotion needed:** One production-backed R4c ADR plus affected SPEC/routing/traceability/roadmap updates after checks pass.
