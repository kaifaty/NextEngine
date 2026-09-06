# Physical sound V36 sealed-owner rebaseline research

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Decision | `PRESERVE_UNOBSERVED_V35_HYPOTHESIS / REPLACE_EXPERIMENT_EXECUTION_CONTRACT` |
| Trigger | [V35 D0 repeat-exact post-access owner fault](physical-sound-v35-d0-development-tournament-result-2026-09-03.md) |
| Access while deciding | `ZERO_NEW_TARGET_MODEL_REAL_PROTECTED_OR_HOLDOUT_VALUES` |
| Product consequence | `RESEARCH_ONLY / AUTHORED_FALLBACK` |

## Problem statement

V35 did not answer whether its geometry-conditioned hybrid beats continuous
local interpolation. It spent fresh train/development roles on a Python
interface error after training and before metric publication. Earlier V23 and
V33 closures also showed that partial callback/fixture coverage is not enough:
an experiment can be scientifically well preregistered yet fail on a path that
only the official owner executes.

The falsifiable question for V36 is therefore two-part:

1. can the exact official entry point complete every stage and terminal on
   type-identical surrogate roles before it receives target access authority;
2. if so, does the unchanged V35 scientific family pass on genuinely fresh
   development and method-holdout roles?

## Evidence and competing explanations

| Hypothesis | Evidence for | Evidence against | Discriminator |
| --- | --- | --- | --- |
| E1: the failure is one isolated typo | Exact exception points to one `len(RoleData)` call | V23 and V33 expose different missed whole-owner paths | Mutate each owner stage and prove exact-path rehearsal catches it |
| E2: static typing alone is sufficient | A strict checker can reject unsupported operations on a concrete role type | Python annotations are not enforced at runtime and existing `Any` boundaries erase guarantees | Strict no-`Any` owner surface plus runtime contracts and rehearsal |
| E3: another richer mock harness is sufficient | I0 already proved counts, terminals and publication faults | I0 still bypassed the exact development-prediction path | Same owner function and concrete role container; only provider differs |
| E4: full official-shape rehearsal is sufficient | It exercises realistic counts, training, resources and publication | A rehearsal-only branch could still diverge from official access | Stage-trace equivalence and byte-sealed owner capability |
| E5: V35 model family should be abandoned | V35 consumed official targets | No metric/prediction/weight escaped, so quality was not observed | Preserve knobs exactly, use fresh roles, allow one successor trial |

## External evidence

- The [Python typing specification](https://typing.python.org/en/latest/spec/type-system.html)
  describes typing as a static-analysis mechanism rather than runtime
  enforcement. V36 therefore cannot treat annotations as the only gate.
- [Python protocols](https://typing.python.org/en/latest/spec/protocol.html)
  define assignability through required members. V36 uses a narrow typed owner
  interface and concrete `RoleData` rather than permissive `Any` dictionaries.
- [Hypothesis](https://hypothesis.readthedocs.io/en/latest/) generates edge-case
  values, while its [stateful testing](https://hypothesis.readthedocs.io/en/latest/stateful.html)
  generates action sequences. V36 uses bounded property/state-machine tests for
  access order, terminal atomicity and mutation detection; they supplement,
  rather than replace, exact end-to-end execution.
- The scikit-learn guidance on [inconsistent preprocessing and data leakage](https://scikit-learn.org/stable/common_pitfalls.html)
  recommends one pipeline so train and evaluation apply the same transforms
  while fit data stays isolated. V36 applies the same principle to the entire
  experiment owner: one path, distinct capability-limited providers.
- NIST frames AI assurance as testing, evaluation, verification and validation
  across the lifecycle in its [AI Resource Center](https://airc.nist.gov/).
  V36 separates executable verification from scientific evaluation and from
  later real-material validation instead of letting one certificate imply the
  others.

## Selected design

V36 preserves the V35 model, features, controls, ablations, training schedule,
losses, thresholds and resource ceilings. It changes only the experiment
execution boundary and allocates fresh synthetic role/truth identities.

The owner becomes one typed function with four capability-limited providers:

```text
sealed owner bytes
  + structural provider  -> zero-target census
  + surrogate provider   -> exact-path discarded rehearsal
  + fresh D0 provider    -> one-shot train/development
  + fresh H0 provider    -> frozen-candidate holdout only
```

The surrogate and official providers return the same concrete container types.
The owner emits a canonical stage trace. The rehearsal must traverse the same
stage IDs, callable identities and terminal publication path as D0; only role
identity, values and access receipts may differ. There is no `test_mode` branch
inside scientific or publication code.

After strict type/property checks and two full-count/full-step surrogate runs,
the owner/profile/dependency hashes form an execution seal. The fresh-target
provider refuses access unless that exact seal is presented. Any code change
invalidates the seal and returns to pre-access verification.

## Why this is scientifically admissible

The V35 artifacts exposed the exception type/stage and access counts only. They
contain no target values, weights, predictions, losses, metric values or model
ranking. V36 therefore gains no quality-selection surface from V35. Carrying
the hypothesis unchanged onto fresh roles tests the original question instead
of tuning against spent evidence.

The exception repair is an execution-contract change, not a scientific knob.
V36 still receives only one D0 and, after a complete pass, one H0. A metric
reject, resource reject, contract fault or unexpected exception closes V36.

## Rejected alternatives

- Repair and rerun V35: roles are spent after `10,800` target rows were opened.
- Infer quality from internal training completion: no metric artifact exists.
- Add only `__len__` to `RoleData`: it hides the symptom without proving the
  complete official path or clarifying which row count is intended.
- Add only mypy/pyright: dynamic imports, arrays and runtime shapes still need
  executable contracts.
- Keep a separate miniature I0 owner: branch drift caused the current loss.
- Jump directly to a topology/operator model: V35 never produced quality
  evidence, so that larger representation is not yet justified.

## Decision and stop rules

Proceed with Roadmap V36. First build and prove the sealed single-path owner;
then freeze fresh roles without changing scientific knobs. Do not open D0 until
the strict typing, mutation suite, stage-trace equivalence, full-count/full-step
rehearsal, terminal atomicity and resource envelope all pass twice exactly.

If the executable seal cannot be made independent of target values, stop. If
D0/H0 then rejects scientifically, close compact hybrid ML and research a
surface/operator representation. If execution faults after fresh access, close
V36 and redesign the shared experiment framework before allocating another
scientific generation.
