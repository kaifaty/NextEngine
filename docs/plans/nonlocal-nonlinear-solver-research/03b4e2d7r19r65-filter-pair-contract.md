# NSR3-B4E2D7R19R65 projected filter-pair discriminator contract

Date: `2026-08-25`

Status: `FROZEN / MEASUREMENT-ONLY IMPLEMENTATION AUTHORIZED`.

Parent: R65 v7 `PASS / FILTER_MERIT_CONFLICT_CANDIDATE`, solver semantic
`bd568e0f367d34ef75f5ebeeca085f6cd6c36bb9fd56cb6966965a629ae8f0e2`,
diagnostic semantic
`9203252f9f330c4ce92bc2bcbe6ed091c361aadb915e28daf6a7ade26620ea03`.

## Fixed identity

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R19R65-FILTER-PAIR-R1` |
| Fixture | exact R63/R64 6,000-row binary64 replay |
| Candidate path | unchanged 15-HVP face-PCG direction and 16 dyadic projected candidates |
| Coordinates | `f=pure inertia`; `h=L2 positive full-row linearized violation` |
| Claim ceiling | `SUPPORTED_BOUNDED` selection evidence only |
| State authority | none; rollback only |

## Hard gates

1. Reproduce the v7 parent and require both inherited semantic roots exactly.
2. Preserve one Hildreth identification sweep, face-PCG state, dyadic order,
   dual/normal/composed calculations, candidate projections, selection and
   final solver state bit-for-bit. Instrumentation must not change the v6
   solver root or v7 proportioning root.
3. Audit the no-PCG joint baseline once per outer and every dual-decreasing
   projected candidate with a fresh R61 directed JVP over all 6,000 rows.
4. Compute `h^2` in stable row order from `max(raw_i,0)^2`; require finite
   nonnegative `h`, exact positive-row count and canonical audit roots.
5. Compute `f` with the existing R63 pure-inertia owner. Use the already
   computed baseline-minus-candidate difference for the objective envelope;
   do not introduce a fitted subtraction tolerance.
6. A filter-classifiable candidate must retain strict dual decrease and strict
   positive cached-normal model reduction. Filter admission cannot override
   either gate.
7. Test `gamma=2^-k`, `k=1..24`, strongest first. Strong admission requires
   the `h` branch at `gamma=1/2` and the frozen resolution slack. Weak
   admission means the weakest envelope passes but strong admission does not.
8. Record every outer, all 239 dual-decreasing candidates, per-outer best
   violation candidate, all candidate roots and the exact predeclared route.
   No checkpoint-only or selected-candidate reporting is sufficient.
9. Execute dense feasibility/objective/dominated/positive-part/self-rejection
   and route-precedence controls before scientific classification.
10. Exact work: 16 baseline plus 239 candidate audits, 255 fresh directed pair
    passes and 154,311,720 diagnostic slots. Solver work remains exactly
    468,968,743 terms with 528 `A^T`, 272 `A`, 16 baseline and 239 candidate
    projections. No timing is admissible.
11. Candidate and committed state, runtime filter/trust, public schema and
    production state remain untouched. No following outer, nonlinear trial,
    parameter selection or performance claim executes.

## Resolution

```text
all 15 blocked outers have a strong candidate
    -> UNIFORM_FILTER_FEASIBILITY_PATH_CANDIDATE

all 15 have a weak-or-strong candidate, but not all are strong
    -> MARGIN_SENSITIVE_FILTER_PATH_CANDIDATE

none has a weak-or-strong candidate
    -> COMMON_DESCENT_DIRECTION_REQUIRED

any other nonempty split
    -> PHASE_DEPENDENT_FILTER_REQUIRED

parent, control, audit, finite, work or identity failure
    -> FILTER_PAIR_REFERENCE_RETAINED
```

`UNIFORM_FILTER_FEASIBILITY_PATH_CANDIDATE` authorizes research of a complete
finite filter lifecycle only. `MARGIN_SENSITIVE` and `PHASE_DEPENDENT` require
a new controller discriminator without fitting the observed margins.
`COMMON_DESCENT_DIRECTION_REQUIRED` stops filter work on this path and returns
to direction construction. Every route remains report-only.

Rationale:
[projected filter-pair research](../../development/nonlocal-nsr3b4e2d7r19r65-filter-pair-research-2026-08-25.md).
