# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / PRIOR_RETENTION_PARTIAL_GAIN / GENERALIZATION_NEXT / BASE_RETAINED.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifact:** [rain/pouring water/drops, base -> prior-retained](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-prior-retention-2026-09-05/comparison.wav)
  (33 s, seed 42, text-only inference); [glass base -> unregularized -> retained](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-prior-retention-2026-09-05/glass-retention.wav)
  (12 s, both seeds). Step 0/40/240, seeds 42/123, regression/empty controls,
  96 WAVs and automatic AST/CLAP retained. Rain/drops alignment improves on both
  seeds; glass/wood recover partially, not to base. No promotion/demo change.
- **Exact current evidence and reproduction:**
  [text-generation pilot](../physical-sound-text-generation-pilot.md).
  Latest fit is `tangoflux-prior-retention-2026-09-05`; unregularized comparator
  is `tangoflux-water-rain-fit-2026-09-05`; sources are
  `esc50-water-rain-2026-09-05`, under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
  Fit/stage JSONs, adapters, comparisons, `prior.safetensors` and
  `field-audit/result.json`; sources retain CSV/LICENSE/WAVs/`real-clap.json`.
- **Retention discriminator:** real full-MSE plus weight-1 base-field MSE,
  160 paired conditional/unconditional states from 16 seed-7 trajectories,
  captured at steps 0,5,...,45. 121 states visited; synthetic rehearsal, not
  real labels or evaluation seeds. Sources/posteriors/240 sample indices and
  24 baseline WAVs match the comparator. FP32 replay/zero-LoRA controls pass.
  Same 240 updates; development active/full MSE improves 25.01%/16.29%.
  Mean CLAP base -> prior: rain .46055 -> .47141; pour .35234 -> .32604;
  drops .41781 -> .44119; glass .25277 -> .19148; wood .39026 -> .36727.
  AST returns to 10/10 from unregularized 9/10; CLAP top1 still 6/10 vs base
  8/10, both pours prefer drops. Overall .35948 vs base .37475/unregularized
  .33191. Training-bank audit: branch drift -80.25%, CFG4.5 drift -65.20%.
- **Codec discriminator:** official posterior sampling matches our formula and
  retains CLAP 0.371–0.464, spectral error within -.01160/+.01456 of the mean,
  padding -100…-95 dBFS. Three mean WAVs replay exactly. No gross sampling
  corruption in these controls; do not retry glass merely switching to means.
- **Corpus:** ESC-50 `33c8ce9eb2cf0b1c2f8bcf322eb349b6be34dbb6`,
  117 WAVs/100 sources, 93 train folds 1–4 / 24 disclosed development fold 5,
  source-ID-disjoint. Rain/pour/drops 40/37/40; generic captions, physical
  attributes null. CLAP matches 110/117; disagreements retained.
  CC-BY-NC 3.0/individual notices retained; CC-Sampling+ IDs 67152/79220/126433
  unfetched. Ten full-scale PCM sources. Bounded prior-ID screen found no
  collisions; foundation pretraining independence is unknown.
- **Prior glass fits:** CLAP base/balanced/uniform .35858/.33929/.34063;
  AST 10/10 throughout, despite improved fit. Exact source/posterior/order/
  baseline controls match. Frozen base/T5/VAE, rank-8 q/v LoRA, AdamW 1e-4,
  BF16 train/FP32 inference; details in the note.
- **Counterfactuals already run:** 28 WAVs, duration 1.5/5 s, CFG 1/2/4.5,
  base-unconditional branch. None repairs glass; do not repeat. Four historical
  WAV and upstream latent replays pass; scores replay within 1e-6.
- **Validator limitation:** all six real/VAE controls prefer wooden-stick/glass
  over knife/glass in the current wording-confounded caption bank. Their true
  target cosines are 0.38–0.46, above generated examples; ranks cannot establish
  striker identity. Steel remains unscored, and no perceptual risk is calibrated.
- **Next action:** new-seed/unseen-wording/composition comparison of base and
  retained-prior adapter, including regression events, before another fit.
  Test transfer beyond two known seeds; do not turn localized CLAP gains into
  calibrated quality or physical-control claims. Pour/drop confusion remains.
  Guided-field preservation is a possible later discriminator, not permission
  for a weight/epoch sweep. Existing trainer supports `--prior-weight 1`.
  No glass-only fits, model shopping, modal-MLP restart, invented labels or
  protected reuse; no new training stack.
- **Hardware/runtime:** RTX 3080 10 GiB, working CUDA, `lab/.venv/bin/python`,
  Torch 2.13.0+cu130, datasets 2.21.0/fsspec 2024.6.1, peft 0.12.0. Offline
  inference loads hash-reviewed code, verifies weights/T5 alias. Old sandbox
  GPU failures are stale; exact pinned libraries are in the pilot note.
- **Verification:** 96 WAVs, three 24-second previews, 33/12-second comparisons
  pass signal/hash checks; adapters and teacher bank match. 45 focused tests and
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
