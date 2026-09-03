# NVIDIA Kimodo for Next Engine motion training — research report, 2026-09-03

## Question and claim boundary

This report asks where NVIDIA Kimodo can improve Next Engine humanoid motion
training without changing the current authority, determinism, safety or
licensing boundaries. It compares the 2026 Kimodo release with the active R8b
standing/forward-start-stop package and the stopped TRAIN-4 reference lineage.

Repository authority reviewed: the [R8b task state](task-state/r8b-first-learned-locomotion.md),
[stopped TRAIN-4 state](task-state/humanoid-motor-training.md),
[roadmap](../roadmap.md), [SPEC-14](../architecture/14-physical-archetypes-motor-skills-and-policy-lifecycle.md),
[SPEC-27](../architecture/27-motor-observation-action-and-deterministic-inference.md),
[SPEC-34](../architecture/34-model-training-environments-trajectories-and-consolidation-lifecycle.md),
[SPEC-35](../architecture/35-deterministic-humanoid-training-substrate.md) and
[ADR-065](../architecture/adr/065-curriculum-flat-command-locomotion-profile.md).

The report is a research decision, not an implementation or training result.
No Kimodo checkpoint, generated motion, dataset or training artifact was
downloaded; no Next Engine run or ProductCheck was started.

## Executive conclusion

Use Kimodo later as an **out-of-process, offline candidate-motion generator**.
Do not put it in the runtime motor path, do not use it to control PhysX, and do
not add it to the active R8b standing/forward-start-stop runs.

The best bounded use is a new post-foundation experiment:

1. generate short SOMA motions from narrow prompts plus root-path, pose and
   contact constraints;
2. freeze each raw NPZ as untrusted, hash-addressed input in the external
   training store;
3. deterministically retarget and quantize it to the exact Next Engine
   `BodySchemaV2` descriptor and 60 Hz reference representation;
4. admit only candidates that pass a new optimizer-free kinematic and CPU
   PhysX feasibility profile;
5. only after that compare command PPO with and without a separately
   authorized Kimodo-reference initialization under the same final R8b-style
   CPU held-out gates.

This can eventually help with starts, stops, turns, recovery, combat,
interaction transitions and style diversity. It is low-value for the first
learned standing checkpoint: Kimodo generates kinematic poses, while standing
success is a feedback-control and contact-stability problem.

## What Kimodo is — and is not

[Kimodo](https://research.nvidia.com/labs/sil/projects/kimodo/) is a 282M
parameter, two-stage transformer diffusion model for offline kinematic motion
generation. The released SOMA models generate at 30 fps for at most 10 seconds
per prompt and accept text, full-body keyframes, sparse joint
positions/rotations, 2D waypoints or paths, heading and foot-contact
constraints. The authors trained the largest variant on 700 hours of optical
motion capture and show a downstream G1 example where ProtoMotions trains a
separate physics policy to track generated demonstrations
([technical report](https://research.nvidia.com/labs/sil/projects/kimodo/assets/kimodo_tech_report.pdf)).

That distinction is decisive:

- Kimodo produces **kinematic reference candidates**;
- a tracking or command policy still has to learn physically valid feedback;
- CPU PhysX remains Next Engine's canonical dynamics and admission authority;
- text is an offline authoring input, never a runtime physical command;
- generated contact labels are hints, not authoritative contact facts.

NVIDIA also documents residual foot skating, imperfect constraint hits,
single-skeleton checkpoints, weak G1 post-processing and sequential rather
than jointly optimized multi-prompt transitions
([best practices and limitations](https://research.nvidia.com/labs/sil/projects/kimodo/docs/key_concepts/limitations.html)).
Kimodo quality metrics therefore do not replace Next Engine ROM, collision,
impact, effort, contact, replay or held-out policy gates.

## Fit with the current Next Engine program

| Boundary | Current Next Engine fact | Kimodo fit |
| --- | --- | --- |
| Active R8b | Standing first, then only `0..0.75 m/s` forward start/stop; no motion corpus/reference tracker | **Do not integrate now.** It would add a second experimental axis before foundation stability is known. |
| Runtime authority | Exact 23-action residual targets, fixed PD, immutable policy artifacts and procedural fallback | **No runtime dependency.** Kimodo never emits `WorldCommand`, motor actions or authoritative contacts. |
| Training authority | Production headless CPU PhysX admits final trajectories; Isaac is an optional mirror | **Compatible only as data input.** Generated motion must pass CPU admission after retarget. |
| Existing corpus path | CMU ASF/AMC is explicitly mapped, retargeted and quantized to `RetargetedClip` at 60 Hz | **Useful adapter target.** Reuse the engine-owned output representation, not the CMU source assumptions or hashes. |
| Stopped TRAIN-4 | R141 is invalid and the whole reference lineage is `STOP_NO_RETRY` | **No inheritance.** Kimodo cannot relabel, repair or bypass R123–R141; it needs a new source, retarget and tracker lineage. |
| Future motor plan | Synthetic-generated trajectories, specialist teachers, reference tracking, distillation and structured motion priors are Proposed | **Good future candidate.** Evidence must be earned before an architecture or roadmap promotion. |

No accepted architecture or roadmap wording needs to change for this finding.
The current R8b next action remains its no-training preflight.

## Model and format choice

Start a future pilot with `Kimodo-SOMA-RP-v1.1`, conditional on the licensing
gate below.

| Variant | Disposition | Reason |
| --- | --- | --- |
| `SOMA-RP-v1.1` | Preferred quality candidate | NVIDIA recommends the 700-hour RP models; SOMA NPZ exposes global positions plus local/global rotations, making an explicit semantic retarget possible. |
| `SOMA-SEED-v1.1` | Provenance comparator, not automatically safer | The checkpoint has the same NVIDIA model license, but direct BONES-SEED access is gated by a separate eligibility-restricted dataset agreement. |
| `G1-RP/SEED-v1` | Reject for direct ingestion | Its 29 actuated values, robot proportions, MuJoCo z-up frame and contact/post-process behavior do not match the Next Engine 23-action body. |
| `SMPLX-RP-v1` | Reject for the first pilot | It uses a different NVIDIA R&D license and adds another retarget path without solving the authority problem. |

The [Kimodo NPZ format](https://research.nvidia.com/labs/sil/projects/kimodo/docs/user_guide/output_formats.html)
contains `[T,J,3]` joint positions, local/global rotation matrices, four foot
contact channels, smoothed and pelvis-root trajectories, and heading. Current
SOMA exports use 77 joints even though the model operates on 30. The format is
closer to the information Next Engine needs than G1 CSV, but it is not a
drop-in `RetargetedClip`.

Required adapter work is explicit:

- pin a SOMA-77 joint-name and parent table; map semantic joints to the frozen
  23-DoF descriptor without name guessing;
- verify handedness and transform Kimodo's y-up, +z-forward coordinates into
  the exact Next Engine coordinate profile;
- solve root pose and target joint rotations against the target morphology,
  then apply descriptor-owned ROM, velocity and collision rules;
- resample 30 Hz to 60 Hz using one frozen rotation/translation algorithm;
- recompute velocities, center of mass, effectors, phase and contacts from the
  admitted target trajectory; never trust `foot_contacts` as physical truth;
- quantize with the existing micrometre, microradian and canonical-sign Q1.30
  rules before deterministic serialization;
- keep raw output, converted output, previews and all heavy artifacts outside
  Git in the external training store.

Generation itself need not be cross-GPU bit-identical. The immutable raw NPZ
hash becomes the input fact; the engine-owned conversion from that byte string
must be repeatably byte-identical.

## Proposed post-R8b experiment

Name the research lane `KIMODO-MOTION-P0`; do not call it R142 and do not attach
it to TRAIN-4.

### Gate K0 — legal, supply-chain and identity closure

Before installing or generating:

- pin the Kimodo repository to observed commit
  `1aece8c124d73d255ceff5086d983b844c9f4e94` or a later explicitly reviewed
  commit, never an unpinned `pip install` from `main`;
- pin exact checkpoint revision and file SHA-256, model card and license
  snapshot, container/lockfile, CUDA/PyTorch versions, SOMA assets and every
  transitive component used by the generator;
- record canonical prompt bytes, constraint JSON bytes, duration, random seed,
  sampler and all guidance/post-processing settings;
- obtain and cache the gated Meta-Llama-3-8B-Instruct text encoder under its
  own terms; generation must not depend on live network access;
- keep credentials out of manifests, logs and the repository.

The [Kimodo code is Apache-2.0 while checkpoints and data are licensed
separately](https://github.com/nv-tlabs/kimodo). The released SOMA checkpoints
use the [NVIDIA Open Model License](https://www.nvidia.com/en-us/agreements/enterprise-software/nvidia-open-model-license/),
which permits commercial use and says NVIDIA claims no ownership in outputs,
but makes the user responsible for them and imposes additional conditions.
That is not enough to skip Next Engine's artifact-specific provenance review.

Do not download BONES-SEED for this pilot. Despite being called “open” in the
model card, its [dataset agreement](https://bones.studio/info/seed-license)
limits no-fee access to academic users and qualifying startups below a revenue
threshold, restricts redistribution, requires attribution and has termination
conditions. If direct dataset use is later desired, it needs a separate
eligibility and distribution decision.

### Gate K1 — frozen candidate corpus

Preregister a small matrix, for example eight narrow start/walk/stop
prompt-and-constraint templates times four seeds. Keep each sample within the
10-second limit and use constraints, not prose alone, to bind:

- neutral first and final poses;
- a 2D forward root path matching one of the R8b speed bands;
- explicit start, steady-walk and stop timing;
- sparse left/right sole-contact intent.

Split templates and seeds before inspecting results. Generated variants from
one model are not independent human subjects, so they may augment training but
must not become the final held-out oracle. Reject individual failures; never
repair the pass threshold after viewing them.

### Gate K2 — deterministic retarget and physical admission

Create a new source profile and converter that emit the existing engine-owned
reference arrays and complete source provenance. Do not reuse the CMU profile
ID, its license assertions, or any R123–R141 cache.

The admission profile must freeze exact bounds before generation and cover at
least:

- finite values, stable dimensions, joint order and source/output hashes;
- hard/soft ROM and per-joint velocity reserve;
- target-collider and ground penetration;
- stance slip, sole orientation, support state and non-sole contact;
- root/CoM discontinuity and start/stop boundary continuity;
- fresh-scene CPU PhysX contact, impact, effort/power/work and reset behavior;
- byte-identical conversion and same-input overlap witnesses.

No generated clip becomes `training_authorized` merely because it looks good,
scores well on Kimodo's benchmark or loads in ProtoMotions.

### Gate K3 — smallest learning utility test

Only after K0–K2 and a separate roadmap/architecture authorization, compare:

1. the clean command-only standing-parent → forward PPO baseline;
2. a new Kimodo-reference tracking or supervised initialization → the same
   command PPO environment.

Freeze model architecture, optimizer, randomization, command schedules, final
CPU held-out seeds and total simulation/optimization budgets. Report at least
five independent training seeds, time and samples to the first passing
checkpoint, final success distribution, safety events and canonical replay
roots. Kimodo is useful only if it improves sample efficiency or motion quality
without weakening the complete-episode standing and forward-stop gates.

If no candidate survives K2, stop. If K3 does not beat the clean baseline,
retain Kimodo only as an animation-authoring aid and keep command PPO as the
motor route.

## Expected value by use case

| Use case | Value | Timing |
| --- | --- | --- |
| First learned standing | Low | Do not use; prove feedback stability directly. |
| Forward start/stop references | Medium | First bounded post-R8b pilot because path and endpoint constraints are useful. |
| Turns, recovery and style diversity | High if admission yield is acceptable | After foundation gates; these are sparse or expensive in the current corpus. |
| Combat and object interactions | Potentially high | Later, after typed task/contact constraints and environment-aware validation exist; Kimodo itself is not scene-aware. |
| Runtime text-to-motion control | Wrong architecture | Text stays offline; runtime consumes typed intent/chunks and immutable policies. |
| Replacing CPU PhysX or the procedural fallback | None | Never. |

## Rejected alternatives

- **Add Kimodo references to R8b now.** This destroys the diagnostic value of
  the foundation-first run and contradicts its declared no-reference scope.
- **Treat G1 CSV as close enough to the Next Engine body.** Joint count,
  morphology, frame convention and actuator semantics differ.
- **Feed raw SOMA rotations into PPO.** This creates implicit retarget, ROM and
  contact authorities in trainer code.
- **Use Kimodo foot contacts as labels of physical truth.** They are generated
  kinematic annotations and may coexist with skating or penetration.
- **Resume the stopped TRAIN-4 tracker with new clips.** R141 failed a lineage
  identity condition; changing the data source does not make that lineage
  admissible.
- **Ship Kimodo, Llama or generated corpora with the engine.** The useful
  boundary is an optional private/offline tool; the game remains correct and
  playable without it.

## Open uncertainties and reconsideration conditions

The report does not establish:

- how many SOMA candidates survive exact 23-DoF retarget and CPU admission;
- whether Kimodo clips improve PPO sample efficiency over the existing CMU
  corpus or command-only standing initialization;
- cross-machine reproducibility of generation, which is deliberately replaced
  by raw-byte freezing rather than assumed;
- whether the model, text encoder and generated-output lineage passes the
  project's final licensing/provenance review;
- the storage, GPU-time or adapter cost on the actual training host.

Reconsider `KIMODO-MOTION-P0` only after both current R8b foundation gates pass,
or after an explicit roadmap decision changes that order. The smallest next
action then is K0 plus a converter-only spike on a few frozen NPZ files — not a
PPO run.

## Primary sources

- [Kimodo project page](https://research.nvidia.com/labs/sil/projects/kimodo/)
- [Kimodo technical report](https://research.nvidia.com/labs/sil/projects/kimodo/assets/kimodo_tech_report.pdf)
- [Official code and model matrix](https://github.com/nv-tlabs/kimodo)
- [Kimodo installation requirements](https://research.nvidia.com/labs/sil/projects/kimodo/docs/getting_started/installation.html)
- [Kimodo limitations](https://research.nvidia.com/labs/sil/projects/kimodo/docs/key_concepts/limitations.html)
- [Kimodo output formats](https://research.nvidia.com/labs/sil/projects/kimodo/docs/user_guide/output_formats.html)
- [SOMA-RP-v1.1 model card](https://huggingface.co/nvidia/Kimodo-SOMA-RP-v1.1)
- [NVIDIA Open Model License](https://www.nvidia.com/en-us/agreements/enterprise-software/nvidia-open-model-license/)
- [BONES-SEED dataset card](https://huggingface.co/datasets/bones-studio/seed)
- [BONES-SEED dataset agreement](https://bones.studio/info/seed-license)
- [ProtoMotions](https://github.com/NVlabs/ProtoMotions)
