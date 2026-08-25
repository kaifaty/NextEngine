# Research contract and claim ledger

Use this template at the start of a campaign. Keep it proportional: a small
claim may need one page; a cross-domain solver claim may need more. Delete
irrelevant prompts rather than filling them with ceremony.

## Contract

```markdown
# <topic> research contract — revision <N>

| Field | Value |
| --- | --- |
| Research ID | <stable id> |
| Architecture snapshot | <commit; governing SPEC/ADR and status; profile/input hashes> |
| Engineering consumer | <one decision this may inform> |
| Claim class | <universal theorem / counterexample / finite bound / profile-bound numerical claim / empirical calibration / algorithm comparison> |
| Claim status target | <what result would be sufficient, without preselecting direction> |
| Budget | <time, compute, source or experiment ceiling> |

## Exact claim

<Define every object and quantify the statement.>

## Exact negation

<State what a valid counterexample or negative resolution must establish.>

## Fixed definitions and model

- Units and nondimensionalization:
- Coordinate frames, orientation and sign conventions:
- Quantified domain/profile set and evidence that it is defined and non-empty:
- Initial and boundary conditions:
- Topology/discretization/contact or stochastic model:
- Arithmetic model: exact / integer / fixed point / real / IEEE profile:
- Architecture facts treated as constraints rather than research variables:

## Assumptions

| Assumption | Status | How checked or bounded |
| --- | --- | --- |
| <assumption> | given / cited / derived / unverified / excluded | <evidence> |

## Resolution and near-miss firewall

- Counts as a positive resolution:
- Counts as a negative resolution:
- Does not count: <vacuous result over an empty/unresolved domain, finite sweep, changed discretization, weaker norm, different profile, asymptotic-only result, unverified tolerance, etc.>
- Claim ceiling even after success:

## Competing hypotheses

| ID | Hypothesis | Evidence for | Evidence against | Cheapest discriminator |
| --- | --- | --- | --- | --- |
| H0 | <baseline/null> | | | |
| H1 | <leading mechanism> | | | |
| H2 | <non-local alternative> | | | |

## Evidence plan

- Primary sources and known control:
- Direct or alternative derivation:
- Counterexample/degenerate-case search:
- Independent oracle or implementation:
- Numerical protocol and controls:
- Formal or exact verification, if justified:
- Adversarial reviewer input and independence rule:

## Stop and reconsider

- Stop with `INCONCLUSIVE` when:
- Do not retry:
- Reconsider when:
- Promotion boundary after this research:
```

Changing the exact claim, domain, assumptions, discretization, arithmetic
profile or success criterion creates a new contract revision. Preserve the old
claim and result so a near miss cannot silently become success.

## Claim ledger

Use one row per result that matters to the engineering decision.

| Claim ID | Exact bounded claim | Status | Evidence class | Evidence pointer | Independent check | Ceiling / gap |
| --- | --- | --- | --- | --- | --- | --- |
| C1 | ... | ... | ... | ... | ... | ... |

Allowed status values:

- `PROVED`: the complete quantified claim is closed under its recorded
  assumptions by an auditable proof, a kernel-checked proof or an exhaustive
  finite certificate whose checker and range argument are independently
  validated.
- `REFUTED`: a proof of the negation or a valid counterexample has been
  independently rechecked against every premise.
- `SUPPORTED_BOUNDED`: numerical or empirical evidence supports only the
  declared finite/profile/statistical domain.
- `INCONCLUSIVE`: serious hypotheses remain and the current evidence cannot
  distinguish them.
- `NOT_TESTED`: required evidence was not run or unavailable.

Evidence classes are orthogonal; use more than one when appropriate:

| Evidence class | Meaning | Does not establish by itself |
| --- | --- | --- |
| `ANALYTIC_DERIVATION` | Human-auditable derivation with explicit premises | Floating-point or implementation behavior |
| `EXACT_CERTIFICATE` | Exact arithmetic, exhaustive finite enumeration, SAT/SMT/ILP or other checkable certificate | Behavior outside the certified finite model |
| `FORMAL_KERNEL` | Kernel-checked formal statement with recorded imports and axioms | Faithfulness of the model or production implementation |
| `NUMERICAL` | Reproducible approximation with precision, bounds, controls and refinement | A universal theorem outside the sampled/bounded domain |
| `EMPIRICAL` | Measurement from an exact engine/profile/corpus/host identity | Mathematical necessity or another profile |
| `CORRESPONDENCE` | Independent mapping/comparison between model, oracle and implementation | Validity of the source model itself |
| `PRIMARY_SOURCE` | Directly inspected theorem, method, data or official specification | Correct application to this claim without a hypothesis match |

## Report skeleton

1. State the strongest claim first with its status and ceiling.
2. Restate the frozen contract revision and architecture status.
3. Present decisive evidence before secondary observations.
4. Record independent checks and adversarial objections.
5. List refuted/blocked routes and exact reasons not to retry them.
6. Separate mathematical validity, numerical validity, model adequacy,
   implementation correspondence, performance and product status.
7. End with remaining uncertainty, reconsideration condition and the smallest
   next action.
