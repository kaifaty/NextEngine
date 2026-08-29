# NSR3-B4E2D7R20R7 iterative semismooth evidence

Status: `FROZEN NEGATIVE / S3 ERROR-ENCLOSURE REJECTION`.

## Frozen result

Implementation `96af8d88` repeated with stable semantic:

```text
8ca318ee6328d44e3a4a4f6959849d12b6e6a5da79fe271f8a00192b1f2f1c58
```

Emitted route and harness status:

```text
FAIL / SEMISMOOTH_ACTIVE_SET_REJECTED
```

The strict failure is intentional: the frozen active-set contract refuses to
classify a passive coordinate whose forward-error interval crosses zero. All
parent, analytic-control and lifecycle gates pass.

## Successful convergence evidence

- Four quiet transfer/negative cases certify at iteration zero.
- Supported certifies after one unit Newton step.
- Filled edge certifies after eight accepted iterations. The first four steps
  cross 47 projector coordinates in total; the final four accept unit steps.
  Its terminal tuple is approximately:

```text
primal             6.57e-33
dual mapping       1.89e-33
complementarity    7.98e-35
stationarity       0
scaled gap         3.13e-30
```

Thus cold-start relinearized semismooth/NNQP convergence is demonstrated on
one of the two formerly unresolved filled cases.

## Localized corner rejection

Filled corner accepts five certified exact-dual ascent steps. On iteration six
its natural face contains all 64 density rows. The single-pivot NNQP reaches a
60-row passive set after 88 transitions/solves, then reports:

```text
reason             PASSIVE_SIGN_UNRESOLVED
source row         53
computed value     +9.227e-2
forward bound      2.133e-1
```

The value is positive, but the current bound cannot prove its sign. That bound
uses a recursive absolute row-sum estimate of `||L^-1||`, which can be grossly
conservative for a large correlated principal system. The result does not
prove a singular Hessian or an invalid active set.

## Decision

Preserve R7 unchanged. Add a separately contracted, on-demand verified inverse
residual enclosure. If `Q` approximates `H^-1` and
`R=I-HQ` satisfies `||R||_inf<1`, then

```text
||H^-1||_inf <= ||Q||_inf / (1-||R||_inf).
```

Use this tighter bound only when the cheap sign enclosure is unresolved. Do
not change the principal solution, pivot order, sign target or nonlinear
line-search policy.

No generalization, runtime, GPU or production authority exists.
