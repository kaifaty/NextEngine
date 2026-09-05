# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / LATE_EVENT_CONFIRMED / EVENT_EXTRACTION_WORKS / BASE_RETAINED.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifact:** [recovered bottle-fracture excerpt, seed314](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-fracture-event-matched-2026-09-05/break-glass-seed314.wav)
  (5 s); [failed prefix -> extracted event](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-fracture-event-matched-2026-09-05/comparison.wav)
  (11 s). Same generated full horizon, no new weights/reference recording.
  Three descriptions × two seeds recover glass tags on 6/6 versus prefixes
  3/6. This fixes candidate extraction, not neural duration control or the goal.
- **Exact current evidence and reproduction:**
  [text-generation pilot](../physical-sound-text-generation-pilot.md).
  Latest roots: `tangoflux-fracture-timing-2026-09-05` and
  `tangoflux-fracture-event-matched-2026-09-05`, under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
  Each has result/AST-CLAP JSONs and WAVs. Earlier transfer/fit roots are in note.
- **Prior transfer:** 16 prompts × seeds314/2718, mean CLAP .369675 -> .370049,
  14/32 improve; AST21/28 ->22/28, steel unscored. Roof-rain captions improve,
  tap/puddle drops worsen. Glass striker margin fails on314 in both models.
- **Decisive timing evidence:** seed314 original/without-empty/direct-fracture
  prefixes ~-99.7 dBFS, but remaining full horizon -20…-21 dBFS; peaks at
  18.17/17.53/12.67 s. Over 99.9999997% energy follows five seconds. Seed2718
  peaks near 1.68 s. Late event confirmed; wording changes do not fix timing.
  Positive prefix replays exactly; noise-floor replay differs by <=1 PCM unit
  (67 mono/868 stereo samples), so no universal exact-replay claim.
- **Extraction evidence:** first 10-ms RMS crossing max(-50dBFS, 0.1 peak),
  50-ms pre-roll; offsets12.61/0.70 s. Matched empty controls use same policy.
  AST Breaking top1 on all6; CLAP mean .28179 -> .43874, failed original
  .16196 -> .47726. All6 exceed matched empties. Later activity exists beyond
  every five-second crop: useful excerpts, not proven complete isolated events.
  No noise amplification/time stretch. The first `event-window` directory's
  unmatched-empty comparison is superseded by `event-matched`; do not use it.
- **Prior fit:** `tangoflux-prior-retention-2026-09-05`, step240, real full-MSE
  + weight1 field rehearsal. Prefix gains local, not broad quality; exact
  sources/order/teacher controls and drift reductions documented in note.
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
- **Do not repeat:** prior glass duration1.5/5s, CFG1/2/4.5 and base-unconditional
  sweeps. Exact controls passed; none repaired the old prefix scores.
- **Validator limitation:** all six real/VAE controls prefer wooden-stick/glass
  over knife/glass in the current wording-confounded caption bank. Their true
  target cosines are 0.38–0.46, above generated examples; ranks cannot establish
  striker identity. Steel remains unscored, and no perceptual risk is calibrated.
- **Next action:** reassess discrete-event base/prior differences with the same
  event-aware extraction before interpreting prefix regression as forgotten
  timbre. Keep full horizons/prefixes as controls and test multiple seeds.
  CLI now supports `--keep-full-horizon`, then `--extract-events result.json`
  with `--diagnostics`. This policy is not for continuous rain/water; the
  threshold is uncalibrated and late activity must not be silently accepted.
  No further SFT sweep, model shopping, modal-MLP restart, invented labels,
  protected reuse or new stack. Demo/base unchanged; no product admission.
- **Hardware/runtime:** RTX 3080 10 GiB, working CUDA, `lab/.venv/bin/python`,
  Torch 2.13.0+cu130, datasets 2.21.0/fsspec 2024.6.1, peft 0.12.0. Offline
  inference loads hash-reviewed code, verifies weights/T5 alias. Old sandbox
  GPU failures are stale; exact pinned libraries are in the pilot note.
- **Verification:** 48 authoritative full/prefix/extracted WAVs and 11-second
  comparison pass signal/hash checks, not quality acceptance. 51 focused tests and
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
