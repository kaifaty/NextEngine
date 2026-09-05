# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / TRANSFER_MIXED / SILENT_EVENT_DISCRIMINATOR_NEXT / BASE_RETAINED.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifacts:** [new light/heavy rain and drops, base -> prior](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-transfer-prior-2026-09-05/water-comparison.wav)
  (33 s); [glass impacts, scraping and rolling, base -> prior](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-transfer-prior-2026-09-05/interaction-comparison.wav)
  (44 s). First fixed seed 314, no selection; seed 2718 also retained.
  136 WAVs, 88-second full previews and automatic AST/CLAP. Mixed transfer,
  not a broad gain; keep the base and liked demo unchanged.
- **Exact current evidence and reproduction:**
  [text-generation pilot](../physical-sound-text-generation-pilot.md).
  Latest roots are `tangoflux-transfer-base-2026-09-05` and
  `tangoflux-transfer-prior-2026-09-05`; prior fit/source roots remain under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
  Each transfer root has `result.json`, `ast-clap.json`, all WAVs and snapshots.
- **Transfer evidence:** 16 prompts (3 unchanged, 13 exact-new), seeds 314/2718,
  50 FP32 steps, CFG 4.5, five seconds. Mean CLAP .369675 -> .370049;
  14/32 improve, new descriptions 11/26. Top1 19/32 -> 17/32; AST 21/28 ->
  22/28, four steel sounds unscored. New roof-rain captions improve, new
  tap/puddle drops worsen. Glass striker-pair margin fails on seed 314 in both.
  Heavy/light rain RMS ordering holds, but is not calibrated physical control.
- **New decisive failure:** bottle fracture seed 314 is effectively silent
  in both models (-99.64/-99.39 dBFS). Seed 2718 has breaking/glass tags,
  -14.57/-14.10 dBFS. The exact prompt is in the committed transfer profile.
  No full-horizon decode has yet been inspected. Do not equate nonzero PCM
  with an audible event, normalize codec noise, or select away this failure.
- **Prior fit remains:** `tangoflux-prior-retention-2026-09-05`, step 240,
  full real MSE + weight-1 base-field rehearsal, seed7/160 paired states.
  42/123 gains were local: overall CLAP .35948 vs base .37475/unregularized
  .33191. Branch drift -80.25%, guided drift -65.20%; not a broad quality gain.
  Sources/posteriors/order/baseline/teacher controls passed; details in note.
- **Codec discriminator:** mean replays and sampled controls reject gross
  sampling corruption; do not retry glass merely switching to posterior means.
- **Corpus:** ESC-50 `33c8ce9eb2cf0b1c2f8bcf322eb349b6be34dbb6`,
  117 WAVs/100 sources, 93 train folds 1–4 / 24 disclosed development fold 5,
  source-ID-disjoint. Rain/pour/drops 40/37/40; generic captions, physical
  attributes null. CLAP matches 110/117; disagreements retained.
  CC-BY-NC 3.0/individual notices retained; CC-Sampling+ IDs 67152/79220/126433
  unfetched. Ten full-scale PCM sources. Bounded prior-ID screen found no
  collisions; foundation pretraining independence is unknown.
- **Prior glass fits:** balanced/uniform objectives improve fit, not generation;
  exact controls match. Do not repeat; details and frozen configuration in note.
- **Counterfactuals already run:** 28 WAVs, duration 1.5/5 s, CFG 1/2/4.5,
  base-unconditional branch. None repairs glass; do not repeat. Four historical
  WAV and upstream latent replays pass; scores replay within 1e-6.
- **Validator limitation:** all six real/VAE controls prefer wooden-stick/glass
  over knife/glass in the current wording-confounded caption bank. Their true
  target cosines are 0.38–0.46, above generated examples; ranks cannot establish
  striker identity. Steel remains unscored, and no perceptual risk is calibrated.
- **Next action:** base-model timing/prompt discriminator before another fit:
  retain the entire decoded 30-second horizon with the SAME five-second
  condition, replay failed fracture seed314, compare positive seed2718 and
  minimal wording changes (remove `empty`, simplify event sequence). Separate
  an out-of-window event, total omission and prompt sensitivity; emit WAVs.
  Current CLI supports `--prompts lab/profiles/physical-sound-transfer-prompts.json`
  and `--diagnostics`. No new SFT/regularizer sweep, glass-only fit, model
  shopping, modal-MLP restart, invented labels, protected reuse or new stack.
- **Hardware/runtime:** RTX 3080 10 GiB, working CUDA, `lab/.venv/bin/python`,
  Torch 2.13.0+cu130, datasets 2.21.0/fsspec 2024.6.1, peft 0.12.0. Offline
  inference loads hash-reviewed code, verifies weights/T5 alias. Old sandbox
  GPU failures are stale; exact pinned libraries are in the pilot note.
- **Verification:** 136 WAVs, two 88-second previews, 33/44-second comparisons
  pass signal/hash checks, not audibility acceptance. 47 focused tests and
  Ruff/diff/local-link checks pass. All jobs terminal. No Cargo/ProductCheck
  or engine audition: external Python lab only. Reproduction is in the note.
- **All goal requirements remain open beyond this baseline:** independent
  robust validation, audible improvement through local learning, precise physical
  controls, demonstrated new-condition generalization and engine integration.

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
  User liked three fitted glass reconstructions; no generalization.
- **Frozen transfer failure:** [unseen result](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-unseen-2026-09-04/result.json).
  Nearest old parameters beat the MLP on 9/9 later strikes, not new objects.
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
