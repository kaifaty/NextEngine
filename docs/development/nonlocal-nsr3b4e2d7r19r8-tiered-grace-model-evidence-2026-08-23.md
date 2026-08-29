# NSR3-B4E2D7R19R8 tiered-grace model evidence

Date: `2026-08-23`

Status: `PASS / TIERED_GRACE_RESIDUAL_MODEL_CANDIDATE / REPLAY ONLY`

## Outcome

The frozen two-tier completion envelope classifies both observed trust-solve
boundaries without changing the live solver policy. The original boundary
retains its exact one-HVP completion. The later boundary enters only the
wider tier, makes sufficient independently measured progress on HVP 33 and
converges on HVP 34.

At the 34-HVP solution, the residual-derived model image `r_final-g` agrees
with one direct sparse `H(step)` oracle under every frozen `1e-10` bound. The
result selects a research candidate for transaction integration; it does not
authorize a production cap change.

## Parent and identity

```text
R8 identity SHA-256  15b8442aa24bf86d5358a018c4e4f13258dfb6f3ccf65aa9363415617b37221d
R7 stdout SHA-256    db0e5e733b57c9d6b5ffe0ad9fb641de1cee78e2fd05fe8cbf890fcae591fcdb
R7 semantic          c8f350b941e57910b1dc5dc93e6a357319d22de6f2fb16fb3a338592f6596c79
R6 retained          true
R5 retained          true
```

The standalone R7 non-regression run from the final R8 binary also emits
exactly 23,701 bytes with stdout SHA
`db0e5e733b57c9d6b5ffe0ad9fb641de1cee78e2fd05fe8cbf890fcae591fcdb`.

## Tier 1 preservation

```text
prefix root       f478832923673956bc98d8067fff9bdeb5c3dab109c0a8e239ad12c6844d60dd
step root         74a9b58726d5d0279699498c41a70fb0e199f45e531d11befd2bc2dfe692d2bd
ratio after 32    3.027232803424963e-5 = 1.201852669017946 eta
prefix safe       true
last eight fall   true
HVP 33 converged  true
```

This remains inside the original `(eta,1.25*eta]` lane. R8 does not alter its
predicate or add another HVP to it.

## Tier 2 classification

```text
capture root      97c340ed27b33318fb4d6684a7d385380ddc15e3a8b0358d0c86feb189f48ab3
prefix root       adf2edcc243060a540d515f6d8687a2b2b0f0e9e2131fbc402f06d9b50b85d08
step root         35054f8939760fe7baaa4d03250b7d14ec82a284a4f01d44a15cacff6a6b0cef
ratio after 32    1.891245028190834 eta
ratio after 33    1.294361573084484 eta
ratio after 34    0.9334121973834975 eta
HVP-33 contraction 0.6843965503098616
```

The first 32 records are finite, positive-curvature and interior, and their
last eight ratios strictly decrease. The HVP-32 ratio lies in
`(1.25*eta,2*eta]`. HVP 33 remains safe and interior, reaches
`(eta,1.5*eta]`, contracts by less than `0.75` and preserves the updated
last-eight decrease. Only then is HVP 34 classified as admissible, and it
converges.

## Model correspondence

```text
residual image root  d5f21a225af5eaa4183c586c142514254904defdb6055f2bc2c53fb094689f62
direct image root    423c2a90ec18d9c03cb5117641688d6dd17ea97d0345e951aeac4e1cafcbe750
image L2 relative    1.052547858378014e-15
component scaled max 5.823906635497171e-14
quadratic relative   2.8590252270021695e-16
predicted relative   2.8590252270021597e-16
candidate predicted  3.2892171491745966e-22
direct predicted     3.2892171491745976e-22
```

Both predicted reductions are finite and positive. Every comparison is well
inside `1e-10`, so the residual-derived image is the selected model candidate
for this boundary.

## Work and safety

R8 adds exactly one direct oracle HVP and one workspace build/release beyond
its parent replays. It consumes zero candidate model HVPs, precision audits,
trial formations, acceptances or all-pair candidate calls. Static binding,
gradient identity, finite values, lifecycle and rollback all pass. No
candidate substep, transaction continuation, public mutation or timing lane
runs.

## Reproducibility

Implementation commit:
`cb78e6b5` (`research: discriminate tiered grace model`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r8-a.zQN1NF`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r8-b.DfDpRJ`.

Both binaries are `5,965,152` bytes, have SHA-256
`0355537431a3b2381695171fe540e15695f4da9ff5f8c7abb79020b4d0736118`
and GNU build ID `f945aff6701ab84c53dce06388b10eaa9503a8cf`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r8-a.5xGT40`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r8-b.LtTQLx`.

Both exit `0`, emit empty stderr and reproduce:

```text
stdout-with-LF bytes  3,117
stdout SHA-256        def805d3ff7b9b596dd00ec0ff07cd6490dc0f483baa37efc9b8bff511a503d5
semantic result       65b9a51e231f51dd9d693ba55b63b0bfd3c4b16ccd5e0c9c298f7f1df7578f63
route                 TIERED_GRACE_RESIDUAL_MODEL_CANDIDATE
```

## Decision

Retain R8 as a successful replay-only candidate. The smallest next research
question is transaction integration: can the exact tiered completion policy
replace only the guarded-completion branch in one private first-substep
transaction while preserving all R6/R5/R2 anchors and explicit work
ownership?

That next stage must keep ordinary solves on direct model HVPs, admit HVP 34
only through the frozen tier-2 continuation gates, use `r_final-g` only after
an actually converged grace path, and stop at the next exact structural or
physics boundary. Production policy, public state and performance claims
remain out of scope.
