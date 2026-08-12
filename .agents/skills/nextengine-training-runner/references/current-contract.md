# Current reference-training contract

Use this reference only after reading the repository authority routed from
`docs/architecture/agent-routing.md`. Values in tracked profiles and accepted
external manifests outrank prose here.

## Authority and ownership

| Concern | Authority | Consequence |
| --- | --- | --- |
| Runtime physics | CPU PhysX/headless motor-lab | Isaac remains a bounded accelerated mirror |
| Body and actuation | `BodySchema` plus compiled descriptor | No trainer-owned joint order, limits, gains or safety |
| Reference task | frozen environment profile | Observation, action, reward, terminal and reset meanings are fixed |
| Optimization | frozen training profile | Hyperparameters, seed, budget and evaluation matrix are inputs |
| Data | external corpus manifest and TRAIN-4 gate report | Git paths and inferred directory names are not lineage |
| Run identity | generation manifest, repository commit and closed run manifest | A dashboard or filename is not evidence |

The current reference environment applies a normalized residual to the
reference joint target, then engine safety and fixed PD. It is neither raw
torque control nor a trainer-specific drive.

## Required closure before launch

The selected generation must admit the exact file or embedded identity of:

1. frozen reference environment profile;
2. frozen training profile;
3. compiled descriptor file;
4. biomechanics USD;
5. validated locomotion corpus manifest;
6. advancing TRAIN-4 gate report;
7. initial checkpoint, when the training profile declares initialization.

The active-generation index closes the generation *file* SHA-256. The
generation and corpus also contain canonical embedded hashes calculated by
their repository-owned producers with their own hash field omitted. The corpus
canonical form uses UTF-8 JSON plus a trailing newline; generation/index use
their own compact JSON form. Do not interchange file and embedded hashes.

## Initialization is not resume

The current reference-overfit entry point can load a profile-bound
`model-weights-only` checkpoint. Actor/critic parameters are initialized, while
optimizer state and counters are newly constructed according to the target
profile. Describe this as initialization.

A true resume requires an explicit contract for every continuation-relevant
state, including optimizer/scheduler, normalization, RNG/counters and the exact
resolved configuration. Do not infer that contract from a `.pt` suffix or from
the generic Isaac trainer's `--resume` option.

## Run classes and claims

| Run class | Minimum purpose | Claim ceiling |
| --- | --- | --- |
| smoke/sanity | import, reset and bounded stepping | pipeline execution only |
| tiny overfit | prove gradients can improve one narrow task | declared clip/phase stage only |
| curriculum stage | test one frozen curriculum transition | declared stage and matrix only |
| held-out evaluation | fixed, predeclared episodes and final checkpoint | statistical claim in that matrix |
| performance evidence | exclusive-device, frozen report-only overrides | throughput only |

Completion, reward improvement or a checkpoint file does not prove CPU/Isaac
correspondence, portable runtime parity, TRAIN advancement or closure of R5.

## Operational boundaries

- Keep corpus bytes, USD, checkpoints, metrics, logs, videos, notebooks and
  dashboards in the external generation store.
- Never mutate or reuse an existing run directory.
- Record a clean repository commit. Do not hide unrelated changes with reset or
  stash.
- Treat deterministic failures as evidence. Retrying unchanged inputs is not a
  diagnosis.
- For live monitoring, read append-only artifacts without modifying the active
  run.
