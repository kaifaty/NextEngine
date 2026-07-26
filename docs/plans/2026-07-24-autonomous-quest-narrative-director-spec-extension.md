# Autonomous quest lifecycle and narrative director specification extension

## Context

Packet 1.8 is the admitted P0 architecture baseline. ADR-025…ADR-028 are occupied, while SPEC-31 and the next requirement/failure allocations are available. The narrative proposal therefore targets packet 1.9 with ADR-029.

## Goal

Define an architecture-only Proposed track where offered and accepted quests can progress under explicit per-quest policy, committed outcomes can open causally linked quest chains, and an optional asynchronous LLM director may propose bounded graph/text changes without becoming authoritative state.

## Locked decisions

- RPG Framework owns Quest instances, graph revisions, outcomes and hooks.
- World Services owns time/population/schedule facts only.
- Agent Intelligence owns director requests/candidates only.
- Main-story anchors and mandatory ending reachability remain authored.
- LLM proposes graph and text; deterministic validation plus `WorldCommand` admission decides.
- `TemplateNarrativeDirector` remains productive offline.
- Generated content is world/save-local.
- Replay records exact completion/candidate bytes and never regenerates.

## Work

1. Add Proposed SPEC-31 and ADR-029 with `REQ-148`…`REQ-151`, `FAIL-062`…`FAIL-063`.
2. Define quest engagement/autonomy/outcome/hook and content-addressed graph contracts.
3. Define bounded director request/candidate, exact completion assignment, validation order, atomic admission and deterministic fallback.
4. Define save/replay/external-effect semantics, stable diagnostics and four Proposed gates.
5. Create packet-1.9 Pending review record; synchronize the index, conditional integration notes, glossary, evidence register and reserved allocation ledger.
6. Validate the packet-1.9 Accepted/reserved projection, high-water IDs,
   ownership, reverse references and missing-fallback cases with the repository's
   available verification path. The parallel bootstrap simplification removed
   the former `docs-check`/preflight subsystem, so this changeset does not
   silently restore it.

## Bounds

- Request: 256 KiB, 256 facts, 128 quest projections and 64 hooks.
- Candidate: 1 MiB, 32 graph nodes, 96 edges, 16 Quest instances and 64 participants.
- Generated continuation depth: 8.

## Verification

- `QUEST-AUTONOMY-P1`: 1,000 quests ×10,000 ticks over 100 seeds/repeats.
- `NARRATIVE-GRAPH-P1`: 10,000 valid/adversarial candidates with zero partial commits.
- `NARRATIVE-ASYNC-P1`: 1,000 absence/crash/timeout/reorder/duplicate/late/restart/collision injections.
- `NARRATIVE-REPLAY-P1`: 100 save/restart runs across game/headless/capture-worker with zero model/network replay calls.
- Run document consistency checks and the current host-check. The proposal
  remains `Proposed/AwaitingReview`; a future synchronized status change uses
  normal repository review and does not imply implementation gate PASS.

## Non-goals

No runtime implementation, concrete model/provider selection, project-source mutation, mandatory cloud, new VS gate, runtime training or automatic human approval.
