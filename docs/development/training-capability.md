# Local training smoke

The isolated Python 3.12 project under `lab/` is an optional developer tool. It
checks one small path:

```text
generated deterministic 2-DoF fixture
→ tiny feed-forward training
→ ONNX opset 17 export and validation
→ 1,000-observation ONNX Runtime comparison
```

Run from the repository root:

```text
uv run --project lab python -m next_lab doctor
uv run --project lab python -m next_lab smoke --device auto
```

`auto` uses MPS when its probe succeeds and otherwise uses CPU. Explicit
`--device mps` fails if MPS is unavailable.

The smoke uses fixed generated inputs, requires finite output, checks that
training improves the fixture loss and compares PyTorch/ONNX actions with a
maximum absolute error of `1e-5`. Generated files live under ignored
`.local/training/smoke/`.

This proves only that the local train/export/inference toolchain works. It does
not validate gameplay behavior, physics correspondence, policy quality or
runtime performance. It produces only local diagnostics and ignored artifacts,
not a product or release status. It does not replace `fast`, `play`,
`persistence-replay` or `content-package`; `platform` and `performance` remain
conditional.

## Canonical humanoid environment

The engine also exposes two engine-owned motor-lab protocol v2 profiles:
`nextengine.motor.env.humanoid-standing.v1` and
`nextengine.motor.env.humanoid-flat-command.v1`. The latter is the canonical
CPU PhysX authority for fixed 23-DoF flat locomotion trajectories. Its command
schedule, observation/action layouts, reward/termination profiles, RNG
derivation and correspondence profile are manifest-hash bound; callers cannot
override reward or physics settings.

Use `python -m next_lab record-trajectories` as documented in `lab/README.md`.
The configured store must be outside the repository. The recorder writes NPZ
v2 with exact commands, applied actions, Q16 rewards, observations,
root/joint/contact facts and separate termination/truncation. This environment
is suitable for local CPU data collection and algorithm experiments; it is not
a trained policy or a runtime learned evaluator.

The Isaac mirror is non-authoritative until `MODEL-MIRROR-P1` passes on the
pinned Linux NVIDIA profile. A Windows-only golden test proves descriptor,
schedule, PD and frame-transform equivalence but does not replace GPU
correspondence.

An optional CUDA workstation probe remains available:

```text
uv run --project lab python -m next_lab doctor --profile local-rtx
```

It reports the detected Linux/CUDA/VRAM environment for developer convenience.
An unavailable RTX machine affects only RTX experiments and never blocks normal
engine work.
