# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / FIRST_GENERATIVE_LORA_TRAINED / BASE_RETAINED / RESEARCH_ONLY.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifact:** [real glass -> base -> step 40 -> step 120](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-lora-glass-2026-09-05/glass-training-comparison.wav),
  16 seconds, repeated for two seeds. First actual generative fine-tune:
  786,432 LoRA parameters, 120 steps, 16 disclosed train/11 development crops.
  Generation takes only text and duration. Runtime/liked glass remains unchanged.
  [Reloaded adapter, all twelve event prompts](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-lora-reload-2026-09-05/preview.wav).
- **Exact current evidence and reproduction:**
  [text-generation pilot](../physical-sound-text-generation-pilot.md).
  Latest external roots are `tangoflux-lora-glass-2026-09-05` and
  `tangoflux-lora-reload-2026-09-05` under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
  Read fit `result.json`, its step0/40/120 `ast-clap.json` and reload results.
  Original base root is `tangoflux-pilot-2026-09-05`.
- **Decisive training result:** development active flow MSE 1.32718 -> 0.96294
  (-27.4%), but padding error worsens 7.4% and full-horizon error 3.7%.
  Balanced objective improves 16.6%. AST passes coarse tags on 10/10 at all
  three checkpoints; CLAP mean falls 0.35858 -> 0.33929. Trained glass prompt
  seed-123 CLAP falls 0.22953 -> 0.10767. Keep the base: lower denoising loss
  did not establish better free generation. Do not select solely by flow loss.
  Loss gives equal weight to 33 active and 612 padding frames; duration 1.5 s.
  Frozen T5/VAE/base, rank-8 attention q/v adapter, AdamW 1e-4, BF16 autocast,
  FP32 inference. Full train/render cycle took 314.21 s on the 10-GiB GPU.
- **Model comparison:** AudioLDM2/TangoFlux CLAP top-one counts 8/24 vs 15/24,
  better-than-empty counts 21/24 vs 24/24; AST coarse expected top-five tags
  6/20 vs 15/20. Steel remains unscored by the exact material ontology and can
  sound/classify glass-like. Rolling lacks the expected tag; scraping has
  Rub/Filing rather than the fixed Scrape label. Light/heavy rain pair margins
  disagree across seeds. These are useful diagnostics, not physical admission.
- **Precision counterfactual:** FP32 yields the same counts and median PCM
  correlation 0.99912 with FP16; precision is not the main failure cause.
  At 200 steps CLAP moves to 8/24 and 21/24, while AST moves to 6/20;
  doubling compute does not resolve the failures. All three runs are complete.
- **Next action:** discriminate short-duration/padding and guidance effects on
  free generation using the same prompt/seed base control, before longer fits.
  Inspect both active and padding terms; broad AST tags missed loss/semantic
  disagreement. Then expand internet data beyond the single glass family.
  Do not restart model shopping/modal MLPs, optimize only CLAP or use protected
  evidence. First learnability is not physical-attribute generalization.
- **Current hardware:** NVIDIA RTX 3080, 10 GiB, CUDA works in the unrestricted
  environment. Prior sandbox GPU failures are not current evidence.
  `lab/.venv/bin/python` has Torch 2.13.0+cu130 and the optional generation
  dependencies listed in the pilot note. Network is disabled for inference
  via offline flags after public pinned safetensors downloads.
  TangoFlux also imports datasets 2.21.0, which pins fsspec to 2024.6.1;
  LoRA adds peft 0.12.0. Torch/model libraries are unchanged. External source is
  loaded by `lab/scripts/physical_sound_tangoflux_pilot.py`; all checkpoint
  values and the T5 alias are verified before inference.
- **Verification:** cached FP32 training loss equals upstream exactly at
  0.32723280787467957. All 84 fit and 26 reload WAVs passed signal/hash checks.
  Fresh adapter reload reproduces four controls' stereo/mono hashes exactly.
  Thirty focused tests, Ruff and diff checks pass. Tests include active/padding
  gradients and malformed sources/adapters. All jobs are terminal. Detailed
  verification and commands are in the pilot note.
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
