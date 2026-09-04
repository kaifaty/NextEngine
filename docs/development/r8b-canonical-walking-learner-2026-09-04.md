# Canonical walking learner — 2026-09-04

## Observation and discriminating evidence

External probe root:
`/home/kaifaty/NextEngine-training/r8b-walking-action-basis-v5/evidence/`.
These are development-tree counterfactuals, not admitted training runs.

| Probe | Observation | Conclusion |
|---|---|---|
| Loaded masses, COM, solver principal inertias, initial transforms, joint drives | Match canonical solver representation; stiffness/damping/armature of implicit drives are zero | No demonstrated mass/frame/duplicate-PD cause; source off-diagonal tensor is not the compiled solver principal representation |
| First substep | Initial positions/velocities/efforts agree; after one substep maximum joint position difference 625 urad and velocity difference 120,261 urad/s | Divergence begins in physical integration, before controller feedback differs |
| Both models lifted 2 m | First-step joint difference 3,187 urad, velocity 1,794,082 urad/s; post-step contact forces zero | Ground contact is not sufficient explanation. Pre-step contact buffer contains stale setup forces, not airborne contact evidence |
| Explicit contact offset 0.02 m | Loaded offsets now match CPU; tape still ends at tick 96 | Real descriptor omission, insufficient causal repair |
| Position iterations 16 instead of 8 | Canonical biomechanics compiler uses 16/4; corrected probe ends at tick 12 | Real mismatch, insufficient repair |
| Position 16, contact offset .02, linear/angular damping .05 together | Tape ends at tick 11 | Combined known corrections do not close safety/correspondence |

GPU applied actuation-force readback agrees with requested previous efforts to
float32 precision (maximum observed difference below 0.42 uNm). CPU airborne
near-contact records have positive separation and zero impulse. Do not claim
they demonstrate collision. Pinned solver-version differences remain plausible,
not proven. Native physics source and defaults are unchanged after probes.

Exact JSON SHA-256 values:

| File | SHA-256 |
|---|---|
| `gpu-substep-trace.json` | `842902aaa9a9d24ddf8eeed511a71845d4e70bd65e898e0f93e20aac0d7de770` |
| `gpu-airborne-trace.json` | `f3282e2fb5492956daa4595f1f4bab35798bf289d9f9a8bdf7b1a1f2142fc5aa` |
| `gpu-contact-offset-trace.json` | `14eefef89718f409197071103f6612e2c956df1d6d74cc3ab45b285e1d3f2ea4` |
| `gpu-solver16-trace.json` | `38fbd23bc3aee43c74997c16f96b3c57957ff39c2ab1f5b5521905dd0e6f4512` |
| `gpu-canonical-settings-trace.json` | `29252749ab9e5e3a56fec443ddb07fadc988fcc1795990240b70b73c77856b91` |

The airborne report's original paired metrics compare incompatible reset
origins and have no correspondence authority. The retained diagnostic tool now
emits null paired metrics and a failed canonical-reset gate for this override.
Temporary native substep traces are not a restorable training run closure.

## Research and decision

NVIDIA's [107.3 articulation stability guide](https://docs.omniverse.nvidia.com/kit/docs/omni_physics/107.3/dev_guide/guides/articulation_stability_guide.html)
supports checking actual mass/inertia, collisions, force/velocity limits and
solver iterations; it does not establish the cause of this model's divergence.
The [pinned Isaac Lab 2.3.2 environment implementation](https://github.com/isaac-sim/IsaacLab/blob/v2.3.2/source/isaaclab/isaaclab/envs/direct_rl_env.py)
was checked for action/substep/autoreset order. Accessed 2026-09-04.

Decision under [ADR-107](../architecture/adr/107-canonical-cpu-walking-learner.md):
stop speculative mirror tuning and test learning on canonical CPU physics with
CUDA neural computation. Keep safety and final walking criteria. Reconsider
the mirror path when a reproducible single cause closes the original failed
tape and standing control, not when a short prefix improves.

The new adapter's first development control passes 640 transitions including
29 safety terminals/resets, comparing every StepResult field with independent
native processes. Step-root digest:
`72b56fbf59849841840182dcccdbc111b6e5a58908fd7540410d8bb7eb817798`.
Evidence: `/home/kaifaty/NextEngine-training/r8b-canonical-walking-v1/adapter-control-01/run-manifest.json`.
Six focused tests cover reset/final-observation separation, invalid-batch
atomicity, input identity rejection/cleanup, shard roots and correct timeout
bootstrap and a synthetic PPO update through the installed library. This is
adapter correctness, not policy quality.

## Closed optimizer result and post-run diagnosis

Generation-02 TRAIN-1 on clean commit `bb8cbc8e` completed all 250 updates /
1,024,000 transitions. All declared artifact hashes and monotonic metric
counters verify. Average observed throughput was 4,636 samples/s on this
non-exclusive host (not a performance gate). Initial/final 20-update mean
episode lengths were 38.271/220.789; final-window action noise was 0.247584.

External root:
`/home/kaifaty/NextEngine-training/r8b-canonical-walking-v1/generation-02/runs/TRAIN-1/`.

| Artifact | SHA-256 |
|---|---|
| `run-manifest.json` | `06e71127cd1dd72bddd1c3c18427ea5ea8281ccf4ceb43e5e51f3823d946c70f` |
| final `model_249.pt` | `74e6d4793d233179e7852f1013f286059642c511894971c7fb9b0ed2dd84372f` |
| `evaluation.json` | `6df276e51c1ec2aea7a15ac50a220f8b706fa9fa9f1eec7dc15ac9210e33e3b6` |

**Walking FAILED.** Every final deterministic CUDA-policy/CPU-physics episode
ends at tick 299 with `terminal.fall`, forward displacement 0.824657 m,
velocity MAE 0.203355 m/s and **zero single-support ticks on either side**.
There is no final stop interval. The identical nominal-seed results do not
establish robustness. Do not relabel translation while falling as walking.

The report-only exploration matrix uses the same final checkpoint, CPU
inference, 16 slots, first episodes only and common random numbers (seed 2001).
Mean/noise/quarter-noise cases respectively average 301/227.625/217.4375 ticks;
**all have zero single-support samples**. CPU versus CUDA inference and batch
shape differ from the final matrix, so the 301/299 difference is not an exact
replay claim. Within the noise matrix those inputs stay fixed. Output:
`r8b-canonical-walking-v1/exploration-audit-01.json`.
This rejects noise reduction alone as a demonstrated foot-release remedy.

Nine report-only compositions of the successful single-foot template vary
swing amplitude 0.5/0.75/1 and duration 0.5/1/1.5. None completes the fixed
600-tick horizon or a qualified support switch. Full-amplitude variants reach
20/35/37 right-only support ticks before failing. Most failures occur before
the opposite-foot phase. Output `alternation-probe-02.json`; the added terminal
joint audit in `alternation-probe-03.json` reproduces every original case field
exactly. Several failures exceed ankle hard ROM by 1,053–4,994 urad. The
duration=1/amplitude=1 case instead has no final-state hard-ROM or velocity
excess, so its precise safety-envelope failure remains unresolved. These
finite failures do not prove that the body cannot walk.

`alternation-probe-01.json` is **INVALID**: the first prototype passed float
targets to a raw-int mirror helper, erasing the opposite-foot targets. It is
preserved, not used as evidence. Explicit Q1.30 conversion plus a regression
test fixes this report-only tool defect; -02/-03 use the corrected mirror.

Pinned primary-source recheck (accessed 2026-09-04): Isaac Lab 2.3.2
[H1 reward configuration](https://raw.githubusercontent.com/isaac-sim/IsaacLab/v2.3.2/source/isaaclab_tasks/isaaclab_tasks/manager_based/locomotion/velocity/config/h1/rough_env_cfg.py)
uses biped air/contact-duration credit, while its
[reward implementation](https://raw.githubusercontent.com/isaac-sim/IsaacLab/v2.3.2/source/isaaclab_tasks/isaaclab_tasks/manager_based/locomotion/velocity/mdp/rewards.py)
rewards sustained one-foot support up to 0.4 seconds, gated by a moving command.
It also uses yaw-frame velocity tracking and foot-slip costs. These are prior
art, not proof that copying those terms fixes this body.

Decision: the direct data plane is repaired, but coordinate weight transfer,
lift and placement before another full walking optimizer run. Localize the
first stance-ankle/safety-envelope failure on the exact return segment and
test a dense step/weight-transfer curriculum under a distinct environment
identity. Do not relax safety, repeat the noise test or increase PPO budget
without a new discriminator. No second optimizer budget is consumed/admitted.

Verification: full Linux `cargo run -p xtask -- host-check` passed (Rust 1.97.1).
The earlier explicit PhysX runtime replay bootstrap-closure failure is a
separate uncorrected path; full host-check does not waive it. No new Rust
physics/body/control changes were made in this direct-learner change.
Final focused Python verification passes 18 tests, including float/raw mirror
preservation; Ruff and `git diff --check` also pass. All optimizer processes
are finished; no recurring monitor or automatic successor run was created.

### Infrastructure correction before the first optimizer update

Generation-01 TRAIN-1 failed before any PPO update/checkpoint: the initial
adapter incorrectly required slot-sorted native responses. Production
`crates/motor/src/training/runner.rs` sorts reset and step outputs by
`(episode_ordinal, vector_slot)`. Asynchronous terminations reorder a 16-slot
batch. This is an adapter defect, not a physics failure or failed learner.
The original two-slot controls did not discriminate the ordering assumption.

The adapter now aligns by explicit slot and still rejects duplicates/missing
slots and mismatched episode ordinals. A control at the real 16-slots-per-shard
partition passes **5,120 transitions and 399 terminals/resets**, all raw fields
exact. Digest `4dcb9802ee6c91e1b210eb5a4cca9a958e286a6d385805386bfb499b0808fe86`;
external `r8b-canonical-walking-v1/adapter-control-02/run-manifest.json`.
Eight focused adapter/PPO tests and 17 combined tests pass, as do Ruff and
diff checks. Generation-01 remains preserved failed evidence. Generation-02
recloses the repaired infrastructure for the same seed/profile and previously
unexecuted optimizer budget; it is not a hyperparameter or reward retry.

Walking V5 can release either foot but a complete alternating gait is not yet
demonstrated. Its unchanged instantaneous support reward may still favor
standing; the direct-CPU run must answer that before isolated gait-credit work.
The independent runtime PhysX replay failure appears to be bootstrap profile
closure: the PhysX-compatible fixture chooses grounded quantization while the
validator requires the core default profile. It is not fixed here and does
not alter motor-lab's separately validated native data plane.
