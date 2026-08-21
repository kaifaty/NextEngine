# B4C3MA adaptive macro transaction research

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Problem split

B4C3PE1 establishes a durable fixed reference with one canonical publication
per macro frame. It does not establish that adaptive refinement can use the
same ownership boundary. Reusing B4C3TAR2 directly would publish after every
private KKT substep and restore the resolution-dependent perturbation already
rejected by B4C3TR.

The adaptive macro transaction is therefore:

```text
committed canonical macro state
  -> read-only spectrum
  -> private binary64 levels from the same start
  -> adjacent coarse/fine gate
  -> select fine private endpoint
  -> one balanced canonical publication
  -> atomically commit one frame and one macro ledger entry
```

The solver and embedded estimator do not change. Only state ownership,
publication cadence and ledger aggregation change.

## Why this is a separate one-frame stage

Three hypotheses must not be bundled again:

1. private adaptive selection and exact reject-limit recovery;
2. atomic macro publication and mixed representation admission;
3. long-horizon schedule, contacts, convergence and work.

B4C3MA tests the first two on the existing P1/P2 first frames. A later complete
replay may begin only after this transaction passes. The expected nominal pairs
are not hard-coded as acceptance results: the frozen spectral estimator derives
them, while the report records the observed P1 `21/42` and P2 `1/2` paths.

## Failure semantics

The private binary64 interval currently carries failure identity as text.
B4C3MA introduces an exact parser only for the grammar
`SUBSTEP_<decimal>:KKT_SOLVE:REJECT_LIMIT`, and the parsed index must equal the
reported number of completed substeps. No substring or general KKT-family
match is recoverable. The existing adjacent-passing-pair state machine remains
the selector: a failed level breaks adjacency, and no pair means no commit.

All attempted nonlinear work is counted. Failed and discarded private
endpoints own no canonical frame, root, publication ledger or energy delta.

## Representation and ledger

For the selected fine private endpoint `F`, its adjacent private coarse endpoint
`C`, and decoded publication `Q`, apply B4C3PE1 separately to position and
velocity:

```text
e = RMS(Q,F)
D = RMS(C,F)
TEMPORAL if D > floor(C,F) and e <= 0.5D
ABSOLUTE otherwise if e <= 0.01A
```

The accepted macro ledger is exactly B4C3P's aggregate KKT sum-scale ledger.
Its profile and policy hashes remain unchanged. Frame step is macro index plus
one, not accepted private-substep count.

## Decision

Freeze the [B4C3MA contract](../plans/nonlocal-nonlinear-solver-research/03b4c3ma-macro-adaptive-transaction-contract.md).
PASS authorizes only a complete adaptive macro recovery replay. It cannot
authorize adaptive-versus-fixed comparison, nominal corpus, CUDA, runtime,
schema or production work.
