---
name: onnx
description: Export, validate, optimize, or run a model through ONNX when an ONNX artifact or interoperability path is explicitly requested or already exists. Do not use for ordinary model training, early experiments, or speculative deployment.
license: Apache-2.0
metadata:
  author: terminal-skills
  version: 1.0.0
  category: data-ai
  compatibility: 'python 3.8+, onnxruntime 1.16+, Linux/macOS/Windows'
  tags:
    - model-interoperability
    - optimization
    - inference
    - cross-platform
    - edge-deployment
---

# ONNX

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

The primary artifact is a verified model conversion or inference result. Do not
add ONNX before a candidate model exists and the target consumer needs it.

## Export

1. Inspect the source framework, concrete input/output tensors and target
   runtime. Reuse the repository's pinned versions and export path.
2. Choose the lowest opset supported by the required operators and target
   runtime; do not default to the newest opset without checking compatibility.
3. Use fixed shapes unless a real caller needs dynamic axes.
4. Put model payloads outside Git unless repository policy explicitly admits
   them.

## Validate before optimizing

After export:

1. run `onnx.checker.check_model`;
2. create an ONNX Runtime session with the target execution provider;
3. compare source and ONNX outputs on representative and boundary inputs using
   a task-appropriate tolerance;
4. report unsupported operators, shape mismatches and numerical drift directly.

An export command succeeding is not parity.

## Optimize only for a measured need

Simplification, graph optimization, quantization and alternate execution
providers are separate experiments. Apply one at a time and keep it only when a
target benchmark improves while the frozen parity/quality checks continue to
pass. Do not add mobile, GPU or cross-platform variants unless that target is
in scope.

For NextEngine, an ONNX file does not authorize runtime ML. Respect the current
offline-cooking/runtime-fallback contract unless a separate architecture change
is requested and accepted.
