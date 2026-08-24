# NextEngine mathematical research skill — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-24` |
| Task key | `mathematical-research-skill` |
| Scope | Add one repository-scoped workflow for mathematical and computational research, and tighten only the directly overlapping skill boundaries. |
| Definition of done | The new skill has a bounded research contract, falsification/verification workflow, NextEngine authority handoff, current UI metadata, source/provenance note, independent forward-test evidence and documentation-only validation. |
| Authority | Working context only; `AGENTS.md`, Accepted architecture, repository skill sources and exact external primary sources outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** One NextEngine-specific orchestrator is implemented and validated; external literature, CAS, Lean/search and Wolfram remain optional backends.
- **Why:** Existing project skills own architecture, durable context, training operations/diagnostics and CPU/Isaac correspondence, but none owns claim-scoped mathematical research from formal question through counterexample search, numerical verification and decision-grade reporting.
- **Next action:** Use `nextengine-mathematical-research` on the first real unresolved engine-mathematics claim and add deterministic tooling only if repeated campaigns expose the same structural failure.
- **Current blocker:** None.
- **Do not retry:** Do not copy or install several top-level research orchestrators; overlapping authority would make trigger selection and claim status ambiguous.
- **Reconsider when:** A single upstream workflow is shown to satisfy NextEngine authority, provenance, artifact-hygiene and claim-boundary requirements without adaptation.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `.agents/skills/nextengine-architecture/SKILL.md` | Architecture and algorithm selection are owned, but research-campaign verification is not | Keep architecture authority there; add only a narrow handoff to the new skill if needed. |
| `.agents/skills/nextengine-training-{runner,diagnostics}/SKILL.md` and `nextengine-isaac-correspondence/SKILL.md` | Operational training and correspondence workflows are already specialized | The new skill must not launch runs, tune PPO or decide mirror admission. |
| `docs/development/mathematical-research-skill-research-2026-08-24.md` | Primary-source audit records exact upstream revisions, capabilities, dependencies, licensing caveats and selected/rejected patterns | No upstream framework or prose was vendored; optional backends stay subordinate. |
| Six fresh-agent forward tests plus three focused regression reviews | Both research prompts produced bounded contracts; architecture, PPO, implementation and formal-correspondence boundaries routed correctly; all discovered defects were closed | Markdown-only v1 is sufficient; keep real-campaign ergonomics as the remaining observation surface. |
| `docs/architecture/agent-routing.md`, workflow/security rows | Documentation/agent-guidance cheap path applies; external inputs and provenance remain bounded | Use `git diff --check` plus direct reference validation; avoid unpinned external code or research artifacts in Git. |

## Decisions that still constrain the work

### D-001 — One project-specific orchestrator

- **Observation:** MerLean, open-problem-prover and academic-research workflows overlap at orchestration, while NextEngine already has project-specific authority and experiment skills.
- **Evidence:** User-provided comparison, current `.agents/skills` inventory and the completed primary-source audit linked above.
- **Decision:** Create one lean project-authored research skill and treat literature, CAS, Lean and optional external services as selectable backends rather than peer orchestrators.
- **Rejected alternatives:** Install every cited skill or copy an entire framework; both duplicate control flow and weaken project-specific authority.
- **Consequences:** The skill must explicitly route architecture, task-state, RL/training and correspondence work to their existing owners.
- **Uncertainty:** Whether repeated real campaigns will justify a small deterministic contract validator or a project-pinned formal backend.
- **Reconsider when:** Primary-source inspection shows a smaller reusable upstream component with compatible license and clearer responsibility.

## Resolved hypotheses

| Hypothesis | Result | Evidence | Reconsider when |
| --- | --- | --- | --- |
| H1: A Markdown-only skill plus references is sufficient for v1 | Supported | Six dissimilar forward tests produced useful outputs; prose revisions closed every deterministic defect found | Two real campaigns repeat the same missing-field or artifact-validation failure. |
| H2: Formal proof should be optional and claim-driven | Supported | Lean transfer test correctly separated `FORMAL_KERNEL` from `CORRESPONDENCE`; numerical cases needed exact/numerical oracles first | A compact reusable lemma becomes a load-bearing blocker and Lean is available. |
| H3: Existing architecture skill needs only a narrow cross-link | Supported | Ownership routed to architecture while textbook implementation exited to the normal Rust workflow | Real tasks repeatedly bounce ambiguously between the two skills. |

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md` and `docs/architecture/agent-routing.md`.
2. `docs/architecture/README.md`, SPEC-00, SPEC-01, SPEC-11, SPEC-12, SPEC-15, ADR-001 and ADR-030.
3. `.agents/skills/README.md`, `nextengine-architecture`, `maintain-task-context`, training/correspondence skills and the system `skill-creator` instructions.
4. Primary upstream workflow files and the dated research report created by this task.

## Next action

Apply the new skill to the first real solver/model claim. Capture only material
ergonomic failures in this task-state or a successor report; do not add a
validator, Lean stack or external service preemptively.

## Do not retry

- Multiple top-level research orchestrators — responsibilities overlap; reconsider only if one is reduced to a backend-only component.
- Treating CAS output, one floating-point run or an LLM proof sketch as proof — each lacks an independent claim-appropriate verifier.

## Handoff

- **Workspace state:** New skill, references, UI metadata, dated audit and focused handoffs added; no pre-existing user changes were present.
- **Checks:** Five affected skill packages passed `quick_validate.py`; every changed Markdown relative link resolves; all 15 external report links returned success; direct identifiers/metadata exist; `git diff --check` passed.
- **Not run:** Cargo, `host-check` and ProductChecks (`NotRun(NoExecutableChange)`).
- **Remaining risk:** No real engine research campaign has yet exercised artifact retention, tool selection or multi-day resume behavior.
- **Promotion needed:** None beyond this repository skill/documentation change; no ADR/SPEC or roadmap change is justified.
