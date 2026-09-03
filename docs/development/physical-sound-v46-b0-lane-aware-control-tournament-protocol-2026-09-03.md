# Physical sound V46 B0 — lane-aware control tournament protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| State | `FROZEN_BEFORE_B0_TARGET_ACCESS` |
| Input | Exact D1 index, admitted Clatter prior, historical B0R metrics and C0R generator targets |
| Output | Compatibility matrix, grouped control metrics and one terminal decision |
| Authority | Retain a disclosed control floor or return `NoUsefulTeacher` |
| Excluded authority | Candidate training, material selection, validator/protected access, cooker, demo, public contract and runtime |

## Falsifiable question

Does the D1 corpus contain a target-contract and role-compatible evaluation
lane on which a trusted analytic, empirical or synthetic teacher can be ranked
against the disclosed simple controls without treating filenames, repeated WAVs
or missing Recipe V3 coordinates as independent truth?

B0 is successful as an audit even when the answer is no. It publishes exactly
one of two decisions:

- `ControlFloorFrozen` only if at least one non-real teacher/control family has
  independent observed truth on the same target contract and a score no worse
  than the best simple disclosed control;
- `NoUsefulTeacher` when no analytic, empirical, synthetic or structural
  control has such an independently scoreable intersection.

The historical real-acoustic floor is always recomputed as a regression check.
Its existence alone cannot turn Clatter or an analytic formula into a useful
teacher and cannot authorize M0.

## Frozen lane intersections

| Surface | Allowed role/contract | B0 use |
| --- | --- | --- |
| C0R real acoustic | `generator_train` and `generator_development`; `c0r_acoustic_pseudo_target` | Score global prototype, retrieval copy and coarse-material ridge after physical-parent aggregation. |
| C0R validator | `validator_calibration` | Closed. No projection or target value may be read. |
| IETeasy real acoustic | train-only NDAC-75 target | Inventory only. It has no same-contract development project, so its target remains closed. |
| Clatter | `empirical_prior`; five Recipe V3 modal fields | Verify 84 rows collapse to 36 exact modal heads; unscoreable without independent Recipe V3 modal truth. |
| V31 analytic owner | causal plate/beam modal control | Identity/control availability only; unscoreable on C0R because D1 supplies neither its geometry/contact inputs nor a lossless Recipe V3 mapping. |
| External modal teacher | D1 has zero admitted rows | Unavailable after D0. |
| Structural transfer | D1 has zero materialized rows | Unavailable. |

No comparison crosses the C0R 105-dimensional pseudo-target, IETeasy NDAC-75
or Recipe V3 contracts. A missing coordinate is not zero, a nominal material
label is not a geometry descriptor, and one random Clatter render is not a new
physical parent.

## Simple-control computation

The C0R adapter preserves the frozen V44 representation:

- 24 log spectral bands, 48 transient-envelope bins, 24 modal-histogram bins
  and nine global features (`105` dimensions total);
- target values are aggregated arithmetically per physical parent before fit
  or evaluation;
- the scaler uses generator-train parents only;
- unknown material contributes to the unconditional scaler/global prototype
  but not to conditioned retrieval or ridge;
- the three allowed controls are the masked train-parent global prototype, the
  lexicographically frozen same-coarse-material retrieval copy and the fixed
  `alpha=1` coarse-material one-hot ridge;
- development error is masked train-standardized RMSE, reported per parent,
  category, material and source project;
- ranking is median supported-parent RMSE, then project-balanced mean RMSE,
  then control name.

The owner must reproduce the exact corresponding V44 B0R metrics. This is a
regression assertion against an already disclosed result, not a new threshold
chosen after B0 values. The expected winner is therefore not used to select a
new model family.

## Inputs and pre-access seal

The canonical profile binds six flat external JSON inputs by filename, byte
count, SHA-256 and schema:

1. D1 access ledger, corpus index and report;
2. V45 C0 Clatter `prior.json`;
3. V44 B0R `baseline-metrics.json` and report.

It also binds the tracked B0 owner/protocol, SPEC-45, D1 result/profile, V41/V44
baseline code/profile/result and the V31 analytic-owner code/result. Mutable
roadmaps and external directory names are not hash dependencies.

Only after this profile, owner and focused tests are committed may an official
B0 process receive the external C0R corpus root. The owner validates that root
is outside the repository and opens only content-addressed targets named by D1
C0R train/development rows. It does not scan for additional targets.

## Access rules

The owner records exact metadata and target bytes/files read. The following
remain zero:

- network requests;
- PCM/waveform bytes and samples;
- IETeasy target bytes;
- C0R validator projection/target bytes;
- protected values;
- model/checkpoint values;
- candidate training steps;
- generated render or cooker bytes.

Clatter numeric values are already admitted control metadata. Reading them can
verify exact grouping but gives no real-parent, synthetic-teacher or validation
credit. Output is atomic, flat and external; failure publishes nothing.

## Required gates

1. Every tracked and external binding matches exactly.
2. D1 decision, authority, counts and zero synthetic/structural rows match the
   frozen state.
3. C0R train/development rows are disjoint by record and physical parent and
   agree with their D1 roles, contracts and target references.
4. Each parent belongs to one source project and one material-mask state.
5. Exactly `64/70` C0R train/development records become `34/30` parents; 24
   development parents have train-supported coarse material.
6. Exactly 134 unique C0R target objects and 682,733 bytes are read; no other
   content object is opened.
7. The three simple controls reproduce their frozen V44 parent metrics and
   global-prototype remains the real disclosed floor.
8. All project-balanced summaries use equal project weight after parent-level
   errors; no WAV-level vote exists.
9. Clatter still has 84 rows and 36 canonical modal groups matching D1.
10. Lane compatibility is explicit and the terminal decision follows the
    frozen mapping without candidate fitting or post-hoc substitution.
11. Two clean runs publish byte-identical files.

## Consequence

`NoUsefulTeacher` closes B0 without calling the lab a failure: the numeric real
floor remains useful, but there is currently no independently scoreable source
that can supervise M0. The next authorized work is E0 real-source power and V0
validator-source growth, or a successor D0/X0 source with exact generation or
structural lineage. M0, real fitting, protected admission, cooking, demo and
runtime remain closed. Authored clips remain mandatory.
