# NSR3-B4E2D7R20R63ZK independent recurrence checker contract

Revision: `2`.

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZK` |
| Architecture snapshot | SPEC-38/ADR-076 `Proposed`; ADR-081 `Accepted`; R63ZJ `INCONCLUSIVE` |
| Engineering consumer | decide whether the untrusted R63ZJ hybrid trace actually follows the frozen K2 recurrence and verifier |
| Claim class | fixed-profile trace correspondence; no representation or production claim |
| Budget | one producer capture, one independent full replay, negative controls, Dev plus two Release repeats, one initial review plus one batched-repair re-review |

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
   preserve each applicable scalar as an explicit K2 `(high, low)` payload,
   test `rho0`, `denominator0`, `rho1` and `denominator1` for strict positivity
   in the original transaction order, and stop at the first failed guard;
6. derive all three certificates directly from the independently derived K2
   solution components and the frozen verifier profile;
7. compare every binary64 component of solution, residual, direction,
   preconditioned and product input/output, both binary64 components of every
   applicable independently derived scalar, every product/state guard,
   metadata and identity field, and every certificate semantic field with the
   untrusted trace;
8. derive the ladder only from the independent certificates.

Comparisons are bit-exact. Equality of opaque nested roots alone never counts;
roots are additional integrity checks after their represented semantics have
matched. The expected replay must derive `complete`, `positivity_exact`,
`sealed_states`, `failure_stage`, product `exact`, site/dimension/entries,
artifact/input/value identities and state `exact`/index independently.

## Work ownership

Seal three distinct work receipts:

- **expected producer claim:** exactly three operator products, six tangent-kernel
  calls, three factor solves, two rho dots, two denominator dots, three scalar
  divisions, 204 solution updates, 204 residual updates, 102 direction
  updates and three certificates, with the frozen lower-level term counts;
- **actual producer claim:** the independently inspected recurrence and hybrid
  ledgers copied from the untrusted trace, sealed separately from the expected
  producer receipt and compared only after the full semantic trace;
- **checker replay:** the independently executed numerical schedule plus
  fixture admission, identity derivation, positivity guards, component,
  scalar, metadata/root and certificate-semantic comparisons,
  certificate-verifier work, hashing, controls and classifier cases.

The checker must derive the expected producer and checker ledgers from
constants and observed loop counts. It may compare the producer ledger only
after the independent trace is complete. Checker work cannot be credited as
candidate work. Control work is a distinct sealed receipt: every comparison is
counted, and any invocation of the old author validator explicitly owns its
fixture validation, callback replay and six tangent-kernel calls. Repeated
fixture validation, identity derivation and root/hash paths must either be
removed from the checker dataflow or counted; they may not be invisible.

## Controls

Starting from the untrusted captured trace, independently mutate and, where
public helpers permit, reseal:

- each state vector family;
- every applicable state scalar payload family plus a scalar root;
- every product guard/metadata/identity field plus input and output components;
- recurrence guard fields `exact`, `complete`, `positivity_exact`,
  `sealed_states` and `failure_stage`;
- state `exact` and `index`;
- certificate `passed`, counts, `sign_root` and `root`;
- recurrence and hybrid work;
- callback identity/aggregate roots;
- nonfinite and incomplete traces;
- top-level result/classifier inputs.

Each mutation must fail at its intended independent comparison. Where the
public API permits a complete author reseal, require the author validator to
accept the mutation before the checker rejects it; otherwise record the
mutation as checker-boundary coverage without claiming author-valid drift.
Every control comparison and author-validator replay contributes to the
control-work receipt. Exhaustively verify classifier precedence across every
declared mismatch category, work/control/ladder booleans and a fail-safe
unknown mismatch value.

## Routes and allowed claim

If apparatus, independent replay, full trace comparison, independent
certificate ladder, work separation, controls and result sealing all pass,
select `HYBRID_K2_RECURRENCE_CORRESPONDENCE_CANDIDATE`.

That route permits only this bounded claim: for the frozen fixture, the
untrusted hybrid trace is bit-exact with an independently scheduled recurrence
whose only intentional difference from R63ZI is the three component-linear
wide tangent callbacks, and the independently derived certificates form the
reject/reject/pass ladder.

The frozen mismatch precedence is apparatus, callback identity, product,
callback aggregate, recurrence guard, state, scalar, certificate,
recurrence-work and hybrid-work. Select the corresponding first-specific
route:

- `INDEPENDENT_CHECKER_APPARATUS_REJECTED`;
- `HYBRID_TRACE_IDENTITY_REJECTED`;
- `HYBRID_TRACE_PRODUCT_REJECTED`;
- `HYBRID_TRACE_CALLBACK_REJECTED`;
- `HYBRID_TRACE_RECURRENCE_GUARD_REJECTED`;
- `HYBRID_TRACE_STATE_REJECTED`;
- `HYBRID_TRACE_SCALAR_REJECTED`;
- `HYBRID_TRACE_CERTIFICATE_REJECTED`;
- `HYBRID_TRACE_RECURRENCE_WORK_REJECTED`;
- `HYBRID_TRACE_HYBRID_WORK_REJECTED`.

An unknown mismatch enum, checker-work failure, control failure, result-seal
failure or impossible positivity-guard continuation selects fail-safe
`INCONCLUSIVE` (`INDEPENDENT_CHECKER_*_REJECTED` as applicable). The classifier
must never collapse a known trace mismatch into a generic correspondence
route.

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
