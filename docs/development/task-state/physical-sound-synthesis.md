# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / TWO_LORA_FITS_UNPROMOTED / TARGET_PIPELINE_RESEARCH_NEXT.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifact:** [real glass -> base -> balanced fit -> uniform fit](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-lora-uniform-2026-09-05/objective-comparison.wav),
  16 seconds, repeated for two seeds. Two actual generative fine-tunes:
  786,432 LoRA parameters, 120 steps each, 16 train/11 development crops.
  Generation takes only text and duration. Runtime/liked glass remains unchanged.
  [Duration/guidance comparison, problematic seed 123](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-duration-guidance-2026-09-05/comparison.wav).
- **Exact current evidence and reproduction:**
  [text-generation pilot](../physical-sound-text-generation-pilot.md).
  Latest external roots are `tangoflux-lora-uniform-2026-09-05` and
  `tangoflux-duration-guidance-2026-09-05` under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
  Read uniform `result.json` and step40/120 `ast-clap.json`; probe `result.json`,
  `ast-tags.json`, `real-positive-controls.json`. The original balanced fit is
  `tangoflux-lora-glass-2026-09-05`; its step0 scores are the exact-WAV baseline.
- **Decisive training result:** balanced active MSE 1.32718 -> 0.96294 improves,
  but padding worsens; uniform loss improves active/padding/full to
  0.97712/0.59010/0.60990 (base 1.32718/0.60084/0.63801). Neither improves free
  generation: ten-case CLAP mean base/balanced/uniform 0.35858/0.33929/0.34063;
  uniform trained-prompt scores 0.23561/0.10643 versus base 0.27602/0.22953.
  AST coarse tags pass 10/10 throughout. Keep the base; scalar fit loss and
  coarse tags miss this failure. Both runs have identical sources, cached
  posteriors, sample order and bit-exact baseline WAVs. Only the loss changes.
  Balanced weights 33 active/612 padding frames equally; uniform is upstream MSE.
  Frozen T5/VAE/base, rank-8 attention q/v adapter, AdamW 1e-4, BF16 autocast,
  FP32 inference. Balanced/uniform train/render cycles took 314.21/325.71 s.
- **Counterfactuals already run:** 28 same-seed WAVs, duration 1.5/5 s, CFG
  1/2/4.5, plus base-unconditional branch. Longer clips reduce the penalty,
  not create a gain; low CFG has poor absolute scores; base-unconditional is
  not a repair. Do not repeat this sweep. All four historical WAV controls and
  exact upstream latent replay pass; scoring replays within 1e-6.
- **Validator limitation:** all six real/VAE controls prefer wooden-stick/glass
  over knife/glass in the current wording-confounded caption bank. Their true
  target cosines are 0.38–0.46, above generated examples; ranks cannot establish
  striker identity. Steel remains unscored, and no perceptual risk is calibrated.
- **Next action — bounded research after two fits:** inspect official codec
  semantics and test real -> VAE mean versus posterior-sample -> audible WAV.
  Current controls decode only the mean, while training samples the posterior.
  Distinguish target corruption/mismatch, constant-caption sparse-data fitting
  and inadequate free-generation evaluation before a third fit. If targets are
  sound, expand internet data beyond this family. No nearby rank/lr/epoch sweep,
  model shopping, modal-MLP restart or protected-role reuse.
- **Earlier base comparison:** TangoFlux beat AudioLDM2 on the fixed diagnostics;
  AudioLDM2 FP32/200-step counterfactuals did not fix quality. Details are in
  the pilot note. These are not physical-control or generalization certificates.
- **Current hardware:** NVIDIA RTX 3080, 10 GiB, CUDA works in the unrestricted
  environment. Prior sandbox GPU failures are not current evidence.
  `lab/.venv/bin/python` has Torch 2.13.0+cu130 and the optional generation
  dependencies listed in the pilot note. Network is disabled for inference
  via offline flags after public pinned safetensors downloads.
  TangoFlux also imports datasets 2.21.0, which pins fsspec to 2024.6.1;
  LoRA adds peft 0.12.0. Torch/model libraries are unchanged. External source is
  loaded by `lab/scripts/physical_sound_tangoflux_pilot.py`; all checkpoint
  values and the T5 alias are verified before inference.
- **Verification:** 36 focused tests, Ruff and diff/link checks pass. All 56
  probe and 84 uniform-fit WAVs pass signal/hash checks. Earlier fresh adapter
  reload reproduces four controls' stereo/mono hashes exactly. All jobs are
  terminal; detailed evidence and reproduction are in the pilot note.
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
