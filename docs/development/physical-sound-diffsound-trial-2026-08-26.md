# DiffSound glass calibration trial — 2026-08-26

## Result

DiffSound ran end to end on the local RTX 3080, including its real-audio
damping fit, second-order tetrahedral modal solve, differentiable oscillator,
loss and backward pass. This establishes tool compatibility for an isolated
offline laboratory. It does not establish that DiffSound can recover a useful
glass model from the current Next Engine audition WAV.

The bounded glass proxy moved its modes and reduced DiffSound's reported RMSE
by only `0.243610%`, while its optimization loss increased by `0.096294%`.
Next Engine's independent evaluator then measured `45.337802 dB`
gain-matched multiresolution spectral RMSE, modal-assignment cost `2.174618`,
and a `1,469.665 Hz` spectral-centroid overshoot. The proxy is rejected as a
calibration result.

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
controlled multi-microphone capture. To test the nearest available path
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

This analysis is diagnostic, not an autonomous quality score. The magnitude
and direction of several independent mismatches are nevertheless enough to
reject spending more compute on the same wrong-geometry proxy.

## Decision and smallest next experiment

DiffSound is viable as an offline inverse-material tool when object geometry
and recording conditions are known. It is not a prompt-to-sound model and the
evaluated experiment is not a generic WAV-to-modal-profile extractor. Material,
geometry, excitation, microphone response and radiation are confounded when a
single synthetic WAV is repeated over the eight expected inputs.

Do not extend this bowl-proxy run to 3,000 epochs. Resume DiffSound evaluation
only after all of the following are available:

1. one concrete glass object's measured or authored surface mesh and a checked
   tetrahedralization;
2. multiple controlled impulse recordings with impact position, gain, padding,
   sample rate and microphone placement recorded;
3. a train/held-out capture split so recovery and audio similarity are not
   judged on the fitted strike alone;
4. clarified upstream licensing or a clean-room/licensed alternative before
   any code or derived runtime component is considered for distribution.

Until then, use DiffSound as evidence that differentiable geometry-aware
fitting is technically possible, not as calibration of Glass-H or as an
independent sound-quality oracle.
