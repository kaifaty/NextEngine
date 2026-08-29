# NSR3-B4E2D7R2R signed-zero reclosure evidence

Date: `2026-08-22`

Status: `PASS / TRUST_REJECT_POLICY_RECLOSURE_REQUIRED / REPLAY_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,763,736-byte executables at
SHA `c329b5416b58870496a4c8ec868e059762c84625c7bc9dd7c6419b1197e2bf8a`
and Build ID `70b6032b49f01013cb257f705fa30a39239308c1`.

Both fresh processes exit zero with empty stderr and byte-identical 1,352-byte
stdout reports at SHA
`36a6c45b08e595818a4ce86a7bde3d88d3a923f250e0ed9d859973aeb083786e`.
The semantic result is
`238dca8baa3434b284acaf88dac3f0279312bee1aad312bf4f559d7064f33a06`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r2r.eAwTE6`.

All parent stdout reports remain byte-exact at their preserved SHAs:

```text
D7   e0542abc4e0c7ff0b38acc0fe38095e270dec030617ad5183665414ce9b3db11
D7R  4b0272df2d46bb53ec09175ea7095b0e8c699ac9e0e4e093c6e20dfd00240801
D7R1 9a9582d09897f63ed7cd9953cc2fb6153b4266813fb6797ba71a1de98140cbfe
D7R2 f6b18542b1a25a548a11925552f014148152c817c8de557a86f9e97f4c628bcb
```

## Signed-zero closure

The unchanged kernel returns values `[+0.0,-0.0,+0.0]` at `r=h`, with exact
bits `[0,9223372036854775808,0]` and sign bits `[false,true,false]`. All three
compare numerically equal to zero. The narrow reclosure passes without
canonicalizing or changing any kernel result.

## Causal classification

Every D7R2 gate other than its superseded positive-zero representation gate
passes. The failed Newton state has active/fluid/boundary set counts
`8/28/800`. The full step changes only boundary membership (`+72/-120`), but
live and current-branch energy differences agree exactly, so horizon crossing
is not causal.

At `alpha=1/2`, rounded coordinates move, every set is unchanged and the same
direction is admitted decisively:

```text
predicted reduction       = 9.9226915852551228e-16 J
raw actual reduction      = 1.3183898417423734e-15 J
direct actual reduction   = 1.3181955852256048e-15 J
direct/model ratio        = 1.3284657432912772
```

The full direction is a descent direction at the current point, but its
quadratic model is only locally valid. The current reject rule shrinks a large
radius rather than the rejected interior step: it needs 13 quarter-shrinks to
bind the step while the hard cap stops after nine. Thus it re-solves and
re-evaluates the same rejected proposal nine times.

## Decision

Select `TRUST_REJECT_POLICY_RECLOSURE_REQUIRED`. Research a replay-only
globalization discriminator before modifying the inner solver. Compare:

1. backtracking along the already computed descent direction;
2. a step-norm-aware rejected-radius update followed by a new Steihaug solve;
3. the first radius-binding continuation of the legacy policy.

The selected mechanism must retain positive model/actual reduction,
active/topology stability, exact rollback and bounded extra HVP/objective work.
This PASS does not authorize installing `alpha=1/2`, increasing reject cap,
changing tolerance/beta/formula, completing an AL state or running a
trajectory.
