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

## Checkpointed 500-ms follow-up

The smallest proposed follow-up was implemented as an external-only harness
against the same upstream commit. It used the complete 500-ms target, one
audio input, 16 modes, the released 2,001-step damping fit and 151 material
steps. Full eigendecomposition and export ran every 15 material steps. The
loss changed from optimal transport to multiscale L1 at step 60, so scalar
loss values on opposite sides of that boundary are not comparable.

At every one of 11 checkpoints the harness exported four normalized 32-kHz
mono float32 WAVs: equal-amplitude modal synthesis, damping-curve amplitude,
ridge-fitted amplitude, and ridge amplitude plus a bounded three-millisecond
transient. It also exported the 16 frequencies, damped frequencies, damping
coefficients, T60 values, runtime decay poles and both fitted amplitude sets.
The transient peak was limited to 25% of the modal peak. No generated file or
harness source was added to the repository.

The full run completed successfully in 10 minutes 12 seconds. All 44 WAVs had
unique hashes, all 11 modal profiles were finite, and no JSON output contained
NaN, infinity or null. Exact evidence roots and hashes are:

| Evidence | Location or SHA-256 |
| --- | --- |
| Run root | `/home/kaifaty/.cache/nextengine-research/diffsound-3a0be14/glass-checkpointed/glass-500ms-16mode-151_20260826-222834/` |
| `run-report.json` | `a1d3e03e02900fb24b0ccba9a5c3ce42743e196b1b7060d9f4f510a75f0e25bc` |
| `damping-fit.json` | `c582956c3362922305c8820e09b6857bd73365b0a440d51b6ee2937b8da5b41f` |
| frozen normalized target | `691d4b1313b5dbd04caaf9367083c19b45b9c9db85c93d0e3487039f541a94e0` |
| harness | `d205e01af506c561fffc7a8b095b110fa549e4a6d06ed058839d8fd210941c46` |
| run config | `13b68f9c1918bd066024a1dbd2939f735ff9de46340c80835e8f5256cc3c64fb` |

The final modes span `1,538.578` to `14,803.317 Hz`. Damping spans
`53.553` to `78.492 s^-1`, corresponding to T60 values of `88.0` to
`129.0 ms` and 32-kHz recurrence poles of `0.9975501` to `0.9983279`.
The optimized proxy ended at `60.669313 GPa` and Poisson ratio `0.197534`.
These remain artistic latent values because the bowl geometry is wrong and
density, scale, excitation and radiation are not identified.

The independent evaluator accepted all 44 files as matched analysis and
reported `Q1_MATCHED_ANALYSIS / HUMAN_CALIBRATION_REQUIRED`; report SHA-256 is
`9d5ffbd9bc8207acb1307ccd641d5f4d0b30b2fba0fa48239fea9205c4e46c1b`.
The final amplitude variants are materially distinct: their spectral
centroids are approximately `12.386 kHz` for damping-curve amplitude,
`1.710 kHz` for ridge amplitude and `5.821 kHz` for equal amplitude. Adding
the transient changed the final gain-matched spectral RMSE from `54.425 dB`
to `44.500 dB` without changing the ridge modal centroid. These numbers are
diagnostics, not a glass-identity ranking: the accepted 16-step anchor already
demonstrated that closer target match can disagree with human material
identity.

The ten-file audition package is at
`/home/kaifaty/.cache/nextengine-research/diffsound-3a0be14/glass-checkpointed-audition-151-v2/audition/`;
its manifest SHA-256 is
`1db7e93195c48fe5e0a19315a8008220b58ce3d38bf32894121a308c0fed5c60`.
It preserves the accepted 16-step anchor, the synthetic target, five
equal-amplitude checkpoints and all four final amplitude/transient variants.
Product-owner audition selected
`09-step150-ridge-amplitude-transient.wav` as the best candidate and described
it as a strike on a glass container with a fairly thin wall. Its SHA-256 is
`4246dd506d9db8e3d30f7764cdb8fe034508b42313b83924392f8ab864dd6d77`.
This is positive identity evidence for that bounded archetype, not for a
universal glass material or physically identified wall thickness.

## Pre-reproduction decision

DiffSound is viable for two distinct purposes. As a perceptual baker, its modal
bank may be treated as an artistic glass archetype even with proxy geometry.
As an inverse-material tool, it still requires known geometry and recording
conditions. Material, geometry, excitation, microphone response and radiation
remain confounded when one synthetic WAV is repeated over the expected inputs.

The checkpointed run closes the requested compute/export and first perceptual
selection steps. The smallest next experiment is therefore:

1. freeze candidate `09` and its `f_i`, exponential damping `d_i`, ridge gain
   `A_i` and transient parameters as external calibration evidence;
2. reproduce its PCM with an engine-owned offline recurrence and measure the
   residual against the selected WAV; `r_i = exp(-d_i / sample_rate)` and
   `T60 = ln(1000) / d_i`;
3. if a broader vessel palette is needed, change upper-mode tilt and transient
   level one variable at a time around this anchor, labeling the results as
   artistic thin/thick-wall impressions rather than inferred geometry;
4. extend optimization steps, mode count or filtered noise only if the bounded
   recurrence cannot preserve the selected identity or a controlled recording
   exposes a specific residual.

Physical transfer across shapes, strike points and sizes is a later experiment
requiring one exact scaled glass mesh, measured or fixed density, multiple
controlled impacts with force/position metadata, and held-out strikes. Modal
frequencies primarily constrain `E / density`; absolute Young's modulus is not
identifiable if scale and density float simultaneously.

## Engine-owned recurrence reproduction

The selected checkpoint-150 profile was then consumed as untrusted external
input by the new `xtask physical-sound-reproduce` command. The command validates
the external schema, hashes, sibling WAVs, modal relations, sample-rate poles,
Nyquist bounds, transient length and output location. It runs an engine-owned
floating-point second-order damped recurrence; it imports no DiffSound source,
dependency or model state. Both the pure ridge-modal signal and the selected
ridge-modal-plus-transient signal are rendered twice, compared sample by sample
with the frozen DiffSound WAV and emitted only to the external research store.

The 32-kHz, 16-mode, 500-ms reproduction passed its declared `0.001` RMS-error
and `0.999` correlation limits by a wide margin:

| Measurement | Ridge modal | Ridge modal + transient (`09`) |
| --- | ---: | ---: |
| Maximum absolute PCM residual | `0.00004901` | `0.00004901` |
| RMS PCM residual | `0.000003455` | `0.000003455` |
| Signal-to-noise ratio | `86.186 dB` | `86.045 dB` |
| Zero-lag normalized correlation | `0.99999999903` | `0.99999999900` |
| Repeated engine render | exact | exact |

For selected `09`, the independent evaluator measured zero attack delta,
`0.000251 ms` temporal-centroid delta, `0.010295 Hz` spectral-centroid delta,
modal-assignment cost `0.000001236` and no failure tag. Its gain-matched
log-spectrum RMSE remained `5.0086 dB` because the metric includes extremely
low-energy log-spectrum bins; the direct 86-dB sample SNR and matching
modal/envelope descriptors bound the reproduction claim more directly. The
evaluator still correctly returns `NeedsHumanAudit`: this run proves numerical
reproduction of the already selected sound, not autonomous glass-quality
judgment.

Exact external evidence:

| Evidence | Location or SHA-256 |
| --- | --- |
| Reproduction root | `/home/kaifaty/.cache/nextengine-research/diffsound-3a0be14/engine-recurrence-step150-v2/` |
| reproduction `report.json` | `b076ff54098d819a5ec390bd63a421677d647bc03e135898fa297939709b97a8` |
| matched quality manifest | `b3ec41279783a99e9c7e96b35ee828bf5d95493a30da678956b47e4987dae37e` |
| engine `09` WAV | `8db48ed8bd61b3bc44787efa418899e286e5c3efbf4d79f1974f6dbcc81baeb7` |
| independent evaluator report | `20571ac7b90264d6137f90fbd3b277de6d1f17aa8de62ae391415dceea26c2b4` |

This closes the high-precision offline recurrence question for the selected
archetype. It does not yet prove that the existing 48-kHz Q30 demo bank can
retain the same identity after coefficient quantization, resampling and its
bounded mode-count/voice path.

## Isolated 48-kHz Q30 transfer

The selected 16 modes were next rerendered at 48 kHz rather than reduced to the
current demo bank's 12-mode limit. The 96-sample, 32-kHz onset was resampled to
144 samples with deterministic linear interpolation and an explicit zero sample
beyond the source boundary. The same resampled onset fed both the f64 reference
and Q30 candidate, isolating fixed-point error from sample-rate conversion.

The offline cooker quantized recurrence coefficients, per-mode initial state
and onset samples to signed Q30. After cooking, the recurrence and transient
sum use integer state with checked `i128` multiply-accumulate; floating point
returns only for final peak normalization and WAV comparison. This is a
clean-room engine-owned transfer and imports no DiffSound code or dependency.
It is not yet the current demo voice, runtime content or a shipping asset.

Before execution, the transfer limits were fixed at maximum PCM residual
`0.001`, RMS residual `0.0001` and normalized correlation `0.99999`. The
500-ms render passed with substantial margin:

| Measurement | Modal only | Modal plus resampled onset |
| --- | ---: | ---: |
| Maximum absolute PCM residual | `6.448e-7` | `6.271e-7` |
| RMS PCM residual | `1.168e-7` | `1.136e-7` |
| Signal-to-noise ratio | `115.524 dB` | `115.429 dB` |
| Zero-lag normalized correlation | `0.9999999999987` | `0.9999999999986` |
| Repeated integer render | exact | exact |

The independent evaluator found no failure tag for either Q30 pair. For the
onset candidate it measured zero attack delta, `0.0000080 ms` temporal-centroid
delta, `-0.000637 Hz` spectral-centroid delta, modal-assignment cost
`1.379e-7` and gain-matched log-spectrum RMSE `0.480821 dB`. As before,
`NeedsHumanAudit` is correct: numerical equivalence cannot by itself decide
whether sample-rate conversion preserved the selected glass identity.

Exact external evidence:

| Evidence | Location or SHA-256 |
| --- | --- |
| Transfer root | `/home/kaifaty/.cache/nextengine-research/diffsound-3a0be14/engine-q30-step150-v2/` |
| transfer `report.json` | `90d115bb2d15129b3bf16561c14dd0770f9771175605d7301fad7de2c37d3a8e` |
| quality manifest | `ebcb43cbdab70ecee4d0bbdcd3eae14f06f67e54b9852c1f8a2990c2f5c60fdc` |
| independent evaluator `report.json` | `4e52089ca70d858cfc1042c6484e12bb21f53b327e7a76e471db71409f7c9667` |
| 48-kHz Q30 onset WAV | `176d7cd50576f681607057a4917184ae4f16a74695e2455d19a47a2b8657c61a` |
| audition A, f64 reference | `1a648511fc02cc6dd551fb64624c295cd838588e48df97cb61197c1e2db2cca3` |
| audition B, Q30 | `10bb7fdd5c75223d643f01ef4c127a4329c9cc417e4f0ec3e02b3e45ccc0a218` |
| sequential A/B | `169c5009f3bbda2888e26788d81be6dd38fbc9c5091f4de21f4d9704face3b93` |

The first generated transfer directory used an unsorted quality-manifest entry
order and was rejected before analysis. Version `v2` sorts identifiers before
publication; only `v2` is retained as evidence. The failed evaluator invocation
did not alter any engine or external source artifact.

## Product-owner Q30 acceptance and explicit demo cut

The product owner auditioned the aligned pair and judged B, the 48-kHz Q30
render, good and possibly better. This closes the human transfer gate for the
selected thin-container archetype. It does not rank Q30 numerics above f64 in
general or establish physical glass parameters.

The laboratory now contains a pre-cooked voice for the selected 16 recurrence
modes and 144 Q30 onset samples. The constants retain the external
source-profile hash but embed no DiffSound code, dependency, generated PCM or
model state. `physical-sound-lab` still defaults to Glass-H. The separate Cargo
feature `physical-sound-selected-glass` explicitly routes the reference demo's
committed `Begin` contacts to the selected voice so the candidate is actually
audible in that scene. Disabling the laboratory still returns to the
authored-clip baseline.

The lab's full-scale demo render is 24,000 stereo frames with peak `29,490` and
WAV SHA-256 `c912806c...b9c823`. Against the accepted audition B, its integer
post-scale differs by at most one S16 least-significant bit, with RMS residual
`0.470859` S16 units and correlation `0.9999999749`. The exact committed PCM
regression hash is `a85dee33...abde0`.

A 500-run alternating release measurement on an AMD Ryzen 9 3950X with Rust
1.97.1 reports a selected-voice full-render p50/p99 of `1.471/1.753 ms` and
`61.306 ns` p50 per stereo frame, `5.504x` the Glass-H per-frame result. At the
laboratory's 16-voice bound, one 1,600-frame selected-glass tick measured
`1.483 ms` p50 and `1.683 ms` p99, or `5.05%` of its 33.333-ms audio window.
This excludes the rest of the engine mixer and OS callback, is not a product
budget and grants no performance promotion. External report:
`/home/kaifaty/.cache/nextengine-research/diffsound-3a0be14/engine-selected-glass-demo-q30-v1/report.json`,
SHA-256 `ad02bc6e...fce43e`.

The feature-enabled reference demo completed 120 SDL/Ash frames and 56
simulation ticks with active audio, 112,000 queued samples, zero audio drops or
faults and 49 debug underruns. It is functional evidence only. The focused
enabled/disabled regression observes selected-glass admission and different
PCM while preserving runtime, RPG and physics checkpoint state exactly.

## Revised decision and smallest next experiment

The 48-kHz Q30 candidate now clears both its numerical and human transfer gates,
and the explicit demo/cost experiment is complete. Glass-H remains the default
control and authored clips remain the fallback.

The next coherent experiment is no longer another near-neighbor coefficient
tune. The exact-geometry synthetic checkpoint is now complete and documented
in the [controlled glass corpus report](physical-sound-controlled-glass-corpus-2026-08-27.md):
15 force/position conditions pass numeric and physical controls, while the
held-out spatial-IDW baseline reaches only `0.907523` correlation and
`5.858210 dB` signal-to-residual ratio. Continue with surface mode-shape
interpolation and matched real recordings, not another preset search. Runtime
promotion still waits for the contact projection, content closure, whole-mixer
budget and Accepted consumer ADR required by SPEC-45.

Clarify upstream licensing or use a clean-room/licensed implementation before
shipping code. Until then, keep DiffSound and all generated assets external and
use the accepted output only to derive and validate engine-owned modal math.
