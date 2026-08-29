# NSR3-B4E2D7R3 globalization-policy evidence

Date: `2026-08-22`

Status: `PASS / STEP_NORM_AWARE_TRUST_SELECTED / REPLAY_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,785,088-byte executables at
SHA `bb873890366798b7a3cd31701e767b114bdd30cf748e0daa6b804867e2c152c4`
and Build ID `84c5ce4d8b40302f4032ff21339ab3a489652547`.

Both fresh processes exit zero with empty stderr and byte-identical 4,172-byte
stdout reports at SHA
`b52f9a599f2b0312fbe73ed8686f623a7142a967b497f507345ed7a47bc00f3c`.
The semantic result is
`6b64edba428fe38544ad6a9bc4d4a8e2ba5100749e15a57a33b94a9a6072d01c`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r3.n3fTw2`.

D7, D7R, D7R1, D7R2 and D7R2R retain their exact stdout SHAs and exit states.

## Candidate comparison

The quadratic interpolation from the rejected full step gives:

```text
alpha_hat                    = 0.42693869009663243
new radius / rejected norm   = 0.42693869009663243
new radius                   = 1.0585364232894624e-10 m
```

One unchanged Steihaug recompute reaches that radius to `1e-12` relative
closure, changes no active/fluid/boundary set and passes:

```text
HVPs                      = 2
predicted reduction       = 8.8854492836625599e-16 J
raw actual reduction      = 6.4965394175331426e-16 J
direct actual reduction   = 6.4902965076063732e-16 J
direct/model ratio        = 0.7304410053344077
```

Backtrack reuse also passes at its first smaller row, `alpha=1/2`, with direct
ratio `1.3284657432912772`. It remains a viable fallback but is not selected:
the step-norm lane retains the existing constrained-CG and negative-curvature
semantics.

The legacy radius first binds only after 13 shrinks, projecting 13 repeated
objective evaluations and 26 repeated HVPs before the binding recompute. That
recomputed step has positive ratio `1.1432593264228799` but still changes
boundary topology, so it does not pass the candidate stability gate.

## Decision

Select `STEP_NORM_AWARE_TRUST_SELECTED`. Freeze a separate tiny-inner
implementation contract. On a rejected interior proposal it may compute the
same factorized actual reduction and interpolation radius, then rerun unchanged
Steihaug. Acceptance must remain the existing raw objective/ratio gate; invalid
interpolation must fall back to the preserved quarter-radius rule.

The implementation must solve only the exact failed inner replay first, keep
the reject cap and all parent bytes unchanged, and publish no state. Only a
later contract may integrate the candidate into the full outer AL/confirmation
transaction.
