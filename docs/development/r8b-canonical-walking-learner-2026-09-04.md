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

## Remaining uncertainty

Walking V5 can release either foot but a complete alternating gait is not yet
demonstrated. Its unchanged instantaneous support reward may still favor
standing; the direct-CPU run must answer that before isolated gait-credit work.
The independent runtime PhysX replay failure appears to be bootstrap profile
closure: the PhysX-compatible fixture chooses grounded quantization while the
validator requires the core default profile. It is not fixed here and does
not alter motor-lab's separately validated native data plane.
