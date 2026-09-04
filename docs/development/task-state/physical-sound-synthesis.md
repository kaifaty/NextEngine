# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / CODEC_SAMPLING_NOT_CAUSAL_IN_CONTROLS / WATER_RAIN_DATA_READY.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifacts:** [original -> codec mean -> sample](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-codec-targets-2026-09-05/comparison.wav)
  (18 s, reconstruction diagnostic, not a new text generator), and
  [real rain/pouring-water/water-drop training and development examples](/home/kaifaty/.codex/experiments/nextengine/physical-sound/esc50-water-rain-2026-09-05/sources-preview.wav)
  (33 s, internet source recordings). No new weights trained this checkpoint.
  The last text-generative fits remain unpromoted; runtime/liked glass is unchanged.
- **Exact current evidence and reproduction:**
  [text-generation pilot](../physical-sound-text-generation-pilot.md).
  Latest external roots are `tangoflux-codec-targets-2026-09-05` and
  `esc50-water-rain-2026-09-05` under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
  Codec `result.json`/`ast-tags.json`; water `result.json`/`real-clap.json`,
  pinned CSV/LICENSE and `audio/`. Prior balanced/uniform runs are
  `tangoflux-lora-glass-2026-09-05` / `tangoflux-lora-uniform-2026-09-05`.
- **Codec discriminator:** official training samples the posterior; our formula
  matches the installed distribution with the same CPU RNG. Three mean WAVs
  replay exactly. Nine sampled reconstructions retain CLAP 0.371–0.464 and
  spectral error within -0.01160/+0.01456 of the mean. Padding is -100…-95 dBFS.
  No evidence of gross target corruption from sampling in these controls;
  do not launch another glass fit merely switching to posterior means.
- **New corpus ready:** pinned ESC-50 revision
  `33c8ce9eb2cf0b1c2f8bcf322eb349b6be34dbb6`, 117 WAVs/100 source recordings:
  93 train (folds 1–4), 24 disclosed development (fold 5), source-ID-disjoint.
  40 rain, 37 pouring water, 40 water drops; generic class captions, physical
  attributes null. Frozen CLAP matches 110/117 labels; all disagreements remain.
  Dataset CC-BY-NC 3.0 and individual notices retained. IDs 67152/79220/126433
  (CC-Sampling+) remain unfetched. Ten source files touch full-scale PCM.
  Local URL-bearing metadata/filename screen found no prior ID collisions;
  scope is bounded, foundation pretraining independence is unknown.
- **Prior fits:** balanced and uniform losses improve fit but not free-generation
  alignment: CLAP mean base/balanced/uniform 0.35858/0.33929/0.34063, AST 10/10
  throughout. Uniform improves both active/padding regions; still keep the base.
  Exact sources/posteriors/sample-order/baseline controls match. Frozen T5/VAE/
  base, rank-8 q/v LoRA, AdamW 1e-4, BF16 train/FP32 inference; details in the note.
- **Counterfactuals already run:** 28 same-seed WAVs, duration 1.5/5 s, CFG
  1/2/4.5, plus base-unconditional branch. Longer clips reduce the penalty,
  not create a gain; low CFG has poor absolute scores; base-unconditional is
  not a repair. Do not repeat this sweep. All four historical WAV controls and
  exact upstream latent replay pass; scoring replays within 1e-6.
- **Validator limitation:** all six real/VAE controls prefer wooden-stick/glass
  over knife/glass in the current wording-confounded caption bank. Their true
  target cosines are 0.38–0.46, above generated examples; ranks cannot establish
  striker identity. Steel remains unscored, and no perceptual risk is calibrated.
- **Next action:** bounded multi-event learning on the existing water/rain corpus,
  with actual five-second targets, class-specific text conditioning and before/
  after WAVs. Keep the base, glass/wood regression controls and source-disjoint
  development. Generalize the existing trainer instead of a new training stack.
  Use uniform upstream loss as the control and validate free generation, not
  merely denoising loss/CLAP. No more nearby fits on the same glass-only data,
  model shopping, modal-MLP restart, invented physical labels or protected reuse.
- **Hardware/runtime:** RTX 3080, 10 GiB, working CUDA. `lab/.venv/bin/python`
  has Torch 2.13.0+cu130, datasets 2.21.0/fsspec 2024.6.1, peft 0.12.0 and the
  pinned libraries in the pilot note. Inference runs offline. The Tango pilot
  loads only hash-reviewed external source and verifies all checkpoint values
  and the tied T5 alias. Old sandbox GPU failures are not current evidence.
- **Verification:** codec 48 new/6 referenced WAVs and all 117 source WAVs pass
  hash/shape/rate/length checks; 18/33-second previews checked. Sampling and
  ingestion have 45 passing focused tests; Ruff/diff/link checks pass. Jobs terminal;
  checks, limitations and reproduction are in the pilot note.
- **All goal requirements remain open beyond this baseline:** independent
  robust validation, audible improvement through local learning, precise physical
  controls, demonstrated new-condition generalization and engine integration.
  No goal completion or product admission has been claimed.

## Preserve these constraints

- The user will not record impacts, hit glass or supply force-sensor data.
  Use internet sources. Preserve source attribution and applicable terms;
  unknown/incompatible redistribution terms exclude distribution.
- Generate playable media at each meaningful experiment checkpoint. Keep all
  candidates and honest failures; protocols, inventories and validators do
  not replace the audible deliverable. Supporting-only debt is currently zero.
- This broad goal does not authorize runtime neural weights or gameplay
  authority changes. [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md)
  is Proposed; report-only waveform generation/authored-asset research is
  allowed. Production retains ordinary clips and an authored fallback.
  Promotion requires the existing ADR/consumer/ProductCheck workflow.
- Do not repurpose protected holdout, shadow, validator or one-shot roles.
  Opened development/train evidence cannot become independent test evidence.
  Pretraining overlap of foundation models is unknown, not automatically clean.
- Do not invent geometry, composition, exact force, velocity or listener
  metadata from material labels or audio. Perceptual prompt fidelity and
  calibrated physical response are separate claims.
- Generated WAVs, datasets, weights and caches remain outside Git. Do not
  replace the liked glass demo/profile with a weak research candidate.
- Keep one roadmap. [V46](../../plans/physical-sound-synthesis-roadmap-v46.md)
  records the earlier bounded impact admission program, not proof the broad
  goal is achieved. Individual lab runs do not change product roadmap status.

## Previous audible work — controls, not the endpoint

- **Liked training reconstruction:** [first pilot result](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-first-2026-09-04/result.json),
  [comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-first-2026-09-04/comparison.wav).
  Three Freesound wine-glass crops -> fitted modal teachers -> MLP -> PCM.
  User liked it. In-sample acoustic loss 1.31725 -> 0.324965; no generalization.
- **Frozen transfer failure:** [unseen result](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-unseen-2026-09-04/result.json).
  Nine later strikes: spectral error 1.49691 vs training 0.31040; nearest
  old fitted parameters beat the network on 9/9. Same recordings, not new objects.
- **Automatic cycle:** [cycle result](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-cycle-2026-09-05/result.json),
  [analytic preview](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-cycle-2026-09-05/analytic-preview.wav).
  16 strikes from 761160/761161 train, 11 later strikes from excluded recording
  761162 development. Expanded and anchored MLPs fit training parameters but
  fail development: spectral/envelope/attack errors respectively
  3.11714/2.82105/2.43014 and 1.74364/1.97123/1.01343.
  Analytic-only control wins at 0.69560/0.33529/0.27242.
- Do not retry size/epoch variants of that supervised parameter MLP. Sparse
  data, nonunique modal targets and parameter/audio-loss mismatch were not
  isolated. More importantly, audio-to-parameter input is the wrong interface
  for the full goal. An input-dependent analytic fit is not a learned gain.
- The three source MP3s remain in
  `ps2-freesound-wine-glass-v1/research`; same author/pack and generator-train
  family, not known identical objects. Do not infer wall thickness or force.

## Legacy admission evidence and forbidden retries

- [V46 B0](../physical-sound-v46-b0-lane-aware-control-tournament-result-2026-09-03.md)
  remains NoUsefulTeacher, M0 admission blocked, real support 71/105. E0/V0
  source-power work is an admission path, not a prerequisite for report-only WAVs.
- [Corrected corpus](../physical-sound-v43-d2-c0r-identity-repair-result-2026-09-03.md)
  supersedes leaked AV-MSF/ObjectFolder role projections.
  [Disclosed roster](../physical-sound-v40-d0-disclosed-roster-result-2026-09-03.md)
  preserves nine permanently opened families. Opened ObjectFolder Real,
  YCB bj5w8 and REALIMPACT revisions are not clean protected evidence.
- V12 object 41 remains acquisition OOD: do not lower force coverage, select
  favourable contacts or open its response/protected roles. For object 92 do
  not drop contact 35 or download the already-proven irrelevant 12-GiB segment.
- V16–V37 spent protected experiment families stay closed. Do not tune their
  thresholds, roles, seeds, capacities or contacts from opened results, or
  reconstruct missing outputs from failed runs. In particular, M0c physical
  response and QSO-v0 geometry/contact transfer were rejected.
- Stationary random-phase residual, universal codec and prompt-to-waveform
  cannot be relabelled as admitted physical formulas. A waveform model is
  allowed as this separate report-only generator, with its own explicit claims.
- Prior detailed D-001…D-098 history and retired plans are available in Git
  at `ed8b9401:docs/development/task-state/physical-sound-synthesis.md`.
  They are a discovery index, not recursive mandatory reading.
