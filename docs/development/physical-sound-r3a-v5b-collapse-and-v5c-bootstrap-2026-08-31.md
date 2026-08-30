# Physical sound R3A V5-B collapse and V5-C bootstrap — 2026-08-31

Status: `V5B_REJECTED / ROOT_CAUSE_LOCALIZED /
V5C_6KBPS_QUANTIZED_GATE_PASSED / DEVELOPMENT_UNREAD /
CLIP_FALLBACK_REQUIRED`

## Result

The first real `rvq-6kbps` capacity attempt is rejected before development.
The automatic step-2000 gate correctly identifies a stable but
input-independent near-silent decoder. Two bounded counterfactuals isolate the
failure below RVQ: the same encoder/decoder trained without quantization still
collapses under the full perceptual loss, while a normalized continuous
bootstrap learns distinct held internet impacts.

This result authorizes a staged V5-C training experiment, not a long capacity
run, development access, holdout access or product integration.

## Frozen V5-B boundary

- model preflight manifest `c596cbb7b350b5099e3acf4a34826fb58dc55f89f5d506ccbeac8c077055dea8`;
- model preflight report `d945d01fa23c8de2815c9d2f41bc364e81b6273f6e3da20916e93c3b494d72c5`;
- CUDA runner manifest `d53561fdd4cc4a662dfe2d750a2f3109fb5d63dc154b14f73ce92cb475d60b95`;
- CUDA runner report `50ee9871d2c63ff1ea0afd5fdd20b35e14ee005cc20deaab64cdeb226343d2e0`;
- factorized 8-dimensional, L2-selected RVQ, weight-normalized convolution and
  truncated-normal `0.02` initialization;
- `56` internet train clips plus twelve authorized fit contacts; `17`
  group-disjoint internet validation clips;
- zero development and sealed waveform reads.

The stabilization follows primary implementation evidence from
[Descript Audio Codec](https://github.com/descriptinc/descript-audio-codec/blob/main/dac/model/dac.py),
its [weight-normalized layers](https://github.com/descriptinc/descript-audio-codec/blob/main/dac/nn/layers.py)
and [factorized quantizer](https://github.com/descriptinc/descript-audio-codec/blob/main/dac/nn/quantize.py).
It removes the V5-A encoder explosion but does not, by itself, prevent decoder
collapse.

## Automatic rejection

External run:
`r3a-v5b-factorized-rvq-discriminator-6kbps-step2000`.

| Artifact | SHA-256 |
| --- | --- |
| Run manifest | `0eecceae011e6986f61e3602c04f829c66696c1eb24b1326ef0e08affc3fb55d` |
| Metrics | `5c692460a9c0decb3645610bd52d78595105c87b008cf332a0457d18ff45bf11` |
| Rejection report | `857b80665fc95fd221f0c42c5133429aa7d7b152734694bffb4304cb4ffa55f9` |
| Step-2000 checkpoint | `632e404cf7e55ba05e4be9e12691418e2c022b8b34a6646cda8c58356f09e5db` |

At step `2000`, latent RMS `0.00995` and maximum absolute value `0.0269`
pass the stability checks. Four independent anti-collapse checks fail:

| Check | Observed | Required |
| --- | ---: | ---: |
| Mean output/target RMS | `0.0294` | `>= 0.10` |
| Relative log-spectrum improvement | `0.0000` | `>= 0.005` |
| Minimum codes per RVQ stage | `1` (`[1,1,2,2]`) | `>= 2` |
| Normalized output diversity ratio | `2.29e-8` | `>= 0.01` |

Distinct output hashes alone would have missed the failure: all `17` hashes
differ numerically, yet normalized waveforms are effectively identical. This
is why diversity is measured acoustically rather than by file identity.

## Causal counterfactuals

The continuous-autoencoder control removes RVQ but retains the same model,
split, scheduler and full loss. At step `2000` it still has RMS ratio `0.0333`,
zero log-spectrum improvement, correlation `0.00163` and diversity ratio
`2.38e-6`. Artifact SHA-256:
`45e2c4c776c7461b64f7b6f143b683738523d325ea59f4f34b14c52bbc1d2bb4`.

The normalized bootstrap control also removes RVQ, but replaces the initial
objective with target-normalized waveform L1, first-difference L1 and
three-scale complex-STFT loss. It changes validation as follows:

| Metric | Initial | Step 1000 | Step 2000 |
| --- | ---: | ---: | ---: |
| Full validation loss | `450.41` | `407.97` | `360.44` |
| Log-spectrum loss | `26.694` | `24.139` | `22.294` |
| Mean output/target RMS | `8.06e-6` | `0.431` | `0.620` |
| Mean absolute correlation | `0.0130` | `0.300` | `0.351` |
| Output diversity ratio | `1.214` | `1.878` | `1.853` |
| Maximum latent RMS | `0.000215` | `0.0858` | `0.313` |

Artifact SHA-256:
`ed888c4ed46bb27e973010c081289fe09b2367160e5ef3a44445e0bb3d771229`.
The large initial diversity ratio is not quality credit because amplitude is
nearly zero; the step-2000 result jointly satisfies amplitude, spectrum,
correlation, diversity and stability evidence.

## V5-C quantized boundary

V5-C uses the supported continuous bootstrap through step `2000`, initializes
four codebooks from eight distinct trained-latent segments, freezes the
encoder/decoder and ramps quantized latent use from zero to one through step
`4000`. The first joint ramp kept code diversity but exploded the mutable
encoder; freezing the already-passing codec removes that scale freedom.

External pass: `r3a-v5c2-frozen-rvq-discriminator-6kbps-step4000`.

| Artifact | SHA-256 |
| --- | --- |
| Run manifest | `581e0c4105c4d151972db584fa5403afcd1c6c616da05f2a6ab3d29b1b20361f` |
| Metrics | `439392476a3a7b73e6224dc0344fe07b91a6e9ea11ef82276c7c90a283fda854` |
| Pass report | `047cec3f04c2ab098cf30c4be2036d913529fb0d129d465b58b0fc61c58a229a` |
| Step-4000 checkpoint | `2f8fcffd868f7550096873fb404ab66048b274e8418ba5bcf44bc2e48d097922` |

All automatic checks pass: RMS ratio `0.1907`, log-spectrum improvement
`5.47%`, minimum `51` codes per stage, diversity ratio `1.991`, latent RMS
`0.313` and maximum absolute latent `5.618`. This is internal representation
evidence only; it does not admit development quality or a product asset.

## Full-loss falsification and stable refinement

Unfreezing the codec and gradually weighting the full perceptual loss is
rejected at step `5000`. Latent RMS reaches `642.8`, amplitude falls to
`0.0316` of target and the automated report is
`df805d76cf078266f70a19fdc636904ff7aa719350793109c2f0444c2eed32b9`.
This shows the full loss is unsuitable as an optimization objective even after
successful quantization; it remains the independent validation and checkpoint
selection endpoint.

The evaluation-only refinement run
`r3a-v5c4-eval-only-refinement-6kbps-step6000` keeps the encoder frozen and
trains decoder plus quantizer with normalized bootstrap and latent matching.

| Artifact | SHA-256 |
| --- | --- |
| Run manifest | `9cb514dbdfff80bf2f18b7c2a001376219529f5b4ff0b0678c6c1c81b1cd45ef` |
| Metrics | `cf9191672e7e3997bfb694e14827e7020af1e48fae295c05b701349edcfd01df` |
| Pass report | `8a41950be3007bb1d10ceb29f89f518b2fdc80ac55d3ab94b600694f97ce9a60` |
| Step-6000 checkpoint | `cd1e7bb98e5e554500dc79fdd4da2bb458d809e214f341aa297fea480d3d240f` |

Steps `5000` and `6000` remain stable and pass every anti-collapse check, but
full validation loss worsens from `425.92` at step `4000` to `438.37` and
`447.90`. The frozen selector therefore retains the earliest step-4000
checkpoint. An implicit `50,000`-step launch is now disabled.

## Conclusion and next discriminator

The full loss is useful as an evaluation endpoint but is a poor cold-start
objective here: clamped log-amplitude terms dominate its value while providing
little useful gradient below their floors, and lower-weight envelope terms can
be reduced by an input-independent low-amplitude output. RVQ is a secondary
challenge, not the primary cold-start cause.

V5-C proves that one `6 kbps` task-specific quantized representation can carry
non-collapsed, input-dependent held internet impact signal. It does not yet
prove adequate acoustic quality, the best capacity, exact-object development,
source-disjoint generalization or runtime suitability.

The smallest next action is the identical bounded step-4000 discriminator for
`12` and `24 kbps`, followed by a frozen internal capacity comparison. Only
that comparison may authorize a longer run or one development evaluation.
Development contacts, row `2407`, method holdout and admission shadow remain
forbidden meanwhile.
