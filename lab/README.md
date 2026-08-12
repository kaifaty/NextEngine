# Next Engine model lab

`lab/` is a non-authoritative training and correspondence lane. The canonical
execution plane remains `headless motor-lab` protocol v2 on CPU PhysX; Isaac Lab
is an optional GPU mirror and never supplies replay facts. Protocol v2 starts
without arbitrary CLI environment settings: the Python client sends one
engine-known profile ID, slot count and run root through `Create`, then uses
partial `Reset`, action-only `Step`, `Checkpoint`, `Restore`, `Ping` and `Close`
with monotonic request IDs.

## Canonical CPU trajectories

Build `next_headless` with the prepared PhysX feature, then record the canonical
flat-command profile into an external training store:

```text
python -m next_lab record-trajectories \
  --headless <next_headless executable> \
  --run-root <lowercase sha256> \
  --slots 16 --episodes-per-slot 4 \
  --store <external store>
```

The NPZ v2 recorder stores commands, post-clamp actions, observations, ordered
Q16 reward components/total, root/joint/contact facts, exact roots and separate
termination/truncation. `MotorLabClient.step` accepts raw integer arrays only;
`step_normalized` is an explicit finite/clamped ties-to-even adapter. Neither
client nor recorder writes inside the repository.

## Stage 0 mirror workflow

1. Export the engine-owned v2 descriptor to an external training store:

   ```text
   cargo run -p next_motor --example export_isaac_mirror --features physx-sdk
   ```

   The command writes JSON to stdout so the caller can place it in its
   configured store. The tracked v2 golden fixture is emitted with
   `-- --golden` and is guarded by a Rust unit test.

2. Validate the descriptor against the Rust golden and translate it to derived
   USDA:

   ```text
   python -m next_lab motor-mirror-check --descriptor <external descriptor.json>
   python -m next_lab translate-body --descriptor <external descriptor.json> --store <external store>
   ```

3. On a Linux NVIDIA training host, install the exact replaceable profile from
   `profiles/isaac-lab-physx-stage0.v1.json` and run:

   ```text
   python -m next_lab isaac-doctor
   ```

   The profile pins stable Isaac Lab 2.3.2 with Isaac Sim 5.1.0. Its bundled
   GPU PhysX remains a mirror of engine PhysX 5.9.0, not an identical build.

4. Record CPU and GPU `.npz` trajectories with the required canonical keys and
   evaluate the normative sample floor of 256 episodes x 600 motor ticks:

   ```text
   python -m next_lab correspondence --cpu <cpu.npz> --gpu <gpu.npz> --store <external store>
   ```

`--store` may be replaced by `NEXTENGINE_TRAINING_STORE`. Commands reject a
store inside the repository. Generated USD, trajectories, reports, runs,
datasets and checkpoints are never source artifacts.

The DirectRLEnv implementation lives in `next_lab.isaac_env` and selects the
standing, flat-command V1 or curriculum V2 profile from the engine descriptor.
For locomotion it derives each partial-reset command schedule on CPU, transfers
the 1,201 exact integer commands to the GPU and emits the unchanged 84-value
root-local layout. V1 evaluates ten ordered Q16 components; curriculum V2
evaluates eleven, including command-conditioned support. Quaternion ordering
and the engine/Isaac frame transform are explicit golden-tested operations.

`MODEL-MIRROR-P1` requires byte-exact commands, profile hashes and reward
component order, reward-total MAE at most `0.05`, joint RMSE at most `0.02 rad`,
root position at most `0.03 m`, velocity at most `0.05 m/s`, contact agreement
at least `98%`, and done-tick agreement at least `95%`. Passing correspondence
does not make GPU execution replay-authoritative or declare a trained policy.

## Training generation isolation

Every train/evaluate/view/reset/stability entry point requires an external
`--generation-index`. The index hash-closes exactly one generation manifest;
the manifest admits exact training-profile, environment-profile, descriptor
and derived-USD hashes. A legacy checkpoint or an input absent from that
closure fails with `INCOMPATIBLE_TRAINING_GENERATION` before simulator
creation. Run and evaluation output roots must be descendants of the selected
generation root.

`lab/scripts/isolate_training_generation.py` creates a new `prepared`
generation non-destructively. It inventories explicitly supplied retired roots,
hashes the bounded inventory, creates a generation directory containing only
`generation-manifest.json`, and atomically writes `active-generation.json`.
The initial manifest admits no training inputs; `TRAIN-1..4` must complete
before a new profile/artifact closure can activate it. Dataset, run and model
bytes remain outside Git.

## Retired Stage 0 PPO evidence

The tracked
`profiles/isaac-rsl-rl-rtx3080-locomotion-curriculum.v2.json` profile remains
immutable historical evidence for the retired pure-PPO line: 512 environments,
24 transitions per environment, 3,000 iterations/36,864,000 samples, fixed
seed 42, explicit CUDA device, entropy `0.005`, initial action noise `0.6` and
periodic checkpoints. Its old profile/checkpoints cannot resume or seed the
biomechanics rebuild generation. RSL-RL and Isaac remain private tools, while
CPU motor-lab remains canonical.

External operational wrappers pass the active generation index. Until a new
profile and body are admitted after `TRAIN-4`, invoking the old wrapper gives a
stable incompatibility diagnostic instead of starting PPO:

```text
/home/kaifaty/NextEngine-training/train-nextengine-poc.sh
```

After a new training profile and translated body are admitted, force a fall and
verify that every slot returns to the descriptor-authored root pose, zero root
velocity, configured joint pose, and zero joint velocity:

```text
/home/kaifaty/NextEngine-training/check-nextengine-reset.sh
```

This gate also requires the authored colliders to start at or above the ground
and the reset state to survive ten zero-action motor ticks inside explicit root,
joint and height-overshoot velocity bounds. The environment does not use the
already contact-perturbed PhysX startup state as a reset template.

Before a calibration run, stress one full episode across zero, full-range
random and current-policy actions. The check fails on the first non-finite
action, observation, reward or environment fact:

```text
/home/kaifaty/NextEngine-training/check-nextengine-stability.sh
```

The report includes the descriptor-derived PhysX effort/velocity limits,
reset counts and maximum observed joint/root velocities for every action source.

When the new generation is active, explicit overrides MAY provide a quick
integration check:

```text
/home/kaifaty/NextEngine-training/train-nextengine-poc.sh \
  --num-envs 64 --steps-per-env 8 --iterations 1 --save-interval 1
```

Every admitted invocation writes an external `run-manifest.json`, append-only
`metrics.jsonl`, TensorBoard events, and SHA-256-closed checkpoints. The
manifest binds the generation, profile, descriptor, generated USD, seed/run
root, package versions, Git revision, GPU memory preflight, and optional parent
checkpoint.
Training aborts before saving a non-finite policy or loss. Resume is accepted
only from a completed manifest with the exact same resolved training config:

```text
/home/kaifaty/NextEngine-training/train-nextengine-poc.sh \
  --resume <external-run-directory/model_N.pt>
```

Evaluate checkpoints separately and without exploration noise. The profile
starts evaluation at episode ordinal 96 so every result uses the full command
stage. Run all five held-out seeds independently so each result has its own
closed evaluation manifest:

```text
for seed in 1001 1002 1003 1004 1005; do
  /home/kaifaty/NextEngine-training/evaluate-nextengine-poc.sh \
    --checkpoint <external-run-directory/model_N.pt> --seed "$seed"
done
```

Inspect the newest hash-closed checkpoint as one continuously simulated humanoid
in a local browser-based 3D viewport:

```text
/home/kaifaty/NextEngine-training/view-nextengine-training.sh
```

Isaac remains the headless physics/policy process; a localhost-only WebGL page
renders its streamed body facts without depending on the native RTX GUI. The
viewer follows the humanoid, paces playback at the 60 Hz motor rate and prints
its current right/forward/yaw command, reward and root height. Pass
`--checkpoint <external-run-directory/model_N.pt>` to inspect a specific closed
checkpoint, `--no-open-browser` to print the local URL without opening it, or
`--unthrottled` to disable real-time pacing. The viewer is a non-authoritative
checkpoint inspection tool; it does not turn Isaac execution into replay facts
and does not mutate training weights.

The browser lists every hash-closed checkpoint from the selected run. Use the
left/right buttons or the iteration selector to load another checkpoint and
reset the live episode without restarting Isaac. This navigates model history;
it is not random-access replay inside one simulated episode.

An evaluation records returns, episode lengths, termination versus truncation,
and every ordered reward component. Its episode count must be a positive
multiple of `num_envs`: each environment slot contributes exactly the same
number of episodes, so fast-failing slots cannot bias the report. The manifest
closes the evaluator Git revision and records every slot's episode count and
lengths. A completed training run proves that the pipeline works; only held-out
multi-seed evaluation can support a policy-quality claim. Checkpoints,
evaluation manifests, and logs remain outside Git.
