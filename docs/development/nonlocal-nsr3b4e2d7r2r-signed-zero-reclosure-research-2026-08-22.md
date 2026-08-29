# NSR3-B4E2D7R2R signed-zero reclosure research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / REPLAY_ONLY`

## Scope

B4E2D7R2 fails because its contract required the positive-zero bit pattern for
all three kernel quantities at `r=h`. The implementation produces
`[+0.0,-0.0,+0.0]`. IEEE signed zeros compare equal, and the sign of an exact
zero gradient contributes neither density derivative nor force. No physical
kernel change is needed or authorized.

This stage re-closes only the mistaken representation gate. It retains the
exact D7R2 trace, route precedence and solver stop boundary.

## Selected gate

At `r=h`, require:

```text
W(h) == 0.0 && W'(h) == 0.0 && W''(h) == 0.0
```

Publish all three `uint64` bit patterns and sign bits. Require the observed
bits `[0, 0x8000000000000000, 0]` so this is not a broad tolerance. Do not
canonicalize the return value or alter `weight_gradient`.

The reclosure internally recomputes the full D7R2 trace, requires its exact
parent stdout hash, exact full-step/set facts and the topology-stable
`alpha=1/2` descent row. It then applies the already frozen D7R2 route
precedence. On the observed trace this should classify
`TRUST_REJECT_POLICY_RECLOSURE_REQUIRED`, but that outcome is a gate, not an
assumption that may be forced.

PASS authorizes research of a new trust/globalization policy only. It does not
authorize increasing reject count, installing a half-step, changing inner
accuracy, running a trajectory or claiming solver readiness.
