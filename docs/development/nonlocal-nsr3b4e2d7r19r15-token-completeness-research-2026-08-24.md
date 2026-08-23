# NSR3-B4E2D7R19R15 continuation-token completeness research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / V1 INCOMPLETE / V2 RESEARCH NEXT`

## Question

Does the R14 version-1 continuation-token candidate bind every state and
policy fact needed to resume the same private nonlinear substep without
granting work, changing solver semantics or losing deterministic evidence?

R15 answers only projection completeness. It does not execute resume.

## What v1 already binds

R14's token projection covers the following exact facts:

- schema version and `next_outer=6`;
- position, dual and completed outer-state roots;
- previous primal bits, admissibility and provisional index;
- predicted-position root, normalized `theta` bits and static-support identity;
- budget epoch, slice/cumulative HVP, soft limit and per-trial HVP allowance.

Those fields are sufficient to reject state, topology, normalized-scale and
HVP-ledger mismatches when the supplied payloads are independently rehashed.
They are not sufficient to reconstruct the complete solver resource state.

## Exact projected boundary ledger

The R10 boundary owns these observed counters:

```text
outer updates              6
inner trials in outer 5    2
current-trial HVP         14
total HVP                512
workspace builds          31
precision audits          20
accepted/rejected trials  20 / 0
```

The admitted continuation adds ten recurrence and one direct-model HVP, one
trial workspace and one precision audit. Outer finalization adds one final
workspace and zero HVP. The R14 suspension boundary therefore projects:

```text
outer updates              6
inner trials in outer 5    2
last-trial HVP            25
epoch-0 slice HVP         523
cumulative HVP            523
workspace builds          33
precision audits          21
accepted/rejected trials  21 / 0
```

The unchanged limits are outer `16`, inner trials/update `16`, HVP/trial `34`,
workspace builds `288` and precision audits `64`. The HVP `512` boundary is a
soft per-epoch admission limit under the R14 projection, not a reset of the
other substep-lifetime counters.

## Completeness counterexample

The v1 token projection does not include workspace/precision counters, their
limits, the outer/inner limits, accepted/rejected history or solver policy
identity. Therefore two distinct locally valid resume contexts can have the
same v1 token root:

```text
exact context:  workspaces 33, precision 21, accepted 21, outer cap 16
twin context:   workspaces 32, precision 20, accepted 20, outer cap 17
v1 token root:  c06dbfeeac346d4413d114082d18b4e4725edfac81896ed72cbabfacf3f188b5
```

The twin is deliberately within every numeric bound. It is unsafe because it
silently grants more remaining work and loses one accepted-trial receipt, not
because it is syntactically invalid.

Likewise, changing the completion-policy identity while preserving the same
physical/static inputs leaves the v1 root unchanged. This could resume with a
different forcing/grace/model policy.

This is a projection collision, not a SHA-256 collision. The hash correctly
commits to its input bytes; the input projection omits required facts.

## Selected disposition

R15 must prove both sides:

1. one-field mutations of fields already present in v1 change the root;
2. fixed resource-ledger and solver-policy twins omitted from v1 retain the
   same root while representing distinct valid contexts.

If the second proof holds, route
`TOKEN_V1_RESOURCE_LEDGER_COLLISION` and reject v1 as resume authority. The
existing R14 result remains valid as a suspension/policy candidate.

A later v2 design must bind at least:

- formula/solver/completion-policy identity;
- canonical payload framing and actual position/dual payload roots;
- predicted/static/normalized-scale identity;
- outer phase and convergence-history state;
- every structural limit and every used counter;
- accepted/rejected and recurrence/model/precision receipts or one
  independently reproducible history root;
- monotone budget epoch plus slice and cumulative ledgers;
- expected-token ownership for stale/duplicate rejection.

An unkeyed token hash provides deterministic integrity, not authenticity
against a malicious editor. Persisted or untrusted tokens would need a trusted
envelope/MAC and are outside this private research stage.

## Authority boundary

R15 may reproduce R14, construct fixed in-memory projections and compare
hashes. It performs no workspace, HVP, model, trial, precision, outer or
physics work. It may not validate a v2 token, resume outer 6, change the budget
implementation, define a public checkpoint schema or claim production
readiness.

The executable discriminator is frozen by the
[D7R19R15 token-completeness contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r15-token-completeness-contract.md).

The discriminator confirms the resource and policy projection collisions in
the [D7R19R15 evidence](nonlocal-nsr3b4e2d7r19r15-token-completeness-evidence-2026-08-24.md).
