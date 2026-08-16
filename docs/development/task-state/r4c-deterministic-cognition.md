# R4c deterministic cognition — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-16 |
| Task key | `r4c-deterministic-cognition` |
| Scope | One production Strategic Agent cognition consumer over the accepted R4b population/navigation substrate, with separate Agent and Memory owners, deterministic planning, save and replay |
| Definition of done | The bounded R4c consumer passes mapped fast/play/persistence-replay/content-package checks; its exact current-only schemas and promoting ADR are accepted without claiming R4d, B-12 or full R4 |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in schemas and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R4c is complete as the bounded one-subject cognition owner vertical accepted by ADR-073; R4d remains unstarted.
- **Why:** The production path now proves revision-bound beliefs/views, Q16 Utility with inertia/emergency, bounded GOAP, private task state, Decision Trace and separate Agent/Memory persistence without fabricating world outcomes.
- **Next action:** Hand the released WIP slot to R4d planning only when that increment is explicitly started; reuse the accepted R4c owners and do not reopen their semantics casually.
- **Current blocker:** None.
- **Do not retry:** A test-only planner, world-truth snapshot, monolithic NPC aggregate, public task trait, generic scheduler or social/economy expansion; each violates the consumer, ownership or R4c scope boundary.
- **Reconsider when:** Production evidence shows the bounded synchronous planner cannot satisfy correctness/resource limits, or R4d demonstrates a missing owner projection that cannot be added without changing Accepted semantics.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| ADR-073 and SPEC-32 | `Accepted R4c core` | One production cognition subject and paired Agent/Memory owners are current; R4d social/economy breadth remains Proposed/NEXT. |
| Contracts / `next_agent` | `PASS` | Semantic beliefs, Epistemic/Drive views, canonical Q16 goal selection, bounded GOAP, private task lifecycle, typed failure and Decision Trace are deterministic and hidden-fact isolated. |
| Runtime / project / save / replay | `PASS` | Stage 7 planning and stage 9 command/event publication commit eight owner segments; authoring V5 activates V6 with 31 roots/117 entries, and Replay V8 requires Agent plus Memory. |
| Production scenario | `PASS` | `play` runs 32 ticks / 46 events / 13 RPG events with eleven decisions; health `100 → 20 → 100` produces ordinary → emergency interrupt → exact resume at ticks `1/4/7`. |
| Persistence scenario | `PASS` | 19 ticks / 2 generations / 8 RPG events converge at state root `7fe04abc4607668d1ea86a6caaf1ca85956462617b54760dcd34ca8a6ae3e00f`. |
| Promotion gates | `PASS` | Focused checks, `play`, `persistence-replay`, `content-package`, format, strict all-targets clippy, full workspace `host-check` and `boundary-scan` pass on the pinned Rust 1.97.1 Linux host. |
| Retained `r4-100npc` report | `NOT_RUN` | Exact due counts/roots and zero starvation are retained, but the unsupported host and population-only workload provide no B-12 or tier-wide cognition timing claim. |

## Decisions that still constrain the work

### D-001 — Bounded R4c cut

- **Observation:** SPEC-32 separates R4c cognition from R4d structured social/economy execution.
- **Evidence:** `docs/architecture/32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md` and ADR-056.
- **Decision:** R4c will implement one authored cognition profile bound to an existing R4b population subject, immutable epistemic/owner views, beliefs/retrieval, fixed-point goal selection, bounded GOAP, a private executive, Decision Trace and separate Agent/Memory snapshots.
- **Rejected alternatives:** Work, currency, trade, food, commitments, speech acts and physical route following belong to R4d; learned policies remain optional R8.
- **Consequences:** The R4c executive may emit only a typed intent/proposal; it cannot fabricate traversal or mutate RPG/World state.
- **Uncertainty:** None for the R4c cut: current-only names are authoring V5, `ActivatedProjectV6` and `ReplayManifestV8`; broader social/economy semantics remain R4d work.
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

1. Start R4d only as a separate product increment with its own task context and owner-safe systemic scenario.
2. Reuse the R4c Epistemic/Drive, Agent/Memory and proposal-only executive boundaries; add no world-truth shortcut or generic scheduler.
3. Keep R4d work/currency/trade/food and structured speech outcomes behind ordinary RPG/World validation and exact save/replay evidence.

## Do not retry

- Generic scheduler or async planner framework — the bounded planner is the only new consumer; reconsider only after a second accepted workload requires shared infrastructure.
- World-truth planner input — violates the epistemic boundary; reconsider only under a superseding ADR.
- Social/economy or physical-transfer execution in R4c — belongs to R4d and would expand the increment beyond its roadmap guard.
- Learned strategic/tactical policy — optional R8; deterministic cognition is the complete R4/v1 fallback.

## Handoff

- **Workspace state:** R4c implementation and ADR/SPEC/roadmap promotion are complete on `codex/architecture-foundation-promotion`; coherent commits retain the historical R4b boundary.
- **Checks:** Focused contracts/agent/runtime/project/reference/replay tests, format, strict all-targets clippy, `play`, `persistence-replay`, `content-package`, `boundary-scan` and the full workspace `host-check` pass; the retained conditional performance report returns its declared `NOT_RUN` on this unsupported host.
- **Remaining risk:** R4d must not reinterpret the accepted one-subject owner split as tier-wide cognition performance evidence, and must not hide systemic outcomes inside the private executive.
- **Promotion needed:** None for R4c; the next semantic expansion requires its own R4d consumer and promotion decision.
