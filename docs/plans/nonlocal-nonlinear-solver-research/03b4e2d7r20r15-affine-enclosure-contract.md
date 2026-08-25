# NSR3-B4E2D7R20R15 affine-enclosure contract

Status: `FROZEN / REPORT-ONLY SHADOW AUTHORIZED`.

## Parent

- R14 implementation `efcd8e60`, semantic
  `8707531a56963b5ce0a456e33b27d0b831a76061d354188b9a1232d9f959d30c`;
- inverse audit roots `d1ea3272...c217`, `e9110976...b4d`;
- R13 case and step roots unchanged.

## Frozen shadow

Initialize `affine_error=0`. Preserve the original `direction_error` and every
consumer exactly. Update only the unused shadow:

```text
negative.empty(): affine_error = passive_error
boundary update:  affine_error = (1-alpha) affine_error
                               + alpha passive_error
                               + existing_rounding_bound
```

At each update require finite, nonnegative `affine_error <= direction_error`.
At the original `RATIO_ORDER_AMBIGUOUS`, reuse the already report-only verified
inverse and re-evaluate ratio ordering from `affine_error` plus its refined
candidate error. Record final old/shadow errors and ratio bounds.

Require exact R13 case/step roots and exact R14 inverse roots. Classify
`AFFINE_SHADOW_RESOLVES_ALL`, `RESOLVES_SUBSET`, `RESOLVES_NONE` or
`SHADOW_REJECTED` without changing a solver decision. No state update, timing,
solver/runtime/GPU or production authority.

