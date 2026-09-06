---
name: mlflow
description: Add, inspect, or repair MLflow experiment tracking when the user asks for MLflow or the active project already uses it as part of the requested workflow. Do not introduce MLflow merely because a task trains a model or needs reproducibility.
license: Apache-2.0
metadata:
  author: terminal-skills
  version: 1.0.0
  category: data-ai
  compatibility: 'python 3.8+, mlflow 2.0+, Linux/macOS/Windows'
  tags:
    - experiment-tracking
    - model-registry
    - ml-lifecycle
    - deployment
    - pipelines
---

# MLflow

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Use MLflow as a projection over a working experiment, not as a prerequisite for
producing the model's primary artifact.

## Scope

- Reuse the project's existing tracking URI, experiment naming and artifact
  policy. Do not start a server, add a registry or change storage topology unless
  the user requested that operation.
- Do not install MLflow until a requested tracking action actually requires it.
- In NextEngine, repository profiles/manifests and exact result artifacts remain
  authoritative; MLflow is diagnostic unless an Accepted contract says
  otherwise.
- Keep datasets, checkpoints, generated media, credentials and local tracking
  stores outside Git.

## Minimal tracking

For an existing run, log only values needed to compare the declared experiment:

- run name and code/input identity already available from the project;
- causal parameters that differ between candidates;
- primary metric plus a small set of diagnostics;
- pointers to external artifacts rather than duplicate payloads.

Avoid autologging when it captures large, private or unstable payloads. Never
let MLflow run IDs or timestamps become deterministic selection inputs.

## Verify

Run the actual experiment first. Then verify that one local test run records the
expected parameters, metrics and artifact pointers without changing model
behavior. Show the primary model/media result separately from the tracking UI.

Model registry, remote tracking and serving are separate product decisions.
Implement them only when explicitly in scope and after the model itself has a
valid deployment path.
