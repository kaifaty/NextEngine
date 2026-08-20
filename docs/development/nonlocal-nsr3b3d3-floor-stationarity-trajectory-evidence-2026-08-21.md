# NSR3-B3D3 floor-stationarity trajectory evidence -- 2026-08-21

Status: `FAIL / ONE_TRIAL_POLICY_REJECTED / NO_B3_RETRY`

The frozen
[D3 contract](../plans/nonlocal-nonlinear-solver-research/03b3d3-floor-stationarity-trajectory-contract.md)
was executed twice byte-identically. The selected D2 action is useful but not
sufficient as a full-trajectory policy: every row eventually reaches a later
floor state whose single trial does not close the reaction gate.

## Accepted prefix

Before the first new failure, the bounded merit path accepts between one and
121 floor trials. Every accepted trial retains topology, strictly reduces the
residual, finishes inside the unchanged reaction limit, and stays inside the
frozen aggregate HVP cap. Completed prefixes also retain active
stationarity-ratio `<=1`, exact contact reaction and the displacement-based
inactive arithmetic bounds.

| Fixture | substeps/frame | completed | accepted floor trials | first failed substep |
|---|---:|---:|---:|---:|
| face | 96 | 294 | 121 | 294 |
| face | 192 | 370 | 34 | 370 |
| face | 384 | 677 | 1 | 677 |
| corner | 96 | 337 | 5 | 337 |
| corner | 192 | 725 | 61 | 725 |
| corner | 384 | 1332 | 1 | 1332 |

This confirms that D2 did not select a spurious one-state mechanism, but it
also falsifies the stronger claim that one residual trial is always enough.

## Two distinct later failures

The first failed floor states separate into two regimes:

| Fixture / level | current defect / limit | trial defect / limit | trial/current |
|---|---:|---:|---:|
| face / 96 | `5.63e-7 / 1.11e-11` | `4.91e-11 / 1.11e-11` | `8.72e-5` |
| face / 192 | `8.00e-8 / 2.87e-12` | `8.21e-12 / 2.87e-12` | `1.03e-4` |
| corner / 96 | `2.01e-7 / 2.59e-12` | `2.38e-11 / 2.59e-12` | `1.18e-4` |
| corner / 192 | `2.94e-7 / 8.47e-12` | `2.69e-11 / 8.47e-12` | `9.13e-5` |
| face / 384 | `1.51e-12 / 1.02e-12` | `3.17e-12 / 1.02e-12` | `2.10` |
| corner / 384 | `1.24e-12 / 6.06e-13` | `1.48e-12 / 6.06e-13` | `1.20` |

The coarse/mid states get a `~1e4` residual reduction but need another
iteration. The finest states overshoot and increase residual, despite exact
active/pair topology and no negative-curvature exit. Blindly permitting more
full residual steps would therefore be unfounded.

## New consistency question

D1 owns `delta` for velocity and reaction measurement, but the inherited
smooth evaluation still forms inertia from materialized positions
`(y-y*)`. At sub-ULP corrections this is not the same binary64 state as
`(delta-delta*)`. The HVP is unchanged, but the gradient used to compute the
trust step may no longer be the gradient whose impulse residual D3 is trying
to close.

The next diagnostic must compare, at these six exact failed states:

```text
h * sum(legacy smooth gradient)
h * sum(displacement-owned smooth gradient)
measured reaction residual
```

and recompute a single trust step with the displacement-owned inertia gradient.
Only after this identity is checked may a bounded residual iteration or line
search be designed.

## Repeatability

```text
D3 raw SHA-256 (two identical runs):
aa16501e6402c5a4170cec7f58d030506d564362dd928d91ca5d6c36463ca792

D3 JSON-without-newline SHA-256:
65737fcabbe8a7d4e926e38bd39b0c4cef20c933b07c19505195641a8a3d1719

D3 semantic SHA-256:
a550a2e9cd6783bd74a022c612236e47542a1e229f2ec3e4432b7663ef3a0071

D2 raw preserved:
0b9581489e133039c7562f938fdb79d73a8810222341f74aa09f61edf668a1ae

D1 raw preserved:
f500187b0fb02c36d4383e2e9fe1003677afc5b2744c02b587fe9f9d86b8587a
```

B3 remains failed. No B3 retry, physical corpus, CUDA, performance, runtime or
production authority is granted.
