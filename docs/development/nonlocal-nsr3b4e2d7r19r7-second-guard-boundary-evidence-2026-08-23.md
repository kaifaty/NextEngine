# NSR3-B4E2D7R19R7 second guard-boundary evidence

Date: `2026-08-23`

Status: `PASS / SECOND_GUARD_OFFLINE_FORCING_CONVERGED / REPLAY ONLY`

## Outcome

The first later R6 guard denial is reproduced exactly and explained. Its
first 32 recurrence HVPs are finite, positive-curvature and interior; the last
eight next-residual ratios strictly decrease. The denial is owned only by the
upper ratio clause: `ratio_after_32 = 1.891245028190834 * eta`, outside the
frozen `1.25 * eta` window.

An exact offline replay reproduces all first-32 iteration projections and
then converges on HVP 34. HVP 33 alone is insufficient: its residual remains
`1.294361573084484 * eta`; HVP 34 reaches
`0.9334121973834975 * eta`.

The result validates the R6 denial and rejects an unconditional one-HVP grace
as a general solution. It does not authorize a two-HVP runtime grace.

## Parent and target

```text
R6 stdout SHA-256  67dfb6781a5840e228b63f3524bf4999ca6d1d9b650f8c461117575a8172611c
R6 semantic        a2687bac0ba18b58ca4903047de6645c389174512452dad82c5e0d46320f5811
R5 retained        true
R2 retained        true
```

Captured target:

```text
capture root    97c340ed27b33318fb4d6684a7d385380ddc15e3a8b0358d0c86feb189f48ab3
outer           1
trial           3
solve index     9
prior guard     one convergence
current root    96294a6434bab090158a1fd5dda13f2e17482b39e1abfeedbb40b81f1645d7e9
predicted root  36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e
gradient root   5bd82926c108fb1cc8de074a03d493e9af960030c80573d7edda44e3cd49f083
radius          0x3f8999999999999a
live prefix     adf2edcc243060a540d515f6d8687a2b2b0f0e9e2131fbc402f06d9b50b85d08
```

The passive capture occurs at outer `1`, trial `3`, guarded-policy solve `9`.
It observes the exact R6 boundary at 227 total HVPs, 12 workspace builds and
nine precision audits. At capture time one workspace remains live; R6 then
releases it and preserves its exact report bytes.

## Guard decomposition

```text
prefix finite/positive/interior  true
ratio after 32                   2.8508865145849365e-5
eta                              1.5074125626715307e-5
ratio / eta                      1.891245028190834
ratio > eta                      true
ratio <= 1.25 eta                false
last eight strictly decrease     true
denial clause                    RATIO_ABOVE_WINDOW
```

There is no live convergence contradiction, curvature failure, trust-boundary
contact or trend failure. The guard correctly withholds HVP 33 under its
frozen one-shot evidence envelope.

## Offline recurrence

```text
offline prefix root  adf2edcc243060a540d515f6d8687a2b2b0f0e9e2131fbc402f06d9b50b85d08
offline trace root   bd739c698affbcfcd6daa0cb7233ef1c8ed517086934a202911129b070ef8d50
offline step root    35054f8939760fe7baaa4d03250b7d14ec82a284a4f01d44a15cacff6a6b0cef
HVPs                  34
termination           FORCING_CONVERGED
ratio after 33        1.9511368959068355e-5 = 1.294361573084484 eta
ratio after 34        1.4070372724867225e-5 = 0.9334121973834975 eta
```

Every offline curvature is positive and every iterate remains interior. The
maximum residual-orthogonality and adjacent A-conjugacy diagnostics are
`1.97e-14`/`2.28e-14`; the CG-derived Ritz estimate is:

```text
dimension   34
minimum     1.0173657373772955
maximum     36.42732860061941
condition   35.80553901345917
```

This does not justify a preconditioner for the exact barrier.

## Work and safety

The R7 lane builds/releases exactly one offline workspace, consumes 34
diagnostic HVPs and performs zero model HVPs, precision audits, trial
formations, acceptances or all-pair candidate calls. Static binding, gradient
identity, finite values, lifecycle and rollback all pass. No candidate
substep, transaction continuation, state commit or timing lane runs.

## Reproducibility

Implementation commit:
`a9271a2f` (`research: replay second guard boundary`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r7-a.OrlDSO`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r7-b.lDWDDw`.

Both binaries are `5,935,384` bytes, have SHA-256
`d99fc415f226eb3cd4aae321c5ad715dae1b588b9cdf2c02198487b7ddaf1e2d`
and GNU build ID `a009e2d46fbef0a30ac8030e12fe2ac6acb4c899`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r7-a.p2ZQzs`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r7-b.RxDDvK`.

Both exit `0`, emit empty stderr and reproduce:

```text
stdout-with-LF bytes  23,701
stdout SHA-256        db0e5e733b57c9d6b5ffe0ad9fb641de1cee78e2fd05fe8cbf890fcae591fcdb
semantic result       c8f350b941e57910b1dc5dc93e6a357319d22de6f2fb16fb3a338592f6596c79
route                 SECOND_GUARD_OFFLINE_FORCING_CONVERGED
```

## Decision

Preserve R6 and its one-shot guard exactly. Research a replay-only
two-boundary progress-envelope discriminator before changing any transaction
policy:

- the original boundary must retain its one-HVP convergence;
- the later boundary may study a second grace HVP only after HVP 33 remains
  finite, positive, interior and makes independently bounded progress;
- HVP 34 must converge;
- residual-derived `H(step)` at the 34-HVP solution must be compared with one
  direct oracle before any trial is formed;
- no full transaction, trial, cap change or performance lane is authorized.

This separates the mathematical question “can a bounded online certificate
justify two HVPs here?” from the policy shortcut “raise the cap to 34”.
