---
name: nextengine-training-runner
description: "Prepare, run, resume or monitor NextEngine humanoid TRAIN-5+ PPO training, evaluation and performance runs, including generation preflight and checkpoint compatibility. Applies to the humanoid Isaac workflow, not audio synthesis or generic model training."
---

# NextEngine training runner

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Prepare a run only from an exact admitted generation closure. Treat production
CPU PhysX as canonical, Isaac as an accelerated mirror, and the optimizer as a
private stochastic tool.
These launch and evidence requirements belong to the humanoid TRAIN workflow.
Do not apply them to unrelated model experiments.

## Resolve the workspace

1. Locate the repository root containing `AGENTS.md`, `docs/architecture/` and
   `lab/`.
2. Read `AGENTS.md` and the matching rows in
   `docs/architecture/agent-routing.md`.
3. Read the listed governing SPEC/ADRs in full. For the current reference
   tracker, include SPEC-35, ADR-069, ADR-070 and the active TRAIN stage in
   `docs/plans/2026-08-12-humanoid-motor-training-rebuild.md`.
4. Read [references/current-contract.md](references/current-contract.md) before
   selecting inputs or making a readiness claim.

Do not copy a path, version, hash or stage status from this skill when the
repository source says something newer.

## Classify the request

- **Inspect/preflight:** perform read-only validation. Do not initialize Isaac
  or create a run directory.
- **Training:** a request to start, continue or reproduce training supplies
  intent to spend compute within its stated scope. Resolve the run ID, budget
  and output root from that request and the admitted profile. Ask only for a
  material missing budget/scope decision; do not reconfirm an authorized run.
- **Initialization:** accept model weights only when a frozen profile explicitly
  binds `initialization.mode`, source profile hash and checkpoint hash.
- **Resume:** require a trainer-supported complete continuation contract for
  actor, critic, optimizer, scheduler, normalization and counters. Never call
  model-weights-only initialization a resume.
- **Evaluation:** use a frozen evaluation manifest/matrix and deterministic
  transformed mean. Do not add exploration noise or select a best undeclared
  checkpoint.
- **Performance evidence:** keep overrides report-only, use the declared
  exclusive-device/preflight method and make no policy-quality claim.

The current `isaac_reference_overfit.py` entry point supports an explicitly
bound initial checkpoint; it is not a general optimizer-resume interface.
This skill executes or validates a frozen experiment contract; it does not
select the next one-variable PPO experiment from run evidence, redefine the
canonical environment or select a new physical model. Route hash-closed run
diagnosis and next-experiment selection to `$nextengine-training-diagnostics`.
Route only a new mathematical/model-validity claim to
`$nextengine-mathematical-research`, and any resulting semantic change to
`$nextengine-architecture`.

## Run the fail-closed preflight

From the repository root, run:

```text
python .agents/skills/nextengine-training-runner/scripts/preflight_reference_run.py \
  --repository . \
  --generation-manifest <external generation-manifest.json> \
  --generation-index <external active-generation.json> \
  --training-profile lab/profiles/<training-profile>.json \
  --environment-profile lab/profiles/<environment-profile>.json \
  --descriptor <external descriptor.json> \
  --corpus-root <external corpus directory> \
  --gate-report <external TRAIN-4 gate-report.json> \
  --usd <external humanoid.usda> \
  --output-root <external generation>/runs/TRAIN-5 \
  --run-id <new-run-id> \
  [--initial-checkpoint <external checkpoint.pt>]
```

Require `ready: true` and exit status zero. Inspect every reported identity;
do not suppress a mismatch. The script is read-only and intentionally does not
probe CUDA, import Isaac, create directories or launch training.

## Launch conservatively

1. Verify that the source used by the run is clean and its commit is recorded.
   If unrelated workspace edits prevent this, prepare an isolated checkout of
   the intended committed source when possible. Do not stash, reset or discard
   unrelated changes or silently train on a different revision.
2. Confirm the output is below the selected external generation root and that
   the final run directory does not exist.
3. Run the exact repository entry point in the pinned Isaac environment. Pass
   paths explicitly; do not introduce trainer-owned reward, physics or body
   overrides.
4. Start with the smallest run that answers the requested question. A smoke or
   overfit run proves only its declared claim.
5. During a live run, monitor `run-manifest.json`, append-only `metrics.jsonl`,
   process health and free storage. Do not edit an active run.
6. On failure, retain the closed diagnostic prefix and read the stable error.
   Do not retry unchanged stochastic or deterministic failures until the cause
   is understood.

Never place datasets, USD, trajectories, logs, checkpoints, TensorBoard files,
notebooks or generated media in Git. A notebook, MLflow/Trackio view or
TensorBoard event is a secondary analysis projection, not lineage authority.

## Close and report the run

Verify that the final manifest closes its metrics and checkpoint hashes. State
separately:

- execution status and exact run identity;
- canonical/correspondence checks actually run;
- statistical or bounded-stage claim actually supported;
- checks that are `NotRun(reason)`;
- remaining blockers before the next TRAIN stage.

A completed run, an improved training reward or a saved checkpoint never by
itself advances TRAIN-5, proves runtime parity or closes R5.
