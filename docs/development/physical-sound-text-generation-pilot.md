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

Do not train another recording-to-modal-parameter MLP or declare that prompt
generation solves physical control. Compare a materially different pretrained
sound-effects generator on the same conditions before adapting a weak base.
[TangoFlux](https://github.com/declare-lab/TangoFlux) supplies generation and
fine-tuning code, plus an iterative preference-training method; it is a
research-only candidate, not selected or executed here. Its CLAP-based ranking
would still need separate failure checks. Stable Audio 3 Small-SFX is another
candidate, but its official weight access is gated; no gate was bypassed.

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
