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

An optional CUDA workstation probe remains available:

```text
uv run --project lab python -m next_lab doctor --profile local-rtx
```

It reports the detected Linux/CUDA/VRAM environment for developer convenience.
An unavailable RTX machine affects only RTX experiments and never blocks normal
engine work.
