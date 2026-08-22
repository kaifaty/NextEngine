# NSR3-B4E2D7R18R4R2 dimensionless-forcing transaction research

Date: `2026-08-23`

Status: `CLOSED / PASS / FULL_NORMALIZED_DIMENSIONLESS_FORCING_STATE_CONFIRMED`

## Question

Can the complete normalized, pairwise-precancelled private transaction use an
explicit dimensionless Krylov forcing rule while retaining deterministic
cross-profile state, precision and a work ledger derived before implementation?

R4R2 is not a relaxation or rerun of R4. R4 remains a hard negative result
against its frozen 38-HVP active correspondence. R4R2 owns a new policy and a
new independently justified ledger.

## Selected policy

For each trust solve, freeze from its initial gradient:

```text
sigma0 = max_i ||g_i|| / dx
eta = min(0.5, sqrt(sigma0))
stop when ||r_next|| / ||r0|| <= eta.
```

The policy is dimensionless and independent of particle count. `sigma0` is
the same normalized stationarity measure used by outer admission. It is
computed once before the first HVP and remains fixed throughout that trust
solve.

The inherited policy remains the default for all legacy commands. R4R2 must
select the dimensionless policy explicitly; it may not change the production
or public solver.

## Work derivation before implementation

For the frozen eight-particle fixture:

```text
||g||_2 <= sqrt(8) * max_i ||g_i||
sigma0 = max_i ||g_i|| / dx
dx * sqrt(8) = 0.14142135623730953 < 1
therefore sigma0 >= ||g||_2
therefore eta_dimensionless >= eta_inherited.
```

The dimensionless rule can never add a residual-driven iteration relative to
R4. It also cannot change a trust-boundary return that occurs before the
residual comparison.

R4's frozen trace has 19 active trials: 18 use two total HVPs (one trust-step
HVP plus one model HVP), and only outer `11`, trial `0` uses three (two trust
HVPs plus one model HVP). R4R1 proves at that unique first recurrence that:

```text
q                 = 2.2720706714996822e-5
eta_dimensionless = 1.2388346565200968e-5
q > eta_dimensionless
```

so R4R2 must retain the second trust HVP there. At the next return it cannot
continue later than the inherited rule because its threshold is no smaller.
Every other trial stays at one trust HVP. Therefore the pre-implementation
ledger is exact:

| Run | Outer | Trials | Accepted | Rejected | HVP |
|---|---:|---:|---:|---:|---:|
| each active | 14 | 19 | 19 | 0 | 39 |
| each inactive | 3 | 0 | 0 | 0 | 0 |
| five-run total | 48 | 57 | 57 | 0 | 117 |

This derivation precedes the R4R2 implementation. A different observed ledger
is a hard contradiction; it is not grounds to edit the expectation.

## State and precision expectations

Because the stopping iteration is unchanged for every trust solve, every
computed step, accepted state and precision audit must remain byte-exact to
the corresponding R4 transaction:

```text
active root   9a3a57e7fcb29700c72c710936ef02ea7459cf2470b7d59ca663605df00c5ee9
inactive root be761a3c07bc4a7f558486c5e9bc5d3b80e1cc0ba2982e1698e7f5a3d24c2585
```

Active confirmation remains `11/12/13`, inactive confirmation remains
`0/1/2`, every accepted trial receives the existing direct normalized long-
double audit, and every candidate-effect acceptance receives the existing
binary128 audit. Runtime binary128 remains prohibited.

## Controls

- Reproduce R4R1 and R4 complete stdout bytes; R4 must remain a hard FAIL.
- Preserve R3, R2 and D7R13 bytes.
- Execute reference active repeat, reference inactive, aligned active and
  aligned inactive under explicit dimensionless forcing.
- Require 57 explicit dimensionless-policy trust-step selections and zero
  inherited-policy selections across the five candidate runs.
- Require the exact roots, indices, precision ledgers and work table above.
- Reject invalid profile, dual, binding and structural budget before workspace,
  HVP, forcing-policy or precision work.
- Require exact rollback, all-pair zero and maximum live workspace two.
- Run no nominal substep, macro, trajectory or timing lane.

## Routes

1. `DIMENSIONLESS_FORCING_ACCEPTED_SIGN_CONTRADICTION`.
2. `DIMENSIONLESS_FORCING_ORACLE_OR_REDUCTION_BOUND_REQUIRED`.
3. `FULL_NORMALIZED_DIMENSIONLESS_FORCING_STATE_CONFIRMED`.
4. `DIMENSIONLESS_FORCING_INNER_POLICY_STILL_INSUFFICIENT`.
5. `DIMENSIONLESS_FORCING_OUTER_STATE_FORMULATION_REQUIRED`.

Identity, parent bytes, policy provenance, profile derivation, static binding,
transaction roots, work, precision, invalid prework, rollback or route
precedence mismatch is hard FAIL.

## Decision

Freeze R4R2 as five complete rollback-only transactions whose only solver
change from R4 is explicit dimensionless Krylov forcing. A confirmed result
may authorize research and contract design for D7R19; it does not authorize a
nominal substep or production integration.

R4R2 subsequently passes and selects
`FULL_NORMALIZED_DIMENSIONLESS_FORCING_STATE_CONFIRMED`; see the
[dated evidence](nonlocal-nsr3b4e2d7r18r4r2-dimensionless-forcing-transaction-evidence-2026-08-23.md).
Every root, precision fact and the pre-derived 117-HVP ledger closes exactly.
D7R19 research is authorized; execution remains blocked pending its contract.
