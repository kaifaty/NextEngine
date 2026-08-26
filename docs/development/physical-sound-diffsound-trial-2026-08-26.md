# DiffSound glass calibration trial — 2026-08-26

## Result

DiffSound ran end to end on the local RTX 3080, including its real-audio
damping fit, second-order tetrahedral modal solve, differentiable oscillator,
loss and backward pass. This establishes tool compatibility for an isolated
offline laboratory. A subsequent product-owner audition judged its 16-step
prediction recognizably glass-like and better than the earlier local glass
renders, so it is retained as a promising perceptual direction.

The bounded glass proxy moved its modes and reduced DiffSound's reported RMSE
by only `0.243610%`, while its optimization loss increased by `0.096294%`.
Next Engine's independent evaluator then measured `45.337802 dB`
gain-matched multiresolution spectral RMSE, modal-assignment cost `2.174618`,
and a `1,469.665 Hz` spectral-centroid overshoot. Those measurements establish
poor reproduction of the earlier synthetic target, not poor glass identity.
The proxy remains invalid as physical material identification but is no longer
rejected as a perceptual calibration direction.

No DiffSound source, dependency, model weight, target recording or generated
audio is added to the repository. SPEC-45 remains `Proposed`; the clip
fallback, frozen wood-B and provisional Glass-H experiment remain unchanged.

## Source and use boundary

- Upstream: [TechnetiumMan/DiffSound](https://github.com/TechnetiumMan/DiffSound)
- Evaluated commit: `3a0be14b8bfa95dcbcd98f3c233e981c713d021b`
- Paper/project context: [DiffSound, SIGGRAPH 2024](https://hellojxt.github.io/DiffSound/)
- The evaluated Git tree contains no root or depth-two `LICENSE`, `COPYING` or
  `NOTICE` file. The trial therefore treats the repository as inspectable
  research code only, not as code that may be copied, integrated or
  redistributed.
- All mutable inputs and outputs stay under
  `/home/kaifaty/.cache/nextengine-research/diffsound-3a0be14/`; the inspected
  upstream clone was a temporary checkout outside the workspace.

This is an offline presentation experiment. It creates no runtime dependency,
public contract, asset role, authoritative state or product-completion claim.

## Compatibility environment

The upstream README requests Python 3.8 and PyTorch 2.0/CUDA 11.7, while its
requirements mix `torchaudio==2.0.2+cu118` with CUDA 11.7 Torch and
Torchvision. The local environment instead uses one coherent CUDA 11.7 set:

| Component | Version |
| --- | --- |
| Python | `3.10.20` |
| PyTorch / Torchvision / Torchaudio | `2.0.1+cu117` / `0.15.2+cu117` / `2.0.2+cu117` |
| torch-scatter | `2.1.2+pt20cu117` |
| NumPy / SciPy | `1.24.1` / `1.10.1` |
| GeomLoss / meshio | `0.2.6` / `5.3.4` |
| GPU | NVIDIA GeForce RTX 3080 |

The initially resolved NumPy 2.x build emitted an ABI warning with this old
PyTorch stack, so the trial pinned upstream's `numpy==1.24.1`. fTetWild was not
needed because the upstream bowl already includes a tetrahedral mesh.

## Control A — smallest upstream real-audio path

The control retained the upstream bowl mesh, eight provided microphone WAVs,
`Ceramic`, 16 modes and experiment mode 3. It retained all 2,001 damping-fit,
5,000 material-initialization and 2,000 damping-parameter pretraining steps,
but reduced the final material loop from 3,000 epochs to one. Exact config:

`/home/kaifaty/.cache/nextengine-research/diffsound-3a0be14/upstream-smoke-config.json`

Observed results:

- loaded tetrahedral mesh: 3,007 vertices and 8,840 tetrahedra;
- damping fit: approximately 2 minutes 35 seconds;
- one differentiable modal/material step: approximately 51 seconds;
- 16 reported frequencies: `1,647.898` through `15,829.234 Hz`;
- DiffSound RMSE: `34.571831`;
- process exit: success.

Artifacts:

| File | SHA-256 |
| --- | --- |
| `gt.wav` | `852242c99cc1fa2135cd49085fe864090bf2829849f7e2377e8bbcd5e1e2c966` |
| `predict.wav` | `35cc0c6de6953de34d49ecfbac2e1c40d30c27eab2e7ad7a410f500c578a9bf6` |
| `result.txt` | `350abde1e30c31c4d4f9f0d03be3a2f9e4dfe43d2925aa07ef2984754e6ca095` |

The successful control validates the local execution path only. One epoch
cannot validate convergence or material recovery.

## Control B — Next Engine thin-goblet proxy

DiffSound's real-audio experiment assumes known geometry and multiple
recordings. Next Engine currently has neither an exact goblet mesh nor a
controlled multi-impact capture. To test the nearest available path
without misrepresenting it as identification, the trial used:

- target: frozen `glass-thin-goblet.wav`, source SHA-256
  `0ab121b00b3c195df6d2101e8e5137d4dc0295b5a7243ba6247ba890df302423`;
- input conversion: mono 48 kHz WAV, repeated as eight identical inputs, with
  zero gain and zero pad; converted-input SHA-256
  `7b1fb51442ece0982a2e761ca4e734dc0af76217739008498264f35c0d116f4a`;
- geometry: upstream bowl mesh as an explicitly wrong proxy;
- initializer: upstream `Glass` table entry (`62 GPa`, Poisson `0.20`);
- solver: second-order mesh, 16 modes, 16 material epochs and full modal
  decompositions at epochs 0 and 15.

Exact config:

`/home/kaifaty/.cache/nextengine-research/diffsound-3a0be14/glass-proxy-config.json`

Training observations:

| Measure | Epoch 0 | Epoch 15 | Change |
| --- | ---: | ---: | ---: |
| DiffSound early loss | `528071.375` | `528579.875` | `+0.096294%` |
| DiffSound RMSE | `35.757381` | `35.670273` | `-0.243610%` |
| Actual weighted Young's parameter | `61.959971 GPa` | `61.364838 GPa` | `-0.960511%` |
| Poisson ratio | `0.200090` | `0.201430` | `+0.001340` absolute |
| First modal frequency | `1,554.401 Hz` | `1,546.399 Hz` | `-0.5148%` |

Upstream's `result.txt` and TensorBoard `youngs` scalar multiply the already
dimensional weighted Young's value by a hard-coded density of `2700`. The
reported `165685063680000.0` is therefore not accepted as pascals; the table
above divides that diagnostic by 2700. This local interpretation does not make
the proxy physically identifiable.

Final artifacts:

| File | SHA-256 |
| --- | --- |
| normalized target `gt.wav` | `69b90098b0b7018710c68cd3530a4e84873b8cfd976bcb5eae2b97341c73a654` |
| final `predict.wav` | `14648fa40079dae0abf4be7632b6fd503b829097adf91ba9efb343def0ee6294` |
| `result.txt` | `d51c48d27501d26bc7ee54346d6b423019012e741a60257270abafba7ddd0301` |

Both audio files are mono float32, 32 kHz and 250 ms. They remain outside the
repository.

## Independent matched analysis

`xtask physical-sound-eval` compared the final prediction with DiffSound's
normalized target under the existing uncalibrated classical profile:

| Descriptor | Prediction | Target | Matched delta/cost |
| --- | ---: | ---: | ---: |
| RMS | `-24.251 dBFS` | `-18.791 dBFS` | `-5.459 dB` |
| Spectral centroid | `3,439.132 Hz` | `1,969.466 Hz` | `+1,469.665 Hz` |
| Temporal centroid | `6.501 ms` | `32.184 ms` | `-25.682 ms` |
| Broadband T20 | `37.477 ms` | `76.536 ms` | mean-band delta `-126.522 ms` |
| Multiresolution log-spectrum RMSE | — | — | `45.337802 dB` |
| Modal assignment | — | — | `2.174618` |

The evaluator tagged spectral and modal-structure mismatch and kept the result
at `NeedsHumanAudit`. Report SHA-256:
`3540968ce71f7bb4902160421557f3a178709e112af8af7ff5949819d21b8adc`.

This analysis is diagnostic, not an autonomous quality score. Product-owner
audition subsequently preferred the prediction as a glass sound despite its
reference mismatch. This pair is therefore a concrete counterexample to using
synthetic-target fidelity as a proxy for material identity.

## Post-audition correction

The positive audition changes the admissible conclusion:

- keep the 16-step prediction as an external perceptual baseline;
- do not force a future run toward the earlier synthetic target merely to
  reduce spectral RMSE;
- measure glass identity/naturalness separately from exact target fidelity;
- retain the geometry/material numbers as latent artistic parameters until an
  exact mesh, scale, density and controlled captures make them identifiable.

The released real-audio path is also materially less complete than the method
described in the paper. The current proxy used only 250 ms of the 500-ms target.
Its final `forward_curve` path comments out learned modal amplitudes, emits all
modes at equal amplitude, normalizes each result, and does not add the learned
filtered-noise component. The paper instead describes learned per-mode
amplitudes, damping factors and an LTV-FIR filtered-noise component for natural
recordings. It reports 10,000 damping and 10,000 material steps, whereas the
released config uses 2,001 and 3,000 and the smoke proxy used only 16 material
steps. These gaps leave useful fitting capacity unexplored.

## Decision and smallest next experiment

DiffSound is viable for two distinct purposes. As a perceptual baker, its modal
bank may be treated as an artistic glass archetype even with proxy geometry.
As an inverse-material tool, it still requires known geometry and recording
conditions. Material, geometry, excitation, microphone response and radiation
remain confounded when one synthetic WAV is repeated over the expected inputs.

The smallest next perceptual experiment is a checkpointed external run that:

1. uses all 500 ms (`16,000` frames at 32 kHz), initially retains 16 modes and
   saves a preview plus `frequency`, `damping`, `amplitude` and loss values at
   every eigen-decomposition checkpoint;
2. fits per-mode amplitudes and a bounded 1–3 ms excitation/noise residual
   after fixing frequencies and damping;
3. stops first at 151 material epochs for audition, then extends to 1,001 and
   the released 3,000-step schedule only when a checkpoint improves glass
   identity or naturalness;
4. compares 16, 32 and 64 modes only after the 16-mode full-tail control, with
   the current preferred prediction and Glass-H preserved as blind anchors;
5. exports each accepted mode as `f_i`, exponential damping `d_i` and gain
   `A_i`; the runtime decay pole is `r_i = exp(-d_i / sample_rate)` and its
   `T60` is `ln(1000) / d_i`.

Physical transfer across shapes, strike points and sizes is a later experiment
requiring one exact scaled glass mesh, measured or fixed density, multiple
controlled impacts with force/position metadata, and held-out strikes. Modal
frequencies primarily constrain `E / density`; absolute Young's modulus is not
identifiable if scale and density float simultaneously.

Clarify upstream licensing or use a clean-room/licensed implementation before
shipping code. Until then, keep DiffSound and all generated assets external and
use the accepted output only to derive and validate engine-owned modal math.
