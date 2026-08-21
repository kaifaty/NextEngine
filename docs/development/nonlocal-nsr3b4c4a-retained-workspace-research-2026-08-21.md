# B4C4A retained accepted-workspace design

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Selected ownership change

B4C4M0 proves an exact local lifecycle:

```text
accepted KKT current workspace
    -> release
    -> rebuild identical state as RUN_SUBSTEP_DIAGNOSTIC
    -> read density/energy
    -> release
```

B4C4A changes only ownership:

```text
accepted KKT current workspace
    -> transfer to interval owner
    -> read density/energy once
    -> record reuse receipt
    -> release exactly once
```

Rejected trials, failed solves, frame spectrum, macro publication and all other
queries keep their existing lifecycle. The diagnostic uses the already exact
`Evaluation` stored in the retained workspace; no physics is recomputed and no
cache becomes durable authority.

## Predeclared result

The B4C4M0 one-macro trace contains exactly one substep-diagnostic rebuild for
every completed private substep:

```text
P1: 63 reads, 327 - 63 = 264 workspace builds
P2:  3 reads,  12 -  3 =   9 workspace builds
```

Transfers, reads and releases must all equal `63/3`; final live workspace count
must be zero and the maximum must remain two. All transaction state, adaptive
selection, physical diagnostics, canonical frame and ledger roots must equal
the record-disabled legacy control exactly.

The optimized query-chain root is necessarily different because successful
diagnostic queries no longer exist. A separate receipt root binds each omitted
request to the retained full-state hash and the exact mechanical/strain values;
the legacy query root remains reported as comparison evidence.

## Failure ownership

A synthetic consumer-abort control obtains one valid retained workspace,
releases it without publication and requires zero live workspaces and empty
retained storage. A non-finite current-state build must fail without creating a
retained workspace. No destructor-side hidden publication is allowed.

## Decision

Freeze the
[B4C4A contract](../plans/nonlocal-nonlinear-solver-research/03b4c4a-retained-workspace-contract.md).
A PASS authorizes only complete-lane application of this same ownership policy.
B4C4B static support indexing, B4C4C flat-only CSR, B4D, nominal execution,
CUDA, runtime and production remain blocked.
