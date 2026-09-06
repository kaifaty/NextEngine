---
name: nextengine-mathematical-research
description: "Prove/refute/bound NextEngine math/numerical claims with counterexamples/reproducible evidence. Use for stability, convergence, conservation, conditioning, discretization, fixed-point/overflow/error bounds, solver/control/geometry, numerical oracles, frozen-model math, or math/model-validity blocker after 2 repair cycles. Russian: математическое исследование, докажи/опровергни, контрпример, устойчивость, сходимость, закон сохранения, граница ошибки, numerical oracle, формализуй лемму. Excludes ADR/SPEC/roadmap ownership, routine implementation, PPO diagnosis/runs, Isaac admission/task-state."
---

# NextEngine mathematical research

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Resolve one exact mathematical or computational claim inside the architecture
that NextEngine actually declares. Produce the smallest evidence that can
falsify or support the claim; do not turn research into a parallel architecture
or certification system.

## Choose the required depth

- For read-only triage, review or claim criticism, answer inline: identify the
  missing owner/definition/evidence, cap the claim and name the cheapest next
  discriminator. Do not create task-state, a dated report or a portfolio merely
  to say that a claim is underspecified or exceeds its evidence. Do not open
  the method references for an obvious negative routing case.
- For an actual research campaign, follow the full workflow below and preserve
  the versioned contract and decisive evidence.
- For settled ordinary implementation, leave this skill and use the normal
  repository/Rust workflow. Invoke `$nextengine-architecture` only when
  ownership, public semantics, roadmap scope, constraints or ProductChecks are
  unresolved; return there after research when a product decision remains.

A generic triage answer that asserts no NextEngine contract may stop at the
missing definitions and authority boundary. Before making a repository-specific
claim or starting a campaign, perform the routed architecture reads below.

## Preserve ownership

| Concern | Owner |
| --- | --- |
| Accepted/Proposed semantics, state owners, contracts, roadmap and ProductCheck mapping | `$nextengine-architecture` |
| Exact mathematical claim, counterexamples, numerical oracle and claim-scoped evidence | this skill |
| Durable resume state, rejected paths and handoff | `$maintain-task-context` |
| Hash-closed PPO diagnosis/next-experiment selection; run preflight/start/resume | `$nextengine-training-diagnostics`; `$nextengine-training-runner`, respectively |
| CPU PhysX versus Isaac mirror admission | `$nextengine-isaac-correspondence` |

Use generic RL, determinism, CAS or theorem-prover skills only as subordinate
references. Repository authority and frozen NextEngine profiles take
precedence over their defaults.

## Run the campaign

### 1. Route before reasoning

1. Read `AGENTS.md` and the matching rows in
   `docs/architecture/agent-routing.md`.
2. Read every routed SPEC/ADR in full and record each relevant status. Treat a
   Proposed model as a research candidate, never as shipped behavior. A
   blocker on production code may still permit an external or tool-only
   research oracle; verify that scope instead of implying promotion.
3. Read the matching task-state when the work is long-running or resumed. If
   none exists and the work qualifies, use `$maintain-task-context` to create
   it before producing large logs or experiments.
4. Identify the engineering consumer and the one decision the research may
   inform. If the question is really ownership, public semantics, roadmap
   scope or ProductCheck selection, hand it to `$nextengine-architecture`. If
   it is settled ordinary implementation, leave for the normal implementation
   workflow without manufacturing a proof campaign.

### 2. Freeze the research contract

Read [research-contract.md](references/research-contract.md). For triage, use it
to expose missing fields inline. For a campaign, write the versioned contract
before selecting a favored method. State the quantified claim and its negation,
units, frames, signs, boundary conditions, finite-precision model, assumptions,
near-misses, claim ceiling, falsifiers, budget and honest stop condition.

Give the contract a revision. A changed claim, domain, assumption or numeric
profile creates a new revision and preserves the old result; never silently
move the goal to fit the evidence.

### 3. Ground the model and prior art

- Verify definitions, dimensional consistency, limiting cases, symmetries,
  invariants, well-posedness and degenerate inputs before expensive work.
- Confirm that every quantified domain/profile set is defined and non-empty;
  reject a vacuous success over an empty or unresolved set.
- Run the cheapest decisive sign, unit, range, limit or one-element falsifier
  before portfolio search or a large sweep.
- Search current primary sources when prior art, a named theorem, a numerical
  method or novelty matters. Read the actual theorem/method, hypotheses,
  experiment and limitations; search snippets and abstracts are discovery
  evidence only.
- Reproduce a known successful control when practical. Distinguish results
  from another discretization, machine or profile from evidence about this
  one.
- Record stable links, publication status, version/date and the exact bounded
  proposition each source supports. Run a novelty review only when novelty is
  part of the claim.

### 4. Search a bounded portfolio

For a non-trivial claim, keep at least these genuinely different routes alive:

1. a known baseline or direct derivation;
2. an alternative representation or mechanism;
3. a disproof, boundary-case or counterexample route.

When independent subagents are available, give each only the frozen contract,
its route and raw shared inputs. Do not reveal the favored answer before the
first artifacts return. Require an equation, proof step, counterexample,
certificate, executable probe or exact blocking lemma—not a status essay.

Skip portfolio ceremony when one cheap discriminator can already settle the
bounded question. Stop repeating a blocked route unless new evidence changes
its mechanism or assumptions.

### 5. Compute as evidence

Read [numerical-evidence.md](references/numerical-evidence.md) for numerical,
physical, geometric, stochastic or floating-point claims.

- Predeclare precision, tolerances, parameter envelope, seeds, sample counts,
  refinement schedule, successful controls, negative controls and stop rules.
- Keep the reference oracle independent of the production candidate. Shared
  code may share the same defect and is not independent verification.
- Prefer exact or arbitrary-precision small cases, manufactured solutions,
  interval bounds and a second formulation before large sweeps.
- Preserve failures and the first decisive boundary. Never use implicit
  epsilon, undeclared point selection, hidden backtracking or retry-to-green.
- Treat a finite sweep as evidence over that finite domain unless it emits a
  complete independently checked certificate.

Use Python and already available repository libraries as the portable
fallback. Use SymPy, Wolfram, SMT/SAT or other tools only when present and
claim-appropriate; record versions and assumptions. Do not add an external
service, plugin or dependency merely to decorate the campaign.

### 6. Audit the weakest link

Give a fresh reviewer the frozen contract, candidate derivation and raw
artifacts without the discovery narrative or desired verdict. Require checks
for:

- a weaker or adjacent statement substituted for the contract;
- missing units, frames, signs, quantifiers, boundary cases or theorem
  hypotheses;
- circular reasoning or a cited result consumed more strongly than stated;
- conditioning, cancellation, rank ambiguity, overflow, rounding and
  reduction-order defects;
- cherry-picked parameters, seeds, baselines or successful retries;
- a reference oracle that is not actually independent;
- an experiment whose observable cannot distinguish the hypotheses.

For executable research evidence, compilation and author-written tests are
necessary but never sufficient. Before interpreting a decisive `PASS` or
`FAIL`, freeze the candidate diff, command and raw output hashes, then give a
fresh reviewer the contract, relevant source/diff and raw output without the
author's diagnosis or intended result. Require the reviewer to inspect both
candidate and controls for shared defects, exact operation/count/order/sign
correspondence, oracle independence, identity sealing, mutation/failure
precedence and whether the observable actually supports the claimed boundary.
The reviewer must report findings and must not silently repair the candidate.
After a material fix, rerun the evidence and repeat review of the changed
surface. If an independent reviewer is unavailable, record code-review
correspondence as `NOT_TESTED` and cap the result; do not substitute a second
self-review or a successful build.

Bound review cost per executable experiment lineage:

- allow at most two independent passes: one initial review and one re-review
  after a single batched repair;
- batch all initial findings into that repair instead of opening one loop per
  finding;
- if the re-review still finds a load-bearing defect, stop the experiment as
  `INCONCLUSIVE`; do not start a third repair/review loop under the same
  contract or research ID;
- open a new revision or successor experiment only when the engineering
  consumer still needs the result and the apparatus/discriminator changes
  materially, not merely to chase a clean verdict;
- ask the reviewer to verify hashes first and budget at most one clean rebuild
  plus one focused rerun. Re-execute ancestor chains only when their captured
  hash mismatches or their live behavior is itself load-bearing and otherwise
  unverified.

Document non-load-bearing review notes without recursively re-reviewing them.
A reviewer may accept a bounded result with explicit limitations when those
notes cannot alter the declared observable, route, identity or claim ceiling.

Recompute load-bearing steps or use a different representation. If a
load-bearing objection remains after the allowed re-review, replace the
approach or return an explicit unresolved gap rather than adding more
ceremonial review.

### 7. Escalate formal verification selectively

Read [formal-verification.md](references/formal-verification.md) before using a
CAS, exact certificate or Lean. Formalize only a faithful, compact statement
whose proof closes a material uncertainty. A proof over real numbers does not
prove the behavior of the Rust, `f32`/`f64`, SIMD, PhysX or GPU implementation;
that requires a separate correspondence argument and executable checks.

### 8. Report the strongest honest result

For every material claim, report a status and one or more evidence classes
from [research-contract.md](references/research-contract.md). Do not collapse
analytic proof, exact computation, numerical evidence, empirical measurement,
formal proof and implementation correspondence into one medal or score.

Use `PROVED` only when the stated claim and assumptions are completely closed
by an auditable derivation, kernel-checked proof or exhaustive finite
certificate. Otherwise use `REFUTED`, `SUPPORTED_BOUNDED`, `INCONCLUSIVE` or
`NOT_TESTED` and state the exact domain.

## Preserve research artifacts safely

- Put the bounded synthesis in
  `docs/development/<topic>-research-YYYY-MM-DD.md` when it must survive the
  task. Keep task-state compact and link to the report.
- Keep raw runs, datasets, checkpoints, downloaded papers, notebooks, caches,
  generated plots and large solver outputs outside Git. Record exact commands,
  tool versions, input identities and hashes needed to find or reproduce them.
- Do not copy third-party code or skill text without verified source, license,
  version and required notices.
- Commit a small reference implementation or fixture only when it becomes an
  intentional reviewed repository artifact, then apply the normal routed
  checks.

## Hand off without promotion by implication

For a research campaign, cover the following in a concise handoff. For bounded
triage or a side question, include only the applicable result and limitations:

1. the exact claim status and ceiling;
2. decisive evidence and independent checks;
3. refuted routes and reconsideration conditions;
4. remaining uncertainty and the cheapest next discriminator;
5. the architecture, training, correspondence or implementation decision that
   remains outside this result;
6. ProductChecks passed, failed or `NotRun(reason)`.

A research report never changes an Accepted decision, promotes a Proposed
track, authorizes a training run or passes a ProductCheck by itself.
