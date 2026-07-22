# Local training capability

This document describes bootstrap capability, not a production physical-policy certification path. The normative decision is ADR-011 in `docs/architecture/adr/`.

## Mac developer-host lane

The isolated Python 3.12 project under `lab/` verifies a bounded path:

```text
generated deterministic 2-DoF fixture
→ tiny feed-forward training
→ ONNX opset 17 export and validation
→ 1,000-observation ONNX Runtime parity
```

Run from the repository root:

```text
uv run --project lab python -m next_lab doctor
uv run --project lab python -m next_lab smoke --device auto
```

The smoke uses four fixed seeds, eight environments per seed, 256 steps per environment, no network access at execution, and a maximum ten-minute budget. `auto` selects MPS when its capability probe passes; otherwise it records a diagnostic and uses CPU. Explicit `--device mps` fails when MPS is unavailable rather than silently falling back.

The gate requires 1,000 generated parity observations, finite outputs, and maximum absolute PyTorch/ONNX action error no greater than `1e-5`. The generated corpus and parity input hashes are deterministic. Exact Python packages and artifact hashes are recorded by `lab/uv.lock` and the local RunManifest.

All generated files are written under ignored `.local/training/train-mac-p0/`. They are local evidence and must not be committed.

## What this gate does not prove

`TRAIN-MAC-P0` does not prove PhysicsBackend correspondence, useful humanoid/creature behavior, policy safety, runtime performance, or `PhysicalCertified` eligibility. Isaac Lab/Isaac Sim is not installed on macOS.

Full training and promotion require a supported local Linux/NVIDIA workstation and `TRAIN-RTX-01`, followed by all POLICY/PHYS/TRAIN gates in the architecture packet. Until then:

```text
TRAIN-RTX-01 = AwaitingCapability
PhysicalCertified promotion = blocked
PrototypeFallback development = allowed
```

The future preflight command is:

```text
uv run --project lab python -m next_lab doctor --profile local-rtx
```

It exits with incompatibility status when the required Linux/NVIDIA capability is absent.
