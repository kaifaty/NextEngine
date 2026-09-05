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

### Exact-magnitude phase control and failed source-free phase repair

`physical_sound_pouring_phase_oracle.py` separates two paths. The oracle uses
the same first TRAINING record/container1/VID_20240116_230040_2.1_16.7, first/middle
crops and three CPU seeds. It requires the original audio and is NOT a generator
for new conditions. The separate refinement path uses only the stored full neural
decoder, metadata controls and a seed when sampling; references enter metrics only.

[Oracle comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-phase-oracle-2026-09-05/comparison.wav>)
is54.96s: first/middle, each real/learned/noise-phase/iterative-phase/coarse65/
expected-noise-gain, seed2718, gain1. All use FFT1024/hop256/Hann and4.08s clips.
Exact real magnitude is combined with Gaussian-noise phase, either directly or
after32 alternating STFT/ISTFT phase updates. Coarse65 reduces log magnitude to65
linear-frequency samples then interpolates back; this is not exactly the learned
gain parameterization. Expected-noise-gain multiplies white-noise STFT by real
magnitude divided by sqrt(sum(window²)), without inverting the current noise draw.
The original signal is never used as the reconstruction phase.

Matched windows, explicit length and the least-squares inverse follow the
[PyTorch ISTFT documentation](https://docs.pytorch.org/docs/2.14/generated/torch.istft.html).
Roundtrip and phase-refinement tests cover the implementation; no library upgrade.
The learned control reuses the completed single-record probe. CPU generator
draws differ from its prior CUDA audition, so its sample-level values are new
matched CPU evidence, not a claim of CPU/CUDA waveform identity.

| Oracle,6 clips per variant | Learned | Noise phase | 32 iterations | Coarse65 | Expected noise gain |
|---|---:|---:|---:|---:|---:|
| Spectrum RMSE,dB | 4.621 | 1.253 | 0.332 | 2.467 | 1.118 |
| Centered spectrum RMSE,dB | 2.133 | 0.519 | 0.324 | 1.841 | 0.951 |
| CV absolute error | 0.435 | 0.159 | 0.045 | 0.181 | 0.148 |
| AST water top5, raw AND RMS0.005 | 0/6 | 0/6 | 6/6 | 0/6 | 1/6 |
| Fixed-six-prompt CLAP positive water margin | 1/6 | 6/6 | 6/6 | 6/6 | 6/6 |

Real controls pass2/2. CLAP mean margins for noise/iterative phase are0.098/0.311,
versus real0.324. Thus phase consistency materially affects this reconstruction
and AST recognition; CLAP already recognizes several weaker reconstructions.
Do not erase this disagreement or call phase the sole universal quality cause.
This is one training record, not independent data or physical generalization.

The executable follow-up then applied exactly32 updates to the neural decoder's
own generated complex spectrogram, without teacher, original magnitude or new fit.
[New source-free variant](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-phase-refinement-2026-09-05/glass10-refined-2718.wav>)
and [comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-phase-refinement-2026-09-05/comparison.wav>)
remain failed candidates. Comparison is36.64s: glass10/glass16/PET10/glass10fast,
each native/refined, seed2718, gain1. Native zero-step sampling matches the old
decoder numerically; no original model weights are modified.

On the same two disclosed development objects/first records/two phases/three
CUDA seeds, native/refined spectrum8.256/8.259dB, centered5.067/5.064, CV error
0.7913/0.7911. AST is0/12 for both on development AND hypothetical profiles at
both raw and normalized levels. CLAP hypothetical-profile water margins remain
negative12/12 for each, real controls positive4/4. Inference-side phase repair
alone therefore does not rescue the learned representation. No phase-iteration,
65-band or smooth-noise capacity sweep follows from this result.

Next selected experiment: preserve time-local spectral structure in a learned
conditional latent decoder/prior, using consistent phase reconstruction.
[Sohn et al.,NIPS2015,section4](https://proceedings.neurips.cc/paper/2015/file/8d55a249e6baa5c06772297520da2051-Paper.pdf)
provides the conditional posterior/prior formulation for one-to-many structured
outputs. Its image experiments do not establish audio success. Our proposed
audio use is an inference: training posterior may read target audio, while the
generation prior receives only object/event controls and randomness. Compare
posterior reconstruction and source-free prior WAVs in the same bounded run;
good reconstruction, low KL or a changed waveform cannot substitute for prior
quality/control. Preserve the existing base and disclosed data roles. This
changes the research model, not the product roadmap or an Accepted contract.

Reproduction: oracle CLI `--source SOURCE --fitted SINGLE_RECORD_MODEL --output
NEW_ORACLE`; source-free refinement evaluation `--source SOURCE --refine-model
FULL_MODEL --output NEW_REFINEMENT --device cuda`. Pure inference function
`refine_generated(model, controls, seed)` never reads a source recording.
33 oracle and53 refinement WAVs pass hash/PCM16/16kHz/finite/headroom checks.
90 focused tests, Ruff, local links and `git diff --check` pass. All jobs terminal;
no training, downloads, runtime/default promotion, Cargo or ProductCheck run.

### Conditional latent model: posterior and source-free prior both rejected

[First reference-free prior sample](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-cvae-2026-09-05/first-audition/prior.wav>)
and [four-profile comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-cvae-evaluation-2026-09-05/comparison.wav>)
are generated from object/event controls and randomness, without a recording,
teacher or base model at inference. Comparison36.64s: glass10/glass16/PET10/
glass10fast, each base/prior, latent2718/phase314, gain1. These remain rejected
candidates. [Reconstruction comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-cvae-evaluation-2026-09-05/reconstruction-comparison.wav>)
is36.64s: glass18/PET30 middle, each real/base/posterior/prior. Posterior explicitly
requires the original recording; neither that path nor its metrics meet the goal.

`physical_sound_pouring_cvae.py` implements690449 parameters. Four stride2
convolutional blocks16/32/64/128 map a512x256 encoded magnitude to an8x32x16
Gaussian latent. Posterior and prior heads output means/log variances, bounded
[-6,2]; the prior sees only11 controls plus frequency/time coordinates. No skip
connections carry target audio to the decoder. Bilinear upsampling/convolutional
blocks restore512x256; a2*tanh(output/2) bounds the encoded output. FFT1024/hop256,
16kHz,4.08s, DC removed; normalization follows the prior fixed[-100,0]dB range.

One fit uses all93 training recordings/13 containers, uniform record/crop draws,
batch6,2000 AdamW updates, seed53, lr3e-4, wd1e-4, gradient clip1. Reconstruction
loss is encoded L1 plus0.25 times re-encoded waveform L1 after TWO differentiable
phase-consistency iterations. KL is the mean analytic Gaussian posterior/prior
divergence; beta linearly rises to0.01 by step500. First/last100-update means are
reconstruction0.6222/0.2102, KL0.4219/0.4710. This bounded fit is not proof of
convergence. At inference, sample a latent and reconstruct with32 phase updates;
phase seed314 is fixed across the three latent seeds. Full-fit checkpoint SHA256
`ce216d80add22ba62dca0eed324f6dfc295c79574e6c8b99c1a8507a787e0351` is preserved.

The first training record and first source-order recordings of disclosed excluded
objects18/30 are each evaluated in first/middle phases, seeds314/2718/1618. These
are one training object and TWO development objects, not an independent test.

| Development mean,12 clips | Base | Posterior | Prior |
|---|---:|---:|---:|
| Spectrum RMSE,dB | 7.655 | 5.613 | 8.626 |
| Centered spectrum RMSE,dB | 5.990 | 2.558 | 5.284 |
| CV absolute error | 0.441 | 0.435 | 0.697 |
| AST water top5,raw | 10/12 | 0/12 | 0/12 |
| AST water top5,RMS0.005 | 12/12 | 0/12 | 0/12 |
| Fixed-six-prompt CLAP water margin>0 | 12/12 | 0/12 | 1/12 |

Training-side base/posterior/prior spectrum6.339/5.115/7.154, centered5.572/1.837/
4.015, CV error0.315/0.423/0.657. Both AST levels and CLAP give6/0/0 of6.
Four hypothetical profiles ×three seeds give base/prior12/0 of12 for both AST
levels and CLAP. All six real controls pass both classifiers. No seed or threshold
selection. Classifier corpus overlap remains unknown; semantic tags are not
physical calibration or a complete naturalness test. Their agreement rejects a
quality claim here. The failure is not only a posterior-to-prior distribution gap:
even reconstruction is not recognized as water despite better spectral metrics.

The source-dependent `pouring-cvae-codec-probe-2026-09-05` uses the first training
record/container1, two phases, three phase seeds. It fixes posterior latent seed53
and compares exact encoded target, posterior output and target log magnitudes
area-reduced/bilinearly-expanded on time only(512x16), frequency only(32x256),
or both(32x16). Every variant uses32 identical phase updates. This is a simple
information-removal control, NOT an equivalence to a learned multichannel latent.

| Codec control,6 clips | Exact | Posterior | Coarse time | Coarse frequency | Coarse both |
|---|---:|---:|---:|---:|---:|
| Spectrum RMSE,dB | 0.341 | 5.087 | 6.310 | 3.445 | 7.261 |
| Centered spectrum RMSE,dB | 0.331 | 1.923 | 1.782 | 2.402 | 2.764 |
| CV absolute error | 0.053 | 0.420 | 0.434 | 0.086 | 0.540 |
| AST water top5,both levels | 6/6 | 0/6 | 0/6 | 0/6 | 0/6 |

Real2/2. The exact preprocessing/reconstruction path is a positive control;
dropping details on either axis damages recognition. This supports investigating
detail preservation rather than blaming only the phase algorithm or the prior.
It does not isolate architecture from the reconstruction objective, undertraining
or latent regularization, nor justify a latent-size/epoch/KL sweep.

Bounded research: [RAVE,v2,section3.1.2](https://arxiv.org/html/2111.05011v2)
freezes an encoder after representation learning and fine-tunes its decoder with
adversarial, feature-matching and spectral reconstruction terms. This motivates
one reversible decoder-only experiment here, not a claim that our representation
is already sufficient or that this spectrogram CVAE reproduces RAVE's waveform
architecture. The paper reports much longer training; our2000 updates cannot be
presented as equivalent evidence. Keep encoder/prior fixed, retain reconstruction,
and learn the critic only from training audio. AST/CLAP must remain separate
diagnostics, never generator training losses. Compare posterior and SOURCE-FREE
prior WAVs in the same run; any improvement still requires control/generalization
evidence before promotion. Do not silently switch to reconstruction-only success.

CLI subcommands: `train --source --output`; `evaluate --source --model --base
--output`; source-free `render --model --controls ELEVEN_VALUES --output`; and
source-dependent `probe --source --model --output`. Render accepts optional seed
and device, and no source argument. Training is separate from evaluation so a
later evaluation failure never requires retraining a completed checkpoint.
Source integrity/roles and unspecified redistribution terms remain unchanged;
all audio, weights and reports stay external. No new dataset or model download.

94 focused tests, Ruff, diff and link checks pass.121 WAVs pass original-writer
hash/PCM16/16kHz/finite/headroom checks: one first prior,86 evaluation/comparison,
one standalone CLI replay and33 codec probes. CLI reproduces the first-prior hash
exactly. Both inference separation and differentiable reconstruction/KL gradients
are tested. All jobs terminal, no runtime/default/roadmap or ProductCheck promotion.

### Matched decoder adversarial continuation: rejected, phase mismatch bounded

[Four-way source-free comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-cvae-adversarial-evaluation-2026-09-05/ablation-comparison.wav>)
is18.32s: retained base-flow, original CVAE, reconstruction-only continuation,
adversarial continuation. Glass H10cm/diameter7cm/duration15s/start0.1,
latent2718/phase314/gain1. These CVAE candidates remain rejected; no source
recording, teacher or critic is required at inference. Full four-profile
base/adversarial comparison is in the same directory's `comparison.wav`.

`physical_sound_pouring_cvae_adversarial.py` runs two matched600-step
continuations from the original CVAE, not sequential fine-tunes of one another.
Same93 training records, uniform3-record/crop batches, seed53, same latent/noise
RNG consumption. Encoder/posterior/prior tensors stay byte-exact;322577 decoder
parameters train. Adam lr1e-4, betas0.5/0.9, clip1. Both runs instantiate the same
347362-parameter critic; only the adversarial arm updates/uses it. Two separate
four-layer strided spectral critics see re-encoded synthesized waveforms at
original/half spatial resolution, with hinge loss. Generator retains posterior
reconstruction plus0.1 adversarial loss on posterior AND prior and0.5 posterior
feature matching. Phase iterations remain2 in training and32 in inference.
The critic is unconditional and training-only, not a physical-condition judge.
AST/CLAP never enter training. This is not a reproduction of RAVE.

External models: `pouring-cvae-reconstruction-continuation-2026-09-05`, SHA256
`37c79a336ff1fd18eea4841d670888fb65a457076f0587597a7cb194116e51a4`;
`pouring-cvae-adversarial-2026-09-05`, SHA256
`f2b06c92cf82dbc379c208271fda3338bd0458c36dfb272ffc06c31796ccd3eb`.
Last100 rec losses0.20830/0.21710; adversarial D/G/FM1.65619/0.24210/0.03848.
Inherited parent metrics in model.json are explicitly labelled historical.
Neither completion nor these losses prove convergence or naturalness.

Evaluations reuse the exact original first training record, disclosed excluded
objects18/30, first/middle phases, three latent seeds and four novel profiles.
`pouring-cvae-rec-cont-evaluation-2026-09-05` and
`pouring-cvae-adversarial-evaluation-2026-09-05` retain all84 individual rows,
AST raw/RMS0.005 and fixed-six-prompt FP32 CLAP. Development is only TWO opened
objects; it is not an independent test or evidence for all physical sounds.

| Development,12 clips per mode | Original CVAE | Rec continuation | Adversarial |
|---|---:|---:|---:|
| Posterior spectrum RMSE,dB | 5.613 | 5.292 | 5.000 |
| Posterior centered spectrum,dB | 2.558 | 2.472 | 2.934 |
| Prior spectrum RMSE,dB | 8.626 | 8.584 | 7.794 |
| Prior centered spectrum,dB | 5.284 | 5.161 | 5.570 |
| AST posterior/prior,both levels | 0/0 | 0/0 | 0/0 |
| CLAP positive posterior/prior | 0/1 | 0/2 | 3/0 |
| Novel prior AST/CLAP,out of12 | 0/0 | 0/0 | 0/0 |

All six real controls pass both classifiers. Base development AST raw10/12,
normalized12/12, CLAP12/12; novel base12/12 throughout. On training, both new
CVAE modes remain0/6 for both AST levels and CLAP. Better partial spectrum
metrics have not recovered water semantics. Adversarial prior level changes
substantially (development mean error-1.782dB versus rec-9.670dB), while centered
shape worsens; do not call this a timbre improvement. No threshold/seed tuning.

Bounded research after this failure considered three explanations: (1) merely
too few additional updates, (2) train/inference phase inconsistency, (3) learned
representation/objective lacking perceptual detail. The matched rec arm tests
the first bounded600-update explanation, not eventual convergence. For the
second, [Khan et al.,2020](https://arxiv.org/abs/2005.07810) identify consistency
of generated spectrograms as a speech synthesis issue; [Masuyama et al.,2019](https://arxiv.org/abs/1903.03971)
describe GLA limitations and learned iteration. These speech results motivate
a control, not a water-quality or neural-vocoder success claim. RAVE's frozen
encoder stage assumes a satisfactory representation; we have not established
that assumption. No further critic/epoch/capacity sweep is justified here.

The executable `--phase-budget-probe` uses the adversarial checkpoint and first
training record, two phases, fixed latent53, phase314/2718/1618, exact target,
posterior and source-free prior at2 versus32 iterations. External output:
`pouring-cvae-phase-budget-2026-09-05`;38 individual WAVs plus a comparison.

| Six clips per row | AST raw/normalized,2->32 | CLAP,2->32 | Spectrum RMSE,2->32 |
|---|---|---|---|
| Exact target | 3->6 at both levels | 6->6 | 0.648->0.341 |
| Adversarial posterior | 0->0 at both levels | 0->0 | 3.280->3.139 |
| Adversarial prior | 0->0 at both levels | 0->0 | 6.425->6.591 |

Thus training-time synthesis itself can harm recognition, but returning the
trained model to2 iterations does not restore it. This does NOT falsify an
effect on training gradients; it rules out a simple inference-only rescue.
`critic-response.json` also measures the saved training critic on these WAVs.
Its fine-scale mean logit real/exact2/exact32 is0.299/-0.419/-0.197; posterior
2/32 is-1.587/-2.341 and prior-0.247/-0.533. Coarse-scale scores disagree in
ordering. It is not an independent validator or a reason to select outputs.
Representation versus objective/undertraining remains unresolved, not proven
to be a hard latent-capacity ceiling. All failures and phase seeds remain.

Next selected discriminator: reuse the already cached, frozen TangoFlux
Oobleck waveform autoencoder on the same water crops. Existing codec evidence
above covers glass, not these recordings. [Stable Audio Open,v2,2024-07-31](https://arxiv.org/html/2407.14358v2)
separates a waveform autoencoder from latent generation, which motivates
reusing a learned waveform representation before another conditional fit.
This is an alternative to jointly learning a lossy spectrogram codec and prior
from93 recordings, not evidence it will succeed. Retain cached model revision,
notices, raw headroom, mean/sample controls and AST/CLAP. If the codec preserves
water detail, next train an object/event-conditioned latent sequence, without
target audio at inference. If not, do not start that generator. A positive
reconstruction control alone will NOT satisfy the user goal or material control.
No new weights downloaded and no extra training launched for this selection.

Reproduction: `physical_sound_pouring_cvae_adversarial.py --source SOURCE
--parent ORIGINAL_CVAE --output NEW` with/without `--adversarial`; use the
existing CVAE `evaluate` with the retained flow base. The same new script with
`--parent ADVERSARIAL_MODEL --phase-budget-probe` runs only the discriminator.
98 focused tests and Ruff pass;212 new WAVs pass writer hashes, mono PCM16,
16kHz, finite/headroom checks. Four-way comparison reads back at18.32s.
Source mismatch, critic/decoder gradient isolation and frozen representation
are tested. No runtime/default/roadmap change, Cargo or ProductCheck run.
All jobs terminal; full multi-event goal remains active.

### Frozen waveform codec and conditional latent flow (2026-09-05)

[Source-free four-profile comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-latent-flow-evaluation-2026-09-05/comparison.wav>)
is36.64s: glass10/glass16/PET10/glass10fast, each retained STFT base then new
latent flow, seed2718/gain1. [Standalone generated sound](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-latent-flow-2026-09-05/first-audition/generated.wav>)
needs only eleven object/event controls and randomness, not a reference/cache.
This candidate remains rejected: partial gains do not establish water quality.

`physical_sound_pouring_wave_codec.py` reuses cached TangoFlux revision
`367005e963cb3a9fb2e03a46104d7de23e34ceea`,624490208-byte Oobleck weights SHA256
`d73619a1d1e1dc48e606632931ffce440b4959ce2a4ed5a3522c3bb573b103be`.
No download or codec training. Strict state load, eval/frozen weights; notices
travel with external artifacts. Powered by Stability AI; TangoFlux/Hung et al.,
local research only. Sound of Water dataset redistribution remains unspecified.
Current4.08s16kHz crops are resampled44.1kHz and duplicated to two channels,
padded from179928 to180224 samples; this does not restore original high bands.
Codec latent is64x88; output trimmed back to4.08s. Native44.1k stereo and16k mono
diagnostics are retained without gain/clipping. This is not spatial calibration.

`pouring-wave-codec-2026-09-05` uses first/middle of the same first training
record and disclosed objects18/30:6 real,6 posterior means,18 posterior samples,
seeds314/2718/1618. AST raw AND normalized accepts5/6 means and15/18 samples;
CLAP6/6 and18/18. All real controls pass. Glass18 middle fails AST for every
codec variant (Drip is near the top, but fixed Water/Pour criteria are unchanged).
Mean/sample spectrum RMSE3.579/3.630dB, CV error0.02971/0.02965. Thus useful
water detail survives most of these codec controls, unlike the rejected CVAE;
not a universal codec-quality pass. [Reconstruction comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-wave-codec-2026-09-05/comparison.wav>)
is41.22s: train/glass18/PET30 middle, each real/mean/sample2718. It is NOT
reference-free generation or success on the user's physical-control goal.

`physical_sound_pouring_latent_flow.py` caches279 fixed first/middle/last crops
of93 training recordings/13 objects in `pouring-oobleck-cache-2026-09-05`.
Only those training targets supply channel center and scale, including posterior
variance; std floor0.1. No excluded-object audio enters fitting/statistics.
The892992-parameter1D conditional U-Net uses residual64/128/192/128/64 blocks,
GroupNorm, time and eleven controls through128-dimensional affine context,
two stride2 reductions and skip-connected interpolation. It predicts velocity
from noise/data interpolants in the normalized64x88 waveform-codec latent.
Targets sample cached posteriors. One2000-step fit uses seed53,batch16,AdamW
lr3e-4,wd1e-4,clip1; first/last200 mean loss1.898/1.530.64 Euler steps generate
a latent, then the frozen decoder produces audio. This is not converged by fiat.
Checkpoint `pouring-latent-flow-2026-09-05` SHA256
`72045123754eb5b47edae56aaf1a281e2ca9ece4f35f869b096714cf514f2bb4`;
cache metadata SHA256 `743e1f2ea8e327e02469c905ac4f3d8faa32e068fc0b836d148ccfdf7f6a4307`.

Evaluation retains66 individual rows:6 real,18 paired base/generated for first/
middle train/objects18/30, plus12 paired base/generated hypothetical profiles.
Zero unsafe outputs. All three seeds remain. Same AST raw/RMS0.005 and frozen
six-prompt FP32 CLAP; they are not losses or complete naturalness/physics judges.

| Development12 clips | Retained STFT base | Waveform latent flow |
|---|---:|---:|
| Spectrum RMSE,dB | 7.655 | 7.819 |
| Centered spectrum,dB | 5.990 | 6.724 |
| CV absolute error | 0.441 | 0.290 |
| AST raw/normalized | 10/12,12/12 | 0/12,0/12 |
| CLAP positive water margin | 12/12 | 9/12 |
| Novel AST/CLAP,out of12 | 12/12 | 0/8 |

Training6: latent spectrum6.114 vs base6.339,CVerror0.236 vs0.315; AST0/6,
CLAP2/6 versus base6/6. This improves an envelope statistic and some CLAP scores
over CVAE, not overall quality or correct material/geometry transfer. Only two
opened development objects are represented. Runtime/base/default stays unchanged.

Bounded causal probe: [Flow Matching,v2,2023-02-08,sections3–4](https://arxiv.org/html/2210.02747v2)
defines conditional probability paths and regression of their vector fields.
Here the straight interpolant is x(t)=(1-t)noise+t target, target velocity is
target-noise. To discriminate decoder failure from imperfect learned transport,
`probe` starts at fixed steps0/32/56/64 of the SAME64-step grid, using encoded
target information only for nonzero starts. No training or solver-step sweep.
`pouring-latent-trajectory-probe-2026-09-05` retains72 WAVs and a54.96s preview;
posterior seed=seed+1000, noise seed314/2718/1618. Start1 is codec control;
only start0 is source-free. Other starts cannot be promoted as the solution.

| Start t | Train AST raw/normalized,out of6 | Dev AST both levels,out of12 | Dev CLAP,out of12 |
|---|---|---|---|
| 0 | 0/0 | 0 | 9 |
| .5 | 1/0 | 1 | 9 |
| .875 | 6/6 | 9 | 12 |
| 1 | 6/6 | 9 | 12 |

Late privileged paths preserve recognition, unlike paths beginning far from
the target. This localizes a learned-transport deficit; it does not prove an
exact failing timestep, insufficient capacity or a particular loss fix.
`field-diagnostic.json` uses all279TRAIN cached crops, three seeds, times
0/.25/.5/.75/.875/.984375,batches32. For each seed, draw posterior target, then
noise, then a single random condition permutation, reusing these across times.
Compare learned correct/shuffled controls against k(t)x, with
k(t)=(2t-1)/((1-t)^2+t^2), the unit-diagonal-Gaussian marginal field.
This is a control, NOT a true data lower bound. Correct conditions beat shuffled
on764–815/837 pairs per time, so the model does not wholly ignore controls.
Its velocity MSE is nevertheless worse than the Gaussian control at t0
(1.349>1.000),t.875(1.487>1.280),t.984375(1.448>1.032), while better at t.5
(1.631<1.998). Conditioning use does not certify physical correctness.

Next bounded implementation: Gaussian skip k(t)x plus a learned residual,
against a matched plain-velocity control; same cached data,2000steps/seed53,
both zero-initialized output layers. This targets the measured endpoint-field
deficit, not an unsupported epoch/capacity increase. Judge new source-free WAVs
and endpoint errors together; retain decoder/base and all failed seeds. No
epoch/lr/capacity or seed/threshold sweep. No additional fit launched yet.

CLI paths: waveform codec `--source --output`; latent flow `cache --source
--output`, `train --cache --output`, `render --model --controls ELEVEN_VALUES
--output`, `evaluate --source --model --base --output`, `probe --source --model
--output`. Render never opens source/cache. CLI replay before/after shared ODE
helper extraction reproduces first-audition hash exactly:
`22c73aef2ee7a04b9ef0019ce61f6cf19f77dcdaec523e788e100f59473c0180`.
105 focused tests, Ruff and198 new WAV hash/format/finite/headroom checks pass.
All jobs terminal; no new download, runtime/roadmap/ProductCheck promotion.

### Gaussian velocity skip: endpoint error improves, audio does not

[Three-way source-free comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-latent-gaussian-skip-evaluation-2026-09-05/ablation-comparison.wav>)
is13.74s: retained STFT base, matched zero-output plain latent flow, Gaussian-skip
latent flow; glass H10cm/diameter7cm/duration15s/start0.1,seed2718,gain1.
All four profiles/three seeds remain in each evaluation; no favourable selection.
Both new models remain rejected. Correcting a measured field defect did not
demonstrate a perceptual improvement.

The existing latent-flow CLI adds `train --zero-output` and `--gaussian-skip`.
Two fits share the279-crop cache, normalization,892992parameters,seed53,
batch16,2000AdamW updates,lr3e-4,wd1e-4,clip1 and random draws. Both final layers
start at zero; only one adds k(t)x, where k(t)=(2t-1)/((1-t)^2+t^2), to its
learned velocity. No codec training or new data. The Gaussian form is a control
distribution, not an assumption that water latents are actually independent
unit Gaussians. Learned residuals can modify the endpoints after training.
Separate format `pour-latent-flow-gaussian-v1` preserves the generation semantics
on reload; original `pour-latent-flow-v1` checkpoints keep the unchanged path.

`pouring-latent-zero-plain-2026-09-05` checkpoint SHA256
`643568f1ee124a7841ef8c7ad5bcca7ae3f4d377ccfb28e26904a43983db113b`;
`pouring-latent-gaussian-skip-2026-09-05` SHA256
`d58f4a325e9b49a5e7b9c8b4f0dd813041a5fb5c243f94ba165f5c9b9d690a55`.
First/last200 loss plain1.8743/1.5394, skip1.5196/1.3839. Completed, not proven
converged. Model directories retain `field-diagnostic.json` with the same279
TRAIN crops/three seeds/six times/matched condition permutations as above.
Correct-control MSE at t0/.5/.984375 is plain1.3849/1.6302/1.4391 versus
skip0.9952/1.6386/1.0460. The intended endpoint defect is substantially reduced;
mid-path error is not. Lower aggregate training loss is not a quality pass.

Separate `pouring-latent-zero-plain-evaluation-2026-09-05` and
`pouring-latent-gaussian-skip-evaluation-2026-09-05` each preserve66 WAV rows,
fixed AST raw/RMS0.005 and six-prompt CLAP. Zero unsafe outputs. Same disclosed
training record/two development objects and hypothetical profiles, not clean test.

| Development12 clips | Matched plain | Gaussian skip |
|---|---:|---:|
| Spectrum RMSE,dB | 8.512 | 8.860 |
| Centered spectrum,dB | 6.751 | 7.165 |
| CV absolute error | 0.241 | 0.378 |
| AST raw/normalized | 0/0 | 0/0 |
| CLAP positive | 9 | 10 |
| Novel AST/CLAP,out of12 | 0/10 | 0/7 |

Training6: plain/skip spectrum6.875/6.064,CVerror0.248/0.200; AST0/6 both,
CLAP5/6 versus3/6. Real controls pass throughout; retained base results unchanged.
Thus the hypothesis that this endpoint correction is enough for useful audio
is not supported. No skip-weight, epoch, learning-rate, capacity or seed sweep.

Bounded alternative hypothesis: posterior sampling might spend most learning
effort on unused/noisy latent channels. [RAVE,v2,section3.2](https://arxiv.org/html/2111.05011v2)
distinguishes informative latent means from posterior noise and analyses the
mean representation's rank. This motivates measuring our frozen codec, not
assuming its channels can be discarded. The skip directory's
`latent-variance-diagnostic.json` uses only the279TRAIN posteriors. Per-channel
Var(mean)/(Var(mean)+E(std^2)) is0.9076–0.9988, average0.9518; ALL64 channels
exceed0.9. Normalized posterior-mean covariance needs54/61 components for95/99%
variance (eigenvalue variance ratio, not RAVE's singular-value fidelity rule).
This rejects the predominantly-posterior-noise explanation on these data;
it does not measure decoder importance or authorize masking/PCA/mean-mode fits.

Next discriminator is one-crop learnability, not another full-corpus sweep:
cache row0,first training record/first phase, same frozen Oobleck and Gaussian
skip,2000updates. Compare the resulting reference-free trained samples against
the exact Gaussian posterior decoder control for that one crop. If it cannot
learn even this, diagnose optimizer/parameterization before broader fitting;
if it can, investigate multi-example transport. A memorized crop is explicitly
NOT novel-object synthesis or completion of the user's multi-event goal.
This single-crop fit has not yet been run; the completed full fits stay frozen.

106 focused tests, Ruff, diff and local references pass.138 new WAVs pass writer
hashes,PCM16/16k mono,finite/headroom checks. New-format CLI exactly reproduces
its first-audition. Tests cover the analytic field's endpoints/midpoint and
format-aware checkpoint reload. All jobs terminal; no source download, codec
training, runtime/default/roadmap or ProductCheck promotion.

### Single-crop learnability, longer fit and evaluator-confound check

[Single-crop comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-latent-single-crop-evaluation-2026-09-05/comparison.wav>)
contains real / exact posterior / learned audio, seed2718, gain1,13.74s.
The learned generator needs no reference recording, but explicitly memorizes
one TRAIN crop; this is a learnability control, NOT new-object generation.
`train --single-crop --gaussian-skip --zero-output` uses cache row0, first phase
of `VID_20240116_230040_2.1_16.7`,2000steps, unchanged optimizer/seed53.
Normalization still uses all93 TRAIN recordings; fitted IDs contain only one.
`single-probe --source --cache --model --output` compares seeds314/2718/1618
against decoder samples from that crop's exact Gaussian posterior. Full-source
evaluation cannot silently treat this model as a full-corpus fit.

`pouring-latent-single-crop-2026-09-05` checkpoint SHA256
`1341e6ddf4d4b235ec72d6c1039bf8d0b228e9aa510f7d7eb2cbf55afe1b759b`.
Real1/1, exact posterior3/3 and learned3/3 pass both unchanged AST levels and
the original CLAP comparison. Spectrum RMSE oracle3.036, learned3.964dB.
First/last200 loss1.204/0.585. This falsifies inability to learn even one crop,
not multi-example failure. The evaluation's `analytic-field.json` compares
768 draws (three seeds ×256) against the exact single-posterior field. For
normalized mean mu/std s, S²=(1-t)²+t²s², that field is
`mu + (t*s²-(1-t))/S² * (x-t*mu)`. Its uniform-time irreducible sampled-target
MSE is pi/2*mean(s)=0.24954. Empirical model/analytic-target MSE0.56450/0.24642;
model-versus-analytic-field error0.31822 remains. No convergence claim.

The positive control and still-decreasing loss justified ONE longer full fit,
reconsidering the previous blanket no-epoch-extension instruction. This was
not an epoch/lr/capacity sweep. `train --gaussian-skip --steps 8000 --parent
GAUSSIAN_SKIP_MODEL --cache CACHE --output NEW_MODEL` warm-starts the existing
2000-step model on the same279 TRAIN crops: total10000updates. **AdamW and RNG
reset**, not exact optimizer resume; lr3e-4,wd1e-4,batch16,clip1,seed53 unchanged.
Cache, fitted IDs, scope, model mode and exact normalization must match before
creating output. Codec is frozen. `pouring-latent-gaussian-long-2026-09-05`
checkpoint SHA256
`1ddab4d6298d8b93e0f58f8e6c8367afd9dbf7bb47d7904ed1c36d1f09c9da49`.
Phase first/last200 loss1.39094/1.31266; all40 windows retained in model.json.

[Long-fit comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-latent-gaussian-long-evaluation-2026-09-05/comparison.wav>)
is36.64s: retained base/new latent, glass10/glass16/PET10/glass10fast,
seed2718, gain1. All66 individual outputs remain available; no unsafe outputs.
Same disclosed training/development and hypothetical profiles, not clean test.

| Long latent subset | Spectrum RMSE,dB | CV absolute error | AST raw/normalized | Original CLAP |
|---|---:|---:|---:|---:|
| Train6 | 5.152 | 0.162 | 2/2 | 6/6 |
| Development12 | 7.305 | 0.180 | 0/0 | 10/12 |
| Hypothetical12 | no paired target | no paired target | 1/1 | 12/12 |

Relative to the2000-step skip, development spectrum8.860->7.305 and CVerror
0.378->0.180 improve. Neither improvement nor CLAP12/12 establishes quality.
AST mostly predicts Scrape/Crunch/Tearing, not alternative water tags.
The [AudioSet ontology](https://github.com/audioset/ontology) is hierarchical;
[Drip](https://research.google.com/audioset/ontology/drip.html) describes liquid
drops, but this taxonomy ambiguity does not explain these dominant negatives.
No existing AST tag set or threshold was broadened.

The long evaluation's `clap-hard-negatives.json` retains all six original
prompts and adds seven AST-motivated alternatives: scraping, tearing, crushing,
chewing, toothbrushing, cutlery/dishes and rattling. **Post-hoc diagnostic**, not
a calibrated quality gate or independent test. Same frozen FP32 CPU CLAP and
all73 long/single evaluation rows, no retraining. Original versus expanded water
margin positives: long hypothetical12/12->2/12; long paired16/18->2/18.
Real6/6, retained base30/30, single oracle3/3 and learned3/3 remain positive.
Thus weak negatives explain much of the apparent CLAP/AST disagreement;
neither evaluator proves naturalness or both interacting materials.

### Numeric solver counterfactual: no rescue, generation default unchanged

[Solver comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-latent-solver-probe-2026-09-05/comparison.wav>)
is13.74s: Euler64 / midpoint128 / midpoint256, glass10,seed2718,gain1.
[Flow Matching's official solver documentation](https://facebookresearch.github.io/flow_matching/generated/flow_matching.solver.ODESolver.html)
identifies solver choice/step size as separate inference controls. The prior
STFT256-step failure did not test this new latent field. Hypothesis: coarse
integration rather than learned-field error might explain its scratchy audio.

The external `pouring-latent-solver-probe-2026-09-05` applies all three methods
to the same12 hypothetical conditions/seeds and three single-crop controls:
45 individual WAVs, no source/cache/model updates. Explicit midpoint uses
`k=v(x,t,c); x += h*v(x+h*k/2,t+h/2,c)`, h=1/N, t=i/N; two field evaluations
per step. Analytic x'=x control at256steps differs from exp(1) by<1e-5;
input is not mutated. Euler64 replays all15 existing WAV hashes exactly.
Mean normalized endpoint RMSE Euler64->midpoint256 is0.02213 full/0.01421
single; midpoint128->256 is0.0000551/0.0000619. This is strong local numerical
convergence evidence, not proof of globally accurate learned dynamics.

All methods give raw AST1/12 full and3/3 single; original CLAP12/12 and3/3.
Expanded CLAP full2/12->1/12->1/12, single3/3 throughout. Refinement does not
rescue semantics. No solver-default change, further step sweep or threshold
relaxation. Normalized AST was not rerun for this solver-only diagnostic;
both levels remain stored for the original long/single evaluations.

Next bounded change targets multi-example learning, not sampling: compare a
per-crop Gaussian affine transport path against the frozen matched zero-output
plain2000-step control. [Lipman et al.,v2,2023-02-08,section4](https://arxiv.org/html/2210.02747v2)
construct analytic Gaussian paths and explain conditional flow matching.
Our proposed diagonal-posterior extension is `x=t*mu+(1-t+t*s)*noise`, target
`mu+(s-1)*noise`: same initial Gaussian and cached posterior endpoints, with
no second independently sampled posterior noise. This is an inference from
the Gaussian construction, not a published audio-quality guarantee. Preserve
conditioning/noise/time draws, cache, plain architecture, zero output,2000steps
and seed53 to isolate path choice; do not repeat the old control fit. Compare
actual new WAVs, unchanged AST and the now-disclosed harder CLAP diagnostic.
The change has NOT been implemented or trained yet; no corpus/new-object or
physical-calibration claim is admitted by any of the above controls.

109 focused tests, Ruff formatting/static checks and diff pass; warm-start
negative tests cover mode/cache/IDs/scope/center/scale before output creation.
124 new WAVs pass writer SHA256,PCM16/16k mono and finite/headroom checks.
Long-model standalone CLI replay is byte-exact, SHA256
`02c5aea472e223a4a16a798d7845ac9b369d352851b68bb143ddb1bb56497b5e`.
Notices/checkpoint identities verified; artifacts remain external/local research.
All jobs terminal. No runtime/default/roadmap change or ProductCheck promotion;
the full multi-event, new-condition neural-sound goal remains active.

### Affine posterior path: no semantic rescue; full training coverage checked

[Matched source-free comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-latent-affine-evaluation-2026-09-05/ablation-comparison.wav>)
is13.74s: retained STFT base / independent plain flow / affine plain flow,
glass10,seed2718,gain1. The new affine candidate is rejected, not promoted.
`physical_sound_pouring_latent_flow.py train --zero-output --posterior-path
affine --cache CACHE --output NEW_MODEL` implements the preceding Gaussian
coupling experiment. The independent default retains the exact old arithmetic.
Both branches consume posterior noise, initial noise and time in the same order,
even though affine ignores the first draw. Same279TRAIN cache,892992parameters,
plain architecture,zero output,seed53,2000steps,batch16,AdamW3e-4/wd1e-4/clip1.
The frozen matched plain control was reused, not retrained. No skip/capacity/
learning-rate change. Metadata records the training path; warm-start rejects a
different path. Inference remains the same source-free64Euler/frozen decoder.

`pouring-latent-affine-2026-09-05` checkpoint SHA256
`1b6e08132f26e2b244a2447360d66d371e06b78cbec8fe125363e72c5d852c3b`.
First/last200 loss1.50577/1.24655. Loss values across different couplings have
different irreducible components and must not be presented as audio improvement.
All66 evaluation rows plus the36.64s four-profile comparison remain available.
Both AST raw and RMS0.005 give zero positives for train6/dev12/hypothetical12,
as does the same harder13-prompt CLAP diagnostic for all30 latent outputs.
The frozen matched plain also has harder CLAP0/30. Real6/6 and base30/30 pass
harder CLAP; existing base/source hashes are unchanged.

| Paired metric | Independent plain | Affine plain |
|---|---:|---:|
| Train6 spectrum / shape,dB | 6.875 / 5.540 | 6.111 / 5.272 |
| Train6 CV absolute error | 0.248 | 0.275 |
| Development12 spectrum / shape,dB | 8.512 / 6.751 | 8.129 / 6.830 |
| Development12 CV absolute error | 0.241 | 0.263 |

Original easy CLAP train/dev/hypothetical plain5/9/10 versus affine6/9/11;
hard negatives again expose the misleading impression. No affine-strength,
epoch/seed or additional coupling sweep follows this negative result.

The next bounded discriminator questioned an adjacent assumption: only six
source/codec controls had previously been checked, so poor training targets
or an unrepresentative training evaluation could explain the apparent failure.
The [publisher's dataset](https://huggingface.co/datasets/bpiyush/sound-of-water)
has separate `clean` and `bg-noise` columns; `clean=yes` does not imply the
other column is `no`. No new source/download/role or classifier threshold.

`pouring-training-codec-audit-2026-09-05` audits ALL279 cached TRAIN crops,
93 recordings/13 containers, paired original and frozen posterior decode with
seed2718:558 individual WAVs, no output failures. Exact cache/source/codec
hashes and per-row controls checked before use; no recoding or filtering.
[Source/codec comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-training-codec-audit-2026-09-05/comparison.wav>)
is27.48s, first middle crop of the first three training containers in cache
order, each real/posterior,gain1; it is NOT a reference-free generation.

| All279 TRAIN crops | Original | Posterior decode |
|---|---:|---:|
| Raw AST Water/Pour top5 | 252 | 216 |
| Original six-prompt CLAP | 278 | 276 |
| Harder13-prompt CLAP | 267 | 232 |
| Both raw AST and harder CLAP | 252 | 206 |

`group-summary.json` retains all groups and paired changes. Harder CLAP loses
35 source-positive cases after the codec, with no opposite flip. By phase,
source first/middle/last93/92/82 of93; codec83/82/67. Raw AST source90/89/73,
codec86/75/55. Nine crops have publisher background-noise=yes;66 need zero
padding, at most9984samples/0.624s. Padding, recording conditions and phase are
correlated; this does NOT prove a single cause or justify deleting failures.
Codec limitations are real, but the useful majority contradicts a wholly
non-water target corpus. No codec training or classifier-driven dataset purge.
Normalized AST was not run for these all-corpus diagnostics.

Then `pouring-long-training-coverage-2026-09-05` generates from ALL279 TRAIN
control vectors, fixed seed2718, using the frozen10000-step Gaussian model
(SHA1ddab4d6 above). It reads no target waveform or cache tensor at inference.
279 individual WAVs plus a13.74s preview, no unsafe outputs. This is training
coverage, not independent test/new-condition proof. Raw AST11/279, easy CLAP
223/279, harder CLAP6/279, both2/279. Synthetic validator controls are excluded
from these denominators. Thus the earlier small evaluation was NOT hiding
broad success on familiar conditions. No material passes reliably: harder
CLAP glass4/117,plastic2/18,PET0/87,PP0/57. This does not prove that every
possible larger/longer scratch model fails, but rules out treating this model
as useful and the current deficit as only unseen-condition generalization.

Decision: stop similar small scratch-flow variants; use a pretrained GENERATOR
prior, not only its codec, for the next physical-conditioning experiment.
[FoleyCrafter,v1,2024-07-01,sections3.2–3.3](https://arxiv.org/html/2407.01494v1)
trains added conditioning components with a frozen audio generator; its inputs
are video, not our physical measurements. Its substantial training scale is
not evidence that a tiny adapter will solve our problem cheaply.
[Audio ControlNet's project page](https://audio-controlnet.github.io/), inspected
2026-09-05 and labelled under peer review, describes frozen-backbone control of
pitch/loudness/events. This is supporting prior art, not a verified dependency
or a physical-parameter calibration result; no model/code was downloaded.

Next implementation: a zero-initialized numerical conditioning bridge to the
already cached frozen TangoFlux generator, using the eleven published controls.
Keep its text encoder, codec, transformer and existing q/v weights frozen;
do not repeat generic-caption LoRA or introduce video input. Verify adapter-off
and zero-adapter exact baseline output, then produce an actual trained candidate
and matched source-free WAVs in the same checkpoint. Preserve unconditioned
glass/wood/rain regression paths. Use TRAIN-only data and disclosed development
roles, no audio input at generation. First reconcile upstream duration/latent
coordinates: tiny-flow normalization and its88-frame cache are NOT automatically
valid inputs to pretrained TangoFlux. Judge generated audio and condition swaps,
not flow loss alone. This bridge has not yet been implemented; full physical
response, new-condition validation and multi-event goal remain unachieved.

110 focused tests pass; path endpoints/finite-difference derivative, exact old
formula/RNG preservation and warm-start path rejection covered. Ruff and diff
pass. Affine standalone CLI byte-replays its first-audition, SHA256
`7a4ff98f25901107f7e8de7ce6a03e06721affe20159db6f45429a3a429c9052`.
909 new WAVs pass writer hashes,PCM16/16k mono and finite/headroom checks.
Full-training coverage exactly replays the earlier first/middle training clips.
All jobs terminal. All artifacts remain external local research; no runtime/
default/roadmap changes. Cargo/ProductChecks NOT_RUN: external Python lab only.

## Frozen generator numerical bridge and TRAIN-mean counterfactual — 2026-09-05

Implemented `lab/scripts/physical_sound_pouring_bridge.py`: eleven numerical
controls -> Linear11/128,SiLU,Linear128/2048,265728 trainable parameters. A
zero-initialized output adjusts positive text-token and pooled conditioning of
the cached TangoFlux generator. Duration token and unconditional CFG row remain
unchanged. Transformer/T5/codec and original glass/wood/rain paths stay frozen.
This implements the preceding bounded experiment, not a promoted engine model.

`pouring-tango-bridge-2026-09-05` uses the same93 TRAIN recordings/279 crops.
It re-encodes each4.08s crop as44.1k dual mono padded to upstream30s, producing
native645x64 posteriors. It does NOT substitute the old normalized88-frame
cache into the pretrained model. Native posterior SHA256
`20850122c2d963cd8a3d7cf84a02a140524aefd8474bd515f283e128869c23ce`.
Cached loss exactly matches upstream0.4048370122909546 with matched random draws.
Training: seed53,200 updates,AdamW1e-4,weight decay0.01,gradient clip1,
BF16 training autocast/FP32 weights; full-horizon original flow-matching loss,
no classifier reward.147 distinct crops/82 TRAIN recordings visited. First/last
20-update mean loss1.6717/1.5969 is not a perceptual success criterion.

Bridge SHA256 `2370690789d087fdec079f67239d42734f52db480a81697c0cb29db4361009ea`.
Full Tango model state digest before/after training is unchanged:
`23ee7758b8b637389e8d0378484b79c326a362f1069d760c3344df641224b258`.
Zero bridge equals baseline PCM in both formats. Generation uses fixed text,
4.08s,50 steps,CFG4.5,seeds314/2718, four hypothetical control profiles:
glass10cm/glass16cm/PET10cm/glass10cm with shorter pour-duration proxy. Native
output uses the full645-frame trajectory and is decoded before trimming.
No reference recording or posterior is an inference input.

| Eight profile/seed rows | Base | Full bridge | Centered bridge |
|---|---:|---:|---:|
| Raw AST Water/Pour top5 | 8 | 4 | 8 |
| AST with classifier-input RMS0.005 | 8 | 3 | 8 |
| Original six-prompt CLAP | 8 | 8 | 8 |
| Same harder13-prompt CLAP | 8 | 1 | 8 |

Base rows contain only TWO unique waveforms, repeated across profiles; do not
treat them as eight independent generations. Full bridge is rejected: crunch,
toothbrushing or rattling wins seven harder-CLAP comparisons. Evaluation remains
diagnostic, not a calibrated gate or independent physical validation.

`conditioning-variation.json` finds98.397% of bridge-output energy in its common
TRAIN-mean component: RMS0.050223 common versus0.006410 varying. One post-hoc
counterfactual subtracts the mean over ALL279 TRAIN control vectors; no training
repeat, dev-fitted offset or strength sweep. Pairwise embedding differences are
unchanged algebraically, but physical waveform differences are not guaranteed.
`pouring-tango-bridge-centered-2026-09-05` saves eight new WAV pairs and the frozen
offset, SHA256 `c237816597393b13f9834632f9e6866d0e1343bb1e2c3686c41d2465f699bb9d`.
[Listen: base/full/centered](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-centered-2026-09-05/comparison.wav>),
13.74s,glass10/seed2718, published PCM concatenation without extra gain.
This restores water recognition, not yet calibrated material/geometry response.

Export/replay failures were preserved rather than counted as successes:

- Initial fit finished all200 updates and generated16 evaluation rows, then
  comparison export rejected a float32 value0.980000019 above its guard.
  Fixed by concatenating exact published PCM, not an epsilon or relaxed guard.
  A focused test also caught and removed one-LSB re-quantization.
- First recovery used identical weights but unfrozen parameter flags and failed
  glass byte replay. Same-process counterfactuals isolate generator parameter
  flags: requires_grad=True differs,False restores exactness; VAE flags alone
  do not. No specific backend-kernel cause is asserted. `generate` now explicitly
  freezes both models before inference; a regression test covers this setting.
- `failed-export.json`, `unfrozen-reload.json` and the mismatched WAVs remain.
  Final recovery reran export/regression only, not training. All three adapter-off
  glass/wood/rain before/after pairs now match both mono/native SHA256 exactly.

Standalone `render --model BRIDGE --controls <11 floats> --output NEW` byte-replays
the full bridge. Optional `--offset CENTERED_DIRECTORY` also byte-replays centered
glass10/seed2718 in both formats. The loader checks bounded files, tensor shape,
finite FP32, bridge/full-model/TRAIN-posterior identities and offset hash. Default
bridge state/checkpoint behavior is unchanged; no source/cache argument needed.
All72 WAVs across the fit, centering, flag controls and both CLI runs pass PCM16,
rate/layout/headroom checks; published result rows pass mono/native hashes.
Native peaks can exceed1 (full bridge up to1.5297); recorded attenuation gains
make audition PCM safe. This is NOT calibrated loudness/force evidence.

136 focused tests, Ruff lint/format pass. No Cargo/ProductCheck/runtime/default
or roadmap promotion. Sources and model notices remain external local research.
The subsequent discriminator is complete in
`pouring-tango-bridge-development-2026-09-05`: first source-order recording of
each already-disclosed excluded container18/30, first/middle phases,seeds314/2718.
Eight centered generations, two unique base generations and four real controls;
other-object/same-phase generated audio is reused for the swapped-control
comparison. All eleven controls are swapped together, not just material.
No extra fitting or seed selection. Real audio enters comparison metrics only.
[Listen: excluded glass/PET](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-development-2026-09-05/comparison.wav>),
36.64s,glass18 then PET30,middle phase,real/base/matched/swapped,seed2718.
Each clip retains its published gain; quiet real PET is not boosted to match.

| Disclosed development,8 matched target/seed pairs | Base | Matched | Swapped |
|---|---:|---:|---:|
| Spectrum RMSE,dB | 18.915 | 18.845 | 19.050 |
| Level-centered spectrum shape RMSE,dB | 7.756 | 7.687 | 7.813 |

Matched shape error beats base5/8 and swapped6/8; mean improvements0.070 and
0.126dB are marginal. Seed variation is larger (for example glass-first base
9.711 versus7.018dB). Two objects and correlated phases/seeds do not establish
physical calibration or independent generalization. Material, geometry, duration
and recording/acquisition effects are not isolated. Water identity survives:
raw and RMS0.005 AST plus harder CLAP all give real4/4,base2/2,centered8/8.
Twenty-five additional WAVs and result-row hashes pass the same audit,97 total.

Decision: retain centered bridge as an audible, reference-free experimental
candidate, not a physical-response success. Before another fit, measure matched
versus fixed swapped TRAIN-control velocity errors with identical posterior,
noise and time draws; compare base/full/centered and active88 versus full645
frames. This distinguishes conditional learning from a common domain shift;
do not assume padded silence dominates merely because it occupies most frames.
Include a TRAIN-real audible control. No offset-strength, epoch or seed sweep.
All jobs terminal; broad multi-event user goal remains active.

## Conditional signal and training-time centering — 2026-09-05

`pouring-tango-bridge-signal-2026-09-05` uses the first source-order recording
of each13 TRAIN objects, middle crop, sigmas0.2/0.5/0.8. All six variants share
one posterior sample and noise per crop, CPU seed10000+cache index, FP32 frozen
generator inference. Wrong controls are a fixed cyclic next-object permutation.
Same verified native645-frame cache; no heldout data, retraining or score selection.

| Mean velocity MSE,39 crop/time pairs | Active88 | Tail557 | Full645 |
|---|---:|---:|---:|
| Base | 1.217021 | 1.590602 | 1.539633 |
| Full bridge,matched | 1.163848 | 1.591441 | 1.533102 |
| Full bridge,swapped | 1.165143 | 1.591455 | 1.533291 |
| Post-hoc centered,matched | 1.211986 | 1.590589 | 1.538935 |
| Post-hoc centered,swapped | 1.213984 | 1.590597 | 1.539214 |
| Common TRAIN mean only | 1.164725 | 1.591360 | 1.533152 |

The silence-dominated-learning hypothesis is contradicted here: full bridge
improves active error while tail error worsens slightly.98.35% of active
improvement is reproduced by the common correction alone. Full matched controls
beat swapped31/39, but by only0.001295 mean active MSE. After centering,22/39,
mean0.001998. These are correlated TRAIN probes, not generalization or estimates
over the full time-sampling distribution.
[TRAIN audible control](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-signal-2026-09-05/comparison.wav>)
is18.32s:first training object,middle crop,real/base/full/post-hoc centered,
seed2718,published gains. References are used for diagnosis, not generation.

One reversible counterfactual adds `run --center-training` to the bridge script.
During each update, subtract the differentiable network mean across ALL279 TRAIN
control vectors. Final output bias cancels; condition-dependent gradients remain.
After fitting, freeze this mean into an offset, drop the training-control bank
and automatically load the verified offset with this model's checkpoint.
No source/cache or extra flag is needed at inference. Existing uncentered and
post-hoc-centered loaders/defaults remain unchanged.

`pouring-tango-bridge-center-trained-2026-09-05`: same200 steps,seed53,optimizer,
BF16 training/FP32 inference,loss,sample sequence and frozen generator. Native
posterior file and all200 sampled cache indices exactly match the earlier fit.
Bridge SHA256 `e5a792889481e2f0f3ac9ec99aa8994449e407100efc249254503e05851d8159`;
offset `4bd29db8631c35830cd2b07c1bb011c390ce303bbc037001df5f05360d7d0dd9`.
Zero/bypass/upstream-loss/full-model hashes and all glass/wood/rain regression
pairs pass. First/last20 loss1.6743/1.6073, not a perceptual criterion.
The matched TRAIN probe active MSE1.207547 versus swapped1.211201 gives22/39
wins,mean gain0.003654: a larger conditional training signal, not transfer proof.

`pouring-tango-bridge-center-trained-development-2026-09-05` repeats the exact
two excluded objects/first-middle/seeds314-2718 comparison. Shape RMSE is
8.106dB versus old centered7.687/base7.756/new swapped8.214. New beats old only
2/8,base3/8,swapped5/8. **Reject as an improvement**; retain the previous centered
experimental model. Water identity still passes raw/RMS0.005 AST and harder
CLAP8/8 on both hypothetical and development generations; semantic recognition
does not rescue the physical-response result.
[Listen: real/old/new/swapped](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-center-trained-development-2026-09-05/comparison.wav>),
36.64s,glass18 then PET30,middle phase,seed2718,published gains.

Bounded adjacent-layer research (inspected2026-09-05):
[PAVAS v2,2026-03-30,sections3.4/E.4/E.5](https://arxiv.org/html/2512.08282v2)
uses zero-initialized residual corrections to per-block adaptive-normalization
parameters rather than directly mixing physical features into the shared
condition. It also updates diffusion blocks; it is NOT evidence that our tiny
frozen-generator bridge should work. Its physical inputs include estimated
mass/velocity and video features, not measured container controls; APCC is an
energy-correlation metric, not material validation. The official
[repository](https://github.com/SonyResearch/PAVAS) currently exposes README and
teaser only, with code/assets pending. CVF PDF fetch returned403; arXiv v2 was
read instead. No downloads, code execution or reliance on pending checkpoints.
The [UPF texture thesis repository](https://github.com/Metiu-Metiu/Neural-Texture-Sound-Synthesis-with-physically-driven-continuous-controls)
describes synthetic-to-real parameter pseudo-labelling before conditional audio
training. This is an alternative data route, not independent measured physical
labels or an adopted implementation.

Next: stop centering/epoch/seed variants of global text/pooled shifts. Inspect
the existing generator's modulation interface and test a bounded zero-initialized
audio-layer residual against the current bridge: zero/bypass exactness, then
matched/wrong-condition signal and playable output before any larger fit.
Keep backbone frozen and existing reference-free fallback; PAVAS does not
authorize full-model retraining, video inputs, invented mass labels or new
runtime authority. Physical parameter transfer and the full user goal stay open.

138 focused tests pass, including common-bias cancellation, nonzero conditional
gradients, exact freezing and automatic verified-offset reload. Standalone CLI
without an offset flag byte-replays both formats.64 new WAVs pass PCM/rate/layout/
headroom and result hashes. Ruff lint/format and diff checks pass. All jobs
terminal; no Cargo/ProductCheck/runtime/default/roadmap promotion.

## Audio-layer modulation and source-information audit — 2026-09-05

Implemented `run --bridge-kind audio-modulation` in the existing bridge script.
The pinned Diffusers Flux implementation has six dual-stream blocks with an
audio `norm1.linear`1024->6144 mixer. A shared11->42->6144 MLP adds a zero-initialized
residual to these six audio mixers. Its264696 parameters closely match the text
bridge265728. Text/pool/duration inputs and context mixers are not directly
changed; joint attention can still propagate changes to context activations.
No single-stream mixer, pretrained weight or runtime engine component is trained.
This is a bounded placement experiment inspired by the preceding PAVAS reading,
not its architecture, video estimator, per-block gates or full-backbone training.

Training hooks remain installed through backward because non-reentrant checkpoint
recomputation re-enters the mixers. Tests verify zero exactness, positive CFG
branch-only injection, nonzero gradients through recomputation, cleanup after
errors, fixed layout, kind-aware checkpoint reload and unchanged text-bridge
conditioning. Every actual training update checks that all bridge parameters
received gradients and no frozen generator parameter did.

`pouring-tango-audio-modulation-2026-09-05`: same native posterior SHA and200
sampled rows as the original bridge, same optimizer/seed53,200 steps/full-horizon
loss/BF16 training/FP32 inference. Bridge SHA256
`0072e9dbb352904b9f21a7da8dd548b877092af6b7e17571c5eb1c5a4f057d8c`.
First/last20 loss1.6726/1.5958. Zero-adapter PCM, upstream loss, frozen full-model
hash and glass/wood/rain adapter-off regressions all pass. Standalone `render`
detects the bridge kind and byte-replays both mono/native glass10/seed2718.

`physical_sound_pouring_bridge_compare.py --source SOURCE --model MODEL
--previous PREVIOUS_DEVELOPMENT --output NEW` makes the repeated comparison
runnable: verify source/split/previous WAV identities, generate eight source-free
clips, reuse matched-seed prior/base and other-object swaps, compute metrics and
write a preview/tag input. Whole first source-order records of excluded
containers18/30, first/middle,seeds314/2718; no favourable crop selection.
Four real controls enter metrics/preview only. Failed generation keeps status
failed rather than inventing completed output. The real run is
`pouring-tango-audio-modulation-development-2026-09-05`.

| Eight development pairs | Base | Previous centered | New matched | New swapped |
|---|---:|---:|---:|---:|
| Legacy spectrum RMSE,dB | 18.915 | 18.845 | 15.167 | 14.340 |
| Legacy level-centered shape RMSE,dB | 7.756 | 7.687 | 11.838 | 12.170 |
| Relative-power shape256 RMSE,dB | 7.764 | 7.692 | 12.459 | 12.793 |

New shape beats base/previous0/8 and swapped4/8 under the legacy diagnostic.
Raw/RMS0.005 AST and harder13-prompt CLAP still recognize water8/8 on both
hypothetical and development generations. Reject this trained instance as an
improvement; retain previous centered model. No layer/width/seed/epoch sweep.
[Listen: real/previous/new/swapped](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-audio-modulation-development-2026-09-05/comparison.wav>),
36.64s,glass18 then PET30,middle,seed2718,published gains.

The adjacent-layer audit found a metric confound: the legacy `p.metrics` first
uses a fixed absolute dB floor, so subtracting the spectrum mean does not make
its shape metric gain invariant. Scaling the same real clip by0.1/0.01 yields
false shape distances0.273/3.616dB. The new separately named
`relative_power_shape_rmse_db` uses normalized power and a relative floor;
the same test is below4e-15dB. Legacy fields and old reports are not overwritten.
`relative-power-audit.json` binds the completed result hash and re-scores its
unchanged WAVs. The rejection survives this correction, also with32 coarse
bands (base7.174/previous7.157/new11.892dB). No EQ, gain or audio changes.

Before another adapter fit, `pouring-control-information-2026-09-05` audits ALL279
TRAIN crops/93 recordings/13 objects using absolute relative-power profiles256.
This differs from the earlier relative first-to-middle timbre probe: it compares
leaving an entire recording out with leaving its entire object out. Ridge0.01
uses the same eleven controls, centering/intercept fitted inside each fold;
fixed shuffle53 and nearest-control baselines, no hyperparameter selection.
All phases of a held recording/object are excluded together.

| Mean per-crop shape RMSE,dB | Global mean | Ridge | Shuffled ridge | Nearest |
|---|---:|---:|---:|---:|
| Leave recording out | 4.390 | 3.630 | 4.427 | 4.468 |
| Leave object out | 4.601 | 5.351 | 4.971 | 5.746 |

Equal-object averaging preserves the direction: ridge/global4.157/4.786 for
record exclusion versus5.476/4.898 for object exclusion. Controls carry useful
within-object information, but this simple predictor does not transfer. This
does NOT prove insufficient metadata or impossibility of a nonlinear model.
[Source comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-control-information-2026-09-05/comparison.wav>),
13.74s: first TRAIN object middle crop/another recording of that object/nearest
other object. These are real recordings, not generated sounds or admission data.

Bounded research revisited the [source paper,v2,2025-01-13](https://arxiv.org/html/2411.11222v2):
it solves inverse pitch/physical-property estimation, not reference-free waveform
generation for new containers. Its success does not validate our forward model
or rescue the previously rejected pitch teacher. The verified publisher TRAIN
CSV has195 recordings/18 objects, only the same four glass/plastic materials;
`physical_parameters` is empty. Broadening existing filters is not a large new
object/material corpus. Our93 TRAIN records have settings ws-kitchen52,
vgg-mrcr37,vgg-coffee3,ws-room1. Objects7,31,40 cross settings; others do not.
These are setting labels, not measured microphones, room responses or forces.

Next discriminator uses those already-TRAIN cross-setting objects to compare
within-object/across-setting variation against between-object variation at
matched phase. Distinguish acquisition nuisance from missing transferable object
coverage before choosing normalization/data acquisition or another model fit.
Keep reference-free generation; do not add a reference recording or an object-ID
shortcut to make the task easier. No protected/author-test roles reopened.

147 focused tests pass, including runnable comparator success/failure, source
order/roles and gain invariance. Ruff lint/format and diff checks pass.64 new
WAVs pass PCM/rate/layout/headroom/result hashes. All jobs terminal. No Cargo/
ProductCheck/runtime/default/roadmap promotion; full multi-event goal stays open.

## Recording-setting factorization — 2026-09-05

The completed TRAIN-only `pouring-setting-nuisance-2026-09-05/result.json`
compares objects7/31/40 across published setting labels. Same-phase pairs use
another recording nearest in eleven-control space, never nearest audio.
Median relative-power shape distance is3.731dB within object/setting,
5.743 within object/across settings, and4.853 across objects/within setting.
Cross-setting distance exceeds within-setting distance for155/162 paired
anchors; repeated recordings/overlapping phases are not independent trials.
Three anchors have no within-setting alternative and remain explicitly missing.
Duration/control matching is imperfect; date, pouring behavior and acquisition
are confounded. This is association, not measured room/microphone causality.

Fixed ridge0.01 on all279 TRAIN crops, excluding whole recordings/objects:

| Mean relative-power shape RMSE,dB | Global | Controls | Setting | Both |
|---|---:|---:|---:|---:|
| Leave recording out | 4.390 | 3.630 | 3.867 | 3.435 |
| Leave object out | 4.601 | 5.351 | 4.062 | 5.152 |

Equal-object averaging preserves both conclusions: settings matter, but adding
them does not establish physical-parameter transfer. No dev fitting or reopened
author tests. [Listen to the source comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-setting-nuisance-2026-09-05/comparison.wav>):
41.22s, glass7/PET31/PP40; each anchor/same-object-same-setting/
same-object-other-setting. These ten verified WAVs are real recordings, not new
neural output. This diagnostic-only checkpoint owes a learned audible candidate.

The next bounded discriminator is a centered text bridge plus a separate
four-setting2048-dimensional learned offset, not an object-ID embedding or an
inference recording. Keep the200-step seed53 training draw trace, frozen
generator and previous post-hoc-centered control. Hypothetical previews fix
ws-kitchen (largest TRAIN group) before scoring. Disclosed development uses
each target's setting, held FIXED when physical controls are swapped. Also
disable the physical branch: improved style matching alone is not success.
The glass18 development setting ws-room has only one TRAIN recording; report
that limitation, not a calibrated acoustic environment or independent test.

Implemented `--bridge-kind setting-text --center-training` in the existing
bridge runner:273920 trainable parameters, centered physical11→128→2048 branch
plus4×2048 setting offsets. Vocabulary comes only from verified TRAIN metadata;
setting offsets initialize at zero without advancing the physical branch RNG.
Scoped setting state survives backward and is removed on errors. No new
backbone, source-at-inference, object-ID input, runtime or product contract.

`pouring-tango-setting-bridge-2026-09-05` completed200 updates. Posterior SHA
and all200 sampled crop indices match the prior centered-training control.
Setting draws: ws-kitchen107,vgg-mrcr90,vgg-coffee2,ws-room1. First/last20 mean
loss1.6741/1.6055. Bridge SHA256
`999885c06d7f1822d4d70eb24abe24e2e11805adbc95ca84ba870cd320ffd699`;
frozen TRAIN offset SHA256
`c495d42f2185d947ed4e0162be9442ff1c8186ed59568f491ac818c318db9a49`.
Zero-adapter PCM, upstream loss, frozen full-model identity and adapter-off
glass/wood/rain PCM regressions pass. Standalone `render --setting ws-kitchen
--controls 0.5 0.35 0.35 0.5 0.1 1 0 0 0 1 0 --seed 2718`
in `pouring-tango-setting-cli-2026-09-05` byte-replays both
formats, loading the centering artifact automatically. `--style-only` disables
the physical branch; this is an ablation, not an alternative trained baseline.

`pouring-tango-setting-development-2026-09-05` uses the unchanged disclosed
first records of excluded containers18/30,first/middle,seeds314/2718.24 new
generations: matched physical controls, other-object controls at the SAME target
setting, and physical-disabled. Tests catch accidentally swapping setting along
with the object. Real audio enters only metrics/preview. Actual style-only WAVs
are byte-identical across phases/physical controls for each setting/seed.

| Relative-power shape RMSE,dB | Base | Previous retained | Matched | Swapped | Style-only |
|---|---:|---:|---:|---:|---:|
| All8 development pairs | 7.764 | 7.692 | 8.025 | 7.820 | 7.817 |
| Glass18,4 pairs | 8.840 | 8.745 | 8.655 | 9.040 | 8.930 |
| PET30,4 pairs | 6.687 | 6.639 | 7.395 | 6.599 | 6.704 |

Matched beats previous2/8, swapped4/8 and style-only2/8. Glass has a small
average gain while PET worsens; no general physical-transfer improvement.
Legacy shape8.019 also loses to previous7.687. Reject this instance as a
replacement and retain the post-hoc-centered bridge. This falsifies improvement
for this fixed trial, not the general possibility of separating recording
conditions or learning nonlinear physical controls.

Raw/RMS0.005 AST and the unchanged harder13-prompt CLAP recognize water8/8
for hypothetical matched clips, and8/8 for EACH development variant. These
coarse checks are insufficient to certify object properties. CPU AST used the
same pinned weights and prior checked CPU/CUDA correspondence; the existing mel
filter warning remains. No score/threshold/prompt tuning or historical rewrites.

[Listen: real/previous/new/wrong-physical/style-only](</home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-setting-development-2026-09-05/comparison.wav>),
45.8s,glass18 thenPET30,middle,seed2718,published gains. The new model renders
without target audio.92 new WAVs across training/development/CLI, plus the10
earlier setting-audit source WAVs, pass PCM/rate/layout/headroom/hash checks.
114 focused tests pass (92 pouring,22 pretrained-generator), Ruff lint/format
and diff checks pass. All jobs terminal; no runtime/default/roadmap promotion.

Next: stop this small-corpus adapter family rather than sweep settings/centering/
capacity/seeds/epochs. Run a bounded internet source search for substantially
broader object and material-pair coverage with defensible physical descriptors
and recording-context information. Acquire the smallest permitted training
slice and audible examples before building more support tooling; exclude
protected roles and unknown-redistribution assets from distribution. Then choose
one learned experiment from actual available coverage. Do not invent missing
force/geometry, ask the user to record objects, or narrow the full goal to water.

## EPIC-SOUNDS material-pair source slice — 2026-09-05

The bounded source search changes event family from pouring to impacts rather
than repeating the13-container adapter family. Competing explanations remain:
too little transferable object coverage; unmodelled acquisition conditions;
and evaluators that cannot actually distinguish physical conditions. The last
trial did not fix transfer by separating setting labels. This checkpoint tests
availability of broader material-pair data and the evaluator's real positives.

[FillImpact](https://arxiv.org/html/2607.17773v1),20 July2026, describes88 objects,
different strikers and fill states; no official downloadable release was found
on the inspected paper page or bounded project search. It remains unavailable,
not a reason to wait. SonicGauss/NISR reuse ObjectFolder lineage; no fresh real
independence or reopening of protected roles is claimed. No payload was acquired
from these leads. Existing source-admission research is not repeated as a gate
on the following independent report-only source audition.

[EPIC-SOUNDS publisher repository](https://github.com/epic-kitchens/epic-sounds-annotations)
and [paper v2](https://arxiv.org/html/2302.00646v2),§IV-A/B andV-C, provide collision
labels involving two materials. The authors describe audio annotation followed
by visual verification and exclusion/correction of ambiguous labels; this is
not measured physical ground truth. Participant/video IDs are recording context,
NOT stable physical-object IDs. Material pairs are unordered: metal/glass does
not identify the striker, a metal alloy, object dimensions, force or velocity.
The corpus also contains action classes such as pouring, scraping and sliding.

The actual publisher TRAIN CSV at revision
`57a922f0d352e9429f1ef8a37eee21758dd3a33c` has60055 annotations,495 videos and32
participants. Training counts include metal/glass472,metal/wood1451,
wood/glass29,metal/plastic1285,metal/ceramic1303,plastic/wood123. This long tail
must not be disguised as equally broad physical coverage. Validation/test
annotations or audio were not acquired. Foundation pretraining overlap is unknown.

`physical_sound_epic_slice.py --output EXTERNAL_NEW_DIRECTORY` downloads only
TRAIN annotations, publisher README and video-path/checksum metadata. The latter
comes from the [official downloader repository](https://github.com/epic-kitchens/epic-kitchens-download-scripts)
at `4f11fb2b579833f360c3c7bb917bf1e24a9787b5`; its code was inspected for endpoint
construction, NOT executed. TLS verification stays on. Some original MP4s are
multi-gigabyte, so ffmpeg seeks to selected intervals through HTTPS partial access.
Whole-video publisher MD5s are recorded but explicitly NOT verified. Local
metadata/decoded/published bytes have SHA256 receipts; these are not evidence
that the entire remote video was downloaded or admitted.

Selection is fixed before waveform access: six classes in the count order above,
first source-order0.25–3s annotation per participant with no overlap with any
other TRAIN annotation; four participants per class. No audio-score selection.
24 clips span19 videos and5 distinct participants overall. Unannotated background
sounds, multiple impacts within one event and imperfect source labels remain
possible. Missing physical fields are explicitly null, not filled from sound.

`epic-material-pairs-source-2026-09-05` stopped before audio access: the checksum
CSV mixes versions55/100 with `errata`, so its version column is string-valued.
The matcher incorrectly compared integers. Preserve that failed result; the
fixed string-compatible matcher is covered by a test and keeps identical selected
annotations. `epic-material-pairs-source-fixed-2026-09-05` completed acquisition.
Original AAC audio is decoded/downmixed/resampled to24kHz monoPCM16; each output
length exactly matches the annotation sample interval. All24 require no gain
attenuation and contain no full-scale decoded samples.49 decoded/published/
comparison WAVs and four metadata files pass identities/layout/length checks.

[Listen to real material-pair recordings](</home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-material-pairs-source-fixed-2026-09-05/comparison.wav>),
37.297s: four clips each of metal/glass,metal/wood,wood/glass,metal/plastic,
metal/ceramic,plastic/wood, separated by0.5s silence.25.297s of actual source
audio. These are internet recordings, NOT new neural generations. The publisher
uses CC-BY-NC4.0; retain attribution, changes and source terms. Local noncommercial
research only; no commercial/distributed engine asset or model promotion.

`clap-material-pairs.json` uses six fixed prompts, one per unordered material
pair, with the existing frozen CLAP. Exact pair recognition is4/24 (uniform
six-way chance expectation4/24). The confusion matrix and all scores are retained;
no prompt/threshold retuning or removal of failed source positives. This small
diagnostic does not prove material perception impossible, but does not qualify
CLAP as the sole material-pair validator or training reward. The source paper
also reports difficulty with bi-material audio recognition; it does not establish
that our frozen zero-shot classifier is competent for this task.

`alignment-check.json` rules out gross timestamp drift for one counterfactual
control: sequential decode from video start versus direct seek forP04_09_4.
54504 samples agree except four1-LSB differences; correlation0.999999999991.
The separately retained12.669s prefix is a diagnostic source decode, not another
selected training clip or exact whole-corpus alignment proof. No source audio
was modified to improve evaluator scores.

Three focused tests pass (selection/overlap/coverage,55/100 mixed-version source
mapping, TLS/crop/headroom/null-physics behavior); Ruff lint/format and diff checks
pass. All jobs terminal. No Cargo/ProductCheck/runtime/roadmap changes. This is
one source/diagnostic-only checkpoint; the next checkpoint must produce a learned
material-conditioned impact candidate and playable controls. Expand TRAIN support
as needed within the stated metadata limits, preserve missing axes, and do not
equate participant separation with proven new-object identity or CLAP scores with
physical calibration. The full multi-event goal remains open.

## EPIC factorized material bridge and corrected event evaluation — 2026-09-05

New learned, reference-free impact WAVs are available. This is a bounded research
candidate, not a reliable material simulator or replacement for the liked glass
demo. Source terms remain EPIC-SOUNDS CC-BY-NC4, noncommercial research only;
TangoFlux attribution: Powered by Stability AI. No runtime/default/roadmap change.

`physical_sound_epic_pair_bridge.py` reuses the frozen TangoFlux bridge hook but
encodes the two unordered materials as five compositional factors (metal, glass,
wood, plastic, ceramic). A zero-initialized 5x2048 linear map has10240 trainable
parameters; differentiable TRAIN-mean centering excludes a common style offset.
Only positive text/pooled conditioning changes, not duration or negative CFG.
The learned material corrections compose algebraically; that construction alone
does NOT prove perceptually correct composition. The shared pouring bridge gains
only a configurable input width; its default eleven-control behavior is unchanged.

Training uses45 unique annotations: first three non-overlapping0.25–3s TRAIN
clips per participant P01/P02/P03 for each of five classes. All wood/glass clips
are excluded. Thirty additional intervals were acquired with the existing TLS
partial-video decoder; previous clips were reused after exact identity checks.
Development uses all seven original source-slice clips from P04/P07, including
two wood/glass clips. No author validation/test data, target audio at inference,
or assumed object identities/striker/force/geometry. Participant separation does
not establish new physical objects or foundation-pretraining independence.

`epic-pair-bridge-2026-09-05` completed200 AdamW steps,1e-4,weight decay.01,
gradient norm1,BF16 training, native45x645x64 posterior mean/std. The unchanged
full-horizon flow objective uses the fixed generic prompt “The sound of two
objects colliding.” Native source gains were retained. First/last20 mean losses
are1.721769/1.651011, not quality scores. Posterior SHA256:
`97d61ef478ffa34784f4bafdbc1aa8eb19e86d1e31962d7cf1514cac6644095f`.
Bridge plus frozen centering-offset SHA256:
`c20a4b1216d95e24fb97f428962a7e27b74136375ad5b85abda93ee96d13a348`.
Full generator tensor digest before/after:
`23ee7758b8b637389e8d0378484b79c326a362f1069d760c3344df641224b258`.
Zero bridge exactly reproduces base PCM; cached loss exactly matches upstream
(.3487389684); adapter-off water/rain PCM remains byte-exact. These are integrity
checks, not physical-quality admission.

**Evaluation correction:** the first run accidentally restored prefix-only
rendering, repeating the previously documented late-event failure. All seed314
prefixes were about -99.64dBFS codec noise, while seed2718 was audible. Neither
nonzero PCM, relative waveform change nor the gain-invariant spectral metric
qualifies such prefixes as impacts. Original reports and WAVs remain untouched,
but their5/14 baseline wins and any material-improvement interpretation are
superseded. No new training, prompt/seed/duration/solver sweep or amplification
was performed to repair this evaluation.

`epic-pair-event-matched-2026-09-05` rerenders the saved weights: base plus all six
material pairs at seeds314/2718,3s duration condition,50 Euler steps,CFG4.5,FP32.
Each retains the raw3s prefix, the complete29.9537415s decode and a3s onset window.
The existing `tango.event_window` rule is identical for base and candidates:
10ms RMS blocks, threshold max(-50dBFS,.1 peak RMS),50ms preroll. It operates on
the published full-horizon PCM; gains never increase. Seed314 starts at11.12s
(plastic/wood11.28s), seed2718 at1.47s. All windows contain active audio; all also
flag activity after the window, so these are bounded attack windows, NOT entire
isolated-event recordings. Full horizons remain available for inspection.
No-event output explicitly fails while retaining its raw evidence. Single-render
CLI now defaults to this policy; `--prefix-diagnostic` is explicitly not quality
evidence. Continuous water/rain diagnostics retain their separate window policy.

[Listen: six pairs, real/base/learned/wrong pair](</home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-pair-event-matched-2026-09-05/comparison.wav>),
71.552s, fixedseed2718. Pair order: metal/glass,metal/wood,wood/glass,metal/plastic,
metal/ceramic,plastic/wood. Each group is real recording, frozen base, learned
matched pair, next-pair control; published gains, no listening-based selection.
[Separate learned wood/glass, seed314](</home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-pair-event-matched-2026-09-05/pair2-314.wav>)
also demonstrates generation of the pair withheld from this bridge's training;
it does not establish that it sounds physically correct.

Gain-invariant relative-power spectral shape compares each full3s generated
window with the entire original development clip, never a reference-dependent
prefix crop. Across14 comparisons: base5.975546,matched5.962152,next-pair5.964974dB.
Matched beats base8/14 and next-pair5/14. The four wood/glass comparisons improve
over both (7.028456 vs base7.122528,next-pair7.130032), but this weak control does
not survive a strong interpretation: `all-material-ranking.json` compares all
six generated material conditions against every reference. Correct pair ranks
first only2/14; held wood/glass ranks1,2,2,3. These correlated small-sample spectral
diagnostics do not demonstrate reliable material control or perceptual quality.

`tags-raw-cpu.json` retains raw frozen AST scores for14 generated and7 real clips,
plus silence/noise/tone controls. Seed314 generations lead with Breaking;
seed2718 leads with Door, including the unadapted baseline. Real recordings have
diverse kitchen/impact/background tags. AST is a coarse event diagnostic, not
a pair validator; no expected material pass threshold is introduced. Its known
mel-filter warning persists. The earlier real-positive CLAP4/24 failure remains
binding: do not use it as the sole material reward or select prompts from this set.

Reproduce from the saved checkpoint, without source recordings at generation:

```sh
lab/.venv/bin/python lab/scripts/physical_sound_epic_pair_bridge.py \
  --model /home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-pair-bridge-2026-09-05 \
  --pair 'wood / glass collision' --seed 314 --output NEW_EXTERNAL_DIRECTORY
```

`--event-matrix` instead of `--pair/--seed` re-evaluates all14 cases and compares
the saved seven development references, without fitting. The separate
`epic-pair-event-cli-2026-09-05` command byte-replays event, prefix and full horizon
in both PCM formats.98 corrected-evaluation/CLI WAVs pass SHA/layout/rate/headroom
checks. The earlier108 training/prefix WAVs, two prefix CLI WAVs and60 acquired
decoded/published source WAVs remain external and unchanged.

53 focused tests pass; Ruff lint/format and diff checks pass. Tests cover
compositional controls, zero/centered gradients, source
roles/overlaps/PCM identity, checkpoint validation, retained late-event recovery,
noise rejection and identical event policy across all14 matrix cases. No Cargo
or ProductCheck is required for this external Python lab path. The learned-media
checkpoint clears the preceding source-only outcome debt; the full goal remains
active. Next, discriminate insufficient material information in these kitchen
labels from weak conditioning transfer using broader TRAIN participant coverage
and a participant-separated real-positive/wrong-label control. Do not repeat a
bridge capacity/epoch/seed sweep or treat tiny spectral gains as physical learning.
The next learned trial must retain a playable comparison and the all-pair control.

## Broader EPIC information discriminator and learned data counterfactual — 2026-09-05

`physical_sound_epic_information.py` tests two competing explanations for the
weak material bridge: insufficient transferable information in the source labels
versus failure to transfer available information into generation. The
[EPIC-SOUNDS paper v2](https://arxiv.org/html/2302.00646v2),§V-C, documents that
recognizing both materials from audio can be ambiguous; this motivates a local
discriminator, not a conclusion that material learning is impossible.

`epic-information-2026-09-05` selects from the same pinned publisher TRAIN CSV,
excluding P04/P07 and intervals overlapping ANY other TRAIN annotation. A sorted
prefix-maximum interval check matches the earlier census, including nested
overlaps. Duration remains0.25–3s. First12 source-order participants per class,
first3 eligible clips each; no audio-score selection. Eligible totals are
metal/glass144,metal/wood225,wood/glass7,metal/plastic306,metal/ceramic401,
plastic/wood35. The actual selected counts are30/34/7/35/33/22:161 clips,19
participants,84 videos. Rare wood/glass cannot be represented as broad support.

114 additional intervals were acquired through the existing verified-TLS partial
decoder;47 cached intervals were identity-checked and reused. All161 retain source
gain1.0; no normalization of published audio.230 newly written WAVs and324 total
referenced decoded/published/audition WAVs pass SHA/rate/layout/sample-bound checks.
Whole remote MP4 MD5 remains unverified. CC-BY-NC4 local research only; missing
striker roles, object IDs, dimensions and force remain unknown. No author val/test
access, claim of new-object independence or dataset/model distribution promotion.

[Source preview](</home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-information-2026-09-05/preview.wav>),
12.443s: first two clips per pair outside P01/P02/P03, fixed source order, not
necessarily two distinct participants. [All161 sources](</home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-information-2026-09-05/all-sources.wav>)
are237.350125s including0.5s separators. These previews are real audio, not neural.

Four fixed representations are evaluated with leave-one-participant-out linear
logistic probes: frozen AST pooled768 features, the same at diagnostic RMS.005,
gain-invariant256-bin spectral shape, and duration/logRMS/logpeak only. Each fold
fits scaling on TRAIN rows only; C1,balanced class weights,max1000 iterations,
no hyperparameter or feature selection. All folds retain all six TRAIN classes.
AST/Tango pretraining overlap is unknown; AudioSet features are not independent
physical ground truth. Source labels enter the probe, not the AST encoder.

| Representation | Macro recall | Overall accuracy | Wood/glass recall |
| --- | ---: | ---: | ---: |
| Frozen AST, raw |24.30%|27.95%|0/7|
| Frozen AST, RMS.005 |20.11%|23.60%|0/7|
| Relative spectrum |25.47%|26.71%|1/7|
| Duration/gain only |21.45%|18.63%|3/7|

Thirty-two fixed within-participant label permutations preserve context/class
frequency associations. Raw AST null macro recall averages16.53%,maximum21.72%,
versus observed24.30%; exploratory Monte Carlo p1/33. This is modest evidence of
some transferable class association, NOT qualified pair recognition. The raw AST
per-class recalls are16.67/26.47/0/31.43/39.39/31.82%. Duration/gain cues and weak
rare-class recall prohibit using this probe as the sole material validator/reward.
Do not tune the probe from these predictions or claim all kitchen labels useless.

Feature cache SHA256:
`b28749db56f41cdd2318aaa1eaa062e309330f4053466d3c5ec234cda4ae4089`.
Versions: NumPy2.5.2,SciPy1.18.0,scikit-learn1.9.0,Torch2.13.0+cu130,
Transformers4.44.2; existing local environment, no package installation. AST uses
the existing pinned revision, CUDA FP32, raw and level-controlled inputs; known
mel-filter warning retained. The new command requires those lab dependencies and
cached AST weights. The source acquisition and probe complete in one command:

```sh
lab/.venv/bin/python lab/scripts/physical_sound_epic_information.py \
  --source SOURCE_SLICE --bridge ORIGINAL_PAIR_RUN --output NEW_EXTERNAL_DIRECTORY
```

The next primary artifact was produced in the same checkpoint, not deferred to
another support-only turn. `epic-expanded-pair-bridge-2026-09-05` trains the same
10240-parameter bridge on154 rows from19 participants, excluding ALL seven
wood/glass examples and retaining the original seven P04/P07 development clips.
`physical_sound_epic_pair_bridge.py --source SOURCE_SLICE --expanded INFORMATION_RUN
--output NEW` verifies source identity, annotation correspondence, overlaps,
participant/pair roles and PCM before fitting. No source redownload is needed.

Architecture, generic prompt,200 updates, optimizer/learning rate, clipping,
BF16 training, sampling seed and frozen generator are unchanged. Data frequency
and the corresponding TRAIN centering distribution change with the corpus:
this is not isolated proof of a participant-count effect. At the fixed compute
budget113/154 rows are sampled, so the larger set has fewer repeated exposures;
do not conclude that more data can never help. First/last20 loss1.725786/1.657109
is not a sound-quality measure. No force/geometry values are invented.
Posterior SHA256:
`b9b5e1539c2ccd7e01b96b76845e78ee445e27eb94373afe8434c35ac52d4f46`.
Bridge plus offset SHA256:
`f78ecf30dbb949fcfbe24255257133bbf7878ddad4373f5bb8d45767547289bd`.
Frozen generator digest remains23ee7758…1224b258; exact full digest is above.
Zero, upstream-loss, full-model and adapter-off water/rain checks pass exactly.

All14 generated cases use the retained-full-horizon/matched-onset policy.
Frozen-base PCM is byte-exact to the earlier event matrix. New mean shape
error5.945757 vs previous5.962152/base5.975546dB; wins8/14 vs previous,7/14 vs
base and7/14 vs next-pair control. All-six-material ranking improves only2/14
to3/14. Held wood/glass worsens7.028456 to7.080724dB; beats previous2/4, with ranks
4,2,4,1. Neither the small mean gain nor increased data coverage establishes
reliable material control. Do not replace the liked demo or claim physical learning.

[Listen: real/previous/expanded model](</home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-expanded-pair-compare-2026-09-05/comparison.wav>),
50.552s, all six pairs in source order, fixedseed2718 and published gains. The full
[real/base/new/wrong-pair comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-expanded-pair-bridge-2026-09-05/comparison.wav>)
and every full horizon remain available. The source-free standalone command with
`--model .../epic-expanded-pair-bridge-2026-09-05 --pair 'wood / glass collision'
--seed 314 --output NEW` retains event, prefix and full horizon independently.

Standalone event/prefix/full PCM is byte-exact in both formats.117 learned,
comparison and CLI WAVs pass identity/rate/layout/headroom checks. A first audit
flagged missing preview receipts, not changed audio; exact regeneration from the
declared source verified those seven WAVs, and future comparison reports now
retain their receipts explicitly. Old completed reports remain unchanged.
Raw AST diagnostics retain Breaking/Smash forseed314 and Door for2718, including
base; no qualified material acceptance.69 focused tests, Ruff lint/format and
diff/link checks pass. All jobs terminal; generated media/data/weights stay external.

Next: two coherent material-bridge cycles leave the main failure unresolved.
Do not run a third data-size/capacity/epoch/seed variant. Use a bounded research
and counterfactual cycle: measure matched/wrong/disabled conditioning on paired
TRAIN posterior/noise, separating attack, decay and silent tail. Test whether
the optimization signal rewards material-dependent active audio or mostly another
part of the horizon. The earlier water signal audit is not evidence for impacts.
Keep these WAVs as inspectable controls; the next learned change needs a causal
discriminator and playable same-policy comparison, not another protocol package.
Full goal active; no runtime, roadmap or ProductCheck promotion.

## Impact-region signal and fixed material-text counterfactual — 2026-09-05

The bounded research cycle tests three explanations before another fit: padded
silence dominates the learned change; BF16 evaluation hides material differences;
or the custom conditioning route suppresses useful pretrained semantics.
[TangoFlux v1](https://arxiv.org/html/2412.21037v1),§§2.1–2.4, describes the frozen
audio VAE, text/duration conditions and flow objective. Its preference-optimization
discussion also cautions that ranking margins alone do not ensure better winning
audio. No CRPO/CLAP reward is imported: our real-positive material evaluator failed.
[PyTorch numerical accuracy](https://docs.pytorch.org/docs/2.14/notes/numerical_accuracy.html),
updated1 June2026, motivates measuring precision effects rather than assuming
identical floating-point outcomes. The runtime remains the existing2.13.0+cu130;
the2.13 documentation URL was unavailable, no library/backend upgrade was made.

`physical_sound_epic_signal.py --model EXPANDED_PAIR_RUN --output NEW` audits
ALL154 TRAIN rows with the same saved native posterior cache and bridge. CPU
seed10000+row draws one posterior sample and noise shared across sigmas.2/.5/.8
and base plus all six material conditions.462 row/time cases use FP32; the first
source-order row per fitted pair provides15 matched BF16 precision controls.
No training, gradients, parameter updates, new source acquisition or protected
roles. Small-sample precision controls are not compared to the different full
FP32 population. Error accumulation is FP64 for both modes.

Source-relative onset uses10ms RMS blocks and.1 peak threshold, not an audibility
gate. Latent frame centers are mapped with2048/44100s stride. Regions partition
all645 frames: before onset, first100ms of activity, remaining annotated body,
250ms post-annotation transition and remaining zero-padded support. The body may
contain repeated strikes, not a pure physical decay. Encoder receptive fields and
posterior noise limit exact waveform attribution. These are operational masks,
not measured acoustic modes or gradients of physical parameters.

| FP32 velocity MSE | Base | Matched bridge | Mean wrong condition |
| --- | ---: | ---: | ---: |
| Attack,462 cases |1.401358181|1.400833318|1.402299973|
| Body,459 nonempty cases |1.347309249|1.346801777|1.348154655|
| Padded tail,462 |1.594076463|1.594083199|1.594090533|
| Full horizon,462 |1.582913589|1.582902174|1.582957486|

Matched controls beat ALL five wrong alternatives only73/462 attack cases and
77/459 body cases. Full-horizon improvement is.000011415; weighted attack/body
contributions are+.000001723/+.000015952, while padded tail contributes
−.000006519. All region contributions reconstruct the full gain, accounting for
empty regions with the full-case denominator. The silent-tail-learning hypothesis
is contradicted: the achieved improvement is in active audio, but very small and
not reliably material-specific. This does NOT justify a new attack-weight sweep.

On the SAME15 precision controls, attack correct-condition rank1 is2/15 in both
FP32 and BF16; body3/15 in both. Tiny aggregate signs do change: full base-minus-
matched is−.000034884 in FP32 and+.000015941 in BF16. Thus numeric precision matters
when quoting tiny gains, but FP32 does not rescue material discrimination here.
This is a saved-offset forward diagnostic, not proof about every training gradient
or a reason to run a precision-only training sweep.

`epic-impact-signal-2026-09-05` completed477 numeric cases and20 individual media
entries, then failed only when the borrowed pouring preview helper required
seed2718. Actual diagnostic seeds are10000+row; they were NOT relabelled or retried.
The fixed diagnostic concatenator keeps all20 entries and verifies published PCM.
`epic-impact-signal-summary-2026-09-05` verifies exact matrix coverage and retains
the source-report hash, corrected weighted summary and assembled preview. The
failed report remains unchanged; no neural inference was repeated to repair it.

[Reference-aided diagnostic preview](</home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-impact-signal-summary-2026-09-05/comparison.wav>),
61.04625s: first TRAIN clip per fitted pair, source/posterior/base-one-step/matched-
one-step at sigma.5. All decoded horizons/prefixes/event windows are retained.
The one-step estimate uses noisy target audio: this is NOT reference-free output
or a new trained generator. No gain increase is applied to any published clip.

The next discriminator produces source-free audio through the unmodified text
path. `physical_sound_epic_pair_bridge.py --model EXPANDED_PAIR_RUN --event-matrix
--text-control --output NEW` disables the bridge and uses exactly the six strings
already frozen for the earlier real-positive CLAP diagnostic. Template:
“The sound of an object made of MATERIAL colliding with an object made of MATERIAL.”
No wording/seed/CFG/duration search or training. All14 cases retain full horizons
and matched onset windows, with the same3s/50steps/CFG4.5 and seeds314/2718.
Generic base controls remain byte-exact to the learned experiment.

`epic-material-text-control-2026-09-05` completes. Spectral shape mean7.693776 versus
learned bridge5.945757/base5.975546; direct text wins1/14 against either. Wood/glass
is8.770939 versus bridge7.080724,0/4 wins; all-pair rank1 remains3/14. This does NOT
prove worse perceptual/material fidelity: coarse AST changes from largely
Breaking/Door to more differentiated categories. Metal/glass seed2718 leads with
Glass(.341),Chink/clink(.274); ceramic prompts also produce clink categories,
while several other pairs remain inconsistent. No score threshold is introduced.
Unmatched kitchen recordings lack object geometry/striker identity, so their
spectral distance is not a sole physical-quality judge. The supervised real-positive
probe and zero-shot CLAP remain unqualified for pair acceptance.

[Source-free fixed-text comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-material-text-control-2026-09-05/comparison.wav>),
71.552s, six groups real/generic-base/material-text/wrong-material-text,seed2718.
[Metal/glass text control](</home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-material-text-control-2026-09-05/pair0-2718.wav>)
is separately playable, without an inference reference. The executed report's
legacy `training_pair` field describes the unused reference bridge, not foundation
pretraining; `bridge_applied=false` is decisive. Future text reports set it null
and name their conditioning explicitly. Pretraining overlap remains unknown.

193 new diagnostic/control WAVs pass identity/rate/layout/headroom checks.
74 focused tests pass; Ruff lint/format and diff/local-link checks pass. Tests
cover temporal partitioning, weighted gain accounting with empty regions, all-
wrong ranking, preserved diagnostic seeds/PCM and disabled-bridge text routing.
No demo/runtime/roadmap/default changes, no model promotion. This checkpoint
includes source-free neural WAVs but no new learned weights. The full goal remains
active. Retain these controls, not another nominal win from a tiny loss difference.

Next: stop the present EPIC material-only bridge/prompt variant family. Return to
the already acquired controlled friction grid, whose commanded speed/load and
separate measured force/position are known. Before any new learned decoder, test
whether the frozen audio codec preserves these paired physical responses, with
clean/main/machine-channel controls and one disclosed shared gain. Use TRAIN roles
and already-open development honestly; do not repeat the rejected stationary-PSD
network/PCA/epoch/width or crossed-velocity family. The next checkpoint must include
an audible codec comparison and reference-free controlled examples, not a new
source inventory. This addresses a missing physical axis, not a smaller goal.

## Controlled friction survives a frozen codec (2026-09-05)

[Codec comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-codec-controls-2026-09-05/comparison.wav>)
is36seconds: wood/steel/glass,20/60mm/s,.5/1N; each condition plays
real -> posterior mean -> posterior sample314. This is reference-aided
reconstruction, NOT a new generator. Full native decodes are retained separately.

`physical_sound_texture_codec.py` uses the existing verified Figshare v5 grid.
The existing loader reads the already-disclosed60 records, but codec execution
uses only the original24 TRAIN scans: repeat0,20/30/50/60mm/s,three surfaces,
two commanded loads, fixed urethane-rubber probe and direction0. Commanded
conditions and sensor-derived force/speed remain separate report fields.
All three channels are checked: published clean mono, raw main microphone0,
raw machine-reference microphone1. The latter two are not scene stereo.

Two fixed arms use input gain1 and17.374337221633088. The latter is
`min(100,.5/max_full_TRAIN_peak)` across all24 records and allthree channels;
peak.028778076171875. No per-recording/channel normalization or output gain.
The full recording is dual-mono encoded in FP32, right-padded to stride2048.
Published PCM24 metrics use the exact original central position-aligned.75s,
without onset selection, time shifts or loudness matching. Native full outputs
are FLOAT WAVs. The36s playlist uses only clean/shared-gain cases. A separate
three-second zero-input control checks codec noise, not acoustic quality.

Frozen Oobleck weight SHA256 is
`d73619a1d1e1dc48e606632931ffce440b4959ce2a4ed5a3522c3bb573b103be`, from the
existing pinned TangoFlux snapshot. No full text generator, optimizer, new
weights or external downloads. Source/model attribution and license notices
are copied into the external result. Runtime Torch2.13.0+cu130,31.66seconds.

Paired deltas hold surface/load fixed for18 adjacent-speed comparisons, and
surface/speed fixed for12 load comparisons. These overlapping pairs are not
independent samples or a calibrated acceptance test.

| Channel / input arm / posterior | Speed direction preserved | Load direction preserved | Speed delta MAE,dB | Load delta MAE,dB |
|---|---:|---:|---:|---:|
| Clean / native / mean | 15/18 | 10/12 | .675 | .365 |
| Clean / native / sample314 | 14/18 | 9/12 | 1.077 | .724 |
| Clean / shared / mean | 16/18 | 12/12 | .327 | .101 |
| Clean / shared / sample314 | 17/18 | 12/12 | .348 | .184 |
| Main / shared / mean | 18/18 | 12/12 | .091 | .096 |
| Machine / shared / mean | 18/18 | 12/12 | .095 | .094 |

Clean mean absolute level error decreases.924 -> .251dB; centered-spectrum
RMSE2.647 ->1.943dB. The two remaining mean speed-sign errors are steel1N,
50->60mm/s (real+.102,decoded-.114dB), and glass1N (+.552,-.029dB).
Sample314 fixes the latter, not the former. These failures remain; no threshold
or seed is chosen to erase them. Zero-input mean/sample levels are-99.958/
-99.862dBFS, below clean source levels. The native-level failure is not complete
erasure into codec noise. Shared input calibration improves amplitude-response
preservation; it is not proof of perceptual realism or machinery removal.
Indeed, machine-reference responses survive equally well. No material-quality
judge or physical-ground-truth claim is obtained from channel discrimination.

Separately, existing rank4 neural weights now have a source-free `--control-demo`
path. [Speed comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-source-free-controls-2026-09-05/speed-comparison.wav>)
and [load comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-source-free-controls-2026-09-05/load-comparison.wav>)
are20.25seconds each: wood/steel/glass, then25/40/55mm/s at.75N, or.5/.75/1N
at40mm/s. Fifteen standalone two-second WAVs use seed314 and one playbackgain100.
No source audio is loaded. Their levels increase along both requested controls
for allthree known surfaces.25/55mm/s and.75N are unrecorded grid conditions,
so this illustrates interpolation, NOT verified realism there. No new training,
no evidence of arbitrary second materials/geometry, no claim this uses Oobleck.

Reproduce with `physical_sound_texture_codec.py --corpus GRID/result.json
--output NEW_EXTERNAL`, or `physical_sound_texture_fit.py --render-model
texture-conditional-rank4-2026-09-05 --control-demo --output NEW_EXTERNAL`.
20 focused tests pass, including exact TRAIN pairs, aligned crops, shared gain,
PCM/headroom, signed response deltas and source-free controls with audio reads
forbidden.743 WAVs pass SHA/layout/rate/finiteness/headroom checks (FLOAT full
decodes are not PCM-normalized). Ruff/diff/local links pass; both jobs terminal.
No Cargo/ProductCheck/engine audition, runtime/demo/roadmap changes or promotion.

Decision: codec representation is feasible for a bounded conditional-generator
trial; it is not the current main blocker. Next implement one small physical-
conditioned latent sequence generator using this frozen codec and the same24
TRAIN scans/shared gain. Compare actual source-free PCM on the already-open
40mm/s/repeat1 development against the existing spectrum/interpolation baselines
and the reference-aided codec ceiling. Keep temporal and paired-response errors,
wrong-condition controls and all failures. Do not resume stationary-PSD/PCA,
EPIC prompt/bridge or precision/attack-weight sweeps. A new latent model must
produce playable WAVs in the same checkpoint; no separate planning package.

## First physical-conditioned latent friction generator (2026-09-05)

[New learned friction comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-conditional-flow-legacy-roles-2026-09-05/comparison.wav>)
is29.791seconds: wood/steel/glass,.5/1N,40mm/s,repeat0,seed314; each group plays
real -> new flow -> old rank4 spectrum -> interpolation -> reference-aided codec.
[Standalone rubber-on-glass generation](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-flow-glass-standalone-2026-09-05/generated.wav>)
is.743seconds. Unlike the earlier codec diagnostic, this branch consumes only
model weights, known surface, commanded speed/load and seed, not source audio.

`physical_sound_texture_flow.py` trains a162432-parameter conditional sequence
model:64-channel input/output convolutions, four residual convolution blocks,
physical/time conditioning. The method follows the linear noise-to-data path
and velocity-regression construction in
[Lipman et al., Flow Matching, v2,8February2023, sections3–4](https://arxiv.org/html/2210.02747v2).
This is a small friction implementation, not a reproduction of the paper's
ImageNet results or evidence of universal physical synthesis.

The same24 TRAIN scans are used, repeat0 at20/30/50/60mm/s.40mm/s and repeat1
remain excluded from fitting; all60 were already disclosed development. The
frozen encoder reads full clean recordings. Each target is32 native64-dimensional
latent frames around central travel, with8frames of real context on each side
of a16-frame scored core. Alignment is snapped to the2048-sample codec grid;
the exact source core is32768samples/.74303855seconds, not the old.75s window.
No context padding enters these targets. Full1.486s generated decodes are kept.
Encoder/decoder weights and shared input gain17.374337221633088 are unchanged.

TRAIN-only per-channel center/scale include posterior variance; there is no
per-recording amplitude normalization.2000full-batch AdamW updates,lr.001,
weight decay.0001,gradient cap1,seed23,FP32. Each update draws fresh posterior
and Gaussian noise samples. Generation uses64 explicit-midpoint steps and
common initial noise across conditions, seeds314/2718; neither source posterior
nor waveform enters generation. No best-checkpoint/seed/capacity selection.
Loss-window mean decreases1.69960 ->1.21357. Training+preparation18.77seconds,
whole run35.31seconds. Model653728bytes,SHA256
`86526a73fb488eb913be9f561b18bf484aa52b36b5b388d7f707b9c8f746ebbe`.

The initial `texture-conditional-flow-2026-09-05` failed before training because
old rank4 metadata predates `heldout_speed_mm_s`. It remains untouched. The
corrected preflight verifies the original complete60-row report, its model
identity and every TRAIN/development role instead of inventing a default.
Only the corrected run trained. It records the verified baseline-report hash.

All comparison PCM uses a common playback gain5.755615234375 after the codec
input gain: equivalent to100times original digital amplitude. No individual
gain, waveform time alignment, onset selection or equalization. This is not
calibrated SPL. Spectrum metrics use the shared22.05kHz mono band; the original
native-level/temporal summaries compare44.1kHz flow against a22.05kHz baseline.
Therefore `bandmatched-analysis.json` separately verifies the unchanged PCM
hashes and recomputes level/envelope/paired responses in the shared band. The
initial report is preserved; no model or output was refitted after this check.

| Disclosed evaluation | Flow | Old rank4 | Interpolation |
|---|---:|---:|---:|
|40mm/s spectrum RMSE,dB |2.41463 |2.33733 |2.41908 |
|repeat1 other speeds spectrum RMSE,dB |2.20725 |2.16442 |2.11538 |
|40mm/s shared-band level MAE,dB |.49008 |1.03316 |1.34150 |
|repeat1 other speeds shared-band level MAE,dB |.45036 |.76100 |.61314 |
|40mm/s shared-band envelope ACF MAE |.28228 |.29697 |.30619 |
|repeat1 other speeds shared-band envelope ACF MAE |.27865 |.28961 |.28387 |

Envelope ACF compares four25–100ms lags; it is a short-clip diagnostic, not a
perceptual judge. Envelope-CV error is worse for flow than the old spectrum
model (.02606 vs.02209 at40mm/s). Native-band claims of a large4dB level or
large temporal advantage are not the fair comparison. DC is not the main
explanation: mean source DC energy fraction.00443 versus flow.01819.
The new model has more audible bandwidth; isolate that from learned temporal
or physical-response improvements.

Across the36 development recordings×2seeds, correct material and correct speed
each beat their wrong-condition controls in72/72 spectrum comparisons; correct
load in61/72. Wrong material rotates the surface ID, wrong speed selects the
opposite20/60 endpoint, wrong load swaps.5/1N. These are known-condition controls,
not unseen surfaces or independent naturalness validation. Repeated recordings
receive the same generation for the same condition/seed;72 is not72 independent
generated cases. Against the old spectrum model flow wins only25/72 overall
spectrum comparisons (8/24 at40mm/s,17/48 at repeat1 other speeds).

Shared-band repeat1 paired speed direction: flow40/48,old42/48,interpolation42/48;
delta MAE.570/.442/.520dB. Load direction30/30all,delta MAE.402/.365/.294dB.
Thus correct-vs-wrong association does NOT establish better fine physical slopes.
Reference-aided cropped-latent codec spectrum RMSE1.98281 over all60 versus
flow's2.21–2.41 development range leaves a generation gap, not codec failure.

The standalone CLI and batched evaluation use identical model/seed/conditions
but are not byte-exact: max PCM difference.00025618,RMS.00004954 in the glass
control. Keep both artifacts; no cross-batch bit-replay claim. Focused tests
forbid source reads during sampling and allow only newly generated PCM reads
during the standalone path. Reproduce with `physical_sound_texture_flow.py
--corpus GRID/result.json --baseline OLD_RANK4 --output NEW_EXTERNAL`, or
`--model FLOW_RUN --texture 74 --speed 40 --force 0.5 --seed 314 --output NEW_EXTERNAL`.

30 focused tests pass; Ruff lint/format,diff/local links and543 new WAV
identity/layout/rate/finiteness/headroom checks pass. All processes terminal.
Source/codec notices stay external with media/weights. No engine audition,
Cargo/ProductCheck,runtime/demo/roadmap changes or promotion. This is a real
new source-free learned model with partially successful physical conditioning,
not achievement of the full multi-material/multi-event goal.

Next: retain the model as an experimental control, not a replacement. Move from
stationary.743s cores to a complete start/slide/stop event using the existing
full recordings and force/position traces. First verify source clock alignment
and whether measured speed/load explain onset/offset on TRAIN; do not invent
sensor/audio synchronization. If supported, extend this same generator with
time-varying physical conditions and publish a complete event in that checkpoint.
Do not respond to the mixed spectrum scores with an epoch/width/seed sweep or
erase the shared-band countercheck. Other probe materials, shape/size, impacts,
rolling/destruction,water/rain and engine integration remain separate missing
parts of the original full goal, not removed success criteria.

## Complete friction event and background counterexample (2026-09-05)

[Source-free rubber/glass event](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-full-event-glass-standalone-2026-09-05/generated.wav>)
is3.15seconds: requested start.3s,90mm travel at40mm/s,.5N,.1s ramps,
stop2.65s,then.5s tail. No recording or sensor file is read by generation.
Shared-band median levels are-52.21dBFS before motion,-42.71 during sliding,
-50.13 after stopping. These show modulation, not verified perceptual realism.
[Glass source/timed/constant/gated/codec comparison](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-full-event-evaluation-2026-09-05/glass-comparison.wav>)
is14.277seconds. The full six-condition comparison is85.707seconds.

The [source paper v4,6November2025](https://arxiv.org/html/2407.16206v4)
describes PC timestamps, an audio-start timestamp and subsequent synchronization.
It does not establish microsecond acoustic timing. Its stated80mm path conflicts
with90mm displacement in all24 inspected TRAIN CSVs; actual CSV trajectories are
used. The inspected pinned repository exposes noise processing, not acquisition/
clock-alignment code. No downloaded code is executed and no per-recording time
shift is fitted. Source audio/position endpoint differences on TRAIN range
-.01424 to+.00696s, not a complete latency measurement.

`physical_sound_texture_event.py` extends the SAME162432-parameter flow: the
existing conditioning projection and residual blocks accept physical features
per latent frame, and the existing integrator accepts bounded sequence lengths.
Constant-feature broadcasting has a focused equivalence check; existing
stationary inference remains unchanged. No new foundation or codec weights.
Input speed is a100ms centered displacement average from position, not literal
instantaneous velocity; force is interpolated from its recorded timestamps.
The32..256-frame domain allows0..80mm/s transitions and0..1.5N sensor values;
steady commands remain20..60mm/s and.5/1N. These bounds are not evidence for
new steady operating points, arbitrary materials or changing probe geometry.

At zero lag, shared-band20ms log-RMS correlates with the speed proxy at mean.848
over24 TRAIN records. Diagnostic best lags on a±.2s/20ms grid fall within±40ms;
all applied shifts remain zero. A preliminary full-band/Savitzky–Golay probe
placed quiet glass20mm/s at the+.2s search edge: the lag estimator depends on
signal representation and is not a synchronization calibration.

Starting from the previous stationary checkpoint,2000 additional updates cycle
first/random/last32 valid frames of each original TRAIN recording. Explicitly
padded frames are excluded; encoder boundary receptive fields still exist.
Parent TRAIN normalization is retained.40mm/s and repeat1 remain excluded from
training.22.30seconds training+preparation,loss1.51005→1.29508; checkpoint SHA256
`b31611a2dd0ec55c2a160b077cb1014ac23138601ad2e56e5786c8ee92e8cb56`.
No source-free output is amplitude-gated after decoding in the **timed** branch.
One playback gain2.8778076171875 after inputgain17.374337221633088 equals50times
original amplitude, reduced globally to retain full-recording transient headroom.

Training completed in `texture-full-event-2026-09-05`; evaluation JSON failed
on a NumPy boolean. Its original result remains stale/running, but the process
is terminal and `failure.json` records the failure after training. Saved weights
and19 first-evaluation WAVs are intact. `--evaluate-model` reruns evaluation only
in `texture-full-event-evaluation-2026-09-05`, with zero training updates; all19
overlapping WAVs replay exactly. The inherited original metadata `context_frames`
field is unused by full-event rendering; the corrected evaluator names effective
full output and future metadata records context0 explicitly.

|40mm/s development,12 records×2seeds | Timed model | Constant parent | Velocity gate | Gate + TRAIN background |
|---|---:|---:|---:|---:|
|Shared-band log-envelope MAE,dB |1.49235 |2.2730 |7.80295 |1.32827 |
|Source half-rise onset error,s |.0100 |.2283 |.0108 |not scored |
|Uncensored source offset error,s |.0633 |.0783 |.0208 |not scored |

Timing uses one source-defined half-rise detector and three consecutive20ms
bins; weak rises and truncated offsets remain unavailable. These errors are
not physical clock accuracy. Repeat1 other-speed timed envelope error is1.44410;
onset.00375s,offset.14105s on38/48 uncensored cases. A five-frame delayed-input
control is worse than correct timing in all72 development envelope comparisons,
as is the constant parent. Neither fact is sufficient quality admission.

The silent gate has exact zeros before movement, so a log-level metric heavily
penalizes it against a noisy recording. The decisive **posthoc** countercheck
adds one TRAIN-only background scalar, RMS.0029587963 in published22.05kHz mono
units, estimated from the first100ms of all24 TRAIN recordings. The same Gaussian
noise rule (seed+17) and gain apply to every condition; no individual fitting,
generator update or seed selection. It uses fixed existing PCM. See
[real/timed/silent-gate/gate-plus-background](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-full-event-noise-floor-2026-09-05/comparison.wav>).
Timed beats this stronger baseline only1/24 at40mm/s and29/48 at other repeat1
speeds. The earlier72/72 wins over silence-gating do NOT prove superiority.
Background resemblance is not necessarily desirable physical source synthesis;
do not promote the noise-added baseline merely because it wins this metric.

Reproduce training with `physical_sound_texture_event.py --corpus GRID/result.json
--parent STATIONARY_FLOW --output NEW_EXTERNAL`; evaluation only adds
`--evaluate-model EVENT_MODEL`. Source-free inference uses `--model EVENT_MODEL
--texture 74 --speed 40 --force 0.5 --seed 314 --output NEW_EXTERNAL`.
`noise_floor_countercheck(EVALUATION/result.json, NEW_EXTERNAL)` reproduces the
fixed-PCM counterexample without ML inference.36 focused tests,Ruff/diff/local
links and1284 WAV checks pass, including19 preserved originals. All jobs terminal;
no engine audition,Cargo/ProductCheck, runtime/demo/roadmap changes or promotion.

Decision: time-conditioned generation is implemented and audible, but not
admitted as better than the stronger control. Do not spend another epoch/width/
loss sweep on matching this apparatus background. The next physical capability
is transfer across surfaces rather than more examples of the same three IDs:
inspect the already-downloaded texture table and published friction coefficients,
identify which genuine descriptors could replace surface one-hot labels, and
run one bounded new-surface generator trial with playable output. Keep the
shared-band, timing and background controls. Do not invent material composition,
geometry or coefficients from names, relabel new surfaces as pristine protected
evidence, or drop other required event families from the full goal.

## New-surface descriptor transfer — 2026-09-05

Primary artifact: [source-free rubber on frosted glass](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-surface-glass-standalone-2026-09-05/generated.wav>),
3.15s,40mm/s,.5N,90mm. No reference audio, sensor file or surface ID is accepted
by the standalone renderer; it takes category, two coefficients and motion.
The independent CLI produces byte-identical PCM to the in-run requested profile.
Compare [oak](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-surface-transfer-2026-09-05/surface-4-comparison.wav>),
[steel](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-surface-transfer-2026-09-05/surface-66-comparison.wav>)
and [frosted glass](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-surface-transfer-2026-09-05/surface-76-comparison.wav>).
Each plays real / descriptor model / equally trained category-only model /
previous timed model using the category representative, with250ms gaps.
These are friction events, not glass impacts or perceived-quality admission.

The selected Figshare v5 grid now contains180 recordings from9 surfaces,
722 source members,242 reused from the earlier grid. All member bytes were
verified against local SHA256 and source ZIP CRC, including reused members.
Whole15.2GB archive MD5 remains **unverified**; only selected ranges downloaded.
External root `cluster-surface-transfer-grid-2026-09-05` retains the original XLSX.
Its SHA256 is `6cd7dfc06e61852d49a2256d7fe8d44d222f3619f48fe9a4031f677d1b0eb4ff`.
The optional spreadsheet skill path was unavailable; direct bounded XML reads
use cached cell values, ignore malformed font styles and reject formulas/entities.
No source spreadsheet repair or generated spreadsheet was made.

The table contains genuine static/dynamic friction measurements, **not** density,
elastic modulus, surface height spectra or object geometry. In the
[primary paper, v4,6Nov2025](https://arxiv.org/html/2407.16206v4),
coefficients are measured with a rubber sheet pulled at10mm/min. Audio uses a
cylindrical rubber probe at20–60mm/s. Therefore these values are candidate
cross-surface features, not calibrated coefficients at audio scan speeds.

| Category | TRAIN surfaces | New-surface development |
|---|---|---|
| Wood |0 Nyatoh,2 Elm |4 Oak |
| Metals |65 Stainless steel,67 Cast iron |66 Steel |
| Glass |74 Float glass,77 Glass(haze) |76 Frosted glass |

Every recording of4/66/76 is excluded from fitting. TRAIN is48 repeat0 scans
at20/30/50/60mm/s on the six TRAIN surfaces.40mm/s and repeat1 stay development.
All surfaces/roles are disclosed development, not pristine final evidence.
The parent has only original24 TRAIN IDs on0/65/74; foundation-codec pretraining
overlap is unknown. No held-surface audio determines initialization, normalization,
training loss, checkpoint selection or the standalone motion profile.

`physical_sound_texture_surface.py` expands the existing time-conditioned flow
from5 to7 physical features: category one-hot3, speed, force, static and dynamic
friction. Coefficients use fixed scaling `(mu-.5)/.25`. Only128 weights are added
to the first context layer; they start at zero and preserve parent predictions
exactly in the warm-start test. Both arms use162560 trainable parameters,
the same parent,48 TRAIN cases,seed23,2000 updates and first/random/last32-frame
windows. The category-only arm zeros both coefficient features during fit/render.
The codec and parent latent normalization are unchanged. The original common
inputgain17.374337221633088/playback2.8778076171875 are retained; no per-case
normalization/clipping or fitted onset shifts. Full decodes remain available.

The entire run completed in245.46s, including preparation and both fits51.93s.
External root `texture-surface-transfer-2026-09-05` has both checkpoints and
the [complete result](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-surface-transfer-2026-09-05/result.json>).
Descriptor checkpoint SHA256 `6d36e47c055512eef37a9b182c19b31f2db246c0c03d33a00c21822a4672a8e0`;
category-only SHA256 `c259a9bca63c6acac813c9da51c10578a37390d38cca1de170962eb01dfc8cd9`.
Evaluation covers60 new-surface scans plus36 old-surface development anchors,
each with two fixed seeds and four variants:768 generated cases. A wrong-control
variant substitutes the farthest same-category TRAIN coefficients, chosen without
audio scoring. No candidate or seed is selected from these results.

| New surface,40 comparisons each | Moving shape RMSE: descriptor / category-only / parent,dB | Descriptor wins vs category-only | Full envelope MAE: descriptor / category-only,dB |
|---|---|---:|---|
| Oak |2.185 /2.183 /2.249 |21/40 |1.704 /1.671 |
| Steel |2.544 /2.589 /2.877 |36/40 |1.969 /2.005 |
| Frosted glass |2.619 /2.518 /2.849 |3/40 |1.702 /1.795 |

Moving metrics use the same measured central.75s interval and shared22.05kHz
band for every variant. They separate shape from absolute level and avoid
mistaking quiet-apparatus resemblance for moving-contact quality. Full-envelope
metrics retain the previous background-confound warning. Steel descriptor
absolute level error1.369 vs category-only1.427/parent2.322dB; frosted glass.793
vs.771/1.236. Wrong coefficients actually improve glass shape to2.563dB; correct
coefficients beat them only16/40 on shape. Thus sensitivity to an input does
**not** establish physically correct transfer. On old-surface anchors descriptor
shape2.305 vs parent2.330 improves slightly, but full-envelope error1.593 vs1.460
and moving-level error.830 vs.743 worsen. Do not replace the existing model/demo.

Reproduction: `physical_sound_texture_surface.py --source GRID/result.json
--parent TIMED_MODEL --output NEW_EXTERNAL` fits both arms; adding
`--evaluate-model SURFACE_MODEL` evaluates saved weights with zero updates.
Source-free: `--model SURFACE_MODEL --category Glass --static 0.3970170073501798
--dynamic 0.3827100881663895 --speed 40 --force 0.5 --seed 314 --output NEW_EXTERNAL`.
The source acquisition command is `physical_sound_texture_probe.py --training-grid
--surfaces 0 2 4 65 66 67 74 76 77 --cached OLD_GRID/result.json --output NEW_GRID`.

Decision: source-free new-surface generation works mechanically and is audible;
measured coefficients give a modest steel improvement, not reliable multi-surface
physical control. The glass counterexample survives the matched category-only
control. Before another neural fit, compare direct TRAIN-spectrum interpolation
in coefficient space against category averaging and observed new-surface spectra.
This cheap discriminator separates an inadequate input description from a learned
mapping failure. If coefficients themselves do not transfer the acoustic shape,
seek additional published surface/contact evidence rather than more epochs/width.
No new protected split, runtime consumer, roadmap promotion or universal-quality
claim; the full impacts/friction/rolling/destruction/water/rain goal stays active.
Verification:44 focused unit tests,Ruff format/check,diff and direct links passed;
all1640 output WAVs pass SHA/finite/layout/headroom checks, and722 source members
pass SHA/CRC checks. Both training/evaluation and standalone jobs are terminal.
No Cargo/ProductCheck or engine audition was run for this isolated Python lab.

## Coefficient versus generator discriminator — 2026-09-05

The promised discriminator is complete, without new neural training. Listen to
[oak](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-coefficient-discriminator-2026-09-05/surface-4-comparison.wav>),
[steel](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-coefficient-discriminator-2026-09-05/surface-66-comparison.wav>),
[frosted glass](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-coefficient-discriminator-2026-09-05/surface-76-comparison.wav>)
or [all three](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-coefficient-discriminator-2026-09-05/comparison.wav>).
Each group contains six central750ms segments: real, neural descriptor,
coefficient interpolation, category mean, **target-aided** best mixture,
**target-aided** self spectrum. The final two are diagnostic controls, NOT
reference-free generators. This probe does not replace the full-event WAVs.

`physical_sound_texture_interpolation.py` retains the exact48-TRAIN and60
new-surface development rows. For each same-category TRAIN surface it interpolates
log spectra along commanded speed, with40mm/s bracketed by TRAIN30/50mm/s.
The requested static/dynamic coefficient pair is projected onto the segment
between the two same-category TRAIN pairs; no target waveform enters this path.
That convex weight blends the two log spectra. Category mean uses weight.5.
The source-aided oracle minimizes shape error along this same two-spectrum span;
the self-spectrum control measures stochastic rendering error with the target
spectrum supplied. Both are explicitly labelled in every generated receipt.

All candidate spectra use the same stationary random-phase renderer and two
fixed seeds314/2718; playbackgain50 is common. Neural PCM is taken from the saved,
hash-verified previous trial and cropped to the identical source-defined central
interval; no weights or source-free request are refitted. Every comparison WAV
is band-limited through the same22.05→44.1→22.05kHz publication/check path. This
additional resampling modestly changes absolute-level errors relative to the
previous full-event report; the within-probe comparisons below are consistent.
It does not restore or judge temporal structure, contact transients or timbre
above11.025kHz. Each source/weight/result retains its earlier disclosed roles.

| New surface,40 PCM comparisons each | Coefficient shape RMSE,dB | Category mean | Neural descriptor | Target-aided best mix | Target-aided self spectrum |
|---|---:|---:|---:|---:|---:|
| Oak |1.543 |1.551 |2.185 |1.535 |.931 |
| Steel |2.044 |2.226 |2.544 |1.997 |.965 |
| Frosted glass |2.271 |2.230 |2.619 |2.186 |.955 |

The coefficient PCM beats the neural descriptor on shape in40/40 oak,39/40 steel
and40/40 glass cases. This is **not overall quality dominance**: absolute-level
error coefficient/neural is.454/1.584dB oak,2.584/1.393 steel and1.579/.747 glass.
The stationary model also has no learned onset/offset behavior. Do not replace
the timed generator with stationary noise merely because a spectrum score wins.

Before stochastic rendering, coefficient/category-mean/best-mix shape errors
are1.314/1.326/1.306dB oak,1.889/2.067/1.853 steel and2.076/2.033/1.999 glass.
Coefficients beat equal mixing on12/20,20/20 and3/20 source cases respectively.
Coefficient weights oak.673,steel.742,glass.370 differ from mean target-aided
optimal weights.618,.858,.611. Two real repeats differ by1.284,1.333,1.267dB
on the same central spectral measure; this is a variability reference, not a
universal lower bound or protected final test. All60 source comparisons and720
generated/cropped cases are retained in
[result.json](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-coefficient-discriminator-2026-09-05/result.json>).

Falsifiable hypotheses and outcome:

- **Only neural mapping is wrong:** contradicted as an exclusive explanation.
  Glass coefficients also underperform equal mixing without a neural model;
  even the target-aided two-spectrum span leaves a substantial residual.
- **Only the two coefficients lack information:** also insufficient. Fixed
  TRAIN-only interpolation already predicts much of the shape that the neural
  output loses. This does not prove that an arbitrary nonlinear coefficient
  mapping could never transfer; only the tested linear span is bounded here.
- **Only stochastic spectrum estimation explains the gap:** not supported by
  self-spectrum rendering error around.95dB versus neural2.19–2.62dB. These
  components are not independent additive errors, so do not subtract them as
  a decomposition or treat the oracle as deployable generation.

A bounded literature check found a relevant counterexample to treating friction
as a unique topography label: [Hsia et al., Phys. Rev. Research3,043204,
21Dec2021](https://journals.aps.org/prresearch/abstract/10.1103/PhysRevResearch.3.043204)
reports a fourfold change in real contact area with only a modest coefficient
change in its studied interface. This supports caution, not a diagnosis of the
Cluster apparatus or an acoustic law. The Cluster paper's10mm/min versus audio
20–60mm/s measurement distinction remains relevant. Two additional publisher/
author pages could not be opened; no claims rely on their search snippets.

Next useful experiment: retain the timed neural event and its level response,
but constrain/calibrate its moving spectral shape with the TRAIN-only predicted
spectrum. First do one reversible **source-free hybrid** full-event countercheck,
not an epoch/width sweep. Keep target recordings out of its inference path and
evaluate all same cases, including onset/offset, quiet background, absolute level
and old-surface regressions. A better central PSD score alone cannot admit it.
Richer surface/contact evidence remains a later input issue, not an excuse to
ignore the demonstrated generator-side loss. Full multi-event goal stays open.

Reproduce with `physical_sound_texture_interpolation.py --source GRID/result.json
--neural-result SURFACE_FLOW/result.json --output NEW_EXTERNAL`.47 focused unit
tests,Ruff format/check,diff and direct artifact links pass; all784 output WAVs
pass SHA/layout/finite/headroom checks. No new neural weights, GPU job, engine
audition, Cargo/ProductCheck, runtime/demo or roadmap changes. The job is terminal.

## Full-event source-free hybrid and DC correction — 2026-09-05

Listen to [standalone neural / corrected glass](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-glass-standalone-2026-09-05/comparison.wav>)
or [the corrected sound alone](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-glass-standalone-2026-09-05/hybrid.wav>).
This is a3.15s rubber/frosted-glass sliding request,40mm/s,.5N,90mm,seed314,
generated from the saved neural model and a TRAIN-only spectrum bank. No source
audio, reference recording, sensor file or surface ID enters standalone inference.
Full neural and filtered decodes are retained. The neural control is byte-identical
to the prior standalone source-free WAV. The new corrected PCM SHA256 is
`eec12325dc70339767846b0f4b3d1bb58f4cd89127ff05637071bee28497fd1b`.

`physical_sound_texture_hybrid.py` cooks only the48 TRAIN central log spectra
and their six surfaces' coefficients into a98576-byte safetensors bank. Its SHA256
is `5c440edd763373c788411c481c48c6690c1093780b6fd5609d1957bf34cd6e20`.
The bank is identical in all three evaluations; no new neural weights are fitted.
Source validation may read the complete disclosed grid, but only declared TRAIN
waveforms supply bank values. The bank loader checks its hash,finite bounds,
48 exact TRAIN IDs/conditions and six-surface membership. Inference interpolates
speed,normal force and same-category coefficient position without a target.

The filter measures a750ms window of its **own generated** moving sound, selected
from input velocity rather than acoustic score. It applies a513-tap FIR to align
spectral shape with the predicted TRAIN spectrum: fixed±12dB pre-normalization
limit and two-bin Gaussian smoothing. Above13kHz the requested unnormalized
response is unity; there is no new high-band truth claim. Intrinsic filter gain
preserves the predicted generated moving power, not a real recording's loudness.
This is an algorithmic EQ gain, **not per-file playback/headroom normalization**.
All publications retain the common inputgain17.374337221633088/playback2.8778076171875
and strict.98 PCM headroom. No clipping or gain change is used to rescue an output.
The fixed256-sample FIR group delay is compensated explicitly and256 extra tail
samples retained. This is not a fitted sensor/acoustic clock alignment.

The first global filter (`texture-hybrid-shape-2026-09-05`) improved shape but
also modified quiet background and shifted some onsets. A posthoc motion-only
countercheck (`texture-hybrid-motion-2026-09-05`) applies the correction through
`clip(input_velocity/commanded_speed,0,1)`, leaving the floating-point neural
signal unchanged when requested velocity is zero. Publication can differ by
one PCM24 LSB from an old float32-multiplied control; do not claim byte-exact idle
PCM. The full requested neural control itself does replay byte-exactly.

Both early variants exposed a calculation defect, not evidence for another
filter-parameter sweep. In the standalone moving window, the raw mean was
-.003092 with RMS.007780; the first corrected mean was-.007041 with RMS.010088.
DC accounted for15.8% versus48.7% of power, creating+2.2565dB RMS drift. Yet the
detrended PSD integrals were almost unchanged,5.054e-5 versus5.077e-5. The
[SciPy1.18 Welch documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.signal.welch.html)
confirms the default constant detrending. Our shape estimator therefore could
not authorize the DC gain2.2766 that its FIR happened to produce.

The bounded discriminator distinguished PSD/DC bookkeeping from insufficient
training or a change in the source clock. The correction is algebraic: project
the FIR onto unit DC gain using a normalized513-sample Hann kernel. It is not a
new fitted threshold, test-selected tap length or target-aided loudness match.
Revision `dc-preserving-fir-v2` has unit DC gain within1.12e-15 on all192 cases;
the standalone calibration-window level change becomes-.0919dB. Across192
cases that change has median-.0138dB and range[-.1943,+.3077]dB: preservation is
approximate, not an exact per-window normalization claim. A nonzero-mean test now
covers the bug missed by the original zero-mean-noise test.

Final [evaluation](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-2026-09-05/result.json>)
uses the same60 unseen-surface and36 original-surface development scans, two
fixed seeds, and three variants: neural / corrected / corrected category mean.
View [oak](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-2026-09-05/surface-4-comparison.wav>),
[steel](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-2026-09-05/surface-66-comparison.wav>)
and [glass](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-2026-09-05/surface-76-comparison.wav>),
each real / neural / corrected / corrected category mean. These comparisons
reuse the original source-defined central interval and shared22.05kHz metrics;
full-envelope/onset/offset measurements remain unchanged in definition.

| Surface | Shape RMSE neural→hybrid,dB | Shape wins | Moving-level absolute error neural→hybrid,dB | Envelope MAE neural→hybrid,dB |
|---|---|---:|---|---|
| Oak |2.185→1.774 |40/40 |1.507→1.492 |1.704→1.715 |
| Steel |2.544→2.273 |38/40 |1.369→1.381 |1.969→1.966 |
| Frosted glass |2.619→2.329 |40/40 |.793→.815 |1.702→1.718 |
| Original-surface development |2.305→1.804 |72/72 |.830→.825 |1.593→1.604 |

Mean onset errors are unchanged for oak/old anchors,steel.0065→.0070s and
glass.0075→.0070s. Mean uncensored offset errors remain unchanged, including
glass.2344s; this existing large error has not been solved. These are detector
errors,not calibrated physical clock accuracy. Glass category-mean shape2.302
still beats the coefficient-conditioned2.329: reliable coefficient transfer
and broad material fidelity remain unproven. Small envelope regressions remain;
no universal perceptual or product-admission claim is made.

All three evaluations and three standalone jobs are terminal; their3780 WAVs
pass SHA/layout/finite/headroom checks.55 focused unit tests,Ruff format/check,
diff and direct artifact links pass. Old failed-design outputs and executed
scripts are preserved,not overwritten. No new training, engine audition,
Cargo/ProductCheck, runtime/demo or roadmap promotion occurred.

Reproduce current evaluation with `physical_sound_texture_hybrid.py --source
GRID/result.json --neural-result SURFACE_FLOW/result.json --motion-only --output
NEW_EXTERNAL`. Source-free inference: `--model SURFACE_FLOW --bank BANK_DIRECTORY
--category Glass --static 0.3970170073501798 --dynamic 0.3827100881663895 --speed 40
--force 0.5 --seed 314 --motion-only --output NEW_EXTERNAL`. Historical pre-DC
variants use their preserved `executed-script.py`; the current default includes
the DC correction. Dropping `--motion-only` runs a current DC-preserving global
filter, not a replay of the historical first variant.

Decision: retain the hybrid as a useful report-only spectral baseline, not a
replacement for the authored demo or proof of physically realistic friction.
Stop EQ/tap/smoothing/gating sweeps. The DC finding also questions prior RMS
interpretations: next compare existing real,reference-aided codec and source-free
generator WAVs with explicit DC and AC components before another neural fit.
Distinguish decoder-conditioned bias from learned latent drift and time-dependent
energy errors; silence controls alone do not isolate those hypotheses. Use the
cached codec audit first, not new protocols/data acquisition. This is a validator/
generator causal check, not permission to redefine quality around an easier metric.
The full multi-event, both-materials, geometry and natural-process goal stays open.

## DC, training-window inference and real-repeat counterchecks (2026-09-05)

Three completed external diagnostics preserve the full physical-sound goal and
report-only boundary. No new training, data acquisition, runtime/demo replacement
or quality admission occurred. The primary new source-free artifact is
[global / windowed glass](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-windowed-field-2026-09-05/requested-comparison.wav>):
3.15s per arm,40mm/s,.5N,90mm,seed314,frosted-glass coefficients, no source audio
or sensor trace at inference. Full decodes are retained. Windowed PCM SHA256:
`cc9cd259a354161c028e4d4f337aad19db05816d4a5d44b31c7a1e2435f79e40`.

### Competing explanations and outcomes

1. **DC bias accounts for the remaining envelope error.** The
   [cached audit](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-dc-causal-audit-2026-09-05/result.json>)
   checks432 codec entries plus36 old-surface development recordings×2 neural
   seeds×5 arms. Real and codec314 repeat across seeds, not independent evidence.
   Mean central DC power fractions are real.01050,reference-aided codec.04688,
   timed generator.04506,current generator.05483,hybrid.05518. Codec alone already
   introduces extra DC. Removing one whole-event mean changes current-generator
   envelope MAE only1.59325→1.59136dB; codec.76240→.65647dB. Per20ms block-mean
   removal gives1.44672 versus.43805dB, but also removes slow physical components:
   it is a diagnostic, not an improved acceptance metric or published filter.
   **DC-only explanation rejected; exact origin of codec-conditioned DC unproven.**
2. **Training32-frame crops versus full-length inference is the dominant cause.**
   The trained network's GroupNorm couples time positions. A fixed perturbation
   outside the21-frame convolutional receptive field changes the remote global
   vector field by RMS.1345775; the windowed wrapper changes it by0. This is direct
   local Torch2.13 evidence, not proof of audio-quality causation. The wrapper
   evaluates32-frame windows with10-frame convolutional halos at every midpoint
   solver stage, averaging overlapping valid velocity fields. Initial full noise,
   weights,64 solver steps and one full codec decode remain unchanged; no audio
   chunks are stitched or postfiltered. At32frames it is exactly the old sampler.
   [Evaluation](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-windowed-field-2026-09-05/result.json>)
   covers24 records:20/60mm/s,.5/1N,new4/66/76 repeat0 and old0/65/74 repeat1,
   two seeds, two arms. All48 global PCM controls reproduce previous outputs
   byte-exactly. **No useful improvement; retain old generator, no window sweep.**
3. **Paired metrics mostly penalize natural randomness between valid sounds.**
   The [repeat check](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-moving-repeat-diagnostic-2026-09-05/result.json>)
   compares the24 source cases above with their other real repeat at the same
   surface/speed/force, and both generated variants. It uses each recording's
   existing source-defined750ms moving crop, common22.05kHz,37 complete20ms bins.
   Sorted log-envelope absolute differences measure empirical1D distribution
   distance, deliberately ignoring order; a permutation test verifies that this
   cannot validate event timing. Real repeat appears twice for neural seed pairing:
   there are24 condition pairs,not48 independent real pairs. Real-repeat variation
   is substantially smaller than neural error, even when envelope order is ignored.
   **Randomness alone does not explain the generator gap.** Two repeats cannot
   establish a universal lower bound, perceptual judge or admission threshold.

| Scope | Full-envelope MAE global→windowed,dB | Moving-shape RMSE global→windowed,dB | Envelope wins |
|---|---|---|---:|
| Old anchors |1.48754→1.48119 |2.24960→2.24225 |13/24 |
| Oak |1.64559→1.65114 |2.08520→2.07550 |4/8 |
| Steel |1.96781→2.00937 |2.63721→2.66162 |0/8 |
| Frosted glass |1.82781→1.83340 |2.62664→2.65086 |4/8 |
| All |1.65064→1.65624 |2.34964→2.35245 |21/48 |

Moving-level absolute error is1.08678→1.08674dB overall. Onset error worsens
.01167→.01458s; uncensored offset is.17889→.17833s on36 paired cases. The glass
offset remains.43333s on this duration-extreme subset; this does not contradict
the earlier.2344s mean over the broader glass evaluation. No fitted time shifts,
per-output gain matching or censored-as-zero errors were introduced.

| Moving-window diagnostic | Real repeat | Global generator | Windowed generator |
|---|---:|---:|---:|
| Ordered envelope MAE,dB |.47625 |1.35737 |1.36618 |
| Envelope-distribution W1,dB |.23440 |1.13471 |1.13627 |
| Absolute level error,dB |.19106 |1.08678 |1.08674 |
| Shape RMSE,dB |1.29155 |2.34964 |2.35245 |
| Envelope standard deviation,dB |.52560 |.82598 |.83422 |

Target envelope standard deviation is.52930dB. Global beats real-repeat shape
in0/48 seed-paired comparisons,level4/48,and either envelope diagnostic1/48.
These750ms figures do not explain the entire full-event/timing error. The
[full-event repeat montage](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-moving-repeat-diagnostic-2026-09-05/comparison.wav>)
plays frosted glass20 then60mm/s,.5N: real / repeated real / global / windowed.
Each recording retains its own full duration; no acoustic clock fitting. The
source-free requested comparison above, not this source-aided montage, proves
the inference interface requires no target sound.

Reproduce using `lab/.venv/bin/python lab/scripts/physical_sound_texture_dc_audit.py
--lab-root ROOT --output NEW_EXTERNAL`, then
`lab/.venv/bin/python lab/scripts/physical_sound_texture_windowed.py --lab-root ROOT
--output NEW_EXTERNAL`; `physical_sound_texture_dc_audit.py --repeat-check` uses
the completed canonical windowed report. Outputs must be fresh external paths.
The three canonical roots are the links above.227 generated WAVs pass receipt
SHA,frames,stereo44100,finite/subtype/headroom checks;62 focused tests and Ruff
pass. No Cargo/host-check/ProductCheck or perceptual acceptance was run.

Decision: close DC-only,window-normalization and randomness-only explanations as
sufficient remedies. Preserve the existing hybrid baseline. Next inspect the
latent-only objective versus decoded acoustic errors and run one bounded
TRAIN-only acoustic training correction with a source-free output and unchanged
controls. Do not substitute sorted-envelope scores for timing/perceptual quality,
launch another EQ/window sweep, or interpret training-case recovery as transfer.

## Paired decoded-acoustic endpoint training (2026-09-05)

New source-free output: [base / ordinary fine-tuning / acoustic fine-tuning](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-acoustic-endpoint-2026-09-05/requested-comparison.wav>).
Each is3.15s of requested frosted-glass friction,40mm/s,.5N,90mm,seed314,
with the existing measured coefficient pair. No recording or sensor trace is
passed to the generator. [Real/base/FM/acoustic](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-acoustic-endpoint-2026-09-05/comparison.wav>)
adds the actual40mm/s,.5N reference for comparison, not for inference.

The prior loss optimized latent velocity MSE without a decoded-audio term.
[WaveFM, NAACL2025](https://aclanthology.org/2025.naacl-long.110.pdf), §§3.2–3.3,
provides primary-source precedent for endpoint prediction with auxiliary spectral
losses in speech vocoding. That mel-conditioned waveform task is not physical
texture generation or evidence that its gains transfer to our latent model.
This experiment preserves our velocity parameterization and uses the algebraic
endpoint estimate `mixed + (1-t)*predicted_velocity`; it is not a WaveFM replica.

Two copies of descriptor checkpoint6d36e47c… receive200 fixed updates each:
same existing48TRAIN records,seed23,AdamW1e-4/weight_decay1e-4,gradient clip1,
same crop/posterior/noise/time sampling recipe. No new surfaces/records enter fit.
The ordinary arm minimizes the original48-case velocity MSE. The acoustic arm
adds.02 times a decoded auxiliary loss on one TRAIN case per update:

- Select `step % 48`; the original prefix/random/suffix crop schedule remains.
  Because48 is divisible by3, the auxiliary crop mode is tied to a record; the
  full latent MSE still covers every record in every mode. This limited exposure
  does not establish comprehensive acoustic fitting of every event phase.
- Decode the predicted32-frame endpoint through the unchanged frozen Oobleck.
  Backpropagation reaches the flow; first auxiliary gradient norm to predicted
  velocity is.05944818. Codec parameters require no gradients and receive none.
- Use a differentiable41-tap shared-band resampler, numerically checked against
  SciPy's default `resample_poly(1,2)`. Discard fixed8-latent-frame margins for
  this auxiliary loss only. This is not proof all codec boundary effects vanish.
- Add absolute log-power-spectrum error at512/2048 FFT sizes, averaged in time,
  and ordered20ms log-energy error. Spectra subtract each frame's mean; energy
  retains DC. No per-output level normalization, phase target, development-derived
  threshold or changed evaluation metric. Natural-log units, not reported dB.

Training finishes in1.80s ordinary and11.08s acoustic after preparation; these
times exclude source encoding,model/codec loading and evaluation. The last48
training losses are not a fixed before/after probe because noise/time/crops vary.
Do not claim an auxiliary training-loss improvement from those histories.
Both162560-parameter weights and complete histories are external:

- Ordinary: `81a67e05f9c5be565c4f9fe29800c1dd8761411a15ca42669719d62e4c582ea9`.
- Acoustic: `4354345aeb9cb2860fd4f52c92f8b15fdfe1f5bef46da0868e5686dc282035f6`.

[Evaluation](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-acoustic-endpoint-2026-09-05/result.json>)
covers36 source events:20/40/60mm/s,.5/1N,new4/66/76 repeat0 and old0/65/74
repeat1; two seeds,three arms=216 generations.40mm/s remains absent from TRAIN.
All cases are disclosed development,not a new protected test. All72 baseline
PCM controls reproduce the earlier surface model byte-exactly.

| Scope | Moving-shape RMSE base/FM/acoustic,dB | Moving-level absolute error base/FM/acoustic,dB | Full-envelope MAE base/FM/acoustic,dB |
|---|---|---|---|
| Old anchors |2.3156/2.3052/2.3100 |.8430/.7510/.9765 |1.5165/1.4580/1.5592 |
| Oak |2.1070/2.0654/2.0722 |1.7065/1.5588/1.9865 |1.7147/1.5670/1.8574 |
| Steel |2.6661/2.6638/2.6635 |1.4788/1.6027/1.3606 |1.9572/2.0367/1.8803 |
| Frosted glass |2.6945/2.6689/2.6634 |.8114/.8316/.7436 |1.6769/1.6218/1.5717 |
| All |2.4024/2.3856/2.3882 |1.0876/1.0410/1.1700 |1.6497/1.5999/1.6645 |

Acoustic versus ordinary wins shape29/72,level28/72,envelope28/72. On oak,
level and envelope win0/12 each. Mean onset base/FM/acoustic is.00972/.00611/
.01056s; uncensored offset.13833/.13200/.13967s on60 paired cases. At unseen
speed40, envelope1.64796/1.61386/1.69210 and level1.08930/1.06877/1.20066dB:
the auxiliary loss does not produce an overall transfer improvement. Steel/glass
averages improve but do not authorize selecting favourable surfaces or promotion.

A separate [standalone reload](</home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-acoustic-endpoint-standalone-2026-09-05/requested-comparison.wav>)
loads only saved candidate metadata/weights and the cached frozen codec, no
dataset or source WAV. It exactly reproduces both published PCM files and every
full FLOAT audio sample. Full FLOAT file SHA differs only in byte60 inside the
RIFF PEAK metadata chunk, not the data chunk; each file's own receipt is valid.
No metadata rewriting or audio regeneration was used to force matching hashes.

Run `lab/.venv/bin/python lab/scripts/physical_sound_texture_acoustic.py --lab-root
ROOT --output NEW_EXTERNAL` for the paired fit/evaluation. `--evaluate-model MODEL`
reuses saved weights with0updates. `--render-model MODEL --output NEW_EXTERNAL`
requires no `--lab-root` and renders the fixed requested glass profile. It rejects
dataset/evaluation arguments,incorrect48-TRAIN identity,codec/parent identity,
weight hash/size and invalid tensors. Existing surface/demo loaders are unchanged.

Both jobs terminal.67 focused tests,Ruff format/check,481 WAV receipt/layout/
finite/headroom checks pass. Current model reloads and all72 baseline controls
verified; FLOAT sample equality distinguished from container-byte equality.
No perceptual admission,Cargo/host-check,ProductCheck,engine/demo or roadmap change.

Decision: keep both candidates as report-only evidence,not an overall upgraded
model. Do not sweep auxiliary weights,training duration or decoder windows from
these results. The penalty acts on a target-aided one-step endpoint during fit,
whereas inference follows64 midpoint steps from independent noise. A remaining
testable explanation is that correcting these endpoints does not correct the
actual generated distribution; this is a hypothesis,not established causation.
Next test acoustic feedback through actual source-free sampling on TRAIN only,
retaining the ordinary-fit control,full decodes and unchanged new-condition
evaluation. No conclusion that all acoustic losses fail or the goal is complete.
