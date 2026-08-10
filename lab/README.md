# Next Engine model lab

`lab/` is a non-authoritative training and correspondence lane. The canonical
Stage 0 execution plane remains `headless motor-lab` protocol v2 on CPU PhysX;
Isaac Lab is an optional GPU mirror and never supplies replay facts. Protocol
v2 starts without CLI profile options: the Python client sends one engine-known
profile ID, slot count and run root through `Create`, then uses partial
`Reset`, action-only `Step`, `Checkpoint`, `Restore`, `Ping` and `Close` with
monotonic request IDs.

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

1. Export the engine-owned descriptor to an external training store:

   ```text
   cargo run -p next_motor --example export_isaac_mirror --features physx-sdk
   ```

   The command writes JSON to stdout so the caller can place it in its
   configured store. The tracked golden fixture is emitted with `-- --golden`
   and is guarded by a Rust unit test.

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
   evaluate the normative 256 × 10-second sample floor:

   ```text
   python -m next_lab correspondence --cpu <cpu.npz> --gpu <gpu.npz> --store <external store>
   ```

`--store` may be replaced by `NEXTENGINE_TRAINING_STORE`. Commands reject a
store inside the repository. Generated USD, trajectories, reports, runs,
datasets and checkpoints are never source artifacts.

The DirectRLEnv implementation lives in `next_lab.isaac_env`. It shares the
23-channel ordering, four-substep cadence, integer ties-to-even PD/safety and
eight ordered reward component IDs with the Rust golden. Passing its
correspondence gate does not make GPU execution byte-exact or production
authoritative.
