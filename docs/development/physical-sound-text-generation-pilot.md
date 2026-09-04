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
