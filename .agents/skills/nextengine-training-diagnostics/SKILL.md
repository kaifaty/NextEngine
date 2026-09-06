---
name: nextengine-training-diagnostics
description: "Diagnose NextEngine humanoid PPO learning failures, run integrity, physics or contact failures, and throughput from existing run artifacts; propose a focused experiment. Applies to humanoid TRAIN runs, not audio synthesis or generic model fitting."
---

# NextEngine training diagnostics

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Diagnose the first broken boundary before tuning the optimizer. Treat dashboards
and plots as views over closed run artifacts, never as the source of truth.

## Establish authority

1. Resolve the repository and read `AGENTS.md` plus the matching
   `docs/architecture/agent-routing.md` rows.
2. Read the current TRAIN stage and the exact environment/training profiles.
3. Read [references/diagnostic-playbook.md](references/diagnostic-playbook.md)
   when interpreting PPO, phase, contact or safety evidence.
4. Work read-only on the external run. Do not repair a manifest, metrics file
   or checkpoint in place.

## Produce a deterministic summary

Run from the repository root:

```text
python .agents/skills/nextengine-training-diagnostics/scripts/diagnose_run.py \
  <external run directory> \
  [--training-profile lab/profiles/<profile>.json] \
  [--window 20]
```

The script verifies declared hashes and counts, checks monotonic counters and
finite values, summarizes early/final windows, evaluates current PPO guardrails
and emits machine-readable alerts. An integrity failure outranks every metric
interpretation.

## Diagnose in order

1. **Artifact/lineage:** schema, status, generation/profile/input hashes,
   metrics count/hash, checkpoint hash and evaluation matrix identity.
2. **Environment/data:** deterministic reset, reference selection and phase,
   observation/action order, termination versus truncation, clip split and
   corpus lineage.
3. **Physics/safety:** non-finite facts, hard ROM, actuator limits, forbidden
   contacts, impact and exact action channel/phase. Reward tuning cannot repair
   a safety or correspondence defect.
4. **Optimizer:** KL relative to its frozen target, early-stop rate, clip
   fraction, pre-clip gradient norm, value loss, entropy and action standard
   deviation.
5. **Outcome:** compare the same fixed evaluation matrix, reference completion,
   episode length and failure distribution. Training return alone is not a
   quality oracle.
6. **Performance:** analyze samples/s and phase timings only after correctness.
   A faster divergent run is a failure.

Classify the primary cause as `Artifact`, `Body`, `Data`, `Environment`,
`Safety`, `Optimization`, `Evaluation` or `Performance`. Cite exact fields,
iterations, phases, clips and action channels.

## Select the next experiment

The following constraints apply when proposing a new humanoid TRAIN run.
Inspecting existing metrics or reporting a cause does not require creating a
new profile, preregistration document or experiment.

- Change one causal variable under a new hash-bound profile; do not mutate a
  frozen profile or reinterpret an old checkpoint.
- Use the smallest seed/clip/phase matrix that can falsify the diagnosis, then
  rerun the unchanged acceptance matrix.
- Debug before changing algorithms. Keep the current PPO baseline until an
  equal-budget alternative is justified.
- If the same blocker survives two coherent remediation cycles, or the next
  step depends on a new dynamics, reward-invariance, feasibility or solver
  claim rather than run evidence, stop tuning and use
  `$nextengine-mathematical-research` for the bounded discriminator first.
- Bind checkpoint selection, seeds and evaluation points in the governing
  profile or evaluation manifest before the run, as required by the TRAIN
  contract. Do not create a duplicate protocol document. Never claim acceptance
  from a best result selected at undeclared points.
- Treat a notebook or local MLflow/Trackio/TensorBoard dashboard as disposable
  analysis outside Git. Preserve the underlying JSON/manifest identity in any
  plot or alert.

If the user asked only for diagnosis, report the cause and experiment without
editing training code or launching a new run.

## Report claim boundaries

Separate pipeline execution, bounded overfit/curriculum progress, held-out
statistical quality, CPU/Isaac correspondence, runtime export parity and TRAIN
stage advancement. Do not let one result stand in for another.
