# NSR3-B4E2D7R20R19 line-frontier contract

Status: `FROZEN / REPORT-ONLY TRIAL AUDIT AUTHORIZED`.

## Parent

- R18 implementation `251e13e7`, semantic
  `6f019b7c314b09406f68438ec4beadfd839b7664952d346a9c898563036d40aa`;
- exact candidate shear root `fdd57c9e...fbb9`;
- late natural face stable, eight positive powers, one mask change, no ball
  change.

## Frozen audit

Replay only candidate shear at cap 32 and require its exact root. For every
existing line trial in steps 25–32 recompute

```text
margin = (new_value - new_bound)
       - (old_value + old_bound
          + 2^-10 alpha (slope - slope_bound)
          + frozen_rhs_rounding_bound).
```

Require `margin >= 0` exactly when stored Armijo acceptance is true. Report
trial power, alpha, mask/ball changes, margin, dual-increase lower and KKT tuple.

For each step define first mask-stable and first accepted powers. Count exact
coincidences, mask-stable rejections and crossing acceptances. Route priority:

1. any stable rejection → `SAME_FACE_MODEL_REJECTION`;
2. at least 6/8 coincidences → `MASK_CROSSING_FRONTIER`;
3. any crossing acceptance → `CROSSING_ACCEPTANCE_TRANSITION`;
4. otherwise `LINE_FRONTIER_UNRESOLVED`.

No trial generation, solver change, timing, cap extension, runtime/GPU or
production authority.

