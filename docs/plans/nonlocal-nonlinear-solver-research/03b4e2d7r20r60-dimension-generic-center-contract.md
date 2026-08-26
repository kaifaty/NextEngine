# NSR3-B4E2D7R20R60 research contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R60` |
| Architecture snapshot | SPEC-38 Proposed; ADR-076 Proposed; ADR-081 Accepted; R59 implementation `b529362a`, semantic `a0881eaa...faa9` |
| Engineering consumer | Decide whether a later separately frozen experiment may replace R50's dimension-65 policy with a dimension-generic, default-null centered callback |
| Claim class | Profile-bound finite exact/numerical certificate for one immutable tuple |
| Claim status target | `SUPPORTED_BOUNDED` or an exact first boundary |
| Budget | One v5 case replay; one captured tuple; exact/Dot2 two-sided audits; depths `4/8/16`; no retry, timing or state update |

## Exact claim

The sole R59 oblique-jet/twist dimension-rejected tuple is captured exactly
once and has dimension `d != 65`. For both `I-A X` and `I-X A`, the exact
dyadic oracle is completely contained by the Dot2 enclosure and the outward
infinity-norm bound is strictly below one. The unchanged practical centered
certificate is exact and underflow-free at depths `4`, `8` and `16`; depth 16
classifies all `d` solution signs with strictly positive minimum separation.

## Exact negation

Any of the following resolves the bounded claim negatively: the exact R59
tuple cannot be reproduced once; `d == 0` or `d == 65`; a Dot2 entry fails to
contain its exact dyadic value; either two-sided norm bound is at least one;
depth 16 is inexact, underflows, is noncontractive or leaves any sign
unresolved.

## Fixed definitions and model

- Units: dimensionless normalized NNQP principal system.
- Norm: matrix/vector infinity norm with outward binary128 bounds.
- Input: immutable v5 problem root
  `5f9c2b59d28c5f4de6f30a0341e76edf01bb3462fe574b55f687b5911f0cd54c`.
- Expected parent trajectory: R59 case root
  `0337d014a68fac594161b90817e534313671008cdc3fc0deb9a16d619e3d6644`,
  `ACTIVE_SET_REJECTED`, 8 accepted iterations and 1,092 transitions.
- Arithmetic: Linux x86_64 GCC `__float128`, 113-bit significand; exact
  dyadic integer residual oracle plus ORO Dot2 and frozen upward operations.
- Reduction/factorization/active-set order: inherited unchanged from R59.
- Center recurrence: `c0=0`, `c(k+1)=X(b-Ax0)+(I-XA)c(k)`.
- Candidate state, solver decisions and default runtime paths are immutable.

## Assumptions

| Assumption | Status | Check |
|---|---|---|
| R59 source/problem identity remains immutable | given | exact manifest/problem root |
| Capture-only refiner reproduces R59 rejection | derived | same dimension-rejection root and complete case root |
| Exact dyadic oracle covers every represented product/sum | established parent | R37 controls and `d*d` exact entries |
| Dot2 bound is outward for every entry | established parent + checked | R38 controls and exact containment |
| Left contraction implies unique correction fixed point | analytic derivation | Banach/Neumann residual bound in infinity norm |
| A passing sign interval preserves the passive face | derived | every component interval excludes zero |

## Resolution and near-miss firewall

- Positive: exact capture, literal controls, two-sided contained contraction,
  and depth-16 `positive + negative == d`, `unresolved == 0`, minimum
  separation `> 0`.
- Negative: first exact failed predicate and its roots/counts are retained.
- Does not count: deleting the guard, using only a residual norm, checking one
  side, trying an unfrozen depth, applying the center, retrying the solver,
  changing a source/cap/tolerance, or treating wall duration as evidence.
- Ceiling: one tuple only; no universal dimension theorem, callback admission,
  6/6 trajectory result, performance or production claim.

## Competing hypotheses

| ID | Hypothesis | Evidence before R60 | Cheapest discriminator |
|---|---|---|---|
| H0 | `d=65` is a genuine certificate-domain requirement | R50 was frozen only at 65 | run unchanged certificate on captured `d` |
| H1 | `65` is policy-only; the same certificate closes | helper formulas are already dimension-parametric | exact/Dot2 plus depth 16 |
| H2 | inverse contracts but sign separation degrades with this tuple | R39/R40 had prior sign ambiguity despite improved bounds | fixed depths and component intervals |
| H3 | represented inverse is genuinely noncontractive | legacy audit rejected it | exact two-sided residual matrices |

## Evidence plan and controls

- Successful control: `A=[2]`, `X=[1/2]`, `b=[2]`, `x0=[1]` must have zero
  two-sided defect and pass depth 4.
- Negative control: `A=[2]`, `X=[0]`, same `b/x0`, must have defect norm one
  and reject the centered certificate.
- Candidate exact oracle and Dot2 audit are separately computed for both
  multiplication orders.
- Evaluate exactly the fixed depths `4`, `8`, `16`; no adaptive stopping.
- Preserve raw stdout outside Git; commit only bounded evidence and hashes.

## Stop and reconsider

- Stop with `INCONCLUSIVE` if capture/control/apparatus identity fails.
- Do not retry R59, modify the v5 source or run another dimension/depth.
- Reconsider a generic callback only after the depth-16 candidate closes.
- Runtime/GPU/performance/production remain out of scope regardless of result.
