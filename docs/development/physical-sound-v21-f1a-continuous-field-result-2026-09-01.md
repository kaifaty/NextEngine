# Physical sound V21 F1a — continuous-field tournament result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Decision | `SINGLE_RUN_RESOURCE_REJECT / NO_ARTIFACT / RUN_B_NOT_RUN / F1_QUALITY_UNOBSERVED` |
| Frozen protocol | [P1a](physical-sound-v21-p1a-continuous-residual-field-protocol-2026-09-01.md), SHA-256 `18465f95b423bc619d2987106332b753de77a65091210b9625b52235277e4bae` |
| Implementation commit | `95886ce06a7806511337a5314961c8d4be462313` |
| Implementation files | common `0f1a0b428ae2420e98ae422eae68a1a5654a196793e15d0f0c9d8ddc7b1eb99a`; model `760d4856446aa372681a7a06988bf4cc82316a1c5b975f2c6ff00bbbb3c80190`; tournament `6ab9d8e31a3caa1b668a4db9a795d9bf3ed3d5aded6068f2c36c1678ba8a22b1`; tests `9fc1eb3f084a6f29d308e097065492d9997bb66d459d4ded50ac0ce1a4eeb242` |
| Published tree | None; both official output paths are absent |
| Product effect | None; authored clips remain complete fallback and runtime ML remains unauthorized |

## Execution

The metadata-only implementation suite passed `8/8` before the value boundary.
The clean committed runner then started official run A with the exact frozen F0
control and target:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/
  v21-f1a-continuous-field-run-a
```

The process generated only the authorized train/development roles and entered
the preregistered seven-model, 1,500-update float64 tournament. It terminated
with the exact diagnostic:

```text
error: F1 runtime ceiling exceeded
```

The runner checks elapsed time before atomic publication. It abandoned staging,
and the target directory is absent. The independent run-B target is also absent.
Run B was not started because identical committed code would test the same known
resource failure rather than establish output-byte reproducibility.

At an elapsed-process observation of about 14 minutes, the Python worker used
`1,013,372 KiB` RSS and remained CPU-active. This is diagnostic only; the exact
gate result is the runner's elapsed time `>1,800.0 s`. The 4-GiB RSS limit was
not the observed blocker.

## Evidence boundary

- Train `1801…1824` and development `1901…1912` are now spent roles. They may
  document this failed execution but cannot choose an optimization, candidate,
  threshold or retry.
- No report, metric row, prediction, model artifact, tree digest, selected
  candidate or quality value was published or inspected.
- Test `2001…2012`, integration `2101…2112`, real, protected, waveform, force,
  source-body and network roles were not opened by the F1a implementation.
- F1a therefore says nothing about whether `ContinuousResidualOperatorV1`
  passes or fails the physical/remesh quality gates. It proves only that this
  execution shape misses its frozen resource gate.

## Attribution and decision

Static inspection identifies avoidable execution work without consulting F1a
quality values: each update performs many small network calls and reconstructs
unchanged analytic probe features/bases inside the optimization loop. Batched
forward evaluation and precomputation can preserve the candidate family,
per-view loss weights, differentiable Cholesky solves, optimizer steps and gates
while materially reducing dispatch and allocation overhead.

V21 closes as `RESOURCE_REJECT`. A successor may test only a preregistered,
algebraically equivalent execution on fresh role identities. It may not lower
1,500 updates, remove candidates, subsample views/probes, change thresholds or
reuse `1801…1912`. If that bounded successor also misses the resource gate, the
continuous family closes and a genuinely smaller F2 family requires new
research and new identities.
