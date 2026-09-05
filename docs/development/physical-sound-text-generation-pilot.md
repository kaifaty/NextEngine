# Text-conditioned physical-sound pilot

External research, 2026-09-05. No product promotion or runtime changes.

## Question and audible result

The user's goal is **object/event description -> sound**, including both
interacting materials, shape/size, force/speed, water and rain. It is not
**recording -> reconstruction**. The earlier liked glass encoder remains useful
as a reconstruction control, but cannot satisfy this goal.

The first executable discriminator is a pretrained text-conditioned generator
on 12 fixed descriptions and two fixed seeds. No reference waveform enters
generation, no local training occurs, and every output is published before
scoring. Conditions cover six material-pair impacts, pouring, light/heavy rain,
scraping, rolling and glass breaking. These are qualitative prompts, not
measured SI-valued physics inputs.

- [100-step FP16 preview: all 12 conditions, seed 42](/home/kaifaty/.codex/experiments/nextengine/physical-sound/text-pilot-2026-09-05/preview.wav)
- External root: `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
- `text-pilot-2026-09-05`: 24 candidates + two empty-prompt controls,
  `preview.wav`, `result.json`, `ast-tags.json`.
- `text-pilot-fp32-2026-09-05`: same descriptions/seeds/100 steps in FP32.
- `text-pilot-200steps-2026-09-05`: same descriptions/seeds in FP16, 200 steps.

Preview order: glass/metal striker, glass/wood striker, wood/metal,
wood/wood, steel/metal, steel/wood, pouring, light rain, heavy rain,
scraping, rolling, breaking. Each clip lasts five seconds, separated by 0.5 s.
The preview uses the first seed, not a selected winner. Individual WAVs retain
the seed in their filenames. No amplification or loudness equalization is
applied; only attenuation if needed to avoid PCM overload.

## Bounded research and measured controls

[AudioLDM 2](https://huggingface.co/cvssp/audioldm2) and its
[official pipeline documentation](https://huggingface.co/docs/diffusers/api/pipelines/audioldm2)
provide an ungated, text-only sound-effects baseline. Its weights are labelled
CC-BY-NC-SA-4.0; this experiment does not distribute them or admit generated
assets into the engine. Model revision is
`c8e7e189d324425c05c4c2f81214041ef4107983`.

Two distinct diagnostics run on delivered PCM:

1. The generator's CLAP compares each audio with all 12 descriptions, swapped
   material/intensity descriptions, and an empty-prompt generation from the
   same seed. **Shared generator weights: not an independent validator.**
2. [AST](https://huggingface.co/MIT/ast-finetuned-audioset-10-10-0.4593), revision
   `f826b80d28226b62986cc218e5cec390b1096902`, classifies audio without seeing
   prompts. This is a separate architecture/checkpoint, but pretraining-corpus
   independence is not established. Its coarse AudioSet tags neither identify
   steel/striker materials nor certify naturalness. Scores are uncalibrated.

At 100 steps, FP16 produced all 24 candidates in a 139.32-second complete run.
CLAP favoured the exact intended prompt among the 12 alternatives on **7/24**
outputs; intended similarity exceeded empty-prompt controls on **20/24**.
AST included at least one coarse expected tag in its top five for **5/20**
scorable outputs. Four steel examples have no exact ontology label and are
explicitly unscored, not passed. All five coarse matches are water/rain;
many impacts instead receive bell, music or synthesizer tags.

AST recognizes silence/noise/tone controls, but only **2/3** previously disclosed
real-glass controls have a glass/clink tag in the top five. Thus even this
coarse threshold cannot be treated as a calibrated material gate. The real
controls come from the old training pilot; they are not a fresh holdout.

Competing explanations and counterfactuals:

- **Numerical precision:** FP32 leaves CLAP counts and AST's 5/20 unchanged.
  Median FP16/FP32 PCM correlation is `0.99912` (minimum `0.98011`);
  mean target cosine is `0.18732` versus `0.18618`. Half precision is not the
  main explanation for these failures. The official
  [optimization article](https://huggingface.co/blog/audioldm2) also describes
  FP16 inference; this is supporting evidence, not a substitute for the run.
- **Too few denoising steps:** the matched 200-step run completes in 270.07 s,
  with CLAP rank one on 8/24, better-than-empty scores on 21/24 and unchanged
  AST coarse coverage of 6/20 (versus 5/20). Doubling compute does not resolve
  the broad failures.
- **Insufficient semantic/interaction control:** supported by the disagreement
  between improved CLAP similarity and coarse audio-only tags. This remains a
  model/measurement hypothesis, not proof of a specific training-data defect.

No result establishes precise geometry, force, velocity, calibrated rainfall,
new-object generalization or novelty relative to unknown pretraining examples.
Louder heavy-rain output alone does not establish correct rainfall physics.
AST used the official Transformers NumPy frontend (TorchAudio is absent).
It emitted a zero-valued mel-filter warning; no checkpoint preprocessing was
retuned to improve these scores. The real/control results above and that
frontend limitation must accompany interpretation of the diagnostic.

## Next useful discriminator

The TangoFlux counterfactual below selects a more useful research base. Do not
train another recording-to-modal-parameter MLP or declare that prompt generation
solves physical control. Next: one bounded real-audio fine-tuning experiment on
that generator, with audible before/after output and frozen development checks.
Use the disclosed wine-glass recordings with two recording IDs for training
and the third for development; do not reclassify them as pristine test data.
Adapt a small part of the model, preserve the base, and check unrelated event
prompts for regressions. Render during the first short training cycle rather
than creating a separate protocol or waiting for a long fit to finish.
The input remains event text, never the target recording. Missing exact
geometry/force labels stay missing; this first adaptation tests learnability
of an observed sound family, not the entire goal.

[Simi-SFX (2024)](https://arxiv.org/pdf/2412.18710) demonstrates continuous
timbral conditioning, but its reconstruction pathway extracts loudness and
centroid from input audio. Adopting it unchanged would again miss the user's
no-reference-audio goal. Do not confuse acoustic feature controls with measured
physical parameters or fabricate missing geometry/force labels.

## Runnable path

Scripts: [generation](../../lab/scripts/physical_sound_text_pilot.py),
[audio-only diagnostics](../../lab/scripts/physical_sound_text_tags.py),
[focused tests](../../lab/tests/test_physical_sound_text_pilot.py).
Use `lab/.venv/bin/python`; installed optional research dependencies are
`diffusers==0.30.3`, `transformers==4.44.2`, `accelerate==0.34.2`,
`huggingface-hub==0.34.4`, `sentencepiece==0.2.1`, `soundfile==0.13.1`.
Torch and NumPy were not replaced. Exact package versions accompany each run.

First download the pinned model revisions with `huggingface_hub.snapshot_download`:
for AudioLDM2 allow `*.json`, `*.txt`, `*.model`, `*.safetensors`, `README.md`;
for AST allow `*.json`, `*.safetensors`, `README.md`. No remote model code is used.
Then run offline, selecting a new external directory:

```sh
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_text_pilot.py \
  --output /absolute/external/new-run --steps 100 --seeds 42 123 --seconds 5

HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_text_tags.py \
  --source /absolute/external/new-run/result.json \
  --output /absolute/external/new-run/ast-tags.json
```

The entire goal remains open: generation from physical attributes, robust
independent automatic evaluation, demonstrated improvement through learning,
new-condition generalization and an admitted offline engine-consumption path.

Verification: six new focused tests and all twelve existing audible-glass tests
passed; Ruff and diff checks passed. All 81 WAVs across the three runs were
read back: correct mono PCM16/16 kHz, expected duration, no clipping and exact
individual hashes. No Cargo/ProductCheck was run: this is external-only Python
lab work with no runtime or public-contract change.

## TangoFlux comparison (2026-09-05)

Powered by Stability AI. [TangoFlux](https://huggingface.co/declare-lab/TangoFlux),
Hung et al., non-commercial research only. This Stability AI Model is licensed
under the Stability AI Community License, Copyright © Stability AI Ltd.
All Rights Reserved. The upstream model/data restrictions are retained with
the external artifacts; no runtime or distributable engine asset is promoted.

The exact model revision is `367005e963cb3a9fb2e03a46104d7de23e34ceea`.
The inspected upstream `model.py` must match SHA-256
`209cfe8de77e39e935668b4e13ddb226ea2842b01d56d59898f970067de3481d` before
execution. All checkpoint keys/values are checked, including the tied T5
embedding alias omitted from safetensors. Cached T5 files only scaffold
construction; no unreported AudioLDM weights remain after loading TangoFlux.

[TangoFlux preview, fixed seed 42](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-pilot-2026-09-05/preview.wav)
contains the same twelve prompts in the order above. The external
`tangoflux-pilot-2026-09-05` directory contains 26 native 44.1-kHz stereo WAVs,
26 downmixed/resampled 16-kHz scoring copies, the preview, exact executed
script, `result.json` and `ast-clap-fp32.json`. The run took **194.09 s** at
50 steps/FP32. The full upstream latent horizon is rendered before trimming
to five seconds; no target audio enters the generator. No weights were trained.

[22-second direct comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-pilot-2026-09-05/base-vs-tango-preview.wav):
AudioLDM2 glass -> TangoFlux glass -> AudioLDM2 wood -> TangoFlux wood,
all seed 42, with half-second gaps and no loudness matching.

| Same diagnostic | AudioLDM2, 200 steps | TangoFlux, 50 steps |
| --- | ---: | ---: |
| CLAP exact prompt ranks first among twelve | 8/24 | 15/24 |
| CLAP intended score beats empty-prompt control | 21/24 | 24/24 |
| AST coarse expected tag in top five | 6/20 | 15/20 |

The four steel cases remain unscored by the AST material rule: its ontology
has no exact steel class. TangoFlux steel outputs sometimes receive glass tags.
Both rolling examples lack a high-ranking Roll tag; scraping receives strong
Rub/Filing tags but not the predefined Scrape tag. These are diagnostic
disagreements, not automatic proof every such waveform sounds wrong. Glass
breaking can pass the coarse Glass tag without proving fracture; paired
prompts also vary descriptive wording, so positive prompt margins do not
isolate a physically causal striker-material effect. Light/heavy rain has
opposite-sign CLAP pair margins across the two seeds. Fine control is not solved.

For this comparison, CLAP is loaded directly in FP32 on CPU for **both**
models. Earlier AudioLDM2 CLAP weights were rounded through FP16 before scoring.
The first replay comparison therefore failed the `1e-5` tolerance (maximum
cosine difference `0.001079`). Explicitly reproducing that weight rounding
reduced the error to `5.38e-7`, isolating the cause. Baseline remeasurement is
retained as `text-pilot-200steps-2026-09-05/ast-clap-fp32.json`; old reports
are not overwritten. Its counts remain unchanged. Neither pretrained judge
has established training-data independence or calibrated perceptual risk.

Run [the TangoFlux script](../../lab/scripts/physical_sound_tangoflux_pilot.py)
offline after downloading the pinned model's `*.json`, `*.safetensors`, `*.md`,
`model.py`, `tangoflux.py` with `snapshot_download`. It also uses the already
cached AudioLDM2 scaffold. Additional dependency: `datasets==2.21.0` (upstream
imports it even for inference); this installs `fsspec==2024.6.1`, replacing
`2026.6.0` in the lab environment. Torch and model libraries are unchanged.
Nine current focused pilot tests and four existing pilot tests passed;
Ruff/diff checks passed. All 52 individual WAVs were checked for exact hashes,
sample rates, dimensions, lengths and unclipped PCM. No ProductCheck applies
to this external-only experiment; generalization and integration remain open.

```sh
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_tangoflux_pilot.py --output /absolute/external/tango-run
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_text_tags.py \
  --source /absolute/external/tango-run/result.json \
  --output /absolute/external/tango-run/ast-clap-fp32.json --with-clap
```

## First actual generative fine-tune (2026-09-05)

[Listen: real glass -> base -> step 40 -> step 120](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-lora-glass-2026-09-05/glass-training-comparison.wav).
The sequence repeats for seeds 42 and 123 (16 seconds total, half-second gaps,
no loudness matching). The real example comes from development recording
761162. Generation receives **text and duration only**, never that recording.
[Step-120 preview](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-lora-glass-2026-09-05/step120/preview.wav)
also includes wooden-stick/glass, wood, water and rain controls.

The reversible experiment trains 786,432 LoRA parameters (rank/alpha 8,
attention `to_q`/`to_v` only), freezing all original TangoFlux/T5/VAE weights.
Dependency addition: `peft==0.12.0`; previous pinned libraries are unchanged.
The implementation follows the upstream [SFT flow target and VAE encoding](https://github.com/declare-lab/TangoFlux/blob/main/tangoflux/train.py)
and the installed Diffusers 0.30.3 [adapter interface](https://huggingface.co/docs/diffusers/v0.30.3/api/loaders/peft).
It executes only the previously inspected hash-pinned HF model code, not the
moving GitHub training script. Cached text conditioning reproduces the exact
upstream FP32 SFT loss: both are `0.32723280787467957` on the same RNG control.

Training uses the previously disclosed 16 crops from recordings 761160/761161;
all 11 crops of 761162 are excluded from this fit. Same author/pack is not
proof of independent physical objects; this is **development**, not a pristine
test. Hashes and crop identities are retained. Each 1.5-second crop is resampled
to stereo 44.1 kHz, peak-normalized to 0.5 and padded to the upstream 30-second
latent horizon. VAE posterior means/stds are cached; training samples the
posterior, while fixed development measurements use its mean and three fixed
noise levels. One prompt describes a knife hitting a wine glass once.
There are no invented geometry, force or striker-material measurements.

Unlike upstream uniform MSE, this short-impact experiment gives half the loss
to the first 33 latent frames and half to the remaining 612 frames. Both terms
remain visible: padding must not hide a bad impact, nor can the impact excuse
bad padding. AdamW uses `1e-4`, 120 steps, BF16 training autocast with FP32
weights. All audition generations remain FP32, 50 flow steps, CFG 4.5.
The complete training plus three before/during/after render sets took 314.21 s;
peak CUDA allocation was 4,655,436,288 bytes. Only adapter weights are saved.

| Development metric (lower is better) | Base | Step 40 | Step 120 |
| --- | ---: | ---: | ---: |
| Active-region flow MSE | 1.32718 | 1.26713 | 0.96294 |
| Padding-region flow MSE | 0.60084 | 0.61090 | 0.64510 |
| Balanced objective | 0.96401 | 0.93901 | 0.80402 |
| Full-horizon uniform MSE | 0.63801 | 0.64448 | 0.66136 |

**Learnability changed, quality improvement is not established.** Active error
improves 27.4%, but padding worsens 7.4% and the full-horizon error worsens 3.7%.
Frozen AST retains a coarse expected tag in all 10/10 cases at every stage;
that coarse gate misses the degradation in text alignment. Mean CLAP target
similarity across ten cases falls from 0.35858 to 0.33929. For the trained
knife/glass prompt it falls from 0.27602/0.22953 to 0.23840/0.10767 (two seeds).
The latter changes its highest-scoring description from wooden-stick/glass to
wood. Thus a lower flow loss must not automatically promote an adapter.
CLAP is not a calibrated naturalness judge either: these results support
**keeping the base**, not declaring every adapted sound perceptually worse.

Three original/VAE round-trip pairs are retained as controls. Their prior
multiresolution spectral errors are 0.6464/0.6671/0.6890 and envelope errors
0.0962/0.1352/0.1187: the codec is lossy, not an exact waveform identity path.
AST gives the real and round-tripped examples strong Ding/Clang tags, so exact
glass/striker identity cannot be inferred from those tags. Its existing NumPy
mel-filter warning remains visible; no calibrated quality gate is claimed.

The checkpoint can be loaded in a fresh process using the original pilot's
`--adapter` option. It validates the source revision, recorded checkpoint hash,
fixed adapter shape, complete key coverage and finite FP32 tensors; it cannot
replace arbitrary base weights. A separate reload run generated all 12 original
event prompts with seed 42, including steel, rolling, scraping and breaking:
[reloaded adapter preview](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-lora-reload-2026-09-05/preview.wav).
Four same-prompt controls match the step-120 stereo and mono WAV hashes exactly.
On these twelve original prompts/seed 42, CLAP exact-prompt top-one changes
from base 9/12 to adapter 8/12, mean target similarity 0.34070 -> 0.33703;
AST coarse expected tags remain 8/10 (steel is still unscored). This broader
check does not establish an overall gain either.
All 84 individual training-run WAVs and 26 reload WAVs passed hashes, dimensions,
duration, rate and unclipped PCM checks. The 16-second comparison and all four
previews were read back. Nothing replaces the liked engine glass profile.
Thirty focused pilot, training and existing audible-glass tests passed;
Ruff formatting/static checks and `git diff --check` passed. ProductCheck,
Cargo and engine audition were not run: no runtime/public-contract changes.

Reproduce with external output directories and the existing disclosed MP3 root:

```sh
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_tangoflux_train.py \
  --sources /absolute/external/ps2-freesound-wine-glass-v1/research \
  --output /absolute/external/tango-fit --steps 120
# Score step0, step40 and step120 with the same command, changing the directory:
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_text_tags.py \
  --source /absolute/external/tango-fit/step120/result.json \
  --output /absolute/external/tango-fit/step120/ast-clap.json --with-clap
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_tangoflux_pilot.py \
  --adapter /absolute/external/tango-fit/adapter-step120.safetensors \
  --output /absolute/external/tango-reload --seeds 42
```

Next: discriminate short-duration/padding and guidance effects on **free
generation**, with a same-prompt, same-seed base control, before another longer
fit. Then expand real internet training coverage beyond this family. Lower
denoising loss on these recordings is not proof of unseen material, shape,
force, speed, water-flow or rainfall control; those full-goal requirements and
engine admission remain open. No model-shopping or modal-MLP restart follows
from this result.

## Duration/guidance counterfactual (2026-09-05)

[Audible comparison, previously problematic seed 123](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-duration-guidance-2026-09-05/comparison.wav)
is 37.5 seconds. First the 1.5-second condition, then five seconds; within each,
CFG 1 base/adapted, then CFG 4.5 base/adapted/base-unconditional. All **28**
candidates (both seeds 42/123, both durations, CFG 1/2/4.5 and the conditional-only
adapter counterfactual at 4.5) remain individually available in that directory.
No model was trained or chosen from these scores. Generation plus CLAP scoring
took 256.38 seconds; AST was measured afterwards.

[Classifier-free guidance](https://arxiv.org/abs/2207.12598) combines conditional
and unconditional model predictions. Our counterfactual therefore separately
tests requested duration, guidance strength and retaining the **base**
unconditional prediction at the same evolving latent. Both branches use the
same batch shape; no external classifier guides generation. A local inference
wrapper avoids the pinned upstream revision's unsupported keyword in its
`guidance_scale <= 1` branch. At scale 1 it returns the conditional prediction
exactly, not an unconditional sample or a numerically unstable subtraction.
At CFG 4.5 the new wrapper's latent is bit-exact with the upstream method.
All four historical base/adapted 1.5-second WAV pairs replay exactly, and their
factored FP32 CLAP measurements reproduce the previous scores within `1e-6`.

| Duration / CFG | Base mean target cosine | Adapted minus base, two seeds |
| --- | ---: | --- |
| 1.5 s / 1 | 0.04281 | -0.00502, +0.01051 |
| 1.5 s / 2 | 0.07036 | +0.00429, +0.00718 |
| 1.5 s / 4.5 | 0.25277 | -0.03762, -0.12186 |
| 5 s / 1 | -0.04872 | -0.01134, -0.01908 |
| 5 s / 2 | 0.14132 | -0.01452, +0.00432 |
| 5 s / 4.5 | 0.26089 | -0.00992, -0.00159 |

- Longer requested audio reduces the large adaptation penalty at CFG 4.5 but
  does not reverse it. Changing duration also changes the scored clip length;
  this does not isolate conditioning from evaluator length sensitivity.
- Reducing CFG to 2 yields tiny positive changes at 1.5 seconds, but both
  absolute text scores are poor, and only one of two examples has a coarse
  glass/clink tag in AST. CFG 1 is worse. Selecting by improvement alone would
  mistake an inadequate baseline for useful sound.
- Keeping the base unconditional branch is not a repair: mean target deltas
  are -0.10176 at 1.5 seconds and -0.01710 at five seconds. This rejects that
  specific remedy, not the existence of all possible unconditional drift.
- In the previously bad seed-123/default-CFG example, the 10-ms energy peak
  moves from 670 ms (base) to 0 ms (adapted), or 110 ms with base-unconditional.
  This is a measured timing change, not proof of a particular physical cause.

The six original/VAE controls were additionally scored against the **same**
five-caption bank. Original target cosines are 0.40049/0.40553/0.41696; VAE
cosines 0.38179/0.44653/0.45743. All six nevertheless rank the wooden-stick/glass
caption above the knife/glass caption. The captions differ in wording beyond
the striker, so these ranks cannot establish striker identity or a material-pair
error. `real-positive-controls.json` preserves the scores and WAV hashes. The
probe now includes this measurement in its runnable path; in the first run it
was performed immediately afterwards. Neither prompt similarity nor coarse
AST tagging is a calibrated physical/naturalness admission gate.

All 56 individual WAVs passed hash, duration, dimensions, sample-rate and
unclipped-PCM checks. The comparison is playable and all four historical replay
controls passed. This narrows the next experiment to the **training objective**:
compare the hand-weighted active/padding objective against original uniform
full-horizon flow MSE, holding recordings, LoRA initialization, sampling and
steps fixed. Do not repeat the duration/CFG sweep as a supposed quality fix.

```sh
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_tangoflux_probe.py \
  --checkpoint /absolute/external/tango-fit/adapter-step120.safetensors \
  --output /absolute/external/tango-probe
```

## Uniform-loss control (2026-09-05)

[Real glass -> base -> balanced fit -> uniform fit](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-lora-uniform-2026-09-05/objective-comparison.wav)
repeats for seeds 42/123, 16 seconds total. The new external run is
`tangoflux-lora-uniform-2026-09-05`; its step40/120 directories contain the
five-prompt/two-seed WAV matrices and `ast-clap.json`. It completed 120 steps
and three render sets in 325.71 seconds.

This changes **only the optimized loss** to the original uniform full-horizon
flow MSE. All 27 cached posterior tensors have the same safetensors hash
`03a024064bf211f9f7d2773ac4df62fcf49f1334fd2059571353152810754631` as the balanced
run; source/crop metadata, LoRA configuration and all 120 sampled source indices
match. All twelve baseline stereo/mono pairs replay exactly. The same seeded
sampling recipe and initialization are retained. Baseline scoring therefore
uses the existing exact-WAV report rather than a redundant model run.

| Development metric | Base | Balanced step 120 | Uniform step 120 |
| --- | ---: | ---: | ---: |
| Active flow MSE | 1.32718 | 0.96294 | 0.97712 |
| Padding flow MSE | 0.60084 | 0.64510 | 0.59010 |
| Full-horizon flow MSE | 0.63801 | 0.66136 | 0.60990 |
| Mean CLAP target similarity, ten generations | 0.35858 | 0.33929 | 0.34063 |

Uniform MSE removes the active-versus-padding tradeoff: both regions improve
over the base. But free-generation alignment still degrades. The two trained
glass-prompt cosines are 0.23561/0.10643, versus base 0.27602/0.22953. Coarse
AST tags remain 10/10 and CLAP top-one remains 8/10, again hiding the degree of
degradation. Neither adapter is selected as a quality improvement. No repeated
human audition is needed to refrain from promoting an unproven candidate.

This does not prove insufficient model capacity or that neural sound synthesis
cannot work. Two comparable fits now show better denoising loss without a
free-generation gain. Stop nearby rank/lr/epoch/objective tuning and apply the
bounded-research escalation: distinguish corrupted/mismatched codec targets,
over-specialization to tiny constant-caption data, and insufficient validation
of generative quality. The next inexpensive executable discriminator is
**real waveform -> VAE mean versus sampled posterior -> audible reconstruction**
on the disclosed sources: current positive controls decode only the mean,
whereas training samples the posterior. Inspect the official codec/training
implementation and score both target paths before another fit. If targets are
sound, expand real internet data beyond this one glass family rather than
trying another local hyperparameter variant. The full multi-material,
water/rain, physical-control and new-condition goal remains unchanged.

```sh
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_tangoflux_train.py \
  --sources /absolute/external/ps2-freesound-wine-glass-v1/research \
  --output /absolute/external/tango-uniform --steps 120 --objective full
```

Verification: 36 focused tests, Ruff formatting/static checks and diff/link
checks passed. The probe's 56 and uniform run's 84 individual WAVs were checked
for hashes, dimensions, rate, duration and PCM headroom; both comparison files
and the three new stage previews were read back. No Cargo, ProductCheck or
engine audition was run: this remains an external-only Python lab change.

## Codec target discriminator and water/rain data (2026-09-05)

[Original -> VAE mean -> posterior sample, three recordings](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-codec-targets-2026-09-05/comparison.wav)
is an 18-second **reconstruction diagnostic**, not a new text generator.
The full external `tangoflux-codec-targets-2026-09-05` run retains three means
and nine posterior samples (seeds 0/42/123), each with a 1.5-second audition
and a five-second version exposing padded silence. The preview uses seed 42.

The official [TangoFlux training implementation](https://github.com/declare-lab/TangoFlux/blob/main/tangoflux/train.py)
samples the VAE posterior. In the installed, pinned
[Diffusers 0.30.3 codec](https://github.com/huggingface/diffusers/blob/v0.30.3/src/diffusers/models/autoencoders/autoencoder_oobleck.py),
the sample is mean plus softplus-derived standard deviation times Gaussian
noise. A focused test verifies our cache sampling against that distribution
with the same CPU generator; it does not assert CPU/GPU RNG stream identity.
The exact published posterior hash and all three mean-reconstruction WAV
hashes match. The large padding-latent standard deviation (~0.974 RMS) does
**not** imply noisy decoded audio.

The executable discriminator does not support posterior-sampling corruption
as the cause of the failed fits in these controls:

- Sample-versus-mean changes in relative spectral error range from -0.01160
  to +0.01456. Source-level mean errors remain 0.64642/0.66711/0.68902; the
  codec itself is lossy, but sampling adds no large systematic degradation.
- Sample target CLAP scores span 0.37099–0.46404, near the corresponding
  mean scores 0.38179/0.44653/0.45743. They remain well above the failed
  free-generation examples. AST likewise retains the same coarse ringing tags.
- Padding RMS is approximately -100 to -95 dBFS, 64–70 dB below active RMS;
  there is no large audible-energy tail induced by sampling in these controls.

The run took 12.60 seconds before AST measurement. All 48 newly rendered WAVs
and six referenced original WAVs passed hash/signal checks; the comparison
was read back at exactly 18 seconds. This rejects a proposed **sampling fix**
on the tested controls, not every possible codec limitation. No third fit on
the same three recordings was launched.

Instead, the next curriculum now has real examples beyond rigid glass:
[rain -> pouring water -> water drops, train/development examples](/home/kaifaty/.codex/experiments/nextengine/physical-sound/esc50-water-rain-2026-09-05/sources-preview.wav).
This 33-second preview contains **source recordings**, not generated output.
The [ESC-50 source repository](https://github.com/karolpiczak/ESC-50) is pinned to
`33c8ce9eb2cf0b1c2f8bcf322eb349b6be34dbb6`. Its metadata groups fragments from
one source recording in the same fold. We use folds 1–4 for training and fold
5 as disclosed development, not as an untouched or pretraining-independent test.

Downloaded **117 five-second WAVs from 100 source recordings**: 93 train,
24 development; 40 rain, 37 pouring-water, 40 water-drop clips. The source-ID
sets are disjoint. The dataset-level CC-BY-NC 3.0 notice, original full license
file and individual author/source notices are retained externally alongside
the pinned CSV, URLs and hashes. Three pouring-water source IDs
67152/79220/126433 have CC-Sampling+ notices; their audio remains unfetched
because that path has not been reviewed. No additional restriction is silently
treated as permission, and no redistribution or engine admission is claimed.

A bounded collision screen found no overlap with existing artifact filenames
or the 39 source IDs in 43 URL-bearing JSON metadata files under 2 MB. This
states the screen's actual scope, not a complete foundation-training audit;
legacy protected audio/roles were not opened or reassigned. The new data have
weak **event-class** captions only: physical attributes remain `null`. Neither
water intensity, rainfall rate, geometry nor material pairs are invented.
Ten source files touch full-scale PCM; the original samples and peak metadata
are retained, not falsely certified as artifact-free or silently edited.

An unmodified frozen CLAP diagnostic on all real examples matches their coarse
category on rain 39/40, pouring water 37/37, and water drops 34/40. All seven
disagreements remain in the corpus; these scores did not select recordings or
calibrate an acceptance threshold. `real-clap.json` records the complete matrix.
This is sufficient to start a bounded multi-event learning experiment with
before/after WAVs and unrelated glass/wood controls; it does not yet prove that
a fine-tuned generator will improve or control continuous physical properties.

```sh
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_codec_probe.py \
  --fit /absolute/external/tango-fit --output /absolute/external/codec-probe
lab/.venv/bin/python lab/scripts/physical_sound_water_sources.py \
  --prior-root /absolute/external/physical-sound \
  --output /absolute/external/new-water-corpus
```

The acquisition tool downloads only data, refuses malformed identities,
missing/unknown notices and known source-role collisions, and preserves
source-level splitting. It refuses existing output directories; use the
completed external corpus instead of reacquiring it or reassigning its roles.

Verification: 45 focused tests and Ruff/diff/local-link checks passed. No
Cargo/ProductCheck or engine audition was run; no runtime or public contract
changed. The base generator and both earlier adapters remain untouched.

## Multi-event water/rain learning (2026-09-05)

[Listen: rain, pouring water, water drops — base then step 240 for each](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-water-rain-fit-2026-09-05/comparison.wav).
This 33-second preview uses seed 42, with no recording at inference. Both
seeds 42/123, the step-40 intermediate, empty-prompt controls and glass/wood
regression examples remain in the external run; the preview does not select
the better seed. [Final water drops, seed 123](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-water-rain-fit-2026-09-05/step240/water_drops-seed123.wav)
is also directly playable. These are new generations, not codec reconstructions.

The existing trainer now accepts the completed attributed `--corpus`, validates
source hashes/PCM/roles, caches three event captions and uses full five-second
targets (108 active latent frames). It preserves the old glass invocation.
The 93 permitted training clips and 24 disclosed development clips remain
source-ID-disjoint. Random sampling visited 92/93 training clips in 240 updates:
82 rain, 72 pouring-water and 86 water-drop updates. No development audio enters
the optimizer. Generic captions do not establish flow rate or physical controls.

The run uses the same frozen TangoFlux/T5/VAE, rank-8 q/v LoRA (786,432 learned
parameters), AdamW 1e-4, BF16 training and FP32 50-step generation. Uniform
upstream flow MSE is the objective; neither AST nor CLAP supplies training
reward. Cached conditioning matches exact upstream loss at
0.46225404739379883. All four initial glass/wood mono and stereo controls
replay the prior baseline exactly. `--diagnostics` automatically runs the
frozen AST/CLAP measurements after checkpoint generation, without a separate
manual scoring step. Total execution including diagnostics: 487.99 seconds;
peak Torch CUDA allocation: 4,654,928,384 bytes (not total device usage).

The result is **not a replacement for the base model**. Development active
flow MSE falls 25.43% (1.55279 -> 1.15788), and full-horizon MSE falls 16.11%
(0.76171 -> 0.63897), with improvements in all three event classes. Yet
free-generation alignment regresses overall:

| Mean target CLAP, two seeds | Base | Step 40 | Step 240 |
|---|---:|---:|---:|
| Rain | 0.46055 | 0.46127 | 0.45653 |
| Pouring water | 0.35234 | 0.35083 | 0.31383 |
| Water drops | 0.41781 | 0.41930 | 0.44578 |
| Glass regression control | 0.25277 | 0.25101 | 0.10684 |
| Wood regression control | 0.39026 | 0.38967 | 0.33657 |

CLAP top-1 across five prompts drops from 8/10 to 6/10; all 10 still beat
their empty-prompt controls. AST expected coarse tags appear in the top five
for 10/10 base and step-40 sounds but 9/10 final sounds: glass seed 123 loses
its glass/clink match. Both final pouring-water sounds prefer the water-drop
caption in CLAP, while AST emphasizes taps/water. The water-drop score gain
occurs on both seeds, but this is prompt alignment, not independently calibrated
naturalness or proof of a physical response. Two seeds do not establish robust
generalization; classifier pretraining overlap remains unknown. AST retains
the previously disclosed NumPy frontend/zero-mel-filter warning limitation.

This is evidence that multi-event learning changes generated sounds, and that
the automated diagnostics can expose cross-event regression despite improving
training loss. It is not evidence that simply enlarging the data or training
longer solves the goal. Keep the base and the liked demo unchanged. The next
bounded discriminator should address retention of the base's useful behavior
(for example a researched reference-model/rehearsal control), while retaining
the pouring-water versus droplets distinction; do not extend this adapter's
epochs or declare its best class a general solution. No repeat of the already
negative glass duration/CFG, posterior-mean or modal-MLP experiments is justified.

```sh
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_tangoflux_train.py \
  --corpus /absolute/external/esc50-water-rain-2026-09-05 \
  --output /absolute/external/new-water-rain-fit \
  --steps 240 --objective full --diagnostics
```

All 96 individual WAVs (72 generated, 24 original/codec controls) passed
hash, rate, duration, PCM type, shape, non-silence and headroom checks. Three
24-second stage previews and the 33-second comparison were read back; both
adapter hashes match. The focused suite has 43 passing tests, including new
caption/role/source-split and five-second loss-region cases. Ruff formatting,
static analysis and diff/local-link checks passed. An initial module-qualified
test command failed because an existing test imports a sibling by bare name;
the corrected invocation uses `PYTHONPATH=lab/tests`. No code workaround or
test exclusion was needed. Cargo/ProductCheck and engine audition were not run:
this is an external Python experiment with no runtime/public-contract change.

## Base-behavior retention discriminator (2026-09-05)

[Rain/pouring water/drops: base -> retained-prior fit](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-prior-retention-2026-09-05/comparison.wav)
is a 33-second seed-42 comparison. [Glass: base -> unregularized -> retained,
for seeds 42 then 123](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-prior-retention-2026-09-05/glass-retention.wav)
is 12 seconds. All candidates remain available, including the weaker ones.

The bounded research separated three hypotheses: unrestricted SFT forgets
useful base behavior; coarse water captions leave event ambiguity; and the
diagnostics are imperfect proxies for naturalness. The previous exact source,
loss and PCM controls argue against an accidental input/precision change.
[Diffusers 0.30.3 prior-preservation documentation](https://huggingface.co/docs/diffusers/v0.30.3/training/dreambooth#prior-preservation-loss)
describes retaining learned image behavior with model-generated examples.
This motivates, but does not establish, an analogous audio experiment.
[TangoFlux v2, 10 April 2025](https://arxiv.org/html/2412.21037v2) also distinguishes
flow training from preference alignment and explicitly treats CLAP as a proxy.
Neither source proves our proposed regularizer or makes CLAP a physical judge.

Implemented one reversible `--prior-weight 1` control in the existing trainer.
Before adding LoRA, capture ten states (steps 0,5,...,45 of 50) from each of
16 base trajectories: current five prompts, nonduplicate original pilot
prompts, and empty prompt. Use seed 7 only, CFG 4.5, both unconditional and
conditional branches. The 160 paired targets are frozen BF16 field predictions
on FP32 base trajectories, not real recordings, labels, or physical velocities.
They are stored externally in `prior.safetensors` (135,536,048 bytes) with
prompt/seed/duration/step metadata. This is field-distillation regularization,
not an exact DreamBooth implementation, KL loss, or preference optimization.

Each update adds one randomly sampled prior-field MSE gradient to the real
audio full-horizon MSE gradient, before their shared clipping/optimizer step.
The prior RNG is separate; it visits 121/160 paired states in 240 updates.
The real sources, posterior tensors, all 240 source indices and 24 baseline
WAVs match the unregularized run exactly. Captured FP32 replay matches exactly
at the first state of every prompt; the zero-initialized LoRA also reproduces
the first BF16 teacher target exactly. Evaluation seeds 42/123 never enter the
prior bank. Prompts overlap intentionally: this is not unseen-prompt evidence.

Result: **partial retention and localized alignment gains, not promotion**.
Development active/full MSE improves 25.01%/16.29% versus base, so the penalty
does not simply prevent fitting. Runtime including diagnostics: 639.24 seconds;
peak Torch CUDA allocation 4,654,928,384 bytes. The five-prompt comparison is:

| Mean target CLAP, two seeds | Base | Unregularized 240 | Prior 240 |
|---|---:|---:|---:|
| Rain | 0.46055 | 0.45653 | 0.47141 |
| Pouring water | 0.35234 | 0.31383 | 0.32604 |
| Water drops | 0.41781 | 0.44578 | 0.44119 |
| Glass | 0.25277 | 0.10684 | 0.19148 |
| Wood | 0.39026 | 0.33657 | 0.36727 |

Rain and drops each gain versus base on both seeds. AST expected-tag coverage
returns from the unregularized 9/10 to 10/10, including glass seed 123. However,
glass/wood remain below base alignment, both pour outputs still prefer the
drop caption, and top-1 remains 6/10 versus base 8/10. Overall mean target
CLAP is 0.37475 / 0.33191 / 0.35948 for base/unregularized/prior. All ten
outputs beat their empty-prompt controls. Step 40 remains near baseline.
No claim of calibrated perceptual quality, exact striker material, flow rate,
unseen-object generalization or engine readiness follows from these numbers.

A separate post-fit reload audit compares both saved adapters on all 160
frozen bank states (`field-audit/result.json`). Branch full-horizon MSE drops
0.0106062 -> 0.00209489 (80.25%); active-region MSE drops 80.07%. Recombine
branch errors using the actual sampler formula, `delta_u + 4.5*(delta_c-delta_u)`:
guided full-horizon MSE drops 0.0267267 -> 0.00929973 (65.20%), and active
MSE drops 0.0425852 -> 0.0156231 (63.31%). Thus retention acts on its intended
quantity, but residual guided drift and imperfect semantic targets remain.
This audit reuses disclosed training states, not independent evidence.

Before another fit, test the candidate against the base on new seeds and
unseen prompt wording/combinations, including regression events. Retain the
existing water/pour distinction as an explicit failure. A larger regularizer
or guided-field penalty is only a candidate if further evidence warrants it;
do not launch a weight/epoch sweep or present the two-seed gains as the full
goal. The base and liked demo stay unchanged.

```sh
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_tangoflux_train.py \
  --corpus /absolute/external/esc50-water-rain-2026-09-05 \
  --output /absolute/external/new-prior-fit --steps 240 \
  --objective full --prior-weight 1 --diagnostics
```

Verification: 45 focused tests pass, including frozen-target, nonfinite/shape
rejection and student-gradient cases. Ruff formatting/static analysis,
diff/local-link checks pass. All 96 individual WAVs pass hash/shape/type/rate/
duration/non-silence/headroom checks; three 24-second stage previews and the
33/12-second comparisons were read back. Teacher and adapter hashes were
checked before the separate audit. All jobs are terminal. Existing AST
frontend/pretraining/proxy limitations remain. No Cargo/ProductCheck or engine
audition was run: external-only Python code and artifacts, no product change.

## New prompts and seeds: transfer check (2026-09-05)

[Light rain, heavy rain, individual drops: base -> prior-retained](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-transfer-prior-2026-09-05/water-comparison.wav)
(33 seconds) and [metal/wood rod on glass, scraping, rolling: base -> prior](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-transfer-prior-2026-09-05/interaction-comparison.wav)
(44 seconds) use the first fixed seed 314, not a selected best seed. The full
88-second previews and individual seed-2718 sounds remain in each external run.

The generator now accepts `--prompts` and optional `--diagnostics`; the default
12-case behavior remains available. The committed
[transfer prompt file](../../lab/profiles/physical-sound-transfer-prompts.json)
contains three unchanged training captions and 13 exact-new descriptions.
Cases request different rain intensity, a jug/bowl, a tap/puddle, metal versus
wooden rods on the same jar/board/pipe, scraping, rolling and bottle fracture.
These are requested conditions, **not measured physical ground truth**. None
of the 13 descriptions matches the prior bank; both new seeds 314/2718 differ
from training/rehearsal and earlier evaluation. Foundation pretraining overlap
is still unknown. No fitting or candidate selection uses these outputs.

Both models use identical prompt-file hashes, 50 FP32 steps, CFG 4.5 and five
seconds. Each produces 32 candidate sounds and two empty-prompt controls, at
44.1-kHz stereo and 16-kHz mono: 136 individual WAVs total. Generation takes
263.06 seconds for base and 271.69 for prior, excluding the following automatic
AST/CLAP pass. The already-published prior step-240 adapter is loaded through
the existing hash/config/tensor checks.

**Result: mixed transfer, no broad improvement.** Mean target CLAP is
0.369675 -> 0.370049; only 14/32 paired sounds improve. On the three original
captions with new seeds, 3/6 improve and mean change is -0.005058. On the
13 new descriptions, 11/26 improve with mean change +0.001628. Top-1 among
the 16 closely related prompts falls 19/32 -> 17/32; this bank includes near
synonyms, so the count is not a calibrated accuracy or naturalness measure.
Both versions beat their empty-prompt control on 28/32 cases.

| New description, mean target CLAP | Base | Prior |
|---|---:|---:|
| Light rain on roof | 0.45005 | 0.47299 |
| Heavy rain on roof | 0.45805 | 0.48262 |
| Drops from tap into puddle | 0.37526 | 0.35646 |
| Metal rod on glass jar | 0.25660 | 0.28145 |
| Glass marble on wooden table | 0.30691 | 0.32388 |

AST coarse expected-tag coverage is 21/28 -> 22/28; four steel outputs are
unscored, not accepted. The extra match is light rain seed 314. Both models
miss the requested glass/metal tag at seed 314 and the exact scraping/rolling
tags on both seeds. Scraping is tagged Rub/Wood/Filing, which illustrates the
ontology limitation rather than proving that it sounds wrong. Glass striker
swap CLAP margin is negative for seed 314 in both models (-0.02040/-0.01510),
positive for seed 2718 (0.06883/0.06866). Wood and steel paired margins are
positive, but neither CLAP nor AST establishes the true striker material.

Both models have positive light/heavy-rain swap margins on both seeds. Heavy
rain also has greater raw RMS: +12.76/+11.57 dB for base and +10.27/+10.35 dB
for prior. This is a qualitative response in these samples, not calibrated
rainfall rate, realism, or evidence that the fine-tune created the capability.

The bottle-fracture case exposes an important failure beyond fitting: seed
314 is effectively silent in both models, at -99.64/-99.39 dBFS native RMS,
and AST agrees with the silence control. Seed 2718 produces breaking/glass
tags at -14.57/-14.10 dBFS. A numerically nonzero WAV is **not** proof of an
audible event. Keep the failed outputs; do not normalize their codec noise into
an apparent sound or choose the other seed to declare success.

Next: a bounded timing/prompt discriminator on the base, before another fit.
Retain the whole decoded 30-second horizon with the same five-second duration
condition, replay the failed seed exactly, and compare the positive seed and
minimal wording counterfactuals (e.g. removing `empty` or simplifying the event
sequence). This distinguishes an event outside the cropped window, failure
to generate it at all, and prompt sensitivity. Until that is inspected, do
not assert which is causal or launch another SFT/regularizer sweep. No adapter
promotion or change to the liked demo follows from this transfer check.

```sh
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_tangoflux_pilot.py \
  --prompts lab/profiles/physical-sound-transfer-prompts.json \
  --seeds 314 2718 --output /absolute/external/new-transfer-base --diagnostics
# Repeat with a different output and the existing --adapter checkpoint option.
```

Prompt files are bounded to 64 KiB/32 cases/512 printable characters per
description, with unique safe IDs and no audio-input fields. An optional
`diagnostic_id` selects a known coarse tag group; otherwise unknown IDs are
explicitly unscored by AST, while CLAP still compares their text. Tests cover
defaults, valid custom cases, traversal/duplicate/reserved IDs, unknown fields,
bad captions and diagnostic groups. A final guard-only change also bounds
whitespace-padded descriptions; the executed snapshots preserve the exact run
code, and all experimental prompts satisfy both versions of the guard.

Verification: 47 focused tests, Ruff formatting/static checks and diff/local
links pass. All 136 WAVs have matching hashes, dimensions, rates, durations,
PCM types and headroom; two 88-second previews and 33/44-second comparisons
were read back. This is signal-integrity verification, not an audibility or
quality acceptance. Both jobs and diagnostic passes are terminal. No Cargo,
ProductCheck or engine audition was run; this remains an external Python lab.

## Late-event diagnosis and extraction (2026-09-05)

[Recovered bottle-fracture candidate, same seed 314](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-fracture-event-matched-2026-09-05/break-glass-seed314.wav)
is a five-second excerpt of the **same generated full horizon**, with no new
training or reference recording. [Failed prefix -> recovered event](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-fracture-event-matched-2026-09-05/comparison.wav)
is 11 seconds; its first five seconds are intentionally almost silent.

The bounded research tested three explanations: an event outside our crop,
total omission, and wording sensitivity. An upstream user reported problems
with sub-ten-second duration conditioning in
[TangoFlux issue 31, 6 January 2026](https://github.com/declare-lab/TangoFlux/issues/31).
That report is not a confirmed diagnosis of our run. The
[official demo](https://huggingface.co/spaces/declare-lab/TangoFlux/blob/main/app.py)
also takes a prefix of the requested length; our old prefix convention was
not an independently validated event-timing guarantee.

`--keep-full-horizon` now retains the decoder's actual 29.9537415-second
output while leaving duration conditioning at five seconds. The
[three-prompt control](../../lab/profiles/physical-sound-fracture-timing-prompts.json)
uses the original bottle description, removes `empty`, or simplifies the
sequence to a bottle shattering on a stone floor. Seeds 314 and 2718 retain
the failure and positive control. Eight generations including empty prompts
take 73.09 seconds before automatic diagnostics; all full and prefix WAVs stay
in `tangoflux-fracture-timing-2026-09-05`.

**The tested failure is an out-of-window event, not total omission.**

| Seed 314 wording | First 5 s RMS, dBFS | Remaining horizon RMS, dBFS | Peak time, s |
|---|---:|---:|---:|
| Original | -99.64 | -20.25 | 18.170 |
| Without `empty` | -99.67 | -20.91 | 17.534 |
| Direct fracture | -99.76 | -21.37 | 12.667 |

Over 99.9999997% of the raw energy is after five seconds in all three cases.
Their first detected activity is around 12.66 s. Removing an adjective or
simplifying the sequence does not fix timing at this seed. Seed 2718 instead
peaks at 1.67–1.68 s and contains audible breaking in the original prefix.
The prior seed-2718 mono/stereo WAVs replay exactly. Seed 314 remains at the
codec-noise floor but is not bit-exact: 67 mono and 868 stereo samples differ
by at most one PCM16 unit. This does not explain the approximately 80-dB
head/tail difference; no exact-replay claim or retry-to-green test is made.

Added `--extract-events SOURCE_RESULT` as a separate offline postprocessor.
It reads hash-checked generated full-horizon PCM and finds a candidate onset
using 10-ms RMS blocks, threshold `max(-50 dBFS, 0.1 * peak block RMS)`, and
50-ms pre-roll. It copies a requested-length window without amplification or
time stretching, records its source offset, any zero-padding, and whether
above-threshold activity remains after the window. The threshold is an
experimental energy heuristic, **not** calibrated perceptual acceptance.
Below-threshold outputs are reported as undetected; partial matrices do not
run the complete-matrix diagnostic. File/type/hash/bounds errors fail closed.
This is for discrete-event candidates, not a general policy for rain or water.

Final evidence is `tangoflux-fracture-event-matched-2026-09-05`. Both prompted
and empty-prompt controls use the same extraction rule. An initial directory
`tangoflux-fracture-event-window-2026-09-05` kept the old empty prefixes; its
empty-control comparison is superseded and must not be used. All files remain
available. The corrected extraction selects offsets 12.61 s for seed 314 and
0.70 s for seed 2718, without padding. All six candidates still have later
activity: these are useful excerpts, not proven complete isolated fractures.

AST expected glass tags improve from 3/6 prefixes to 6/6 excerpts; Breaking
is the top tag for every extracted candidate. Mean target CLAP rises
0.28179 -> 0.43874. The formerly failed original prompt rises 0.16196 ->
0.47726 at seed 314, with excerpt RMS -20.37 dBFS. All six exceed equally
processed empty controls. This is an extraction gain from an existing neural
generation, not a learned weight improvement or proof of exact physical
response. Crops use already headroom-attenuated full-horizon PCM; levels are
not force/energy calibration.

Next, reassess discrete-event base/prior differences with identical event-aware
processing before attributing every prefix-score regression to forgotten
timbre. Keep raw prefixes and full horizons as controls. Continuous events
need a different window policy. The model's duration/sequence control is still
unrepaired, and no runtime integration or model promotion follows from this.

```sh
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_tangoflux_pilot.py \
  --prompts lab/profiles/physical-sound-fracture-timing-prompts.json \
  --seeds 314 2718 --keep-full-horizon --diagnostics \
  --output /absolute/external/new-fracture-timing
HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 lab/.venv/bin/python \
  lab/scripts/physical_sound_tangoflux_pilot.py \
  --extract-events /absolute/external/new-fracture-timing/result.json \
  --output /absolute/external/new-event-windows --diagnostics
```

Verification: 51 focused tests pass, including late-event versus silence,
sample-preserving crops, noise rejection, padding/truncation flags, matched
empty-control extraction and source-hash rejection. All 48 authoritative
generation/extraction WAVs pass hash/PCM/shape/rate/length/headroom checks;
the 11-second comparison was read back. Ruff formatting/static checks and
diff/local links pass. Executed versions have source hashes in external
evidence; all 16 extracted WAVs replay exactly with the final tightened guards.
All jobs terminal;
no Cargo/ProductCheck or engine audition, since this is external Python work.

## Event-matched impact comparison and physical-control data (2026-09-05)

Playable comparisons: [glass](</home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-impact-window-prior-crops-2026-09-05/glass-comparison.wav>),
[wood](</home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-impact-window-prior-crops-2026-09-05/wood-comparison.wav>),
[steel](</home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-impact-window-prior-crops-2026-09-05/steel-comparison.wav>).
Each is 22 seconds, seed314 fixed in advance: base/metal striker, prior/metal,
base/wooden striker, prior/wooden, with 0.5-second gaps. PCM levels are retained,
not independently loudness-matched. These are experimental candidates.

The [seven-prompt subset](../../lab/profiles/physical-sound-impact-window-prompts.json)
repeats six material pairs and bottle fracture at seeds314/2718, 50 FP32 steps,
five-second conditioning, with the full 29.9537-second decoder output retained.
The same event rule processes both models and their empty-prompt controls.
External roots are `tangoflux-impact-window-{base,prior,base-crops,prior-crops}-2026-09-05`.
All four result/diagnostic sets are complete. No weights were trained here.

| Diagnostic, 14 candidates | Base prefixes | Prior prefixes | Base event crops | Prior event crops |
|---|---:|---:|---:|---:|
| Mean target CLAP | .318445 | .317866 | .373407 | .363659 |
| Target rank1 among seven captions | 6 | 5 | 7 | 8 |
| Beats equally processed empty prompt | 10 | 10 | 14 | 14 |
| AST expected tag in top5, ten scored | 8 | 8 | 10 | 10 |

Steel's four cases remain unscored by AST. After matching extraction, prior
improves target cosine in only **1/14** cases versus 4/14 prefixes; every
two-seed class mean is lower than base. Rank1 moves in the other direction,
illustrating why one diagnostic cannot establish perceptual superiority.
The decision is **retain base, do not promote or repeat this LoRA sweep**.
Timing affected our earlier comparison but does not establish a learned gain.

Glass/metal seed314 is also late: base crop starts6.67s, prior7.91s. All other
paired offsets agree, ranging0–12.61s. No padding; all28 crops have subsequent
above-threshold activity, so these are excerpts, not complete isolated events.
All six striker-swap margins per model are now positive (base .02198–.06146,
prior .01986–.06273). This revises the old negative glass margin at314: that
failure is window-sensitive, not proven striker confusion. Neither positive
margin nor broad glass tags validate the physical identity of both materials.

Replay against the prior transfer artifacts: prior28/28 candidate mono/stereo
prefixes match exactly; base26/28 do. Base glass/metal314 differs by at most
one PCM unit in1150 mono/7015 stereo samples. All192 full/prefix/crop WAVs
pass SHA256/PCM/rate/dimension/headroom checks, and three comparisons read back
at22s. Run the earlier full-horizon/extraction commands with this new profile
for reproduction; add the existing prior-retention step240 adapter for prior.

### Next physical axis: published controlled friction recordings

The [Cluster Haptic Texture Dataset paper, arXiv v4, 6 November2025](https://arxiv.org/html/2407.16206v4)
describes118 surfaces, a fixed urethane-rubber probe, five commanded velocities
20–60mm/s, eight directions and0.5/1N loads. This offers measured sliding
controls, **not** arbitrary impact pairs, fluid parameters or object geometry
transfer. [Figshare article v5](https://api.figshare.com/v2/articles/29438288/versions/5)
identifies the files and CC-BY4.0 terms. Attribution is retained with the data.
The paper distinguishes noise-cancelled mono audio from raw main/machine-noise
microphone channels and records force/position separately. The two raw channels
are sensors, not a spatial stereo scene. These sources motivate the experiment;
they do not prove our eventual model's physical accuracy.

The bounded [acquisition script](../../lab/scripts/physical_sound_texture_probe.py)
downloads12 disclosed conditions: Nyatoh wood0, stainless steel65, float glass74;
20/60mm/s ×0.5/1N, direction0, repeat0. It preserves both audio versions and
force/position CSVs. This is a feasibility/development probe, not a held-out
test or training run. No existing protected roles were reopened. A bounded
name/article-ID scan found no prior local references, not a pretraining audit.

Authoritative root: `cluster-texture-controls-canonical-2026-09-05`.
All50 selected files total19,495,822 bytes and pass member CRC/local SHA256.
Only ZIP ranges were fetched, not the15.2GB archive. Whole-archive MD5 is NOT
verified; pinned version metadata, multipart ETag and member hashes are recorded.
The miniature archive lacks raw audio despite its README: the first acquisition
failed explicitly. The full archive contains it. A second attempt exposed
ten-second signed-redirect expiry; the final reader resolves Figshare's canonical
URL per range. Both failed directories remain, and expired signed query details
were removed from the failed diagnostic. Do not reuse a resolved signed URL.
The texture spreadsheet has malformed font-only `&quot` attributes; inspection
repaired those in memory only, preserving the downloaded original unchanged.

[Recorded friction preview](</home/kaifaty/.codex/experiments/nextengine/physical-sound/cluster-texture-controls-canonical-2026-09-05/clean-controls-preview.wav>)
is46.584s: wood, steel, glass; within each, slow/light, slow/heavy, fast/light,
fast/heavy. One shared gain78.515 preserves relative levels; this is **recorded,
not generated** sound. A separate raw two-microphone preview uses gain34.054,
so its absolute playback level must not be compared with the clean preview.
Original files are unmodified. `signal-audit.json` records segment order,
gains and measurements over the central54mm of travel, derived from position.

Measured central speeds are19.755–19.828 and59.459–60.155mm/s; median measured
forces .529N and1.029–1.049N. Labels therefore remain **commanded**, with sensor
observations separate. Clean central RMS is−77.19…−65.66dBFS; main-microphone
RMS−54.52…−41.97dBFS. Faster motion increases clean RMS for all six paired
conditions; heavier loading increases it in all six pairs. This small probe
does not distinguish contact response from motion-dependent machine residuals
or preprocessing. Raw noise channels and repeat/velocity transfer are necessary
controls before a learned physical-response claim, not reasons to withhold a
clearly labelled experimental synthesis.

Next end-to-end checkpoint: expand this fixed friction grid to repeated scans
and intermediate speeds, then produce a conditional neural sound with a
held-out-speed/repeat comparison and a non-neural baseline. Keep sensor labels,
machine-noise controls and waveform outputs together; do not add another
generic caption-only SFT sweep or a separate validator-only milestone.
This is one missing physical axis of the full goal, not a replacement objective.

```sh
lab/.venv/bin/python lab/scripts/physical_sound_texture_probe.py \
  --output /absolute/external/new-texture-probe
```

Verification:22 focused acquisition/pilot tests pass; Ruff/static/format and
diff/link checks pass. The four new tests cover the fixed physical grid,
separate raw channels, malformed audio rejection and silence preservation.
All jobs terminal. No Cargo/ProductCheck or engine audition: external lab only;
the base/demo and product contracts remain unchanged.

## Neural friction from physical conditions (2026-09-05)

[Generated glass friction,40mm/s,0.5N,seed2718](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-neural-rank4-glass-40-2026-09-05/generated.wav>)
is two seconds, made **without an input recording**. This is a rubber probe
sliding on float glass, not glass impact/ringing. [Six-second comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-conditional-rank4-2026-09-05/glass-comparison.wav>)
plays real -> neural -> interpolation at0.5N, then the same at1N. Each clip
is0.75s with a0.25s gap. One shared playback gain100 preserves their levels.

The acquisition script's `--training-grid` adds intermediate velocities and
repeat1 without changing its original12-condition default. The new external
`cluster-texture-training-grid-2026-09-05` has60 recordings,242 files and
86,691,095 selected bytes, including both microphone channels and sensors.
All source hashes pass. Training is24 repeat0 scans at20/30/50/60mm/s.
All12 scans at40mm/s are disclosed unseen-speed development; the other24
repeat1 scans test repeat transfer. These are the same three surfaces/fixed
probe, not independent objects, hidden tests or pretraining-independent sound
categories. Commanded controls and measured force/speed remain separate.

[The bounded fitting script](../../lab/scripts/physical_sound_texture_fit.py)
takes surface one-hot, normalized commanded speed and normal force. It predicts
a stationary log power spectrum. Input audio is used only in offline training
and comparison. The standalone `--render-model` path reads model metadata and
safetensors, not a corpus or recording; domain, checkpoint hashes, finite values,
tensor shapes, duration and amplitude are checked. A test forbids audio reads
during standalone inference. Unknown surfaces and out-of-range controls fail.

This follows the general learned-controller plus signal-processing approach
described by [DDSP, Engel et al.,2020](https://arxiv.org/abs/2001.04643), not its
trained model or a reproduction of its reported quality. Our renderer shapes
fresh Gaussian noise using the predicted one-sided power density, removes DC
and adds10ms endpoint fades. It cannot reproduce impacts, deterministic phase,
contact sequences or arbitrary nonstationary structure. It is an external
stochastic texture baseline, **not** an admitted physical formula/runtime model.

Each source contributes the same0.75-second central sliding window, located
from position CSVs. Mono is resampled44.1 ->22.05kHz; Welch spectra use1024
samples/50% overlap. No per-recording loudness normalization. Both fits use
CPU FP32, seed23,1500 full-batch AdamW updates at1e-3, weight decay1e-4;
only training rows determine the mean, optimizer targets and optional basis.
No pretrained weights. Baseline interpolates training log spectra in speed
for the same surface/load. Oracle rendering uses the target's own spectrum
as an explicitly reference-dependent representation control, not an inference
result. Raw machine-microphone spectra are mean-level-matched shape controls,
not SNR estimates or proof of noise removal.

First model:5 ->64 ->64 ->513,37,889 learned parameters. It fits training
spectra well but loses all12 unseen-speed comparisons. The single corrective
experiment restricts outputs to four PCA components derived only from the24
training spectra:5 ->64 ->64 ->4,4,804 learned parameters plus fixed basis.
This tests fitting of incidental spectral detail; it is not an epoch/width sweep.

| Spectrum RMSE,dB, lower is better | Full network | Four-component network | Interpolation |
|---|---:|---:|---:|
| Train24 | .22379 | 1.08692 | 0 (stored training spectra) |
| Unseen speed40mm/s,12 | 2.07522 | 1.68983 | 1.72887 |
| Repeat development24 | 1.31552 | 1.34025 | 1.33773 |

Full/rank4 win0/12 and6/12 unseen-speed cases against interpolation; repeat
wins20/24 and15/24. The smaller model improves this narrow network prediction,
but its mean advantage over interpolation is only0.039dB, not a robust benefit.
The outcome is an audible, physically conditioned neural **candidate**, not
quality acceptance or superiority of neural synthesis.

Both runs also publish real/neural/interpolation/oracle WAVs for all six
surface/load combinations at40mm/s, repeat0, noise seed314. Each full preview
is24s. `waveform-audit.json` evaluates actual PCM, not just predicted spectra:
mean spectral RMSE full2.00855, rank4 1.77531, interpolation1.79547,
oracle .92227dB. Mean25ms envelope coefficient of variation: real .05813,
rank4 .04445, oracle .06197. Thus neither an exact waveform match nor a severe
temporal-representation failure is established. Mean-level-matched machine
shape RMSE is10.48dB on unseen-speed sources; this alone cannot rule out a
motion-dependent recording/preprocessing shortcut. Metrics are diagnostic,
not a calibrated perception/realism validator.

Roots: `texture-conditional-spectrum-2026-09-05`,
`texture-conditional-rank4-2026-09-05`; standalone inference roots
`texture-neural-glass-40-2026-09-05` and
`texture-neural-rank4-glass-40-2026-09-05`. Checkpoints are154,136/30,068 bytes.
The standalone examples use a second noise seed2718. Generated/artifact hashes
and attribution stay external; model/data are not installed in the demo.
Later code adds the same waveform diagnostic to future fit results; executed
reports preserve their original hashes and separate PCM audit files.

Before a third model variant, use a bounded residual/repeat/noise discriminator
and crossed velocity checks to establish whether the apparent gain persists
beyond the chosen40mm/s split. Keep development reuse disclosed and do not
turn an opened fold into independent evidence. The next checkpoint must still
include generated sounds at other velocities, not a validator-only package.
Do not infer that more units/epochs or a time-varying decoder fixes this result.
The full impacts/water/rain/geometry/both-materials goal remains open; this is
one narrow forward-conditioning capability, not a replacement objective.

```sh
lab/.venv/bin/python lab/scripts/physical_sound_texture_probe.py \
  --training-grid --output /absolute/external/new-texture-grid
lab/.venv/bin/python lab/scripts/physical_sound_texture_fit.py \
  --corpus /absolute/external/new-texture-grid/result.json \
  --rank 4 --output /absolute/external/new-texture-fit
lab/.venv/bin/python lab/scripts/physical_sound_texture_fit.py \
  --render-model /absolute/external/new-texture-fit --texture 74 \
  --speed 40 --force 0.5 --seconds 2 --seed 2718 \
  --output /absolute/external/new-texture-inference
```

Verification:29 focused tests pass, including source grid/split, input domain,
PSD scale, noise seeds, reference-free inference, checkpoint rejection, rank
shape and waveform metric controls. Ruff/static/format and diff/link checks
pass.242 source files,52 individual/control/preview WAVs and the additional
six-second glass comparison pass hash/signal checks. Both fits and inference
jobs terminal. No Cargo/ProductCheck/engine audition; no production promotion.

## Crossed velocities and recording-channel countercheck (2026-09-05)

[Glass30/50mm/s comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-cross-speed50-2026-09-05/glass-cross-speed-comparison.wav>)
is six seconds: real -> neural -> interpolation at30mm/s, then50mm/s,
0.5N/seed314. Each velocity has its **own excluded-velocity model**, not one
promoted model. One gain100, no individual loudness matching. All six material/
load cases, including poor results, remain in each24-second full comparison.

Added `--heldout-speed {30,40,50}` to the existing fitter. Default40 and its
previous artifacts remain unchanged. New roots `texture-cross-speed30-2026-09-05`
and `texture-cross-speed50-2026-09-05` retain the same rank4 architecture,
seed23,1500 updates and24 training scans, excluding the chosen velocity and
all repeat1 recordings. Mean/PCA basis are recomputed on each fold's training
rows only. This explicitly reuses disclosed development data in cross-validation;
the models trained with40mm/s are not evidence that those models generalize to40.
No protected/one-shot/test objects were reopened and no new capacity sweep ran.

| Excluded speed | Neural mean spectrum RMSE,dB | Interpolation | Neural wins |
|---|---:|---:|---:|
| 30mm/s | 1.90424 | 1.75858 | 2/12 |
| 40mm/s, previous run | 1.68983 | 1.72887 | 6/12 |
| 50mm/s | 1.84997 | 1.78802 | 3/12 |
| All three disclosed folds | 1.81468 | 1.75849 | 11/36 |

The hypothesis of a stable gain across these speeds is contradicted. A third
width/epoch/basis sweep on these three surfaces is not the next action.
The neural baseline remains playable; neither it nor interpolation is promoted.

`texture-cross-speed50-2026-09-05/cross-speed-audit.json` records the bounded
discriminators. For each surface/load/velocity, both repeats receive the same
prediction. Their mean squared error decomposes exactly into squared deviation
from the two-repeat mean plus one-quarter of their squared difference.
Repeat-to-repeat spectral RMSE is1.394/1.379/1.336dB for30/40/50; the latter
scatter term contributes only13.3/16.8/13.0% of observed neural MSE. The
two-repeat mean is not ground truth or an unbiased population estimate, but
this check does not support explaining the entire error as repeat randomness.

For a channel countercheck, a repeat1 recording retrieves one of three repeat0
surface templates at the **same speed and load**, using centered log spectra
(constant level removed). Clean audio is30/30 correct; so is the supposedly
machine-noise microphone. This is a counterexample to treating this retrieval
score as independent acoustic-quality validation. It does NOT prove that the
generator learned only machinery, that clean audio is worthless, or that the
reference microphone contains no actual contact sound.

Read, but did not execute, the source's
[NLMS implementation at e05d6b0](https://raw.githubusercontent.com/cluster-lab/Cluster-Haptic-Texture-Dataset/e05d6b022d127e24f73583146f0aa229c6934449/preprocessing/noise_cancel/active_filter/LMSnoise_cancel.py)
and its [processing wrapper](https://github.com/cluster-lab/Cluster-Haptic-Texture-Dataset/blob/e05d6b022d127e24f73583146f0aa229c6934449/preprocessing/noise_cancel/active_noise_filter.py).
The wrapper chooses noncausal700-tap normalized LMS, step1, leakage .001,
without prewhitening. The implementation subtracts an adaptive estimate from
the main channel and starts with random coefficients. This supports considering
recording/preprocessing effects, not asserting an exact replay of the published
archive or identifying which physical component was removed.

Verification:30 focused tests pass, including all three excluded-velocity
partitions and recording separation.50 new WAVs and the six-second comparison
pass hash/PCM/rate/headroom checks. Ruff/static/format and diff/local links pass.
Both jobs terminal; no Cargo/ProductCheck/engine audition or demo replacement.
Reproduce with the previous fit command plus `--heldout-speed 30` or50 and a
new external output directory. Original40mm/s fits remain unmodified.

### Next missing physical axis: measured rainfall

A bounded Internet search found
[Measuring Amazon rainfall intensity with sound recorders, DataSuds V2](https://dataverse.ird.fr/dataset.xhtml?persistentId=doi:10.23708/I0QYNM&version=2.0).
The published terms are CC-BY4.0. It provides48,208 training spectra and only
three complete example recordings, plus separate cross-site spectral tables.
The README identifies `total_rain` as **accumulated millimetres over five minutes**,
not instantaneous mm/h; numeric columns label spectral frequencies. The
notebook's class labels also relabel isolated0.2mm readings as no rain, so do
not substitute those labels for the measured quantity or execute the notebook.
This is a candidate rain-control source, not sufficient waveform evidence
for universal rain synthesis, arbitrary struck surfaces or exact event timing.

The external `amazon-rain-source-probe-2026-09-05` contains pinned-version
API metadata, original README/notebook and the three original60s/48kHz/mono/
PCM16 WAVs (no rain/light/heavy). All five files pass publisher MD5 and local
SHA256. Notebook code was inspected as text only. No numeric intensity was
invented from the three qualitative descriptions. A bounded local DOI/name
scan found no earlier reference, not an exhaustive overlap audit.
Train file43944 and cross-site files43958/43957 are **not downloaded**.

Next: acquire the training spectral table, identify its units/frequency grid
and match the three source filenames before another fit. Reconstruct those
spectra from the supplied WAVs as a source-unit discriminator, then produce
an explicitly experimental rain sound conditioned on measured accumulation.
Do not use the two cross-site tables for tuning; keep any temporal splits
storm/day-grouped rather than assuming adjacent rows are independent. Only
three full WAVs means temporal realism will remain under-validated; that limits
claims, not the ability to produce a report-only audible experiment.

## Measured-rain neural waveform and representation check (2026-09-05)

[Standalone neural rain, 2mm accumulated over five minutes](</home/kaifaty/.codex/experiments/nextengine/physical-sound/amazon-rain-neural-2mm-2026-09-05/generated.wav>)
is eight seconds at48kHz, generated from model weights, accumulation and
noise seed2718, **without a reference recording**. This is a stationary forest
soundscape baseline, not isolated droplets, a physical rainfall calibration,
or demonstrated generation of unseen intensities/surfaces. The three source
example times are development-only; an excluded day is not an unseen condition.
[Rain comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/amazon-rain-neural-2026-09-05/rain-short-comparison.wav>)
plays real -> neural -> reference-spectrum control at0.4, then6.2mm/5min,
eight seconds each with0.5s gaps. Shared gain0.980778; no individual matching.
All interpolation controls remain in the102s full comparison and individual WAVs.

### Source, fit and measured result

The preceding acquisition status is superseded: original file43944 was obtained
with `https://dataverse.ird.fr/api/access/datafile/43944?format=original`.
`amazon-rain-source-probe-2026-09-05/psds_training.csv` is381,270,141 bytes,
publisher MD5 `d91a06cecf3af48a205bdf43c48abee1`, verified before fitting.
Default access returned converted TSV exceeding its published size; that
incomplete `.part` is rejected and unused. Original CSV has48,208 rows,
190 days,1,679 nonzero measurements, and513 LINEAR raw-PCM16 power densities.
Divide by32768² before converting to dB. Frequency names are rounded labels
for the exact48kHz/1024 FFT grid, not the grid itself. Welch1024 reproduces
all three full60s WAV spectra within0.000020dB.33 filename timestamps start
seconds after their table minute; identities are unique and their minute bins
match. This does not establish sample-level alignment with the rain gauge.
No qualitative class relabeling was applied. Cross-site43958/43957 stay unfetched.

Source attribution remains Xavier, Fleischmann, Gosset, Maciel, do Nascimento,
Ramalho and Bicudo, DataSuds DOI10.23708/I0QYNM V2, CC-BY4.0. Source metadata,
README/notebook and originals remain outside Git. The [source study](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2024GL108210)
uses sound to estimate rainfall; it does not validate this forward generator.

`physical_sound_rain_pilot.py`: log1p(accumulation) ->32 ->32 ->8 coefficients
of a training-only PCA log-spectrum basis;1,384 learned parameters. Seed41,
2,000 AdamW updates,128-row batches,lr0.001. All948 wet training rows plus
948 randomly selected dry rows determine fitting AND the interpolation control.
Day hashing yields104 eligible training days/25,397 rows,35 development
days/9,259 rows, and51 adjacent guard days/13,552 rows. May10 is explicitly
development. Guard days are excluded from optimization; multi-day storm
independence is not established. The report's `train` aggregate includes
unselected dry rows, not just the1,896 actual optimizer examples.

| Disclosed development | Neural mean spectrum RMSE,dB | Interpolation | Neural wins |
|---|---:|---:|---:|
| All9,259 recordings | 7.61800 | 7.61738 | 4,239 |
| Wet364 recordings across19 days | 7.46889 | 7.57973 | 191 |

The wet-row difference is-0.11083dB; a5,000-resample day-cluster bootstrap
gives[-0.28139,-0.01032]dB. This is descriptive same-site development evidence,
not a protected test, perceptual acceptance or independence from multi-day
storms. `result.json`, `rows.json` and `audit.json` in
`amazon-rain-neural-2026-09-05` preserve the exact membership and results.

### Why neither the spectral score nor AST accepts this model

The existing frozen AST diagnostic was run without text input on all12 clips,
the standalone clip, silence, noise and a tone. It identifies the simple
controls, but neither REAL wet clip has a rain tag in its top5. It calls the
real clips boat/vehicle-like and most synthesized clips noise-like. Therefore
it fails the relevant positive control and cannot decide rain naturalness.
The NumPy frontend also warns about zero mel filters; AudioSet pretraining
disjointness is unestablished. Exact model revision and scores are in `tags.json`.
Do not lower thresholds or treat a higher synthetic rain tag as improvement.

A separate reference-dependent probe removes the60s-versus8s spectrum mismatch:
Welch1024 of the SAME first8s drives stationary synthesis, compared with
the complete time-varying STFT magnitudes (1024 samples/hop256;32 alternating
consistency/magnitude projections, noise seed314). Exact original-phase inverse
STFT round-trip passes<1e-12. Control RMS is matched to the original, then ONE
shared gain0.415226 prevents clipping. This probe is NOT neural inference.

| 10ms RMS coefficient of variation | Real | Exact8s stationary spectrum | Temporal reference |
|---|---:|---:|---:|
| No rain | 0.1133 | 0.0927 | 0.1116 |
| 0.4mm/5min | 0.8544 | 0.3011 | 0.8450 |
| 6.2mm/5min | 0.4121 | 0.3093 | 0.4166 |

[Light-rain representation comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/amazon-rain-neural-2026-09-05/light-rain-temporal-comparison.wav>)
is real -> exact stationary spectrum -> temporal reference,25.5s total.
`temporal-probe.json` records all nine clips and measurements. Preserving a
target spectrogram naturally improves its reconstruction metrics; this is
evidence of information loss, NOT perceptual superiority or learned transfer.
Temporal reconstruction also overshoots the heavy clip's crest factor
(9.10 versus3.78), so it is not an artifact-free decoder solution.

This motivates the next change rather than another capacity/epoch sweep:
use full waveforms and learn temporal event/envelope structure. The publisher's
pinned inventory contains only three WAVs; the other tables cannot supply that
missing structure. A broader waveform source with trustworthy physical labels
is needed before making a generalization claim. This direction is consistent
with [McDermott and Simoncelli,2011](https://mcdermottlab.mit.edu/papers/McDermott_Simoncelli_2011_sound_texture_synthesis.pdf):
their experiments distinguish power-only synthesis from representations with
envelope statistics and cross-channel dependencies. This probe is not a
reimplementation of their auditory model or a reason to abandon neural generation.

Reproduce fitting (fresh external output required):

```bash
lab/.venv/bin/python lab/scripts/physical_sound_rain_pilot.py \
  --source /home/kaifaty/.codex/experiments/nextengine/physical-sound/amazon-rain-source-probe-2026-09-05 \
  --output /absolute/external/new-rain-fit
lab/.venv/bin/python lab/scripts/physical_sound_rain_pilot.py \
  --render-model /absolute/external/new-rain-fit --amount 2 --seed 2718 \
  --output /absolute/external/new-rain-render
```

Verification:34 focused Python tests and Ruff pass. WAV hashes/formats/headroom
are checked; source units, selected training membership and standalone no-audio
input are verified.48kHz synthesis is opt-in; friction's22.05kHz default is
unchanged. The fit's script hash precedes whitespace-only Ruff formatting.
No Cargo/ProductCheck, engine audition, demo replacement or production promotion.

## Geometry-conditioned temporal pouring flow (2026-09-05)

[Neural pouring audition](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-audition-2026-09-05/generated.wav>)
is4.08s of independently sampled audio: glass cylinder, height10cm,
top/bottom diameter7cm,15s pouring event at elapsed fraction0.2, seed2718.
Audition gain10 is explicitly recorded; it is not calibrated acoustic loudness.
The [unamplified standalone output](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-standalone-2026-09-05/generated.wav>)
uses the same conditions. Inference reads only weights, metadata and controls,
not a source WAV or an audio-derived embedding. This new combination is a
generation example, not a physically validated new case.

### Source and model

[Sound of Water](https://huggingface.co/datasets/bpiyush/sound-of-water),
by Piyush Bagad, Makarand Tapaswi, Cees G. M. Snoek and Andrew Zisserman,
provides full pouring recordings with container measurements and material/shape
annotations. We pin revision `12575460ee39d6adaebbe5aff531a5f4a24a627b`.
Its dataset card/root inventory does not specify redistribution terms; the
separate GitHub software/model MIT license is NOT inherited by the recordings.
Data, weights and generated audio remain local research artifacts, excluded from
distribution. No YouTube samples or publisher Test I/II/III recordings are used.

From the195 publisher-training rows, annotation-only filters select123:
`clean=yes`, `flow_rate_appx=constant`, `liquid=water_normal`, supported materials
glass/plastic/plastic_pet/plastic_pp and cylindrical/semiconical shapes.
`sound-of-water-source-2026-09-05` contains123 original48kHz mono PCM16 WAVs
(110,261,540bytes), README and original training CSV. All125 files pass their
publisher Git-blob SHA1 or LFS SHA256 plus local size/SHA256 checks. Every
recording's length agrees with its annotated trim duration within0.05s.
The source's numeric dimensions are used; approximate constant flow is NOT
converted into measured ml/s, nor elapsed fraction into an exact liquid height.

Entire containers18(glass,13 recordings) and30(PET,17) are excluded before
optimization. Remaining93 recordings from13 objects train the model. This is
a disclosed new-container development experiment inside the publisher training
split; repeated recordings of TWO excluded objects are not30 independent objects.
All30 are evaluated in source order, without selecting favourable examples.

`physical_sound_pouring_pilot.py` implements a245,985-parameter conditional
2D U-Net with FiLM blocks, frequency/time coordinates and11 physical/event
inputs:3 dimensions, duration, elapsed fraction,4 material indicators and2
shape indicators. It learns rectified-flow velocity on256×256 log-magnitude
STFT patches, not a constant average spectrum. Audio is resampled to16kHz;
FFT512/hop256, fixed floor-100dB and fixed scale `(dB+50)/25`. No evaluation
statistics set normalization. Seed53,1,500 AdamW updates, batch6,lr0.0003,
weight decay0.01, gradient norm cap1; random patches from training files only.
Final inference uses64 Euler steps and32 phase-reconstruction iterations.
Training loss first/last100 averages0.78788/0.36540; no quality claim follows
from that training-loss decrease. The decoder/source checks preceded fitting.

The [paper](https://arxiv.org/html/2411.11222v2) discusses changing resonances
during pouring and a reference-conditioned DDSP simulator. This experiment
instead learns a spectrogram distribution conditioned on numeric/object inputs;
it neither downloads the authors' model nor executes their repository. Their
inverse-property results do not establish this generator's physical accuracy.

### New-container results and automatic checks

Each withheld recording supplies its FIRST4.08s only. Baseline retrieval chooses
a training recording by distance in the same normalized metadata, then decodes
its first patch. The oracle decodes the exact target spectrogram. Neither
baseline nor oracle is presented as learned generation. Shared playback gain1
preserves level differences. All120 WAVs, exact controls and selected baseline
IDs are in `pouring-flow-2026-09-05/result.json`.

| Excluded object | Neural spectrum RMSE,dB | Nearest training example | Reference decoder |
|---|---:|---:|---:|
| Glass18,13 recordings | 13.6119 | 16.9067 | 0.1810 |
| PET30,17 recordings | 8.8309 | 7.0877 | 0.1581 |
| All30 | 10.9027 | 11.3426 | 0.1680 |

Neural wins16/30, but loses the PET group; this is not robust material transfer.
Median neural level error is-8.995dB. Mean10ms envelope CV is0.599 versus
real1.179 and oracle1.106. The oracle has median level error-0.104dB.
Thus the existing representation/decoder can preserve these measurements much
better than the first learned model: the next discriminator belongs in learned
level/envelope prediction and conditioning, not another data-source search.
The comparison contains the first source-order example of EACH held-out object,
real -> neural -> oracle,27.48s, rather than selected classifier winners.

Frozen AST, with the unchanged `Water`/`Pour` diagnostic labels, places at least
one expected tag in its top5 for30/30 real,30/30 oracle and29/30 neural clips.
Silence/noise/tone controls also retain their expected tags. Unlike the rain
pilot, these relevant positive controls pass. This supports coarse water-event
recognizability, NOT naturalness, correct vessel material, dimensions or flow.
The same NumPy mel-filter warning persists; pretraining disjointness is not
asserted. Raw scores/revision/provenance are in `tags.json`; no thresholds were
changed, no AST score trained the generator, and no result is promoted.

Reproduce with fresh external outputs:

```bash
lab/.venv/bin/python lab/scripts/physical_sound_pouring_pilot.py \
  --source /home/kaifaty/.codex/experiments/nextengine/physical-sound/sound-of-water-source-2026-09-05 \
  --output /absolute/external/new-pouring-fit
lab/.venv/bin/python lab/scripts/physical_sound_pouring_pilot.py \
  --render-model /absolute/external/new-pouring-fit --height 10 \
  --diameter-top 7 --diameter-bottom 7 --material glass --shape cylindrical \
  --duration 15 --progress 0.2 --seed 2718 --playback-gain 10 \
  --output /absolute/external/new-pouring-audition
```

`--acquire --output /absolute/external/new-source` reproduces bounded acquisition.
`--device cpu` is available for standalone inference; fitting currently uses
CUDA. Timing/geometry CLI bounds are numerical guardrails, NOT an empirical
generalization envelope. Do not promise unsupported extrapolation.40 focused
tests pass, including reference-free inference, checkpoint identity, whole-object
exclusion, bounded annotation parsing, phase-transform controls and neural
conditioning gradients. Ruff passes. No runtime/demo or product-roadmap changes.

### Power/envelope objective, onset sampling and validator level confound

[Power/envelope candidate](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-power-envelope-audition-2026-09-05/generated.wav>)
uses the previous standalone conditions/seed2718 and audition gain10, with no
reference input. [Matched revision comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-power-envelope-2026-09-05/revision-comparison.wav>)
plays real -> base -> candidate for the first glass and PET examples,27.48s,
gain1. No old artifact or demo is overwritten. These remain research candidates.

Before training, `pouring-flow-discriminator-2026-09-05` checked the ORIGINAL
checkpoint on all30 disclosed recordings, seed314.64 versus256 Euler steps
give10.894/10.902dB spectrum error and0.599/0.610 envelope CV; integration
resolution does not explain the approximately9dB level deficit. Slight absolute
differences from the original report arise from rereading PCM16 references.
Changing only the material label improves glass spectral error on13/13 cases
and worsens PET on17/17. This contradicts using these scores as a reliable
material-identity check, not proof of a unique causal material mechanism.

Two matched fits retain all93 training IDs, source hashes, seed53,245985
parameters,1500 updates and30 source-order excluded-object evaluations:

- `--objective power-envelope`: endpoint estimate `xt+(1-t)*velocity` adds
  a0.25×t²-weighted loss on log mean power spectra and frame-envelope CV.
  Statistics are trained from source data, not an independent quality validator.
  Default `velocity` is unchanged. Root `pouring-flow-power-envelope-2026-09-05`.
- `--patch-sampling onset-balanced`: original velocity loss, half of training
  patches start at zero; the remainder retain uniform internal crops. Random
  draws are still consumed, preserving the recording-selection sequence.
  Root `pouring-flow-onset-balanced-2026-09-05`. This tests a data-phase hypothesis:
  real training first/middle median RMS is0.01156/0.00567; excluded recordings
  0.01230/0.00451. More onset exposure is not a calibrated flow/force change.

| First4.08s,30 recordings | Base | Power/envelope | Onset-balanced |
|---|---:|---:|---:|
| Spectrum RMSE,dB | 10.9027 | 9.5814 | 8.1949 |
| Median level error,dB | -8.9945 | -7.4410 | -5.8067 |
| Mean envelope CV | 0.5993 | 0.8697 | 0.5797 |
| AST Water/Pour top5 at stored level | 29/30 | 16/30 | 15/30 |
| AST at common RMS0.005, PCM control | 30/30 | 30/30 | 21/30 |

Power/envelope improves the spectrum metric on28/30 recordings. Mean absolute
CV error falls0.5794 ->0.3266 (about44%); real mean CV is1.1787. Nevertheless
neither model establishes realistic material response. The power candidate's
material counterfactual still favours the wrong label for all13 glass cases,
including after centering spectra to remove constant level. PET favours the
correct label17/17. `material-counterfactual.json` retains all30 switched WAVs.

The raw AST regression initially suggested retaining only the base. A common
RMS control then removed the power candidate's deficit. In
`pouring-flow-gain-validator-check-2026-09-05`, all150 real/base/oracle/power/onset
WAVs were scaled to RMS0.005 with no clipping. Real/oracle remain30/30. Thus
the raw top5 difference cannot be attributed solely to content degradation.
This does NOT license discarding the raw result or claiming perceptual parity;
normalization is a disclosed development countercheck, not a protected gate.
The AST frontend warning and lack of calibrated naturalness/material authority
remain. `physical_sound_text_tags.py --ast-rms 0.005` now exposes this optional
classifier-input-only control, preserving raw defaults and WAVs, logging gain,
preserving silence and rejecting insufficient headroom. CLAP stays separate.

`pouring-flow-middle-check-2026-09-05` additionally checks a centered internal
patch from ALL30 excluded recordings at its actual elapsed fraction. Base/onset
spectrum error is9.6315/7.5492dB, median level error-5.4648/-4.0913dB, but raw
AST Water/Pour top5 is13/30 and12/30 versus real30/30. The earlier29/30 base
result applies only to beginnings, not entire pouring events. Power/envelope
middle-phase and multi-seed robustness are not yet established.

A bounded research check read [Flow Matching for Generative Modeling,v2,
2023-02-08](https://arxiv.org/html/2210.02747v2), specifically the squared
vector-field objectives and their gradient equivalence. Our two-pattern toy
test finds nonzero gradient0.011879 at the original optimum after adding the
nonlinear endpoint statistic. The modified objective need not preserve the
original optimum. This is a counterexample to assuming equivalence, NOT proof
that objective bias caused the audio scores; the gain control weakens that
simple explanation. No downloaded research code was executed.

Next: measure multiple seeds and middle-phase power-candidate behavior with
BOTH raw and level-controlled checks before another fit. Do not select the
onset candidate solely for lower spectral error, repeat loss-weight/sampling
sweeps, or equate one noise seed across two objects with broad generalization.
Reproduce each fit with the prior command plus its one named flag and a fresh
external output. Focused tests cover loss gradients, the non-equivalence
counterexample, unchanged uniform sampling/RNG consumption, shape versus level
metrics, opt-in AST level control, silence and headroom. No production promotion.

Verification:46 focused Python tests and Ruff check/format pass. All525 new
WAVs pass16kHz mono PCM16, finite-sample and headroom checks;615 distinct WAV
paths referenced across reports (including preserved controls) match their
recorded SHA256. All three fits retain identical source/train IDs/exclusions,
seed, update count and parameter count; checkpoint hashes match. The new
in-memory float normalization rerun (`tags-float-normalization.json`) reproduces
the PCM-control counts exactly: real/base/oracle/power30/30, onset21/30.
Local documentation links and `git diff --check` pass. Cargo/ProductCheck and
engine audition not run: no runtime, contract or engine-content changes.

### Frozen-model phase/seed check: spectrum gain is not temporal control

Middle-pour comparisons, real -> base -> power/envelope for first source-order
glass then PET,27.48s each, shared gain1:
[seed314](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-phase-seeds-2026-09-05/middle-comparison-seed314.wav>),
[seed2718](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-phase-seeds-2026-09-05/middle-comparison-seed2718.wav>),
[seed1618](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-phase-seeds-2026-09-05/middle-comparison-seed1618.wav>).
No training, new source, protected evidence or old-WAV replacement in this check.

`physical_sound_pouring_compare.py` evaluates ALL30 disclosed development
recordings at first and centered internal patches, with seeds314/2718/1618 for
both frozen models. Only numeric metadata enters neural sampling; target audio
enters metrics and explicit real/oracle controls. Root
`pouring-flow-phase-seeds-2026-09-05`:480 individual WAVs plus3 comparisons,
`result.json`, `tag-input.json`, raw/normalized CUDA tags and `analysis.json`.
The original first/seed314 and middle/seed314 values reproduce exactly.

| Phase/seed | Spectrum RMSE base -> power,dB | Power wins /30 | Mean absolute CV error base -> power |
|---|---:|---:|---:|
| First/314 | 10.903 ->9.581 | 28 | 0.579 ->0.327 |
| First/2718 | 10.461 ->9.079 | 26 | 0.349 ->0.288 |
| First/1618 | 10.235 ->9.250 | 21 | 0.574 ->0.501 |
| Middle/314 | 9.631 ->8.426 | 24 | 0.280 ->0.119 |
| Middle/2718 | 9.167 ->8.104 | 23 | 0.127 ->0.372 |
| Middle/1618 | 9.149 ->8.117 | 29 | 0.265 ->0.197 |

Spectrum improvement survives all six groups (151/180 paired wins); centered
spectrum shape improves only seed1618, not the other two. Pooled middle CV
error slightly worsens0.2238 ->0.2295. Earlier44% CV improvement is confined to
first/seed314, NOT a robust global gain. Three noise draws are not three new
objects: generalization evidence still concerns just glass18 and PET30.

More decisively, mean middle-minus-first CV is real-0.2969, exact-spectrogram
oracle-0.2681, base+0.0073 and power+0.0138. The representation/phase decoder
preserves most of the observed change; both learned generators largely miss it.
This is a paired diagnostic, not proof of the unique cause or a guarantee that
every individual stochastic draw should reproduce one recording's envelope.

`--training-controls` repeats the matrix on the FIRST source-order recording
from each of13 training objects, excluding18/30, with no score-based selection.
Root `pouring-flow-training-phase-control-2026-09-05`:208 individual WAVs plus
3 comparisons. Mean phase CV change: real-0.5731, oracle-0.5230, base+0.0207,
power+0.0286. Mean spectrum base/power: first8.904/9.052, middle9.552/9.671dB;
middle CV error0.1373/0.2869. Thus failure is NOT solely new-object transfer.
This sample of13 disclosed training recordings is a fit diagnostic, not a new
test set or a complete training-distribution audit.

AST at RMS0.005 reports Water/Pour top5 for ALL360 generated development WAVs
and ALL156 generated training-control WAVs. Nevertheless real development is
60/60, oracle58/60; real training26/26, oracle23/26. Together with missing
temporal response, these positives show that coarse event identity cannot
stand in for naturalness, temporal control or material correctness. Raw
seed314 first base/power29/16 and middle13/13 remain; other seeds are30/30 in
both phases/models. No raw evidence discarded, thresholds unchanged.

`physical_sound_text_tags.py --device cuda` now accelerates AST; CPU remains
default, device is recorded, CLAP unchanged. Compared by identical WAV SHA256
against preserved CPU reports:180 raw and120 normalized rows retain identical
top10 label order and expected-top5 flags; max score difference2.24e-6.
The two slow duplicate CPU jobs were deliberately terminated (exit143) after
this cross-check; their incomplete outputs are NOT evidence. Both full GPU
development runs and the normalized training-control run completed. The NumPy
mel-filter warning remains; this is not a new calibrated validator.

Reproduce with `physical_sound_pouring_compare.py --source SOURCE --base BASE
--candidate POWER --output NEW_EXTERNAL_ROOT`, optionally `--training-controls`.
Run `physical_sound_text_tags.py --source NEW_EXTERNAL_ROOT/tag-input.json
--output NEW_EXTERNAL_ROOT/tags-raw.json --device cuda` and separately add
`--ast-rms .005` with another report path. All51 focused tests, Ruff and694 WAV
hash/rate/PCM/finite/headroom checks pass. No runtime/ProductCheck or demo change.

Next: bounded research and a training-side discriminator before further full
fits. Competing explanations are weak learned phase conditioning/optimization,
insufficient predictive information in elapsed fraction, and representation
loss. The oracle weakens the last explanation; training-object failure weakens
an OOD-only explanation. Test a small known-object first/middle conditional fit
against shuffled-phase and exact-spectrogram controls, retaining playable WAVs.
Do not resume generic capacity/epoch/loss sweeps, promote power on AST alone,
or replace the broad user objective with matching these summary statistics.

### Phase conditioning: learnable in a small probe, unstable at broader scale

[Two-phase learning comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-two-phase-probe-2026-09-05/comparison.wav>):
first then middle; real/parent/matched-label/shuffled-label, seed314,36.64s,
gain1. This is SAME-recording training evidence, not generalization.
[Paired-record standalone candidate](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-paired-records-audition-2026-09-05/generated.wav>)
uses the usual glass10cm/7cm,15s event, fraction0.2, seed2718, audition gain10,
no audio input. Retained as an experimental candidate, NOT accepted as an
all-round improvement. No demo or previous artifact was overwritten.

Bounded research read [Guided Flows,v2,2023-12-07](https://arxiv.org/html/2311.13443v2)
§3/Algorithm1 and [Flow Matching,v2](https://arxiv.org/html/2210.02747v2).
Guidance combines conditional and unconditional fields; training includes
null conditions. Our model was not trained that way, so inserting an arbitrary
guidance coefficient is not a supported fix. No foreign code executed.

`physical_sound_pouring_phase_probe.py` first selected the FIRST training row,
`VID_20240116_230040_2.1_16.7`, plastic container1, duration14.65359s. Its fixed
patches start at sample0/84480, elapsed fraction0/0.360321. Both600-step fits
start from identical base weights, retain245985 parameters, AdamW3e-4/wd0.01,
batch6, seed53 and velocity loss. Targets/noise/times/RNG consumption match;
the negative control permutes only phase labels. No holdout enters training.

Root `pouring-flow-two-phase-probe-2026-09-05`:22 WAVs plus comparison, matched/
shuffled checkpoints, result and normalized AST. Matching the correct target
spectrogram beats the other phase for matched6/6, parent3/6, shuffled3/6 noise/
phase cases. Mean spectrum error parent6.338, matched3.823, shuffled6.443dB.
CV change real-0.3279, oracle-0.2471, matched approximately-0.086; the temporal
variation is still underfit. This falsifies completely disconnected conditioning
on this example, not an architectural sufficiency/generalization claim.

`path-identifiability.json` computes an exact balanced two-endpoint Gaussian-path
control: encoded endpoint distance158.171, optimal phase accuracy from noisy
target alone `Phi(t*D/(2*(1-t)))` exceeds95% for `t>0.020375`. Thus approximately
98% of uniform flow times allow phase inference without its label in this toy.
This makes weak incentive to use labels a plausible mechanism; it is NOT a
measurement proving that the network adopted that mechanism.

The evidence-backed `--all-training` extension selects the SAME93 training IDs.
Each batch samples3 recordings, pairing each recording's first and middle patch;
its matched600-step and shuffled600-step fits share all random draws and parent
weights. Shuffling changes only elapsed fraction, not geometry/material/duration.
Root `pouring-flow-paired-records-2026-09-05`: all30 disclosed excluded recordings,
two phases, three seeds, parent/matched/shuffled plus real/oracle:660 WAVs and
one73.28s comparison. These remain TWO objects, not180 independent objects.

| Across three seeds | Parent | Matched pairs | Shuffled phase |
|---|---:|---:|---:|
| First spectrum RMSE,dB | 10.533 | 8.442 | 13.512 |
| Middle spectrum RMSE,dB | 9.316 | 7.862 | 10.365 |
| Mean middle-first level,dB | -2.585 | -3.050 | -0.486 |
| Mean middle-first CV | +0.0073 | -0.0174 | -0.0013 |
| First CV absolute error | 0.501 | 0.526 | 0.509 |
| Middle CV absolute error | 0.224 | 0.252 | 0.236 |
| Raw AST Water/Pour top5 | 162/180 | 96/180 | 111/180 |
| RMS0.005 AST Water/Pour top5 | 180/180 | 120/180 | 146/180 |

Real mean level change-5.390dB, CV change-0.2969. Matched improves spectrum
on158/180 pairs versus parent, but CV errors worsen and seed2718 fails normalized
AST on ALL60 cases;314/1618 pass. No seed blacklisting or relabelling this as a
general improvement. Correct-phase level change improves both objects: glass
parent/matched/shuffled-2.968/-3.391/-0.499dB versus real-6.551; PET
-2.293/-2.789/-0.476 versus real-4.503. Physical response remains underestimated.

An adjacent-layer discriminator, `pouring-flow-crossed-decoder-seeds-2026-09-05`,
crosses generator seeds314/2718/1618 with independent phase-decoder seeds on the
first source-order glass/PET recordings, both phases.88 WAVs, no training.
Base AST36/36; matched generator2718 passes only1/12 across decoder seeds, while
314 passes11/12 and1618 passes12/12. Real4/4, oracle10/12. The failure cannot
be explained solely by shared decoder randomness, though decoder effects are
not zero. Keep learned-magnitude and reconstruction hypotheses distinct.

Reproduce the two fits with `physical_sound_pouring_phase_probe.py --source
SOURCE --parent BASE --output NEW_EXTERNAL_ROOT`; add `--all-training` for the
paired-record extension. Both classifier levels use the existing AST CLI on
`tag-input.json`, separate output paths and `--device cuda`. Stored checkpoints
remain compatible with source-free `physical_sound_pouring_pilot.py --render-model`.

An independent frozen CLAP check (`clap-semantic-cross-check.json`, same crossed-
decoder root) compares32 raw-level clips using the existing six fixed water/
tap/bird/whistle/metal/static prompts and decoder seed314. Real/oracle8/8 and
base12/12 favour a water prompt; matched10/12 does. Both PET phases at generator
2718 favour birds. Water-versus-nonwater margin worsens in11/12 matched versus
base pairs. This partially corroborates the regression, not every AST failure:
glass2718 still favours water. Neither embedding margin is calibrated naturalness
or material authority. Preserve the disagreement; no threshold/seed retries.

All four fine-tunes retain parent exposure to93 training recordings; metadata
separates `finetune_ids` (one or93) from inherited `train_ids`, with parent hash
and1500 parent updates versus600 additional updates. Fine-tuning never makes
other parent training objects unseen. Metadata was clarified without altering
weights or WAVs. This experiment does not change redistribution restrictions.

Next: keep parent/power artifacts and reject paired-record promotion. Inspect
training-side per-flow-time error and phase ablations, especially near pure
noise, before another full fit; use the two-endpoint oracle as a successful
control. The candidate demonstrates partial controllability but sacrifices
semantic stability and still underfits temporal variation. Do not hide this by
choosing only seed1618, tuning AST thresholds or claiming a decoder-only fix.

Verification:54 focused tests, Ruff and `git diff --check` pass. All773 new WAVs
pass SHA256,16kHz mono PCM16, finite-sample and headroom checks; all four
checkpoints and inherited/fine-tuning exposure match their metadata. Local
links resolve. All experiment jobs completed. No Cargo/ProductCheck or engine
audition: isolated Python research, no runtime/content-contract changes.

### Flow-time localization and separate envelope experiment

[Stage-splice comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-stage-ablation-2026-09-05/comparison.wav>)
uses the first glass recording, first then middle; real/base/matched/base-early/
base-late, generator2718 and fixed decoder314, gain1. Frozen-field splices are
counterfactuals, not samples certified to follow either trained distribution.
`physical_sound_pouring_stages.py` keeps64 Euler steps and switches at t0.25;
endpoint switches0/1 exactly reproduce the original pure-model sampler.

On the first source-order recording from each of13 TRAINING objects, both
phases, per-time velocity MSE is lower for matched than base at all eight
times0/.01/.05/.1/.25/.5/.75/.95. At t0, base/matched0.28109/0.24462; swapped-
phase penalties0.001929/0.000957. At t0.95,0.89196/0.84584. The exact-endpoint
oracle has maximum MSE9.4e-13. High late-time MSE alone is not proof of the
failure's cause; data/noise ambiguity also changes with flow time.

On first source-order glass/PET, both phases and three generator seeds, fixed
decoder314: AST normalized base12/12, matched9/12, base-early11/12 and
base-late9/12. At generator2718 alone:4/4,1/4,3/4,2/4 respectively. Early base
partially helps, but neither splice cleanly restores the baseline. Spectrum
RMSE7.655/6.461/7.124/6.551dB. Root `pouring-flow-stage-ablation-2026-09-05`
contains56 WAVs plus comparison and312 per-time/phase-ablation records.

The next reversible experiment separated a frozen texture model from a learned
32-bin amplitude envelope (127.5ms bins). A primary-source check of the
[DDSP paper abstract,2020-01-14](https://arxiv.org/abs/2001.04643) supports modular
neural/signal-processing controls as prior art, NOT this water model's accuracy.
No DDSP code/dependency was imported; this is a separate small experiment.

`physical_sound_pouring_envelope.py` trains27424-parameter MLP flows on the same
93 recordings' first/middle envelopes: two1500-step fits, matched versus randomly
permuted phase only, seed53, batch32, AdamW3e-4/wd0.01. Encode is
`log(max(RMS,1e-5))/3+2`; sampling64 Euler steps; decoder bounds[-2,1.5] are
numerical guards, not physical calibration. Interpolated predicted/base coarse
RMS ratios modulate the frozen base waveform; no target recording at inference.

Evaluation initially stopped at the strict0.98 peak guard AFTER both fits
completed. No training was restarted. Evaluation resumed from those exact
checkpoints and retained raw failures. For report-only listening, ALL660 clips
(including real/base controls) receive the SAME gain0.63339858, with no waveform
clipping. Raw metrics/peak failures remain in `result.json`; scaled auditions
do not constitute passing level validation. Base first/middle values differ
slightly from old reports because decoder314 is now fixed for every generator.

[Envelope experiment comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-envelope-flow-2026-09-05/comparison.wav>)
plays first glass then PET; first/middle; real/base/matched/shuffled, seed314,
shared audition gain above. This is a FAILED candidate, not an improvement.

| All30 disclosed recordings ×3 seeds | Base | Matched envelope | Shuffled phase |
|---|---:|---:|---:|
| First spectrum RMSE,dB | 10.532 | 12.572 | 13.057 |
| Middle spectrum RMSE,dB | 9.315 | 15.077 | 15.113 |
| First absolute CV error | 0.504 | 0.506 | 0.669 |
| Middle absolute CV error | 0.225 | 0.766 | 0.713 |
| Mean middle-first level,dB | -2.583 | -0.034 | +0.176 |
| Mean middle-first CV | +0.0076 | +0.1432 | -0.0577 |
| Raw headroom failures | 0/180 | 60/180 | 60/180 |
| Normalized AST Water/Pour top5 | 180/180 | 88/180 | 89/180 |

Real mean changes remain-5.390dB/-0.2969 CV. The envelope identity control
reproduces real with mean spectral error~2.1e-6dB; real/identity AST60/60 each.
Raw AST at the shared audition level gives120/180 for all three generated
groups; retain alongside normalized scores, not evidence of semantic parity.
Root `pouring-envelope-flow-2026-09-05` has660 WAVs plus comparison, two completed
checkpoints and both classifier reports. No base/demo replacement or promotion.

Next: before another flow fit, test whether a simple condition-to-relative-level
predictor on93 training recordings transfers to the disclosed containers better
than zero-phase/shuffled controls. Compare per-record normalized and absolute
targets. Missing recording gain/listener information is a hypothesis, not an
established cause. Do not repeat absolute-envelope capacity/epoch sweeps or
weaken peak guards; the broad realistic physical-sound goal is still open.

Verification:61 focused tests, Ruff check/format, local links and
`git diff --check` pass. All718 written WAVs pass SHA256,16kHz mono PCM16,
finite/headroom checks; the120 raw generation-level failures remain failures.
Checkpoint hashes and common audition gain verified. All jobs terminal;
Cargo/ProductCheck and engine audition not run (isolated Python lab only).

### Relative pouring level: limited gain, not better timbre

[Source-free audition](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-relative-level-audition-2026-09-05/generated.wav>)
is4.08s, glass/cylinder H10cm/top-bottom diameter7cm, duration15s,
elapsed fraction0.2, generator2718/decoder314, explicit audition gain10.
[Comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-relative-level-2026-09-05/comparison.wav>)
is54.96s: first source-order glass then PET, first/middle; real/base/relative,
generator314/decoder314, gain1. First-window base/relative are identical.

`physical_sound_pouring_relative_level.py` fits ridge0.01 to first/middle RMS
levels from the SAME93 training recordings. Within-record demeaning cancels
recording-level offsets and ALL static covariates: the relative model learns
only a global elapsed-fraction slope, not material/geometry interactions.
Its slope is-25.2779dB per unit fraction; absolute fit-24.1940, record-paired
random-sign control-2.2978, zero-phase0. The constant training-mean delta is
-6.5126dB. No neural weights changed. At inference, desired level is the
generated first-window level plus slope times progress; a single scalar gain
adjusts the generated current patch. No reference audio is read.

| Middle-first level RMSE,dB | Base | Relative | Absolute | Shuffled | Zero | Train mean |
|---|---:|---:|---:|---:|---:|---:|
| Glass,13 recordings | 5.252 | 4.069 | 4.029 | 7.088 | 7.635 | 3.921 |
| PET,17 recordings | 4.142 | 3.735 | 3.669 | 5.310 | 5.697 | 4.027 |
| Pooled | 4.656 | 3.883 | 3.829 | 6.144 | 6.607 | 3.981 |

These are TWO disclosed objects, not90 independent cases. Relative calibration
reduces this error16.6% versus base but barely beats the constant training mean,
and loses to the absolute fit. Recording-gain confounding is NOT established.
Across180 generated clips, absolute spectrum RMSE worsens9.923->11.177dB;
CV error is unchanged0.3642. The generated anchor is already too quiet relative
to real recordings. Do not present this as an overall realism improvement.
Raw AST real60/60, base/relative143/180 each; RMS0.005 AST60/60,180/180,180/180.
Independent CLAP, same six fixed prompts, first glass/PET ×two phases ×three
seeds: real4/4, base/relative12/12 positive water margins. Semantic consistency
does not prove calibrated level, material or naturalness. Scalar negative
controls were explicitly excluded from AST/CLAP inference.

Root `pouring-relative-level-2026-09-05` contains960 individual WAVs, comparison,
`model.json`, full results and classifier reports; the audition root contains
one WAV. `render(parent, calibration, output, controls, seed=2718,
device="cuda", audition_gain=10)` is the source-free Python entry point;
controls come from the existing `physical_sound_pouring_pilot.condition`.
Parent/calibration hashes are checked and all gains explicit. No automatic
attenuation or peak-guard weakening.64 focused tests and962 WAV/hash checks pass.

### Gain-invariant timbral evolution: conditioned correction rejected

[Timbral comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-relative-timbre-2026-09-05/comparison.wav>)
plays first source-order glass then PET, middle phase: real/base/global/
conditioned/shuffled, seed2718/decoder314, gain1,45.8s. This is a FAILED
correction, not a new best model or demo replacement.

`physical_sound_pouring_timbre_probe.py` tests the same93 training recordings
before another neural fit. Target is middle-minus-first centered log-power
in32 linear-frequency bands. Relative normalization removes scalar recording
gain; targets retain time-averaged spectral color, not within-patch dynamics.
The global ridge head uses elapsed fraction only. The conditioned head uses
fraction times static controls plus an intercept (11 features); ridge0.01.
Controls are training-mean delta, zero change and record-wise random signs,
seed53. Each of13 training objects is excluded in turn from fitting the head.
This is grouped head validation, NOT independent validation of the base neural
model, which has already seen all13 objects. No hyperparameter/seed sweep.

| Relative spectral prediction RMSE,dB | Conditioned | Global | Train mean | Zero | Shuffled |
|---|---:|---:|---:|---:|---:|
| Equal-object mean of13 excluded-group RMSEs | 3.956 | 3.867 | 3.916 | 4.212 | 4.480 |
| Disclosed glass,13 recordings | 4.051 | 3.555 | 3.594 | 3.852 | 3.775 |
| Disclosed PET,17 recordings | 2.898 | 2.876 | 2.861 | 2.988 | 3.094 |
| Disclosed pooled RMSE | 3.445 | 3.188 | 3.199 | 3.390 | 3.406 |

Metadata adds no robust advantage over the global curve here. This rejects
this feature/target combination, NOT all learnable physical information.
For audition, interpolate the predicted coarse correction into STFT bands
of the frozen generated middle clip, anchored to its generated first clip.
Preserve the current generated RMS; no real audio enters this correction.
Generated inputs are hash-checked cached base samples, not a new neural fit.
All30 disclosed recordings ×three seeds were rendered, not just previews.

| Mean over90 middle clips | Base | Global | Conditioned | Shuffled |
|---|---:|---:|---:|---:|
| Relative32-band shape RMSE,dB | 3.177 | 3.004 | 3.213 | 3.242 |
| Absolute32-band shape RMSE,dB | 5.060 | 5.455 | 5.605 | 5.172 |
| Existing spectrum RMSE,dB | 9.315 | 10.199 | 10.204 | 9.415 |
| CV absolute error | 0.225 | 0.242 | 0.234 | 0.231 |
| Raw AST water top5 | 56/90 | 81/90 | 76/90 | 58/90 |
| RMS0.005 AST water top5 | 90/90 | 90/90 | 90/90 | 90/90 |

Global correction improves the narrow relative metric but worsens actual
spectral match and CV. AST does not detect that degradation. Preserve the base;
do not start another static gain/EQ/capacity sweep from these scores.

Bounded adjacent-layer research: [Bagad et al.,2024-11-18,v1](https://arxiv.org/html/2411.11222v1),
sections3–4 and6.1, motivates testing time-resolved resonance instead of
time-averaged color. Axial pitch rises with shrinking air column; radial
resonance can fall. Their generic pitch baselines also fail substantially;
therefore a spectral maximum is not reliable ground truth. Their DDSP synthetic
generator conditions loudness/residual on real audio and does not itself meet
our reference-free interface. This is prior art, not validation of our model.

Hypotheses: recording level alone explains poor control (not supported by the
relative-vs-absolute level comparison); static coarse color captures the missing
physical information (no robust gain above); time-resolved resonance/latent
state matters (plausible, unproven here). Next discriminator: on the existing
training-only recordings, compare temporal resonance tracking against known
synthetic rising/falling controls and shuffled-time controls before using it as
a training target. Require a playable reconstruction/control and report misses;
do not invent liquid-height labels, force measurements or open protected data.

Root `pouring-relative-timbre-2026-09-05` contains360 clips plus comparison,
linear weights, group/development errors and separate raw/normalized AST reports.
Reproduce with `physical_sound_pouring_timbre_probe.py --source SOURCE
--base-outputs RELATIVE_LEVEL_ROOT --output NEW_EXTERNAL_ROOT`.

Verification for both relative-level/timbre changes:67 focused tests,
Ruff check/format and `git diff --check` pass. All1323 written WAVs pass SHA256,
16kHz mono PCM16, finite/headroom checks; parent/source/training identities
match. Both AST timbre reports have360 clips plus three synthetic controls;
the pre-existing real-water positive controls remain disclosed, not new tests.
All local links resolve and experiment jobs are terminal. No Cargo/ProductCheck
or engine audition: report-only Python lab, no runtime or roadmap change.

### Temporal resonance controls and frozen Sound of Water detector

[Diagnostic comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-sow-pitch-probe-2026-09-05/comparison.wav>)
is67.064s: first source-order training objects1(plastic),5(glass); full real
recording, classical ridge component, neural-pitch band component; gain1.
[Spectrogram/track overlay](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-sow-pitch-probe-2026-09-05/tracking-diagnostic.png>)
was rendered and inspected. These are SOURCE-DEPENDENT diagnostic extractions,
not new reference-free neural sounds. The previous generator is unchanged.

`physical_sound_pouring_resonance_probe.py` uses16kHz/FFT2048/hop256, a
250–5993Hz log grid with96 bins/octave, frequency-median subtraction over31
FFT bins, and a dynamic-programming ridge. Max jump12 grid bins/frame,
penalty0.5 per grid bin; the path may rise or fall. These are fixed numerical
choices, not calibrated physical limits. A Gaussian80-cent band extracts the
selected component, with residual defined by subtraction; reconstruction is
an arithmetic control, not synthesis quality. No material/geometry prior enters
the tracker. All first source-order recordings from13 TRAINING objects used;
no excluded or protected object opened.

Known synthetic rising/falling/stationary/crossing tones plus noise use seed53.
All four tonal cases have100% of interior frames within50 cents of an active
mode. Crossing checks nearest mode, NOT identity through crossing. Reversing
time yields the same reversed classical path. However, pure noise also gives
a smooth path, median prominence7.09dB and score0.876 ABOVE the strongest of
nine time-shuffled nulls. Thus positive shuffle margin/smoothness alone cannot
admit labels. Stationary tone correctly has zero shuffle margin. On real
recordings, selected component energy spans0.0005–0.342 of input; low-frequency
background sometimes wins over the moving resonance. Reject automatic labels
from this unconstrained tracker; do not tune its threshold from these cases.
Root `pouring-resonance-probe-2026-09-05` has55 WAVs and full diagnostics.

Following the [paper's explicit multiple-mode limitations,section6.4,v1](https://arxiv.org/html/2411.11222v1),
the adjacent-layer alternative is the authors' specialized pitch network,
not another classical ridge parameter sweep. The [official model card](https://huggingface.co/bpiyush/sound-of-water-models)
marks model weights MIT and describes synthetic pretraining followed by real
visual co-supervision. This does not change dataset redistribution terms.

Acquired only the real-finetuned checkpoint, card and backbone configs:
`sound-of-water-pitch-model-2026-09-05`, model revision
`60c7b81251923b0116ffb1f12464c8170b377b9a`,377980520 bytes,
SHA256`2fa3d8cec1488ee65bb5a6e30f1b79716d8243bbe4ddc4c0687ce2a02c84303c`.
Backbone config revision`22aad52d435eb6dbaf354bdad9b0da84ce7d6156`.
`physical_sound_sow_pitch.py` adapts the reviewed forward path from
[upstream2599de7](https://github.com/bpiyush/SoundOfWater/blob/2599de7f11d565ed78f48e4340938e0fc6ef6455/sound_of_water/audio_pitch/model.py),
retaining its MIT notice. No downloaded Python executed or dependencies added.
Load is tensor-only `weights_only=True`, fixed publisher hash/size, all215
finite tensors and strict key matching. Config is wav2vec2-base:768 hidden,
12 layers/12 heads,512 CNN channels; do not substitute the paper's8-head prose.
Time encoding matches upstream49Hz flooring, inclusive clip endpoints and
0.01 scale. Input normalization uses the stored feature extractor config.
Axial output is the probability-weighted wavelength on64 bins spanning0–100cm,
converted with34000cm/s. Radial weights load but are not claimed as validated.

The54 frozen evaluations cover13 full training recordings plus five synthetic
controls, each original/reversed/250ms-block-shuffled. These weights are
independent of the generator, but their training corpus OVERLAPS ours. They
are not an independent unseen-data test or an authoritative naturalness judge.

| Synthetic control | Median pitch error,cents | Frames within50 cents |
|---|---:|---:|
| Rising | 59.8 | 42.6% |
| Falling | 1381.8 | 6.2% |
| Stationary | 115.4 | 20.0% |
| Crossing,nearest active mode | 76.3 | 30.8% |

The neural model follows a plausible rising line in the two inspected real
spectrograms and avoids the glass5 low-frequency classical path. But falling
tones and reversal reveal strong direction/context dependence. Real reversed-
versus-original pitch disagreement has per-object medians~316–2477 cents;
it is not a general pitch tracker. Noise has median normalized entropy0.674,
versus0.329–0.397 for original real recordings, but falling-tone confidence
overlaps real examples. These observations do NOT establish an abstention
threshold. Every result retains `automatic_label_admission=false`.

`crop-context-check.json` adds52 inference comparisons:13 objects ×first/middle
4.08s crops ×reset/absolute timestamps. Compare to the SAME full-recording
prediction after interpolation, excluding0.25s from crop edges. Median of
per-object median differences: first91.9 cents, middle67.8(reset)/51.2(absolute).
Worst first crop is container23 at2561 cents; absolute middle worst179 cents.
These are context-consistency errors, not errors against true pitch. Supplying
the absolute crop start usually helps; short-clip output cannot silently replace
a full-recording pseudo-target. Reproduce individual queries with
`infer(model, extractor, crop, start_seconds=offset)`.

Decision: use the frozen model only as a candidate training-side full-sequence
pseudo-target, with uncertainty and the classical/synthetic controls retained.
It must not become the sole validator or certify material identity. The next
checkpoint owes a new REFERENCE-FREE waveform: a small condition-to-resonance
learner around the retained neural texture, compared against unmodified texture
and a simple trajectory baseline. Do not add another detector/validator stack
first, or count the diagnostic components here as meeting the generator goal.

Reproduce diagnostics with `physical_sound_pouring_resonance_probe.py --source
SOURCE --output NEW_PROBE`, then `physical_sound_sow_pitch.py --model-dir
MODEL --probe NEW_PROBE --output NEW_NEURAL_PROBE`.74 new WAVs and54 posterior
NPZs verified, as well as four downloaded model/config files.73 focused tests,
Ruff and `git diff --check` pass. No jobs remain; no Cargo/ProductCheck or
engine audition. No new generator training, protected-data use or promotion.

### Learned resonance trajectories: reference-free WAVs, renderer not promoted

[New neural sound](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-resonance-head-evaluation-2026-09-05/glass10-2718/neural.wav>)
is4.08s, glass/cylinder H10cm/top-bottom diameter7cm, event15s, start fraction0.1,
generator2718/decoder314, gain1. [Four-profile comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-resonance-head-evaluation-2026-09-05/comparison.wav>)
is54.96s: glass H10, glass H16, PET H10, glass H10/event8s; each base/neural/simple,
seed2718, all gain1. All profiles share diameter7cm/start fraction0.1. Changing
height at fixed duration also changes implied fill rate; no measured ml/s claim.
These clips genuinely require NO recording or teacher at inference.

`physical_sound_pouring_resonance_head.py` trains a4993-parameter MLP
11->64->64->1/SiLU on the first source-order recording from each of13 training
objects.64 points at fractions0.02–0.98 interpolate the frozen full-record
Sound of Water teacher; target is `(log2(wavelength_cm)-5)/2`.1000 AdamW updates,
seed53, batch128, lr1e-3, wd1e-4, gradient clip1; sampled loss0.2242->0.01544.
Teacher predictions are uncertain pseudo-targets, NOT true physical labels.
The frozen texture still inherits its original93 recordings. Source identity
is verified through file hashes and PCM16 quantization correspondence.

The head predicts frequency over the4.08s patch. A fixed STFT2048/hop256 response
`1+3*exp(-0.5*(cents/150)^2)` emphasizes that moving band, then restores the
original generated RMS. Neither width, strength, damping nor loudness is learned.
The simple control uses the cylindrical air-column/end-correction approximation;
it is only approximate for semiconical vessels. End-of-event fractions saturate
at1 for fixed-length evaluation patches; no new tail-modeling claim.

The first gain10 audition attempt stopped AFTER the full fit: glass10/seed314
has three completed WAVs; glass10/2718 has only the already-written base WAV.
All four remain in `pouring-resonance-head-2026-09-05`; the partial file has
explicit post-failure metadata and is excluded from completed evaluation.
No missing output was reconstructed. `--trained-head` reused the SAME weights
in the fresh `pouring-resonance-head-evaluation-2026-09-05` root at explicit gain1.
SHA256`462793195961551827d5c18abdeef4e7b8fb6e29241baba6644087214b02b2c2`
matches both copies. There was one full fit, not two; peak guards were preserved.

Thirteen additional1000-step fits each exclude one object from head training.
Mean of per-object median pseudo-target errors: neural277.5 cents, simple357.0,
training-mean trajectory438.8. Neural beats simple on6/13 objects, so the22.3%
mean reduction is not a uniform gain. Teacher corpus overlap prevents an
independent-data generalization claim. On hypothetical profiles the learned
start/end frequencies are912/1157Hz(glass10),769/919(glass16),537/665(PET10),
989/2014(glass10fast). Responsiveness alone does not prove physical accuracy;
especially the material-only axial-pitch change remains uncalibrated.

All30 disclosed excluded recordings ×two phases ×three seeds were evaluated.
Cached PCM16 bases/real references are reread, so base values differ slightly
from earlier pre-quantization metrics. These remain TWO objects.

| Mean over180 generated clips | Base | Neural trajectory | Simple trajectory |
|---|---:|---:|---:|
| Spectrum RMSE,dB | 9.918 | 11.101 | 11.481 |
| Centered spectrum RMSE,dB | 5.928 | 5.988 | 5.952 |
| CV absolute error | 0.364 | 0.328 | 0.290 |
| Raw AST water top5 | 143/180 | 166/180 | 161/180 |
| RMS0.005 AST water top5 | 180/180 | 170/180 | 161/180 |

Neural spectral wins11/180, CV wins150/180. Thus no overall quality improvement
or default replacement. On the four hypothetical profiles ×three seeds, AST
base/neural/simple12/11/12 of12 at BOTH levels. The neural miss is glass10fast/
2718; Water ranks6, no seed/threshold tuning. Independent fixed-six-prompt CLAP
still gives positive water margins12/12 for each variant; real controls4/4.
This disagreement is retained, not relabelled as an all-pass naturalness check.

Bounded research/discriminator: is the frequency predictor the only bottleneck,
or is fixed band emphasis insufficient? [Sound of Water,v1,section4.2](https://arxiv.org/html/2411.11222v1)
uses pitch AND loudness/residual with an audio-reconstruction-trained decoder;
it does not establish that a fixed moving filter suffices. No new upstream code
or data was imported. `pouring-resonance-oracle-control-2026-09-05` tests the
privileged full-record teacher trajectory on13 training objects ×two phases,
seed2718, alongside real/base/neural/simple.131 WAVs include a clearly labelled
[privileged-control comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-resonance-oracle-control-2026-09-05/comparison.wav>).

| Training-side diagnostic,26 clips | Base | Neural | Simple | Privileged teacher |
|---|---:|---:|---:|---:|
| Spectrum RMSE,dB | 9.238 | 9.658 | 9.584 | 9.873 |
| Centered spectrum RMSE,dB | 5.754 | 5.696 | 5.759 | 5.725 |
| CV absolute error | 0.309 | 0.300 | 0.333 | 0.317 |

Even the source-conditioned teacher curve does not restore spectral fidelity.
This weakens the predictor-only explanation; it does not prove the teacher
is true pitch or isolate every decoder failure. Stop fixed-band/head-capacity
sweeps. Next smallest experiment: feed the retained trajectory into a narrowly
trained waveform/spectrogram decoder adapter and optimize reconstruction, with
an exact zero-adapter baseline and new reference-free WAVs. Preserve the current
base and head as controls; do not make teacher confidence the quality objective.

Source-free Python API: `render(parent, head, fresh_output, controls, seed=2718)`;
no dataset, teacher checkpoint or target waveform is opened. Main CLI takes
`--source --teacher-probe --parent --base-outputs --output`, optionally
`--trained-head` for exact full-fit reuse. Evaluation contains360 new corrected
WAVs,36 hypothetical-profile WAVs and one comparison. Together with the four
retained first-attempt WAVs and131 privileged-control WAVs,532 WAVs pass format,
finite/headroom checks.531 have original writer hashes; the partial base has an
explicit observed hash and matches gain1 base within PCM16 quantization bounds.
76 focused tests, Ruff, local links and `git diff --check` pass. All evaluations
terminal; no Cargo/ProductCheck/engine audition, runtime or roadmap promotion.

### Trajectory-conditioned input adapter: audible, no demonstrated quality gain

[New source-free WAV](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-pitch-adapter-2026-09-05/matched-first-audition/adapter.wav>)
uses the same glass H10cm/diameter7cm/event15s/start0.1 profile, generator2718,
decoder314, gain1. [Four-profile comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-pitch-adapter-2026-09-05/comparison.wav>)
is54.96s: glass10/glass16/PET10/glass10fast, each base/matched/shuffled. No source
recording or teacher is needed at inference. These are experimental candidates,
not a better default. Dataset redistribution remains unspecified/local-only.

`lab/scripts/physical_sound_pouring_pitch_adapter.py` adds a zero-initialized,
bias-free1->16 Conv2d3x3 at the frozen PourFlow input:144 trainable parameters.
The input is a150-cent Gaussian plane around the supplied frequency trajectory,
not a fixed output filter. Every original parent tensor remains exactly frozen;
a zero adapter exactly reproduces the parent sampler. The4993-parameter head
and parent245985-parameter model are reused without retraining.

Two600-step AdamW fits use the same13 first source-order training recordings,
first/middle crops, six patches per batch, seed53, lr1e-3/wd0.01/gradient clip1.
The matched fit receives full-record teacher curves at the correct crop offsets;
the shuffled fit permutes only curves, keeping audio, controls, time and noise
draws identical. Loss is flow velocity MSE plus0.25 times the time-squared-weighted
one-step endpoint L1, averaged across pooling scales1/4/16. Final100-step mean
losses0.343387/0.343618 do not establish a quality advantage. Teacher curves are
uncertain and corpus-overlapping; they are not independent physical truth.

The first source-order recording of each disclosed excluded object18/30, two
phases and three seeds were evaluated. This is12 generated clips per model,
not12 independent objects. No expansion to all30 recordings after this failure.

| Development mean | Base | Matched guide training | Shuffled guide training |
|---|---:|---:|---:|
| Spectrum RMSE,dB | 7.655 | 8.026 | 8.086 |
| Centered spectrum RMSE,dB | 5.990 | 6.034 | 6.137 |
| CV absolute error | 0.441 | 0.431 | 0.425 |
| Raw AST water top5 | 10/12 | 11/12 | 9/12 |
| RMS0.005 AST water top5 | 12/12 | 12/12 | 12/12 |

Both adapters win0/12 spectral comparisons against base; CV wins9/12 and12/12.
All four reference clips pass both AST levels. Four hypothetical profiles ×three
seeds give12/12 AST water detections for every variant at both levels. These
coarse semantic checks do not demonstrate realistic water, physical parameter
accuracy or independent generalization. Spectrum error against one stochastic
recording is also not a complete perceptual metric; the observed result supports
withholding an improvement claim, not a universal impossibility theorem.

Bounded research asks whether teacher-to-predicted-guide mismatch is responsible,
whether the frozen decoder/input-only adapter cannot express the correction, or
whether the one-step training objective fails to improve free-running audio.
[ControlNet,v3,section3](https://arxiv.org/html/2302.05543v3) trains copies of deep
encoding blocks joined through zero convolutions; it does not justify treating
one144-parameter input convolution as an equivalent architecture. This is an
image-model analogy, not proof of our audio capacity bottleneck.
[DDSP](https://arxiv.org/abs/2001.04643) motivates jointly learned signal-processing
components. More directly, [Sound of Water,v1,section4.2](https://arxiv.org/html/2411.11222v1)
trains a pitch/loudness/residual decoder with multiscale spectrogram reconstruction.
Its published generator draws loudness/residual from a real conditioning sample;
that interface does NOT itself meet our no-reference-input goal. No new upstream
code, weights or datasets were imported for this research cycle.

The executable discriminator reused the completed matched adapter on13 TRAINING
objects ×first/middle, generator2718/decoder314. Compare predicted head curves
against privileged full-real-record teacher curves and frame-permuted teacher
curves (NumPy53), with all other inputs fixed. This is an intentionally
source-dependent diagnostic, not the source-free generation path.
`pouring-pitch-adapter-guide-check-2026-09-05/result.json` and its
[comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-pitch-adapter-guide-check-2026-09-05/comparison.wav>)
retain all130 clips; comparison includes the first two training objects, each
first/middle, in real/base/predicted/teacher/scrambled-teacher order.

| Training-side diagnostic,26 clips | Base | Predicted | Teacher | Scrambled teacher |
|---|---:|---:|---:|---:|
| Spectrum RMSE,dB | 9.238 | 9.306 | 9.308 | 9.280 |
| Centered spectrum RMSE,dB | 5.754 | 5.790 | 5.793 | 5.758 |
| CV absolute error | 0.309 | 0.305 | 0.304 | 0.305 |

Privileged guidance does not rescue the result. This weakens train/deployment
guide mismatch as the sole cause, but does not distinguish capacity from loss
mismatch or establish correct teacher pitch. Do not run more tiny-adapter/epoch
sweeps. Next experiment: train a temporal resonance/noise decoder directly with
multiscale audio reconstruction, learning time-varying signal components rather
than nudging frozen flow features. It must generate from object/event conditions
and randomness alone. Include a synthetic learnability control and new source-free
real-domain WAVs; reconstruction-only diagnostics cannot become the endpoint.
Preserve base/head and reject promotion on semantic tags or training loss alone.

Training CLI takes `--source --teacher-probe --parent --head --output`.
Source-free CLI uses `--parent PARENT --head HEAD --render-adapter ADAPTER
--controls 0.5 0.35 0.35 0.5 0.1 1 0 0 0 1 0 --output FRESH_OUTPUT`, with optional
`--seed` and `--device`; it explicitly excludes source/teacher arguments.
The controls encode normalized height/top-bottom diameters, duration, phase,
material and shape. `pouring-pitch-adapter-cli-2026-09-05` reran the real CLI and
reproduced both first-audition WAV hashes. Weights are integrity-checked at load;
oversized metadata, bad parent/head identity and invalid sampling bounds fail.

Verification:81 focused tests, Ruff and `git diff --check` pass;81 experiment,
131 guide-check and two CLI WAVs pass hash/PCM16/16kHz/finite/headroom checks.
Both fits and all evaluations are terminal. No CLAP rerun for this rejected
candidate, no Cargo/ProductCheck/engine audition, no runtime/default/roadmap
promotion. The full multi-event physical-sound goal remains open.

### Direct temporal noise/resonance decoder: reconstruction gain is not water

[New standalone generated waveform](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-temporal-decoder-2026-09-05/first-audition/generated.wav>)
and [four-profile comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-temporal-decoder-2026-09-05/comparison.wav>)
are real outputs, but this candidate is REJECTED as an improvement. Comparison
order is glass10/glass16/PET10/glass10fast, each base/temporal/static, generator2718,
gain1,54.96s. The decoder requires only its own combined checkpoint, object/event
controls and a seed. It loads no source recording, teacher or previous PourFlow.
No learned physical calibration or successful new-condition quality is claimed.

`physical_sound_pouring_temporal_decoder.py` trains63619 parameters:12->96/SiLU,
GRU96->96, output67. Inputs are the11 metadata/time controls and log frequency
from the retained4993-parameter head. Its weights remain frozen and are packaged
inside the new checkpoint. Outputs are65 linearly spaced log-noise-gain bands,
resonance strength and width. Log gains interpolate to513 FFT1024 bins, bounded
[-10,0] before exponentiation. A learned Gaussian resonance multiplier uses
softplus strength and width50–900 cents. Predicted coefficients change each16ms.
They filter Gaussian excitation with differentiable STFT/ISTFT, hop256,16kHz,
4.08s. There is no iterative phase reconstruction. A static ablation preserves
each bin's mean-square filter energy while removing temporal changes; a separate
ablation removes the dedicated resonance multiplier.

The loss operates on the generated waveform: mean log-magnitude L1 plus spectral
convergence at FFT256/1024/2048, plus0.5 times20ms log-RMS-envelope L1. This differs
from the previous frozen-flow input adapter and its one-step velocity objective.
[Sound of Water,v1,section4.2](https://arxiv.org/html/2411.11222v1) motivated direct
audio reconstruction, but its pitch/loudness/residual-conditioned DDSP generator
is NOT reproduced here. In particular, this model has no audio-conditioned
residual or learned stochastic event latent; only Gaussian excitation is random.

An in-family positive control supplies known rising/falling frequency tracks
and opposite amplitude ramps for two synthetic filtered-noise targets.300 updates
reduce loss against a fresh excitation seed from2.807 to1.323; static-response
ablation is1.819. Both specified checks pass (below70% initial and below static).
Eight WAVs are retained. This verifies learnability in the selected signal
family, not pitch inference, physical truth or real-water quality.

One fresh real-data fit uses all93 existing training recordings/13 containers,
uniform recording and crop sampling, batch6,1200 AdamW steps, seed53, lr1e-3,
wd1e-4, gradient clip1. No teacher extraction or new data download. First/last
100-update loss means3.217/2.337. The source, training roster and inherited head
identity are checked. Dataset terms remain unspecified/local-research-only.

The first source-order recording of disclosed excluded containers18/30, first/
middle phases and seeds314/2718/1618 form12 generated clips per variant.

| Development mean | Base | Temporal | Static | No resonance |
|---|---:|---:|---:|---:|
| Spectrum RMSE,dB | 7.655 | 8.256 | 8.238 | 8.507 |
| Centered spectrum RMSE,dB | 5.990 | 5.067 | 5.065 | 4.961 |
| CV absolute error | 0.441 | 0.791 | 0.969 | 0.811 |
| Raw AST water top5 | 10/12 | 0/12 | 0/12 | 0/12 |
| RMS0.005 AST water top5 | 12/12 | 0/12 | 0/12 | 0/12 |

Every new variant wins only3/12 spectral comparisons and0/12 CV comparisons
against base. Four real controls pass both AST levels. For the four hypothetical
profiles ×three seeds, base/temporal/static AST is12/0/0 of12 at both levels.
The same frozen six-prompt CLAP contrast independently gives12/0/0 positive water
margins, with real4/4. Common wrong AST categories are white/pink noise and leaves.
Classifier data overlap is unknown; neither classifier is a calibrated quality
judge, but the agreement is strong evidence against promoting this candidate.

A bounded fit-versus-transfer discriminator trains only the FIRST source-order
training record, container1/VID_20240116_230040_2.1_16.7, first/middle crops.
It reuses the completed decoder and performs300 updates with fresh excitation,
same optimizer/seed and three fixed evaluation seeds. This is memorization,
not a new-condition result. `pouring-temporal-decoder-fit-check-2026-09-05` failed
before its first optimizer update because loaded eval-mode cuDNN GRU cannot
backpropagate. Its eight before/real WAVs remain explicitly labelled partial.
The corrected train-mode run uses the same original weights in fresh
`pouring-temporal-decoder-fit-check-retry-2026-09-05`; no full fit was repeated.

| Single-record diagnostic,6 clips | Before | After | After, static |
|---|---:|---:|---:|
| Spectrum RMSE,dB | 4.452 | 4.622 | 4.593 |
| Centered spectrum RMSE,dB | 3.585 | 2.136 | 2.082 |
| CV absolute error | 0.840 | 0.390 | 0.792 |
| RMS0.005 AST water top5 | 0/6 | 0/6 | 0/6 |

Real controls pass2/2. Learned dynamics are possible and improve on this record,
but do not restore water recognition. This weakens an unseen-object-only failure
explanation. It does not yet isolate insufficient spectral detail, inadequate
excitation/phase structure, stochastic event averaging or optimization.
Do not start a65-band/capacity/epoch sweep. Next discriminator is source-dependent
exact-record FFT1024 magnitude with Gaussian-noise phase versus iterative phase
reconstruction on this same first training record. Produce labelled oracle WAVs
and check existing semantic controls before selecting the next learned model.

CLI training uses `--source --parent --head --output`; fit diagnostic uses
`--source --probe-model --output`. Standalone rendering uses `--model MODEL
--controls 0.5 0.35 0.35 0.5 0.1 1 0 0 0 1 0 --output NEW_OUTPUT`, optionally
`--seed`/`--device`, explicitly excluding training inputs. The real standalone
CLI reproduced the first-audition WAV hash exactly. Checks cover synthesis
identity, temporal ablation, finite gradients including exact reconstruction,
frozen head, input/seed rejection, checkpoint integrity and reference-free render.

86 focused tests, Ruff and diff/link checks pass.127 written WAVs pass format,
finite/headroom checks:98 full experiment,8 retained failed-probe inputs,
20 corrected-probe clips and one standalone CLI output.119 have original writer
hashes; eight partial files carry explicitly observed hashes. Original head
tensors match both final checkpoints exactly. Every job is terminal. No runtime,
default, protected-data, roadmap or ProductCheck promotion; full goal remains open.
