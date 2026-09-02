# Physical sound V34 F0 target-safe profile freeze protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `FROZEN_BEFORE_V34_TARGET_OR_MODEL_VALUES` |
| Parent | [V34 rebaseline](physical-sound-v34-protocol-closure-rebaseline-2026-09-02.md) |
| Failed lineage | [V33 D0](physical-sound-v33-d0-fresh-development-tournament-result-2026-09-02.md), immutable and never retried |
| Claim | `TARGET_SAFE_SYNTHETIC_PROFILE_FREEZE_ONLY / NO_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_OR_PRODUCT_AUTHORITY` |

## Purpose

F0 freezes one new synthetic corpus and the machine-readable proof obligations
that must pass before its targets can materialize. It evaluates no truth
expression, creates no feature/target row, initializes no model and decodes no
real/protected signal.

The canonical profile is an overlay over the hash-bound V33 profile. The
overlay replaces only fresh evidence identities and truth coefficients, adds
the witness/terminal-publication contracts and changes access order. It must
prove that V33's feature, model, control, training, gate, family, count and
resource sections remain canonical and unchanged.

## Frozen fresh evidence

- three new synthetic material rows `synthetic-g/h/i`;
- ten new geometry multiplier cells, none byte-equal to V32 or V33;
- `12 / 6 / 6` train/development/method-holdout contacts, pairwise disjoint and
  disjoint from V32/V33;
- one exact boundary-node contact with `u=0` in every contact set;
- the unchanged V33 role algebra: `648 / 432 / 432` cases and
  `6,480 / 4,320 / 4,320` modal rows;
- fresh contact/decay/global-gain truth expressions, committed before any
  evaluation.

The V34 role identity includes profile ID, role, stratum, family, material,
geometry cell, contact set/index and modal ordinal. Consequently no V33 role ID
can recur even when the role algebra has the same shape.

## Preserved hypothesis

F0 hash-compares these effective sections with V33 and rejects any drift:

- features and normalization;
- `1,811`-parameter spectral candidate and `1,491`-parameter raw MLP control;
- identity, nearest, raw/spectral ridge and raw-MLP controls;
- optimizer, steps, seeds, dtype, device and thread policy;
- development/method-holdout numeric and hard gates;
- P1 family/support declarations, mode count, role plan and corpus counts;
- resource and repeatability bounds.

Changing one of these sections requires a new roadmap; F0 is not a model-tuning
surface.

## Witness contract

For both development and method holdout, C0 must report all three strata and:

- at least `180` exact nodal-zero modal rows per stratum;
- at least one positive and one negative nonzero pickup-signed row per role;
- at least one finite non-silent render witness per family and stratum;
- at least one exact common-contact remesh pair per family and role;
- at least one material-sensitive and contact-sensitive row per stratum;
- complete modal-row participation for frequency/order, decay, bounds and peak
  checks.

Counts and row IDs derive only from P1 structural output. C0 may not import or
call the oracle, target builder, model owner, optimizer or development metric
code. A missing/target-dependent witness rejects before training.

## Terminal publication contract

T0 uses discarded non-official full-shape fixtures. Post-access Pass,
metric-reject, hard-reject and resource-reject outcomes are returned as atomic
data closures; they do not raise. A true pre-access contract reject records
zero target/model access and leaves no partial directory. Every terminal path
repeats twice byte-exactly before D0 becomes authorized.

## F0 exit

F0 passes only if two fresh processes publish identical profile conformance,
evidence and report artifacts, all dependency hashes close, V32/V33 freshness
is exact and this access receipt remains all zero:

```text
train/development/method-holdout target rows = 0
oracle values = 0
feature rows = 0
model parameters initialized = 0
real/protected signal values = 0
network requests = 0
```

Any failure leaves no partial artifact and authorizes neither C0 nor training.
Even a pass grants C0 only; T0, D0 and H0 remain separately gated.

