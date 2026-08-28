# NSR3-B4E2D7R20R63ZK independent recurrence checker contract

Revision: `1`.

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZK` |
| Architecture snapshot | SPEC-38/ADR-076 `Proposed`; ADR-081 `Accepted`; R63ZJ `INCONCLUSIVE` |
| Engineering consumer | decide whether the untrusted R63ZJ hybrid trace actually follows the frozen K2 recurrence and verifier |
| Claim class | fixed-profile trace correspondence; no representation or production claim |
| Budget | one producer capture, one independent full replay, negative controls, Dev plus two Release repeats, one review cycle |

## Problem and hypotheses

R63ZJ deterministically observes a reject/reject/pass ladder, and its callback
products replay exactly, but its mutable DTO can be consistently resealed after
state, certificate or recurrence-work drift. It is closed as `INCONCLUSIVE`
and must not be repaired or reviewed again.

R63ZK tests four competing explanations:

- H0: the apparent ladder depends on stale or mutable certificate/state data;
- H1: the raw state trace independently satisfies the frozen K2 recurrence and
  verifier;
- H2: the callback products are correct but a non-product transition differs
  from the declared R63ZI K2 recurrence;
- H3: the numerical trace closes, but candidate and validation-replay work
  cannot be separated without changing the claimed algorithm.

## Trust boundary

Call the frozen R63ZJ producer once and copy its returned
`FormulaProbeHybridRecurrence` as untrusted trace data. The checker may read
component arrays and fixed site labels. It must not use these producer values
as an oracle:

- `exact`, `complete`, `positivity_exact`, `failure_stage` or author route;
- state/certificate/product/recurrence/callback/hybrid roots;
- certificate booleans, counts, signs or error bounds;
- recurrence or hybrid work ledgers;
- state scalar roots as expected values.

The checker independently schedules the recurrence from the immutable parent
fixture through small public K2 arithmetic primitives. Sharing the frozen K2
arithmetic and R63Y verifier implementations is allowed; sharing the author
transaction scheduler, validator or classifier is not.

## Required independent replay

From the fixture only:

1. validate exact fixture identity `7780543a...4553`;
2. project the frozen projected RHS and projected scale to canonical K2;
3. execute the start factor solve;
4. at `Kx0`, `Kp0` and `Kp1`, split the independently derived K2 input into
   exact high/low binary64 vectors, run the frozen binary128 tangent kernel on
   each, project both outputs to K2 and add canonically;
5. execute the initial residual solve, both rho/denominator dots, both alpha
   divisions, beta division and every solution/residual/direction update in
   the frozen order;
6. derive all three certificates directly from the independently derived K2
   solution components and the frozen verifier profile;
7. compare every binary64 component of solution, residual, direction,
   preconditioned and product input/output, every independently derived scalar
   identity, and every certificate semantic field with the untrusted trace;
8. derive the ladder only from the independent certificates.

Comparisons are bit-exact. Equality of opaque nested roots alone never counts.

## Work ownership

Seal two distinct ledgers:

- **producer claim:** exactly three operator products, six tangent-kernel
  calls, three factor solves, two rho dots, two denominator dots, three scalar
  divisions, 204 solution updates, 204 residual updates, 102 direction
  updates and three certificates, with the frozen lower-level term counts;
- **checker replay:** the same independently executed numerical schedule plus
  explicit component comparisons, scalar comparisons, certificate semantic
  comparisons and certificate-verifier work.

The checker must derive both expected ledgers from constants and observed loop
counts. It may compare the producer ledger only after the independent trace is
complete. Checker work cannot be credited as candidate work.

## Controls

Starting from the untrusted captured trace, independently mutate and, where
public helpers permit, reseal:

- each state vector family;
- a state scalar root;
- a product input and output component;
- certificate `passed`, counts, `sign_root` and `root`;
- recurrence and hybrid work;
- callback identity/aggregate roots;
- nonfinite and incomplete traces;
- top-level result/classifier inputs.

Each mutation must fail at its intended independent comparison even if all
author-side validators and roots are made self-consistent. Exhaustively verify
classifier precedence across its boolean input space.

## Routes and allowed claim

If apparatus, independent replay, full trace comparison, independent
certificate ladder, work separation, controls and result sealing all pass,
select `HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE`.

That route permits only this bounded claim: for the frozen fixture, the
untrusted hybrid trace is bit-exact with an independently scheduled recurrence
whose only intentional difference from R63ZI is the three component-linear
wide tangent callbacks, and the independently derived certificates form the
reject/reject/pass ladder.

If products match but a transition or certificate differs, select the first
specific `HYBRID_TRACE_*_REJECTED` route and localize the mismatch. On any
apparatus, work, control or sealing defect select `INCONCLUSIVE`.

## Stop and firewall

- Never change the R63ZJ snapshot, validator, evidence or review verdict.
- Do not consume the R63ZJ route as parent authority.
- Do not retry the rejected R63N/R63O direct matrix-free equation.
- Do not design width three, a dynamic builder or a portable representation
  before R63ZK receives independent `GO`.
- Timing, corpus/generalization, adaptive stopping, runtime/Rust/GPU
  integration, ProductChecks and production promotion remain blocked.

One initial independent review plus at most one batched-repair re-review is
allowed. Any remaining load-bearing finding closes R63ZK as `INCONCLUSIVE`.
