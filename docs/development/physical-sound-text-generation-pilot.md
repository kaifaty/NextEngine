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
