# NextEngine mathematical research skill — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-25` |
| Task key | `mathematical-research-skill` |
| Scope | Add one repository-scoped workflow for mathematical and computational research, and tighten only the directly overlapping skill boundaries. |
| Definition of done | The new skill has a bounded research contract, falsification/verification workflow, NextEngine authority handoff, context-bounded metadata, current UI metadata, source/provenance note, independent forward-test evidence and documentation-only validation. |
| Authority | Working context only; `AGENTS.md`, Accepted architecture, repository skill sources and exact external primary sources outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** One NextEngine-specific orchestrator is implemented and validated; startup metadata is distilled, while detailed methods and optional backends load only after a matching trigger.
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
| Six fresh-agent forward tests, three focused regression reviews and the dated implicit-routing probes below | Research prompts produced bounded contracts; architecture, PPO, implementation and formal-correspondence boundaries routed correctly; shortened descriptions still selected mathematical research and architecture without explicit `$skill` mentions | Markdown-only v1 is sufficient; keep real-campaign ergonomics and implicit synonym recall as the remaining observation surface. |
| [Official OpenAI skill documentation](https://developers.openai.com/codex/skills) plus YAML-parsed local metadata measurement | Startup context contains skill name/description/path; mathematical and architecture descriptions were the two project-authored outliers at 839 and 758 characters | Distill them to 598 and 597 characters; keep the 10.7 kB mathematical workflow and 15.2 kB references conditional. |
| `docs/architecture/agent-routing.md`, workflow/security rows | Documentation/agent-guidance cheap path applies; external inputs and provenance remain bounded | Use `git diff --check` plus direct reference validation; avoid unpinned external code or research artifacts in Git. |

### Metadata routing probes — 2026-08-25

- **Mathematical prompt:** `До реализации нового fixed-point coupler докажи или опровергни сохранение импульса и отсутствие overflow на всех допустимых профилях; сначала сформулируй исследовательский контракт и самый дешёвый контрпример. Только анализ, без изменений файлов.`
  **Observed route:** Fresh agent `/root/routing_probe_math_0825` selected `nextengine-mathematical-research` without an explicit skill name and returned a bounded research contract before implementation analysis.
- **Architecture prompt:** `Нужно изменить публичный WorldCommand и решить, требуется ли новый ADR, какие SPEC затрагиваются и какие ProductChecks обязательны. Только анализ, без изменений файлов.`
  **Observed route:** Fresh agent `/root/routing_probe_arch_0825` selected `nextengine-architecture` without an explicit skill name and routed the question through the governing SPEC/ADR and ProductCheck workflow.

## Decisions that still constrain the work

### D-001 — One project-specific orchestrator

- **Observation:** MerLean, open-problem-prover and academic-research workflows overlap at orchestration, while NextEngine already has project-specific authority and experiment skills.
- **Evidence:** User-provided comparison, current `.agents/skills` inventory and the completed primary-source audit linked above.
- **Decision:** Create one lean project-authored research skill and treat literature, CAS, Lean and optional external services as selectable backends rather than peer orchestrators.
- **Rejected alternatives:** Install every cited skill or copy an entire framework; both duplicate control flow and weaken project-specific authority.
- **Consequences:** The skill must explicitly route architecture, task-state, RL/training and correspondence work to their existing owners.
- **Uncertainty:** Whether repeated real campaigns will justify a small deterministic contract validator or a project-pinned formal backend.
- **Reconsider when:** Primary-source inspection shows a smaller reusable upstream component with compatible license and clearer responsibility.

### D-002 — Budget startup metadata, not research rigor

- **Observation:** Codex loads every discovered skill's name, description and path before choosing a skill; the full body and references are progressive-disclosure layers.
- **Evidence:** Official documentation above plus the local description/package measurement.
- **Decision:** Target at most 600 characters for project-authored descriptions unless forward tests show a concrete recall loss. Keep only triggers and ownership boundaries there; keep the core workflow in `SKILL.md` and specialized playbooks one reference level deep.
- **Rejected alternatives:** Remove claim-critical workflow from the conditional body, or install external research frameworks as peer skills; neither improves startup routing safely.
- **Consequences:** YAML-parsed project-authored descriptions fall from 3,742 to 3,340 characters without changing the mathematical workflow or optional backend policy.
- **Uncertainty:** Shorter descriptions may miss an implicit synonym not represented in the retained English/Russian triggers.
- **Reconsider when:** A fresh-agent trigger test or real task fails to select the correct skill.

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

- **Workspace state:** New skill, references, UI metadata, dated audit and focused handoffs added; mathematical/architecture metadata subsequently distilled under the context budget; no pre-existing user changes were present.
- **Checks:** All six project-authored skill packages passed `quick_validate.py`; the YAML-parsed mathematical/architecture descriptions meet the 600-character policy target at 598/597 and all six total 3,340 characters; the two dated implicit-routing probes above passed; every changed Markdown relative link and the official documentation link resolve; all 15 earlier report links returned success; `git diff --check` passed.
- **Not run:** Cargo, `host-check` and ProductChecks (`NotRun(NoExecutableChange)`).
- **Remaining risk:** No real engine research campaign has yet exercised artifact retention, tool selection or multi-day resume behavior.
- **Promotion needed:** None beyond this repository skill/documentation change; no ADR/SPEC or roadmap change is justified.
