# Physical sound R3A V5-B collapse and V5-C bootstrap — 2026-08-31

Status: `V5B_REJECTED / ROOT_CAUSE_LOCALIZED / V5C_BOOTSTRAP_SUPPORTED /
DEVELOPMENT_UNREAD / CLIP_FALLBACK_REQUIRED`

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

## Conclusion and next discriminator

The full loss is useful as an evaluation endpoint but is a poor cold-start
objective here: clamped log-amplitude terms dominate its value while providing
little useful gradient below their floors, and lower-weight envelope terms can
be reduced by an input-independent low-amplitude output. RVQ is a secondary
challenge, not the primary cold-start cause.

V5-C must therefore freeze and test this curriculum:

1. continuous encoder/decoder bootstrap with the normalized objective;
2. deterministic codebook initialization from trained latents;
3. gradual continuous-to-quantized latent mixing while retaining bootstrap
   reconstruction and latent-matching losses;
4. a fully quantized anti-collapse gate before the full perceptual objective;
5. only after that gate, a bounded transition to the full objective.

The smallest next run ends at the first fully quantized checkpoint, expected
at step `4000`. It may proceed only if amplitude, spectrum, every RVQ stage,
output diversity and latent stability all pass. Development contacts, row
`2407`, method holdout and admission shadow remain forbidden.
