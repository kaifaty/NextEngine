# NSR3-B4E2D7R19R11 total-budget offline replay evidence

Date: `2026-08-23`

Status: `PASS / TOTAL_BUDGET_OFFLINE_FORCING_CONVERGED / REPLAY ONLY`

## Outcome

The recurrence interrupted by R9's global HVP budget is a healthy ordinary
Krylov solve. An exact offline replay reproduces every one of the 14 captured
iterations and reaches dimensionless forcing convergence on HVP 24.

Therefore the HVP-512 boundary omitted exactly ten recurrence HVPs from this
solve. This is not evidence for adding an unconditional ten HVPs to every
solve or every transaction. It is evidence for studying one bounded atomic
completion of the already-started solve.

## Parent and target identity

```text
R11 identity SHA-256  9cf1761428463b72901c61503466128576e113b9a75f21da3149dda8a7af7a40
R10 stdout SHA-256    15719465931a21421e094abde1200d685bee44e4b112dc314d5857f863074953
R10 semantic          30f339247a76484071494b26db54c385a0bea637c4f9e744628a0ff83371b30c
boundary root         002f3b63423de2e0db8715b599e064043502c3881ba89cb55aaf7394bee850df
live recurrence root  a900c9452fa173f1c60fdf7c670de747653742348340056f05d7336cbaaa974c
```

R9/R8/R7/R6/R5/R2 remain transitively exact through the R10 parent.

The live and offline first-14 prefix roots are identical:

```text
b3165d58a442deccd262812fdafb4d385337a7dc81ceddf2200dde890bdc5768
```

The rebuilt sparse workspace reproduces the captured gradient root
`51388c4b...62f95` exactly and retains the same current, predicted, dual,
theta, radius and static-support identities.

## Offline convergence

```text
termination                 FORCING_CONVERGED
recurrence HVP              24
new tail after live prefix  10
offline trace root          0fb21740e7137234fe17332d451b0767b78ae3fc636a98a829ea175c482458e0
step root                   e63fd50c794d9c6679e9a3d63beb5105e05fbe0a56f7bd128465a9ba69993983
forcing eta                 1.0314622868968171e-4
final residual ratio        8.7450766826289097e-5
final / eta                 0.8478329061296769
```

The continued ratios are:

```text
HVP 15  2.6233651e-3
HVP 16  1.8611968e-3
HVP 17  1.2431107e-3
HVP 18  8.4216624e-4
HVP 19  5.8142015e-4
HVP 20  3.9498755e-4
HVP 21  2.8309418e-4
HVP 22  1.8268955e-4
HVP 23  1.3170322e-4
HVP 24  8.7450767e-5  forcing converged
```

All 24 iterations are finite, positive-curvature and interior. Maximum
residual orthogonality is `8.45e-15`; maximum adjacent A-conjugacy is
`1.01e-14`. The 24-dimensional Ritz estimate is:

```text
lambda min  1.1094833759
lambda max  29.5916543877
condition   26.6715617662
```

There is no conditioning or preconditioner signal at this boundary.

## Work and scope

R11 consumes one offline workspace and 24 recurrence HVPs. Fourteen are
replay correspondence work; ten are the diagnostic continuation. It consumes
zero model HVP, trial, acceptance, precision audit or all-pair candidate call.
Workspace build/release is `1/1`, maximum live is one and rollback is exact.

No transaction continuation, candidate/second substep, macro, trajectory,
timing or public state mutation was run.

## Reproducibility

Implementation commit:
`80b638c1` (`research: replay total budget recurrence`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r11-a.iG5ELP`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r11-b.T2pSLF`.

Both binaries are `6,078,360` bytes, have SHA-256
`36d7256e646ad6d81c0b9030dccac397baf5f27f3e5cceb8a61d27d4670f10e7`
and GNU build ID `ac601437d28b1621d2cc8bbcc2a555b0ff2de0f7`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r11-a.lh8JHK`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r11-b.6Xr3Ru`.

Both exit `0`, emit empty stderr and reproduce byte-exact:

```text
stdout-with-LF bytes  14,812
stdout SHA-256        bd05f7ac3ca76425dbfda1e43882f48adcb4c83addf1fe4ddeaacd37ca8a8efe
semantic result       be9cc63d2376f4c560498aa9d28c1d44070a40e1ec412a30cbe735fba89305ef
route                 TOTAL_BUDGET_OFFLINE_FORCING_CONVERGED
```

## Decision

Preserve both the live total cap and R11 as offline evidence. Research
D7R19R12 as a one-trial shadow of atomic in-flight completion:

- retain exact R11/R10 and first-14 roots;
- credit only ten post-boundary recurrence HVPs;
- use the HVP-24 step and one ordinary direct `H(step)` model HVP;
- evaluate the existing precancelled divided reduction, precision policy,
  acceptance and radius update once;
- do not commit the trial or continue the transaction.

Only after this shadow is classified can a global soft-cap/completion-reserve
policy be designed. Production readiness remains unproven.
