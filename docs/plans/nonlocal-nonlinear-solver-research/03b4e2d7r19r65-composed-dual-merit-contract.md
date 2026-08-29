# NSR3-B4E2D7R19R65 composed dual-merit discriminator contract

Date: `2026-08-25`

Status: `FROZEN / MEASUREMENT-ONLY IMPLEMENTATION AUTHORIZED`.

Parent: v8 `PASS / MARGIN_SENSITIVE_FILTER_PATH_CANDIDATE`, solver semantic
`bd568e0f367d34ef75f5ebeeca085f6cd6c36bb9fd56cb6966965a629ae8f0e2`,
proportioning semantic
`9203252f9f330c4ce92bc2bcbe6ed091c361aadb915e28daf6a7ade26620ea03`,
filter-pair semantic
`bdeeab4bd4d5db6b4f269de61fe84db020726690c22b464434935d8fcf218ab2`.

## Fixed model

For every v8 baseline/candidate pair, in normalized coordinates require

```text
F       = 0.5 ||s-t||^2
raw     = c+A s
d       = F + lambda^T raw

a       = A^T lambda
z       = t-a
d_alt   = 0.5 ||s-z||^2 + lambda^T c + a^T t - 0.5 ||a||^2.
```

Direct and completed-square values must agree under
`gamma(256*N+1024)` times the sum of absolute terms. All dot products use the
canonical row/particle order with long-double accumulation and binary64
publication.

## Hard gates

1. Preserve v6, v7 and v8 semantic roots exactly.
2. Preserve the 16-outer trajectory, all candidates, selected v6 line, solver
   state and every old work counter exactly.
3. Reuse v8's 255 fresh directed audits. Add zero `A`, `A^T`, projection,
   PCG, candidate or nonlinear work.
4. Compute `F` directly in normalized coordinates and compare
   `SPACING^2*F` with existing R63 physical inertia under the inherited R63
   model bound. Never add unscaled physical inertia to `lambda^T raw`.
5. Compute direct and completed-square dual values for all 16 baselines and
   239 dual-decreasing candidates. Record formula gap/bound, dual change and
   normal-safety for every candidate.
6. Scientific ascent requires: finite formulas, bounded agreement, strict
   positive direct dual change, strict fixed-density dual decrease and strict
   positive cached-normal model reduction. No fitted epsilon may turn equality
   into ascent.
7. Scalar control: `D=R`, `t=0`, constraint `1+s<=0`, `lambda=1/2` gives
   `s=-1/2`, `F=1/8`, `raw=1/2`, `d=3/8`; `lambda=1` gives feasible optimum
   `s=-1`, `d=F=1/2`. The wrong sign `F-lambda*raw=-1/8` at `lambda=1/2`
   must be rejected.
8. A box-active control must verify the completed-square identity when
   projection is not the unconstrained minimizer. Route-precedence controls
   cover all four outcomes.
9. Report all candidate roots, all 16 outer classifications and a separate
   composed-dual semantic. No candidate is applied.
10. Exact diagnostic work remains 255 directed pair passes / 154,311,720
    slots; solver work remains 468,968,743 terms. Timing is forbidden.

## Routes

```text
all 15 blocked outers contain safe strict composed-dual ascent
    -> COMPOSED_DUAL_PATH_CANDIDATE
none contain it
    -> FILTER_CONTROLLER_REQUIRED
otherwise
    -> PHASE_DEPENDENT_DUAL_FILTER_REQUIRED
failure
    -> COMPOSED_DUAL_REFERENCE_RETAINED
```

The strongest result authorizes only a separately frozen best-dual candidate
transaction experiment. It does not authorize runtime, production, outer
nonlinear acceptance or removal of filter-SQP from the later nonlinear
globalization roadmap.

Rationale:
[composed dual-merit research](../../development/nonlocal-nsr3b4e2d7r19r65-composed-dual-merit-research-2026-08-25.md).
