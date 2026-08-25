# NSR3-B4E2D7R20 v1 operator preflight evidence

Date: `2026-08-25`

Status: `STRUCTURAL PASS / SCIENTIFIC FAIL / CORPUS EXCITATION INSUFFICIENT`.

## Result

The v1 preflight materialized source constraints, the R64 sparse operator,
contact/trust domain and the projected predictor without running candidate or
oracle iterations.

```text
schema    nextengine.nonlocal.nsr3b4e2d7r20_operator_preflight.v1
route     GENERALIZATION_OPERATOR_PREFLIGHT_CANDIDATE
semantic  bfc57b45a3cd74b7b2e635637b1758b7a5866d3daa926e0145477ef36e696f93
stdout    50c9a816aa7d84aae27346ef6ec509c323d2f305ae384dac293d4077873d721a
```

All six operators and lifecycle/scaling checks pass. Positive projected-target
row counts are:

```text
face             0
corner           0
supported       16
released         0
oblique edge     0
opposed corner   0
```

Thus only the supported-column case exercises the inequality solver. Uniform
wall-directed motion becomes feasible translation/contact clipping in the
free-surface cases; their density deficit absorbs the small compression.

## Decision

Reject v1 as a generalization corpus before solver execution. Preserve face,
corner and released cases as transfer/zero-step controls. Reclassify the two
new quiet states as kinematic negative controls. Add two source-frozen filled
box holdouts, whose admission requires positive projected-target rows before
any solver/oracle execution. No threshold, outer count or solver path changes.
