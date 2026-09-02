# Physical sound V32 M1 execution mechanics protocol

| Field | Value |
| --- | --- |
| Status | `FROZEN_BEFORE_OFFICIAL_VALUES` |
| Date | `2026-09-02` |
| Parent | [V32 M0 physics-locked residual protocol](physical-sound-v32-m0-physics-locked-residual-protocol-2026-09-02.md) |
| Planning authority | [Roadmap V32](../plans/physical-sound-synthesis-roadmap-v32.md) |
| Claim | `SYNTHETIC_KNOWN_TRUTH_TOURNAMENT_ONLY / NO_REAL_MATERIAL_QUALITY_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Purpose

M0 freezes the corpus, oracle, candidate, controls, gates and resource envelope.
This document freezes the remaining execution mechanics that must be
unambiguous before the M1 owner materializes an official train, development or
method-holdout value. It does not change an M0 feature, target, model,
threshold, role, seed, contact, geometry cell or authority claim.

The official M1 experiment is one tournament, executed in two fresh processes
`A` and `B` solely to establish byte-exact repeatability. `B` is not a retry and
cannot repair or replace `A`. A mismatch is terminal `REJECT_NONDETERMINISTIC`.

## Corpus enumeration and role access

The owner enumerates rows in this exact nested order:

1. role in `train`, `development`, `method_holdout` access order;
2. family/support binding in M0 profile order;
3. material in M0 profile order;
4. geometry-cell index in ascending role-owned order;
5. contact in M0 profile order;
6. P1 modal ordinal in ascending order.

The canonical row ID is
`<role>/f<family-index>/m<material-index>/g<cell-index>/c<contact-index>/o<ordinal>`.
Object-group identity omits contact and ordinal. A material is passed to P1 as
`synthetic-elastic-reference`; its M0 index and physical values remain in the
row identity/features. Geometry values are Decimal products of the selected P0
base fixture and M0 multipliers. No forbidden identity field enters a branch
feature.

The owner materializes all train rows, fits the three controls and candidate,
then materializes development exactly once. It may create
`candidate-freeze.json` only after every development and hard gate passes.
Only that immutable freeze permits one materialization of method holdout.
Failure before freeze leaves method-holdout rows at zero.

## Control mechanics

All three controls are fit and evaluated independently per output branch.

- Identity predicts exact binary64 zero for every row.
- Ridge uses the branch-local M0 feature vector. It centres train features and
  target using arithmetic binary64 means, solves
  `(Xc.T @ Xc + 1e-6 I) beta = Xc.T @ yc` with `numpy.linalg.solve`, and uses
  the unregularized intercept `target_mean - feature_mean @ beta`. No output
  clipping is applied to a control.
- Nearest uses squared Euclidean distance over the unchanged branch-local
  normalized features. It scans train rows in canonical row-ID order; exact
  distance ties select the lexicographically smallest row ID. Its prediction
  is that train row's oracle target for the same branch.

The control report retains per-branch RMSE and the aggregate defined below.
The best control on method holdout is the minimum measured RMSE among identity,
ridge and nearest independently for each comparison; this comparison does not
select or alter the frozen candidate.

## Candidate training and serialization

The owner uses the M0 NumPy-PCG64 initialization, three independent float64
PyTorch heads, full-train batch, 1,200 AdamW steps, inclusive fixed learning
rate, weight decay and global gradient-norm clip exactly as frozen. CPU intraop
and interop thread counts are one. Deterministic algorithms are required.

Candidate weights use a repository-owned binary envelope: fixed magic and
version, then lexicographically sorted tensor name, rank, dimensions and
contiguous little-endian float64 values. Development predictions use fixed
magic and version, canonical row IDs and three little-endian float64 values in
`decay`, `global_gain`, `contact` order. Framework checkpoint containers,
timestamps and pickle are forbidden.

## Metrics and ablations

For branch `b`, `RMSE_b = sqrt(mean((prediction_b - oracle_b)^2))` over the
complete role. The normalized aggregate is the arithmetic mean of
`RMSE_decay/0.20`, `RMSE_global_gain/0.16` and `RMSE_contact/0.22`; these are
the frozen oracle bounds, not learned output ranges.

Ratios use direct binary64 division. If a denominator is exact zero, the ratio
passes only when the numerator is exact zero; the reported ratio is `0.0` in
that case and otherwise the gate fails with `zero_denominator_violation`.

Development ablations do not retrain:

- material-zero sets decay/global-gain feature indices `4..7` to exact zero,
  evaluates the trained heads and reports
  `mean(ablated_decay_rmse, ablated_global_gain_rmse) /
  mean(full_decay_rmse, full_global_gain_rmse)`;
- contact-zero sets contact feature indices `9..11` to exact zero, evaluates
  the trained contact head and reports `ablated_contact_rmse / full_contact_rmse`.

Every M0 development and method-holdout inequality is conjunctive. Equality at
a maximum/minimum threshold passes.

## Hard physics-locked checks

For every materialized candidate role the owner requires:

- finite corrections inside the exact M0 bounds;
- P1 frequency and ordinal byte values unchanged by composition;
- positive corrected decay;
- exact nodal zero and non-zero signed-gain sign preservation;
- P1 common-vertex remesh equality for every case;
- exact linear `0.5`, `1`, `2` impulse-response ratios without refitting;
- a direct 144,000-frame, 48 kHz corrected render with peak strictly below
  `0.95` for every development and, if opened, method-holdout case;
- unchanged hash identities for the T0 mutation and V0a validator releases.

Train rows are checked for finite features/targets and P1 construction, while
the expensive corrected-render peak gate is evaluated on protected evaluation
roles. A hard-gate failure is terminal and cannot be converted into a metric
reject or fallback pass.

## Publication and terminal decisions

Each process writes to a fresh external directory through staging and atomic
rename. Common files are exactly:

1. `corpus-manifest.json`;
2. `control-report.json`;
3. `candidate-weights.bin`;
4. `development-predictions.bin`;
5. `evidence.json`;
6. `report.json`.

`candidate-freeze.json` exists only after development pass.
`method-holdout-report.json` exists only after that freeze and holdout access.
No other file is published. Total file count is therefore six or eight and
the complete output remains inside the M0 64 MiB bound.

The terminal decision is exactly one of:

- `PASS_KNOWN_TRUTH_TOURNAMENT`;
- `REJECT_DEVELOPMENT`;
- `REJECT_METHOD_HOLDOUT`;
- `REJECT_HARD_GATE`;
- `REJECT_RESOURCE`;
- `REJECT_NONDETERMINISTIC` at the A/B comparison boundary;
- `CONTRACT_REJECT` when input/dependency/publication validation fails before
  a scientific decision.

Only `PASS_KNOWN_TRUTH_TOURNAMENT` permits Roadmap V32 to await real S1 roles.
Every reject closes this compact residual family. All outcomes retain the P1
classical owner as research machinery and authored clips as product fallback.

## Resource and access accounting

The owner records deterministic within-limit flags, output byte count, role
row/case/group counts, network requests and real/protected signal values in its
artifact closure. The outer execution harness records the actual monotonic wall
duration and process peak RSS for each fresh process. Operational wall/RSS
numbers are deliberately excluded from byte-exact artifact preimages; their
per-process pass/fail flags are included. Network requests and real/protected
signal values must remain exact zero. Wall time must not exceed 300 seconds,
peak RSS must not exceed 1 GiB and output must not exceed 64 MiB. A resource
failure is terminal; thresholds, roles and steps are not reduced.

Generated corpus rows, weights, predictions, reports and WAV-equivalent
renders remain external artifacts. Git receives only owner code, tests,
profiles, protocol and the bounded final result summary.
