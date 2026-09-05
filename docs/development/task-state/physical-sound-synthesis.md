# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / RAIN_WAVEFORM_BASELINE / TEMPORAL_STRUCTURE_MISSING.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifact:** [neural rain](/home/kaifaty/.codex/experiments/nextengine/physical-sound/amazon-rain-neural-2mm-2026-09-05/generated.wav),
  8s/48kHz,2mm accumulated over5min/seed2718; no recording at inference.
  Stationary forest soundscape, not isolated rain or unseen-intensity evidence.
  [Rain comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/amazon-rain-neural-2026-09-05/rain-short-comparison.wav):
  real -> neural -> reference-spectrum control at0.4, then6.2mm/5min,51s.
  Shared gain0.980778, no individual matching. Friction/demo/base unchanged.
- **Exact current evidence and reproduction:**
  [text-generation pilot](../physical-sound-text-generation-pilot.md).
  Latest roots: `amazon-rain-neural{,-2mm}-2026-09-05`, under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
- **Matched impact result:** prior improves only1/14 event crops, all seven
  class means decline. Retain base; no LoRA sweep. Positive striker margins
  after cropping revise earlier window-confounded failures, not material proof.
- **Extraction boundary:** first10ms RMS crossing max(-50dBFS,0.1peak),50ms
  pre-roll, same policy for empties; no amplification/stretch. Uncalibrated,
  discrete-event policy, not continuous rain/water. Old unmatched-empty
  `tangoflux-fracture-event-window` evidence is superseded; do not reuse it.
- **Physical-control source:** `cluster-texture-training-grid-2026-09-05`,
  Figshare29438288v5, CC-BY4.0,60 records/242 files/86.69MB: wood0/steel65/glass74,
  urethane probe,20/30/40/50/60mm/s ×0.5/1N, direction0/repeats0/1. Hash checked.
  Train24 repeat0 at20/30/50/60;12 scans40mm/s and24 repeat1 development.
  Same surfaces, not new objects. Keep measured and commanded controls separate.
  Mini has no raw audio; full ZIP range reader must resolve canonical Figshare
  URL per request (signed redirect expires after10s). No full-archive MD5 check.
- **Crossed-velocity result:** fixed rank4/4,804params/1500steps. Neural versus
  interpolation spectrum RMSE30:1.904/1.759;40:1.690/1.729;50:1.850/1.788dB.
  Pooled1.815/1.758,11/36 wins. Stop this three-surface capacity/epoch/basis
  tuning; the apparent40mm/s gain is not robust. All folds disclosed development.
  No pretraining; physical inputs -> stationary Gaussian texture, not impacts.
- **Counterchecks:** repeat scatter contributes13–17% of neural MSE.
  Matched speed/load surface retrieval is30/30 for BOTH clean and machine-mic
  spectra. That score is not independent quality validation. It does not prove
  the generator uses only noise. Upstream noncausal NLMS inspected, not executed;
  no exact archive-preprocessing replay. Audit JSON in speed50 root.
- **Rain source:** `amazon-rain-source-probe-2026-09-05`, DataSuds DOI
  10.23708/I0QYNM V2, CC-BY4.0. Three60s/48kHz WAVs match0/.4/6.2mm/5min;
  original CSV43944 `?format=original`,381270141bytes, MD5d91a06cecf3af48a205bdf43c48abee1.
  Converted TSV exceeded publisher size; incomplete `.part` is rejected/unused.
  48,208 rows,513 linear rawPCM16²/Hz bins; divide32768², then10log10.
  Welch1024 matches all three WAVs within0.000020dB.33 filenames start seconds
  after their valid table minute, not sample-level gauge alignment. No class
  relabeling. Cross-site43958/43957 unfetched; only three full WAVs published.
- **Rain fit:**1,384params/rank8,seed41/2000steps,948wet+948dry selected train.
  Day split and adjacent-day guards; May10 development. Wet364 development
  rows/19days: neural/interpolation7.469/7.580dB,191 wins. Small spectral gain,
  not quality or unseen-intensity proof. Rows/model/results external, note below.
- **Rain discriminator:** AST misses BOTH real wet clips; not an acceptance
  validator here. Exact8s stationary-spectrum control loses10ms envelope
  variation (light0.301 versus real0.854); temporal reference restores0.845.
  [Representation comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/amazon-rain-neural-2026-09-05/light-rain-temporal-comparison.wav),25.5s,
  real -> stationary -> temporal. Reference-dependent reconstruction, NOT neural
  generation/quality acceptance; heavy temporal crest overshoots9.10 versus3.78.
- **Prior controls:** codec checks reject gross corruption. No posterior-mean,
  duration/CFG or caption-bank threshold retries; exact evidence in pilot note.
  ESC-50 physical attributes are null; no material-identity validation for steel.
- **Next action:** acquire a broader FULL-WAVEFORM source with trustworthy
  physical controls and train temporal/event structure; produce new playable
  neural audio, not only a validator. Three rain WAVs/spectral tables cannot
  establish this. No rain spectrum MLP width/epoch sweep or AST threshold tuning.
  Preserve reference-dependent controls as controls. Keep cross-site tables out
  of tuning; group temporal evidence by storm/day, not adjacent random rows.
  No repeated generic-caption/glass SFT sweeps, modal-MLP restart, invented labels,
  protected reuse or new stack. Demo/base unchanged; no product admission.
- **Verification:** rain fit/inference/AST/temporal probe terminal;34 focused
  tests and Ruff pass. Source units/hashes, no-reference inference and WAV
  checks verified; exact evidence in pilot note. No Cargo/ProductCheck
  or engine audition: external Python lab only.
- **All goal requirements remain open beyond this baseline:** independent
  robust validation, robust audible gains from learning, broader physical
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
- **Automatic cycle:** [cycle result](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-cycle-2026-09-05/result.json).
  16 train strikes,11 later development strikes. Expanded/anchored MLPs fit
  training but lose to the analytic-only control; exact errors are in the note.
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
