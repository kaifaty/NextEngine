# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / MATCHED_IMPACT_LORA_REJECTED / PHYSICAL_FRICTION_DATA_ACQUIRED.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifact:** [glass base/prior comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/tangoflux-impact-window-prior-crops-2026-09-05/glass-comparison.wav)
  (22s): base/metal striker, prior/metal, base/wooden, prior/wooden, seed314.
  Wood/steel comparisons are in the same directory. Same event extraction;
  no new weights, reference input or independent loudness normalization.
- **Exact current evidence and reproduction:**
  [text-generation pilot](../physical-sound-text-generation-pilot.md).
  Latest roots: `tangoflux-impact-window-{base,prior,base-crops,prior-crops}-2026-09-05`, under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
  Each has result/AST-CLAP JSONs and WAVs. Earlier transfer/fit roots are in note.
- **Earlier broad transfer:**16 prompts × two seeds:14/32 improve, roof-rain
  gains but tap/puddle drops regress. Exact evidence is in the pilot note.
- **Matched impact result:** seven prompts × two seeds, base/prior prefix CLAP
  .318445/.317866 -> crops .373407/.363659. Prior improves only1/14 crops;
  all seven class means decline. Retain base; do not repeat this LoRA sweep.
  AST8/10 ->10/10 both, steel unscored; rank1 crops7/14 ->8/14 is not quality.
  All crops beat equally processed empties. Glass/metal314 onset6.67/7.91s;
  fracture31412.61s. All28 have later activity, no padding: not isolated events.
  Striker-swap margins now positive on both seeds/materials/models; the former
  glass314 failure is window-sensitive, not proven striker confusion.
- **Extraction boundary:** first10ms RMS crossing max(-50dBFS,0.1peak),50ms
  pre-roll, same policy for empties; no amplification/stretch. Uncalibrated,
  discrete-event policy, not continuous rain/water. Old unmatched-empty
  `tangoflux-fracture-event-window` evidence is superseded; do not reuse it.
- **New physical-control source:** `cluster-texture-controls-canonical-2026-09-05`,
  Figshare29438288v5, CC-BY4.0,12 disclosed recordings: wood0/steel65/glass74,
  urethane probe,20/60mm/s ×0.5/1N, direction0/repeat0. Audio/raw two-mic,
  force/position and original metadata:50 files,19.50MB, CRC/SHA checked.
  [Recorded friction preview](/home/kaifaty/.codex/experiments/nextengine/physical-sound/cluster-texture-controls-canonical-2026-09-05/clean-controls-preview.wav)
  uses one gain78.515; recorded, NOT generated. `signal-audit.json` has order.
  Actual speed19.755–19.828/59.459–60.155mm/s; force medians .529/1.029–1.049N.
  Clean signal−77…−66dBFS; machine residual/preprocessing is a possible shortcut.
  Preserve raw noise channel; commanded labels are not actual sensor readings.
  Mini archive has no raw audio. Full archive range access works; canonical
  Figshare URL must resolve per request (signed redirect expires after10s).
  Earlier failed mini/full directories remain, not authoritative. No full ZIP
  download or whole-archive MD5 verification. No protected evidence reopened.
- **Prior fit:** `tangoflux-prior-retention-2026-09-05`, step240, real full-MSE
  + weight1 field rehearsal. Prefix gains local, not broad quality; exact
  sources/order/teacher controls and drift reductions documented in note.
- **Codec discriminator:** mean replays and sampled controls reject gross
  sampling corruption; do not retry glass merely switching to posterior means.
- **ESC-50 corpus:**117 WAVs/100 sources,93 train/24 disclosed development,
  source-ID-disjoint. Generic rain/pour/drop labels, physical attributes null.
  CC-BY-NC3.0/individual notices retained; excluded Sampling+ IDs unfetched.
  Revision, source list and limitations remain in the pilot note.
- **Prior glass fits:** balanced/uniform objectives improve fit, not generation;
  exact controls match. Do not repeat; details and frozen configuration in note.
- **Do not repeat:** prior glass duration1.5/5s, CFG1/2/4.5 and base-unconditional
  sweeps. Exact controls passed; none repaired the old prefix scores.
- **Validator limitation:** all six real/VAE controls prefer wooden-stick/glass
  over knife/glass in the current wording-confounded caption bank. Their true
  target cosines are 0.38–0.46, above generated examples; ranks cannot establish
  striker identity. Steel remains unscored, and no perceptual risk is calibrated.
- **Next action:** expand friction data to repeats/intermediate velocities and
  produce a conditional neural sound with held-out-speed/repeat comparisons
  and a non-neural baseline. Include raw machine-noise controls in that same
  audible experiment; no validator-only milestone. The new bounded source
  script is `lab/scripts/physical_sound_texture_probe.py`. One fixed probe and
  three surfaces are one physical axis, not the whole goal or object transfer.
  No repeated generic-caption/glass SFT sweeps, modal-MLP restart, invented labels,
  protected reuse or new stack. Demo/base unchanged; no product admission.
- **Hardware/runtime:** RTX3080 10GiB, working CUDA, `lab/.venv/bin/python`.
  Offline weights/code verified; exact libraries in note. Old GPU blockers stale.
- **Verification:**192 generated WAVs,24 acquired WAVs plus sensor files,
  three22s comparisons and recorded previews verified.22 focused tests and
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
