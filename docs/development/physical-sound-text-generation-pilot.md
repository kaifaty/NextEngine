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
