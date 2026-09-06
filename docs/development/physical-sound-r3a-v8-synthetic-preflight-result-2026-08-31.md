# Physical sound R3A V8 — explicit-modal synthetic preflight result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `READY_FOR_FRESH_REAL_MODAL_PROTOCOL` |
| Preregistration commit | `3275edba` |
| Implementation commit | `9187e26e` |
| Manifest SHA-256 | `b2bb931d38cfa53b49f99338e81f4d49b986d45c18b46e9cdbe79d70fcdb6934` |
| Report SHA-256 | `391854fe141d5e3e3cd5c71b828ee4f0377475f56b7cfae2d5bc1f8b82c63b78` |
| Product effect | None; real quality, R3B, atlas and runtime remain unauthorized |

## Question

Can a materially different explicit-modal neural substrate pass two controls
before any fresh real development audio is selected or read?

1. Recover known frequency, damping and gain through the differentiable modal
   renderer from bounded perturbed initialization.
2. Learn held boundary `3 x 20` mode-shape components from only `20%`
   farthest-point context positions on two published FEM objects, and beat
   frozen constant and nearest-position controls.

The complete hypothesis, sources, thresholds and stop/go boundary were frozen
before this run in the [V8 rebaseline](physical-sound-v8-explicit-modal-neural-rebaseline-2026-08-31.md).

## Exact inputs

Source: [NISR dataset](https://huggingface.co/datasets/BumsooKim00/nisr-dataset)
revision `20368791bcd7829e04ae3eb07c10aa0bb370e38a`.

| Object | Points | Context | Query | Source SHA-256 |
| --- | ---: | ---: | ---: | --- |
| Glass object `1` | `501` | `101` | `400` | `6548be1d819cef4c010aed787d5277e0c4de8021fa901b5345ad1dac115f36aa` |
| Glass object `2` | `2810` | `562` | `2248` | `d527053ef8c06f96ba4435a2e46ac03cbc690138f37dd2b9273f5496828322b9` |

Both inputs are opened synthetic development controls. Neither is a hidden
holdout and neither can provide real acoustic quality evidence.

Environment identity:

- Python `3.11.15`;
- NumPy `1.26.4`;
- SciPy `1.11.4`;
- PyTorch `2.12.1+cu130`, forced deterministic single-thread CPU execution;
- no source payload, prediction, model weight or generated WAV was added to
  Git.

## Repetition

Two independent output directories were produced from the committed runner.
Both runs emitted the same manifest and report hashes shown above. Each run
also repeated its modal optimization and every object field internally;
`modal_exact_repeat=true` and `field_exact_repeat=true`.

## Modal recovery

| Metric | Observed | Frozen maximum | Result |
| --- | ---: | ---: | --- |
| Waveform MSE | `2.000678189118631e-12` | `1e-6` | `PASS` |
| Maximum frequency error | `1.505046653734882e-8 cents` | `1 cent` | `PASS` |
| Maximum relative damping error | `1.1559082836787304e-5` | `0.02` | `PASS` |
| Maximum relative gain error | `6.455083159662145e-6` | `0.02` | `PASS` |

This is a favorable-initialization recovery control. It proves the renderer,
parameter constraints and optimizer are coherent; it is not blind modal
extraction from real audio.

## Neural mode-shape field

All RMSE values use context-only per-component normalization.

| Object | Neural RMSE | Nearest RMSE | Constant RMSE | Neural / nearest | Frozen maximum | Result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `1` | `0.2182470560` | `0.8314744234` | `0.9739170671` | `0.2624819836` | `0.50` | `PASS` |
| `2` | `0.0484857187` | `0.4998642206` | `1.0193434954` | `0.0969977779` | `0.50` | `PASS` |

Mean component-wise query prediction standard deviation is `0.9690534472` and
`1.0178934336`, both above the frozen `0.01` anti-constant gate. Predictions
and parameter tensors repeat by exact SHA-256 within and across runs.

## Access audit

| Boundary | Observed |
| --- | --- |
| Fresh real V8 waveform samples decoded | `0` |
| Prior V5 development waveform samples decoded | `0` |
| Sealed waveform samples decoded | `0` |
| Method holdout accessed | `false` |
| Admission shadow accessed | `false` |
| Real quality credit | `false` |
| R3B authorized | `false` |
| Runtime/public contract changed | `false` |

## Interpretation

The result falsifies the narrow claim that an explicit modal renderer or
coordinate neural mode-shape field is not trainable on exact published FEM
truth. It also shows a substantial held-position advantage over the frozen
nearest control on these two opened synthetic objects.

It does not show that:

- frequencies/damping can be initialized reliably from real recordings;
- one global exponential damping law is sufficient;
- synthetic FEM mode shapes predict microphone waveforms;
- the field beats KNN on real impact positions;
- glass sounds perceptually correct;
- a representation, validator or engine asset should be promoted.

## Decision and next action

Return `READY_FOR_FRESH_REAL_MODAL_PROTOCOL`. The only newly authorized work is
to freeze a fresh [ObjectFolder Real](https://objectfolder.stanford.edu/objectfolder-real-download)
fit/development protocol before reading its audio:

1. exact source archive/object hashes and canonical listener/excitation policy;
2. source/object/contact-disjoint roles with no prior REALIMPACT development
   object reused for V8 selection;
3. analytic STFT/Hilbert modal initializer, global damping, one bounded
   filtered residual and at most one preregistered spatial-damping capacity;
4. nearest/KNN, modal-only and residual ablations plus unchanged acoustic hard
   endpoints;
5. a fit-only stop before development if the representation cannot reconstruct
   its authorized fit recordings.

Authored clips remain authoritative. R3B stays closed until fresh real
development and one later source-disjoint representation holdout both pass.

## Checks

- focused V8 unit suite: `7/7 PASS`;
- Python bytecode compilation: `PASS`;
- two independent external preflight runs: `PASS / byte-identical`;
- documentation whitespace and local-link checks: `PASS` at preregistration.
