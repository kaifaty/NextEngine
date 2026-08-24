# NextEngine mathematical-research skill design review — 2026-08-24

Status: `IMPLEMENTED_AND_VALIDATED / NO_ARCHITECTURE_OR_ROADMAP_CHANGE`.

## Decision

Create one repository-scoped `nextengine-mathematical-research` orchestrator.
Keep literature retrieval, Python/SymPy, exact or interval arithmetic, Lean,
Lean declaration search and Wolfram as optional claim-specific backends. Do not
install MerLean, open-problem-prover and an academic-research suite as peer
orchestrators: their workflow authority overlaps and none knows NextEngine's
Accepted/Proposed, owner, artifact-hygiene or ProductCheck boundaries.

The new skill owns the question “is this exact mathematical or computational
claim true under these frozen definitions and assumptions?”. It does not own
the question “should the engine adopt this semantics?”. That remains in
`nextengine-architecture`.

## Existing local evidence and gap

NextEngine already has strong research practice:

- [the W1 sealed pressure-solver report](continuum-water-w1-sealed-pressure-solver-research-2026-08-18.md)
  compares bounded hypotheses, rejects attractive solver variants, uses an
  independently written checker, preserves exact roots and states a narrow
  claim ceiling;
- [the TRAIN-4 R136 cone-feasibility report](humanoid-train4-r136-cone-feasibility-research-2026-08-15.md)
  separates numerical validity from physical infeasibility and refuses to turn
  an infeasible frozen model into tolerance tuning;
- [the W2 algorithms/data-structures report](continuum-water-w2-algorithms-and-data-structures-research-2026-08-19.md)
  separates exact-layout optimization from a changed numerical model, binds
  external benchmarks conservatively and predeclares stop gates.

`AGENTS.md` also requires a bounded research cycle after two failed coherent
remediation cycles. The missing component was a reusable skill that makes this
practice discoverable, claim-scoped and composable with existing native skills.

| Existing skill | Retained authority | Gap handed to the new skill |
| --- | --- | --- |
| `nextengine-architecture` | SPEC/ADR/roadmap routing, owners, solution and check selection | Truth of an unresolved stability/convergence/conservation/model claim |
| `maintain-task-context` | Compact resume surface and durable negative decisions | Detailed proof, literature and experiment method |
| `nextengine-training-diagnostics` | Exact run lineage, PPO/safety diagnosis and one-variable experiment | New reward, dynamics, feasibility or solver mathematics |
| `nextengine-training-runner` | Frozen run preflight/launch/resume/evaluation | Scientific question and experiment-contract design |
| `nextengine-isaac-correspondence` | CPU/Isaac P1/P2 identity and admissibility evidence | Disputed canonical model or tolerance mathematics |

Imported `using-deep-rl` and `using-determinism-and-replay` remain subordinate
references. Editing their CC-BY-SA copies would invalidate the current
“unmodified” provenance note and could introduce parallel authority.

## Primary-source ecosystem audit

All revisions below were read from the named upstream repository on
2026-08-24. Methods were reviewed; no upstream code or prose was copied into
the project skill.

| Project and exact revision | Actual useful capability | Decision for NextEngine |
| --- | --- | --- |
| [open-problem-prover `fa3ff7f`](https://github.com/meleantonio/open-problem-prover/tree/fa3ff7f92c115114b6be5faa39c03fe7f859ad5f) (MIT) | [Proof contract](https://github.com/meleantonio/open-problem-prover/blob/fa3ff7f92c115114b6be5faa39c03fe7f859ad5f/skills/proof-contract/SKILL.md), independent route ledger, counterexample route and blind audit | Adopt the concepts. Reject its long open-problem contract, target-search prohibition and indefinite multi-hour loop as defaults. Its [numerical verifier](https://github.com/meleantonio/open-problem-prover/blob/fa3ff7f92c115114b6be5faa39c03fe7f859ad5f/templates/numerical_verifier.py) is a `NotImplementedError` scaffold, not a ready verifier. |
| [MerLean `cf1d6f8`](https://github.com/MerLeanProver/MerLean/tree/cf1d6f8e7ae05d5cd68d007e24b5311c1f3dc219) (Apache-2.0) | Lean dependency graph, statement-per-node work, negative memory, route scouts, skeptic and Codex bundle | Do not install into the engine. The [Codex research agent](https://github.com/MerLeanProver/MerLean/tree/cf1d6f8e7ae05d5cd68d007e24b5311c1f3dc219/research-agent/codex) is real, including offline mode, but the normal stack has substantial [Python/graph/LLM dependencies](https://github.com/MerLeanProver/MerLean/blob/cf1d6f8e7ae05d5cd68d007e24b5311c1f3dc219/src/requirements.txt) and external-data implications documented in [PRIVACY.md](https://github.com/MerLeanProver/MerLean/blob/cf1d6f8e7ae05d5cd68d007e24b5311c1f3dc219/PRIVACY.md). Bronze is debate/numerics, Silver proves selected steps under hypotheses, and only Gold targets end-to-end Lean. |
| [official Lean skills `7d3da02`](https://github.com/leanprover/skills/tree/7d3da0282e7b724b07620e45cf212f2e05e19334) (Apache-2.0) | Small Lean setup/proof/debug/review skills and pinned forward-test design | Recommend later as the formal backend when a real Lean task exists. They are not a research orchestrator or model-correspondence verifier. |
| [Numina-Lean-Agent `1c9af8a`](https://github.com/project-numina/numina-lean-agent/tree/1c9af8a52e715f22fede766425ba3d3b95526132) (README says MIT; root `LICENSE` absent) | Released formal-theorem workflow and evidence for its stated Putnam 2025 and paper-level Lean results | Do not install as the engine orchestrator. The current quick start is built around Claude Code, a Lean project, Python 3.13 and several optional/required external LLM/search credentials. Its formal benchmark does not establish physical-model fidelity, floating-point correspondence or numerical-solver validity. |
| [LeanExplore `6b25f86`](https://github.com/justincasher/lean-explore/tree/6b25f8632cc3387cf85f7730375a690a1f1dfb79) (Apache-2.0) | Read-only semantic search over Lean declarations including Mathlib and PhysLean, with summary-first retrieval | Useful optional backend. Verify every result against the project's pinned Lean/Mathlib revision. Hosted mode has a shared 30-request-per-60-second IP limit and sends queries to an external service; local mode downloads a large index/model stack. |
| [research-papers `97131ba`](https://github.com/marciob/skill-research-papers/tree/97131ba7007f62374cc689cf7a85fa8fead8bb2b) (MIT) | Identifier normalization, OA fallback, full-text depth labels and separation of discovery metadata from evidence | Adopt the source-depth discipline, not its mandatory full-paper protocol or helper. Source availability is best-effort; the parser stack includes PyMuPDF/pymupdf4llm with AGPL/commercial licensing considerations and its cache is not a complete research manifest. |
| [Academic Research Agent `41c611c`](https://github.com/ngtiendong/Academic-Research-Agent-Skill/tree/41c611c2e36461596c0c072e7641f9ddba251be8) (MIT) | Cheapest decisive falsifier, novelty and reality checks, stop/drop branch and claim-to-evidence mapping | Adopt a lightweight engineering reality check. Reject the full PhD artifact lifecycle and validator as default ceremony; its validator checks artifact structure, not scientific truth. |
| [Wolfram AgentTools `b76b1f3`](https://github.com/WolframResearch/AgentTools/tree/b76b1f33e038f4deed51e834b111e92ac99d8787) (MIT source) | Real MCP/CLI tools for Wolfram evaluation, definitions, notebooks, inspection and tests; current README lists Codex support | Keep optional. Wolfram Language/runtime rights and kernel/license-seat constraints are separate from the MIT connector source, and current service setup must be read from official live documentation rather than copied into the skill. |
| [Computational Research `f9351ce`](https://github.com/WolframInstitute/ClaudePluginComputationalResearch/tree/f9351cec28cc5c5052119c8af2deb2c72b0d5097) (README says MIT; root `LICENSE` absent) | Provenance ideas, drift fingerprints and separation of settled results, experiments and conjectures | Do not copy or install. It is a working-draft Claude plugin, has Wolfram/project-specific assumptions and makes public Wolfram Cloud publication part of a notebook workflow. External publication is never implicit NextEngine authority. |

## Corrections to the supplied ecosystem proposal

1. The cited projects are not interchangeable maturity levels. Several are
   prompt workflows; AgentTools and LeanExplore are actual backends; MerLean is
   a young integrated runtime with meaningful dependencies.
2. MerLean now has a Codex package and an offline mode, so “Claude-only” is no
   longer exact. It is still not a lightweight drop-in NextEngine backend.
3. Official Lean skills help write and debug Lean. They do not choose a
   faithful physical model, prove source-to-engine correspondence or make a
   learned/PhysX result product-admissible.
4. Numina-Lean-Agent's published formal-math result is meaningful, but its
   current repository remains a Claude-oriented multi-service system rather
   than a drop-in Codex research skill for engine mathematics.
5. A theorem over ideal real arithmetic does not establish IEEE, SIMD, GPU or
   PhysX behavior. A separate `CORRESPONDENCE` claim is required.
6. Wolfram is useful only when the MCP/runtime is actually available and the
   execution/licensing boundary is acceptable. Python and already available
   project libraries remain the portable fallback.
7. A full paper, notebook, global proof graph or Bronze/Silver/Gold campaign is
   not a default NextEngine deliverable. ADR-030 requires risk-proportional
   evidence and rejects unrelated certification ceremony.

## Selected skill design

The skill is named `nextengine-mathematical-research`, not the broader
`scientific-research`, so ordinary literature reviews, UX research and settled
engineering selection do not trigger it. Physics, control, geometry, ML and
simulation enter only through a precise mathematical or numerical claim.

The package follows the current official
[OpenAI skill-authoring contract](https://developers.openai.com/codex/skills):
repository discovery through `.agents/skills`, concise trigger/boundary
metadata for progressive disclosure, `SKILL.md` plus only needed references,
optional `agents/openai.yaml`, an instruction-only default and behavioral
prompt tests before handoff.

Its package is intentionally instruction-only:

```text
.agents/skills/nextengine-mathematical-research/
├── SKILL.md
├── agents/openai.yaml
└── references/
    ├── research-contract.md
    ├── numerical-evidence.md
    └── formal-verification.md
```

No validator script is justified yet. A machine schema would add ceremony
before repeated use has shown stable fields or a recurring validation defect.

### Core decisions

- Freeze an exact claim, its negation, assumptions, near-miss firewall, claim
  ceiling, budget and `INCONCLUSIVE` exit before selecting a favored route.
- Use a bounded portfolio only when the claim is non-trivial: baseline/direct
  derivation, alternative representation and counterexample/disproof route.
- Require successful and negative controls, the cheapest distinguishing
  experiment, and an oracle that is independent in implementation or
  formulation.
- Use primary sources and full theorem/method hypotheses when they carry a
  claim; do not require a novelty campaign when novelty is not claimed.
- Preserve raw research artifacts outside Git and keep only a dated bounded
  synthesis plus hashes/commands/evidence pointers.
- Escalate Lean, exact certificates or CAS only at a material weakest link.
- Hand semantic promotion back to `nextengine-architecture`; hand exact PPO
  operations and CPU/Isaac admission back to their native skills.

### Result model

Reject a single medal or numeric score. Record two orthogonal dimensions:

- claim status: `PROVED`, `REFUTED`, `SUPPORTED_BOUNDED`, `INCONCLUSIVE` or
  `NOT_TESTED`;
- evidence classes: `PRIMARY_SOURCE`, `ANALYTIC_DERIVATION`,
  `EXACT_CERTIFICATE`, `FORMAL_KERNEL`, `NUMERICAL`, `EMPIRICAL` and
  `CORRESPONDENCE`.

This prevents a Lean proof from silently implying model adequacy, a parameter
sweep from becoming a theorem, or a performance result from becoming physics
correctness.

## Focused improvements to existing skills

- Add a two-way handoff between `nextengine-architecture` and the new skill:
  architecture freezes the allowed model; research returns a claim-scoped
  result; architecture owns any semantic decision.
- Escalate `nextengine-training-diagnostics` only when two remediation cycles
  survive or a new mathematical/model claim is required.
- Make `nextengine-training-runner` explicit that it executes a frozen
  contract, sends run-evidence/next-experiment selection to diagnostics and
  invokes mathematical research only for a new mathematical/model claim.
- Prevent `nextengine-isaac-correspondence` from inventing tolerances or
  changing the canonical model to obtain a pass.
- Add the missing `agents/openai.yaml` UI metadata to
  `nextengine-architecture`, matching the other project-authored operational
  skills.
- Leave imported third-party packs unmodified and document composition in the
  skill inventory.

## Forward-test results and remaining risk

Fresh agents received only the current skill, the prompt and permission to
read directly required references. No test changed repository files or ran an
expensive experiment.

| Prompt | Result |
| --- | --- |
| Q24.40 enthalpy conservation and overflow across arbitrary split/merge sequences | `PASS`: produced a quantified contract, caught the unresolved/possibly empty profile domain, made range analysis the first falsifier and separated integer-model proof from Rust correspondence |
| Implicit beam-solver stability region and smallest divergent case | `PASS`: separated spatial, integrator, nonlinear-solve and finite-precision claims; defined independent oracle routes and an honest infimum/bracket when a continuous minimum need not exist |
| Residual-accumulator crate ownership and ADR need | `PASS`: declined a fake proof campaign and routed the Proposed solver semantics to `nextengine-architecture` |
| KL spike plus hard-ROM-dominated PPO run | `PASS`: routed to training diagnostics, put the first safety failure before optimizer tuning and refused retrospective checkpoint selection without artifacts |
| Lean stability lemma over reals promoted to SIMD Rust `f32` stability | `PASS`: capped the Lean result at `FORMAL_KERNEL`, left implementation `CORRESPONDENCE` untested and proposed a bounded independent discriminator |
| Standard binary heap for an internal tools queue | `PASS`: exited to the normal repository/Rust workflow with no research contract, references or mandatory architecture handoff |

The first pass exposed four reusable defects: no explicit triage depth, no
guard against vacuous domains, an overly broad ordinary-implementation handoff
and no definition of a smallest counterexample in continuous domains. The
skill was revised, and focused regression reviews found those defects closed.

The remaining operational cost is mandated architecture routing: a real
repository-specific campaign can require a large routed SPEC/ADR set. The
skill therefore keeps a cheap inline triage path, but it does not weaken the
repository rule for claims about actual NextEngine semantics. Add a script only
after real campaigns reveal a repeated deterministic structural failure that
the current instruction-only contract cannot prevent.
