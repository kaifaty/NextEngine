# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / TEXT_TO_SOUND_PILOT / RESEARCH_ONLY / FALLBACK_REQUIRED.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifact:** [text-pilot preview](/home/kaifaty/.codex/experiments/nextengine/physical-sound/text-pilot-2026-09-05/preview.wav):
  12 descriptions, first fixed seed, five seconds each with 0.5-second gaps.
  AudioLDM2 generated 24 candidates plus two empty-prompt controls offline.
  No input audio and no local training. This is a pretrained generative
  baseline, not an already-trained physical-attribute model.
- **Exact current evidence and reproduction:**
  [text-generation pilot](../physical-sound-text-generation-pilot.md).
  External roots end in `text-pilot-2026-09-05`,
  `text-pilot-fp32-2026-09-05`, and
  `text-pilot-200steps-2026-09-05` under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
  Read their actual `result.json` and `ast-tags.json`, not just this summary.
- **100-step findings:** CLAP exact-prompt rank one on 7/24, target similarity
  beats empty prompt on 20/24. Separate audio-only AST has coarse expected
  tags in top five on 5/20 scorable cases (all water/rain); steel has no exact
  ontology label and is unscored. CLAP is shared with the generator, AST is
  not; neither is a calibrated naturalness or physical-correctness validator.
- **Precision counterfactual:** FP32 yields the same counts and median PCM
  correlation 0.99912 with FP16; precision is not the main failure cause.
  At 200 steps CLAP moves to 8/24 and 21/24, while AST moves to 6/20;
  doubling compute does not resolve the failures. All three runs are complete.
- **Next action:** compare a materially different pretrained base on the same
  event/material cases. TangoFlux has official inference/fine-tuning code and is the next
  research candidate; do not assume it is installed or tested. No further
  similar tuning after two failed counterfactuals without bounded research.
  Then adapt a useful base with physical/event controls and real-data checks;
  do not optimize only the generator's own CLAP score.
- **Current hardware:** NVIDIA RTX 3080, 10 GiB, CUDA works in the unrestricted
  environment. Prior sandbox GPU failures are not current evidence.
  `lab/.venv/bin/python` has Torch 2.13.0+cu130 and the optional generation
  dependencies listed in the pilot note. Network is disabled for inference
  via offline flags after public pinned safetensors downloads.
- **Verification:** 18 focused tests, Ruff and diff checks passed; all 81 WAVs
  read back with valid length/unclipped PCM and individual hashes checked.
  No background processes remain after this checkpoint.
- **All goal requirements remain open beyond this baseline:** independent
  robust validation, improvement through local learning, precise physical
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
