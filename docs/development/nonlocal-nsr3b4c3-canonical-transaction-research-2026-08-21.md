# NSR3-B4C3 canonical transaction research -- 2026-08-21

Status: `COMPLETE / STAGED_LEVEL_TRANSACTION_SELECTED`

## Reused evidence boundary

NPR1-A remains valid only for checked binary64-to-integer conversion and root
encoding. Reuse:

- integer IEEE-754 decode with nearest-ties-to-even at `1,000,000` units;
- negative-zero normalization, nonfinite/overflow/range failures;
- ascending `SampleId`, duplicate rejection and `50,000` sample capacity;
- canonical frame and trajectory root byte layouts;
- `+/-16 m` position and `+/-64 m/s` velocity research bounds.

Do not reuse NPR1's stopped source-shaped formula, profile root, solver,
trajectory roots or physical authority. B4C3 uses the selected FCR2/joint
pressure identity.

## Adaptive transaction problem

The next substep must consume only decoded published integers, including
inside an adaptive candidate level. But a level is not globally accepted until
the embedded coarse/fine gate passes. Publishing its frames globally as they
are computed would leak discarded trajectories.

Selected ownership:

```text
committed frame start
  -> level-local canonical transaction
       solve substep
       stage canonical frame
       decode stage for next substep
       ...
  -> embedded level gate
       accepted: atomically append selected fine frames
       rejected/discarded: destroy all staged frames
```

Provisional frame step numbers are `committed_count + local_ordinal`. Only the
selected fine transaction advances the committed count. A failed solve,
publication or decode discards the complete level transaction.

## Why B4C3 is split

Canonical roundtrip changes every later binary64 input, so B4B2/B4C2T exact
trajectory equality is no longer valid. Transaction mechanics and long-horizon
physical drift have different failure modes:

- **B4C3A:** one-frame P1/P2 staged level transaction, exact roots/order,
  roundtrip ownership and failure atomicity;
- **B4C3T:** complete canonical adaptive/fixed controller with independently
  frozen physical correspondence bounds.

## New roots

```text
profile  345eb8876aec66fc5a94a8ea1c6cf0f148c7868327bf5d6bafe96058a30d4087
P1       71fd23dd299bc892254007dc7dcaa25898794ff456b0c0dfdbec0271ac3e9884
P2       0bfc8b62d52e479b724824b2c0e5886a6b89faec8891986e13f2c1382c7a7f87
```

The profile root binds `nuv-variational-fcr2`, the B4C2T operator and
`canonical-um-r0`. Scenario roots bind the frozen geometry, gravity, box and
macro-frame count.

## Decision

Freeze B4C3A before any full canonical controller. B4C3A PASS may authorize
B4C3T physical-bound design only. B4C4 packaging and B4D nominal execution
remain blocked.

## Preselected implementation defect

The first implementation run rejected only P2 fine publication because a
binary64 decode/subtract diagnostic measured `5.0000000000050004e-7 m/s` at a
true half-microunit rounding boundary. All transaction, ordering, embedded-gate,
binary correspondence and failure-atomicity checks passed. This is preserved as
negative measurement evidence: decimal `1/1,000,000` is not exactly
representable in binary64, so subtracting the decoded binary64 value is not an
exact test of the quantizer's half-unit guarantee.

The repair evaluates the same frozen `0.5e-6` bound after promoting the original
binary64 input and canonical integer to `long double`; solver arithmetic,
quantizer, profile/scenario roots and discriminator thresholds remain unchanged.
