# NSR3-B3 split boundary-composition smoke evidence -- 2026-08-21

Status: `FAIL / SUPPORT_REACTION_ACCURACY_NOT_ESTABLISHED / STOPPED`

The frozen
[B3 contract](../plans/nonlocal-nonlinear-solver-research/03b3-boundary-composition-smoke-contract.md)
was executed without changing its physical coefficients, operation order,
accuracy limits or per-substep momentum gate. B2 and B1R1 remain selected;
B3 does not authorize the physical corpus.

## Exact first failure

```text
NSR3B3_FACE:
  CONTROLLER:
    FRAME_CANDIDATE:
      SUBSTEP_1:
        MOMENTUM_LEDGER
```

At that step:

| Observable | Value |
|---|---:|
| active pressure centres | `1` |
| contact events | `0` |
| smooth outer trials / HVP calls | `6 / 11` |
| absolute ledger residual | `5.6680123733388044e-8 kg m/s` |
| normalized ledger residual | `1.4791141212575156e-7` |
| frozen limit | `1e-9` |

The first failure is therefore **before contact**. It does not falsify the
post-solve sweep or contact impulse closure. It shows that the selected smooth
solver stop does not by itself certify a virtual support reaction at B3's much
stricter per-substep momentum tolerance.

The corner candidate reaches the same gate on a step with 30 contact-feature
events:

| Observable | Value |
|---|---:|
| absolute ledger residual | `5.6072818056180324e-8 kg m/s` |
| normalized ledger residual | `1.5012124695930388e-8` |
| active pressure centres | `1` |
| outer trials / HVP calls | `10 / 23` |

Because the face control fails without contact, contact is not the leading
explanation for either row.

## Solver transcription correction

The first implementation attempt reached `REJECT_LIMIT` while distinguishing
energy reductions below binary64 resolution. The one correction allowed by
the contract replaced the incomplete local stop transcription with the
already selected C2 rules:

- scale-aware displacement residual `<=1e-8`;
- pre-trial numerical energy floor at `1024*epsilon*energy_scale`;
- floor admissible only with scaled residual and scaled-step guards.

This removed the rejection failure. The final momentum failure occurs with
the corrected stop and is preserved; no second formula or threshold change
was made.

## Fixed-reference observations

The fixed runs also stop at the same local ledger gate before they can become
accuracy references:

| Fixture / substeps per frame | active | contact events | absolute residual | normalized residual | smooth outer/HVP |
|---|---:|---:|---:|---:|---:|
| face / 96 | `1` | `0` | `5.29e-11` | `6.62e-9` | `2 / 2` |
| face / 192 | `1` | `0` | `7.46e-12` | `1.87e-9` | `2 / 2` |
| face / 384 | `0` | `0` | `2.10e-12` | `1.05e-9` | `0 / 0` |
| corner / 96 | `1` | `30` | `5.64e-11` | `1.17e-8` | `2 / 2` |
| corner / 192 | `1` | `15` | `1.33e-5` | `5.52e-3` | `1 / 0` |
| corner / 384 | `0` | `0` | `2.62e-12` | `2.19e-9` | `0 / 0` |

The corner `/192` row is the discriminator. Its active pressure state passes
the selected scale-aware displacement stop immediately (`1` outer trial,
`0` HVP), yet the reaction ledger is not accurate enough. The very fine
inactive rows additionally show that a purely relative per-step ledger becomes
ill-conditioned as both gravity impulse and reconstruction error approach
binary64 scale.

## Interpretation

The experiment separates three claims that had previously been conflated:

```text
trajectory stop passes
    != smooth stationarity is exact
    != virtual boundary reaction is certified
```

B2 proved the algebraic virtual reaction at a prescribed state. B3 requires
the **solved** state to be stationary enough that reconstructed momentum and
that reaction agree. The existing `1e-8` displacement stop was designed for
trajectory accuracy, not reaction authority, and can stop before the latter
is true.

It would be incorrect to make B3 pass by merely loosening `1e-9` or by adding
support and contact reactions before checking them separately. It would also
be premature to tighten every nonlinear solve: the fixed rows show both a
real active-state discrepancy and a separate vanishing-scale arithmetic
effect.

## Repeatability and regression

Two complete failed reports are byte-identical:

```text
raw report SHA-256:
c64ad0b8d73f7ada62364a3daa2d3bed66a8bfc5a1fe6c0148b7c1f8f2e5fb2f

semantic result SHA-256:
9e0eb0baf63c6bf1ae0dd5288cd8809c1722ddf354a1fff86f3d70935f2a56fe
```

Historical selected reports remain byte-identical:

```text
B2 raw:      d6ba5f8e802966c25283d0c8384ed01beec20b347acb343cf5c7c2bf360d69d9
B2 semantic: 80a01b2ed0cf844841da322233b121b33e273c71eace8682795d0cad4e1dfb80

B1R1 raw:      af34c3d8e142610bb11d26592a8b9f673af6ff941f89c7b8631f178cd70f0af1
B1R1 semantic: e215b0facc30445541a6f2fa9446fd9d8140bf5f66983180cb435de9863f535e
```

## Next research

Before retrying B3, run a bounded reaction-accuracy diagnostic that does not
abort the trajectory:

1. separate stationarity defect, velocity-reconstruction roundoff and contact
   impulse in every ledger;
2. publish signed cumulative, L1 cumulative, maximum absolute and normalized
   residual over substep ladders;
3. compare the selected displacement stop with a reaction-aware stop
   counterfactual, charging its extra HVP work;
4. derive any mixed absolute/relative certificate from the global B1R1 state
   error budget and binary64 floor before freezing it;
5. resume B3 only if the diagnostic proves either a sound ledger certificate
   or an affordable reaction-aware solve.

Until then, the exact state is `B3 FAIL`: split formula remains valid, boundary
trajectory composition and reaction authority remain unproven.
