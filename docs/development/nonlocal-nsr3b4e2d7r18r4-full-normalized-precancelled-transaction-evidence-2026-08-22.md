# NSR3-B4E2D7R18R4 full normalized precancelled transaction evidence

Date: `2026-08-22`

Status: `FAIL / WORK_LIFECYCLE / NORMALIZED_KRYLOV_FORCING_RESEARCH_REQUIRED / D7R19_BLOCKED`

Implementation commit: `7e12d116ca0d58d5c9fb4d3fc6dafd8743fc4332`.

## Result

R4 deterministically fails the frozen exact-work control:

```text
first_failure = WORK_LIFECYCLE
route         = <none: hard control failed>
```

This is not an inner, outer, sign or state-confirmation failure. All five
transactions reach the expected confirmation/holdout semantics with exact
cross-profile roots, but every active run uses 39 HVPs instead of the frozen
D7R13 baseline of 38. D7R19 remains blocked.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r18r4.EUNGdr`.

Two independent clean GCC 15.2 Release builds produce byte-identical
5,585,272-byte executables:

```text
SHA-256  23d8c1101ec2afc910b54228050dbab8b87201a9c032d6d3b5748e84e1a8b74b
Build ID bfcf692a54e3b05ca5ffbdf3281f2db99ec8b8fb
```

Build directories:

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r4-a.dlDkuD
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r4-b.fF9QVq
```

One R4 process from each build exits one with empty stderr and emits the same
9,386-byte stdout:

```text
stdout SHA-256 32e4369a19564c769148c9d0bc534bce8f0d22dac4a0eeb085aa191a46dc38c6
semantic result 91b311aa592a9d5e8f6845c80d98abec98329c1a00c1eb80b559a2d77c1d49e0
```

Both builds directly preserve all frozen parent bytes:

```text
D7R18R3 e0b36e34a047de1d8bb1435fa6f68cf5a53ab8a4260a95719775c03e4608d3e5
D7R18R2 3659eac888c22eae5bcf7ae8c5f8a426bcbc12bd24bd3320c23be0c176815d77
D7R13   514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de
```

## What succeeds

Reference repeat and reference/aligned active and inactive roots are exact.
Every active transaction reaches provisional `11`, confirmation `12` and
admissible holdout `13`; every inactive transaction reaches `0/1/2`. Each
active run has 19 accepted trials, zero rejected trials and four
candidate-effect binary128 audits. Each inactive run performs no trial or HVP
work.

All 57 accepted trials receive direct normalized long-double audits. Fifty-four
resolve positive, three are unresolved and none resolves negative. All 12
candidate-effect pairs resolve positive in direct normalized binary128 and
pass the 5% magnitude bound. Pair membership, static binding, invalid prework,
workspace lifecycle, pair-union accounting, nonnegative dual state and forced
rollback all pass. All-pair candidate calls remain zero.

## First failing boundary

The only exact-work difference occurs in each active run at outer `11`, trial
`0`:

| Quantity | Value |
|---|---:|
| stationarity before | `1.5347113061952662e-10` |
| step norm | `3.630345433109899e-12` |
| predicted reduction | `3.939590151509977e-23` |
| divided reduction | `3.939799657019704e-23` |
| HVP calls | `3` |
| raw would accept | `false` |
| R2 divided would accept | `false` |
| binary128 candidate effect | resolved positive / within bound |

Every other accepted trial uses two HVP calls. The full ledger is therefore:

```text
expected 48 outer / 57 trials / 114 HVP
observed 48 outer / 57 trials / 117 HVP
```

The `+3` total is exactly one additional Krylov HVP in each of the three
active transactions. It does not change the accepted-trial count or any
confirmation state.

## Mechanism hypothesis

The normalized Steihaug implementation inherits the dimensional stopping
rule:

```text
||r_next|| <= min(0.5, sqrt(||r_initial||)) * ||r_initial||.
```

This rule is not invariant under multiplying the objective by a positive
constant. If `gbar = alpha*g`, both residual norms scale by `alpha`, but the
small-residual right side scales by `alpha^(3/2)`. For the reference mapping
`alpha=dt^2/M=1/7200`, the normalized relative forcing becomes approximately
`sqrt(7200)=84.85` times stricter than the dimensional one. That can explain
why the first CG iteration does not stop at the one observed boundary while a
second iteration does.

This is a falsifiable mechanism, not yet permission to alter the forcing term.
The R4 result also shows that merely changing the frozen HVP expectation from
38 to 39 would be a post-observation relaxation and is prohibited.

## Decision

Preserve R4 as an exact negative work-correspondence result. Do not retry it
unchanged, change `114` to `117`, or open D7R19.

Research/freeze one replay-only R4R1 discriminator at active outer `11`, trial
`0`. It must expose the initial and first-iteration CG residuals and compare
the inherited dimensional-scale-dependent forcing with an explicitly
dimensionless forcing definition. It cannot accept a trial or rerun the full
transaction with a changed policy.
