# NSR3-B4E2D7R19R10 total-HVP boundary evidence

Date: `2026-08-23`

Status: `PASS / TOTAL_HVP_BOUNDARY_SAFE_PROGRESSING / PASSIVE ONLY`

## Outcome

The unchanged global `512`-HVP limit interrupts a safe, still-progressing
ordinary trust recurrence. It does not coincide with a numerical failure,
nonpositive curvature, trust boundary or already-satisfied forcing test.

The exact interruption is outer `5`, trial `1`, guarded-policy solve `20`.
Fourteen recurrence HVPs have completed. All fourteen iterations are finite,
positive-curvature and interior, and every observed next-residual ratio
strictly decreases from `0.531503` to `0.00391546`.

The prefix is not yet near forcing: `eta=1.0314622869e-4`, so its final
residual ratio is still `37.9603 eta`. R10 therefore supports a separate
offline continuation study, not a live cap increase and not a one- or two-HVP
grace assumption.

## Parent and target identity

```text
R10 identity SHA-256  52f069f26f5c47f0ea204b227765cea108970da1a87a553173f2655ad42b9fa7
R9 stdout SHA-256     f1cb461d270e1642e9c0220c60fc59c64cb5bb69039f293ed8e9eade7f6ed1f0
R9 semantic           40e152b8f45f2d2b7b654d0bc629aa76ed160f6c9a9467db1097a5fef877934d
R9 transaction root   c6a03d43a04ba00cef80b618bb983ab23f59e27d79ad073f327f119b145660cf
R8/R7/R6/R5/R2        retained exact
```

The R10 command obtains its capture from the same execution that produces
the exact R9 parent report. The ordinary R9 command was also checked after
the refactor and retained its exact `3,567` bytes and stdout SHA above.

## Exact interrupted solve

```text
outer / trial / solve  5 / 1 / 20
boundary root          002f3b63423de2e0db8715b599e064043502c3881ba89cb55aaf7394bee850df
recurrence root        a900c9452fa173f1c60fdf7c670de747653742348340056f05d7336cbaaa974c
termination            BUDGET
captured HVP            14
theta bits              0x3fc5cccccccccccd
radius bits             0x3f8999999999999a
current root            cefab23603e8cb5b773b346d896a15b6cc3c28516acb747c4b8652cec27aae63
gradient root           51388c4bffce4fd3213844e51510b2ebfdfbad4c8db38baef35005edfcb62f95
dual root               fff30a51545a3ed99ac71070b1279e725806e93ee4528ded93e0ef15efb0030d
```

The residual sequence is:

```text
HVP  1  0.5315034571
HVP  2  0.3521260626
HVP  3  0.2395533145
HVP  4  0.1567533267
HVP  5  0.1093023383
HVP  6  0.0794993115
HVP  7  0.0534123152
HVP  8  0.0346744989
HVP  9  0.0250880305
HVP 10  0.0170731744
HVP 11  0.0113546489
HVP 12  0.0082131057
HVP 13  0.0056694721
HVP 14  0.0039154603
```

The early/late eight-value windows have minima
`0.0346745/0.00391546` and medians `0.133028/0.0142139`. Both improve, and
the late window strictly decreases. The frozen classifier therefore selects
`TOTAL_HVP_BOUNDARY_SAFE_PROGRESSING`, not `SAFE_NEAR_FORCING`.

## Outer progress and work distribution

Completed outer states show monotone primal progress:

| Outer | Recorded trials | Inner HVP | Stationarity | Primal | Position update / dx |
|---:|---:|---:|---:|---:|---:|
| 0 | 6 | 118 | `2.03e-14` | `7.4925e-8` | `1.9804e-7` |
| 1 | 4 | 111 | `1.94e-14` | `5.2356e-8` | `5.6684e-8` |
| 2 | 3 | 81 | `2.29e-13` | `4.1520e-8` | `2.9318e-8` |
| 3 | 3 | 82 | `4.24e-14` | `3.4972e-8` | `1.9530e-8` |
| 4 | 3 | 82 | `1.67e-14` | `2.9914e-8` | `1.4440e-8` |
| 5 | 1 + interrupted | 38 | `1.0639e-8` before interrupted solve | unavailable | unavailable |

Outer `5` has no completed final outer state; its zero-valued final-state
fields are deliberately marked unavailable and are not evidence of primal or
dual progress.

The complete HVP accounting is:

```text
completed trial recurrence  480
interrupted recurrence       14
completed direct models      18
total                        512
```

This is independently equal to both the completion trace
`494 recurrence + 18 direct model` and the sum of per-outer inner HVPs.
There are `20` accepted, zero rejected and two residual-model trials.

## Passive-capture proof

At the capture boundary one workspace is live. The final R9 state has the
same HVP, build, precision, pair-visit and all-pair counters; only that
workspace has been released:

```text
workspace build/release final  31 / 31
live at capture / final          1 / 0
maximum live                     2
all-pair candidate calls         0
new HVP/model/trial/audit        0 / 0 / 0 / 0
```

Thus R10 observes the exact boundary without continuing or changing the
transaction.

## Reproducibility

Implementation commit:
`0cbd6dee` (`research: diagnose total HVP boundary`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r10-a.J5Aq1u`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r10-b.bCkeme`.

Both binaries are `6,048,240` bytes, have SHA-256
`873f1adf777ec2fba2a92c72864f94b37b25d8486ffe31d5817b063f66f55981`
and GNU build ID `8ca428f5d3df32719bfaa104ba0a4772b67eb295`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r10-a.03gNIk`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r10-b.SzvYWk`.

Both exit `0`, emit empty stderr and reproduce byte-exact:

```text
stdout-with-LF bytes  27,165
stdout SHA-256        15719465931a21421e094abde1200d685bee44e4b112dc314d5857f863074953
semantic result       30f339247a76484071494b26db54c385a0bea637c4f9e744628a0ff83371b30c
route                 TOTAL_HVP_BOUNDARY_SAFE_PROGRESSING
```

No CPU/wall comparison, second substep, macro or trajectory was run.

## Decision

Preserve the live total budget at `512`. Research D7R19R11 as a replay-only
continuation of the captured solve:

- reproduce the exact first-14 prefix and all input/static roots;
- continue only that recurrence under the existing offline cap of 128 HVPs;
- classify nonfinite, curvature, boundary, forcing convergence or cap;
- expose recurrence/Ritz diagnostics without model, trial, precision or
  transaction continuation;
- use the result only to design a later global-budget-policy discriminator.

R10 improves diagnosis but does not make the Nonlocal solver production-ready.
