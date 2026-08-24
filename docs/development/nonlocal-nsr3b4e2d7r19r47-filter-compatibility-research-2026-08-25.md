# NSR3-B4E2D7R19R47 filter-compatibility research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Correction to the post-R46 plan

R46 proves that the R43 trial is acceptable in the two-coordinate filter. It
does not prove that R43 is a valid ordinary `theta`-step of trust-region
filter-SQP.

The primary trust-region algorithm requires a compatible normal step before
forming the tangential step. Compatibility includes satisfaction of the
linearized constraints inside the trust region; when this cannot be achieved,
the algorithm inserts the current point into the filter and enters a
restoration phase. Restoration may exit only at a filter-acceptable point
whose next trust-region QP is compatible. See Fletcher, Gould, Leyffer, Toint
and Wächter,
[*Global Convergence of a Trust-Region SQP-Filter Algorithm for General
Nonlinear Programming*](https://doi.org/10.1137/S1052623499357258),
Algorithm 2.1 and equations 2.12, 2.17--2.20.

The previous roadmap jumped from filter admission directly to a complete
switching/trust transaction. R47 corrects that ordering:

```text
filter acceptable
        !=
compatible normal/SQP step
        !=
successful restoration exit.
```

## Existing evidence already exposes the gap

At the immutable source,

```text
source psi                  3.2918458374056909e-15
predicted hinge reduction  2.6064437667433723e-15
linearized residual psi    6.8540207066231862e-16
linearized residual h      3.7024372261047687e-8
```

The projected R43 endpoint is contact-safe and well inside the inherited
global-L2 trust ball, but it leaves a strictly positive linearized inequality
residual. Therefore it fails the first, parameter-independent part of strict
normal-step compatibility. No choice of filter margin, switching exponent or
trust expansion constant can turn that nonzero residual into satisfaction of
the linearized constraints.

This is consistent with the history: R39 drove the unconstrained/contact-
unsafe linearized hinge very close to zero, while R43 had to clamp 1,268
inward contact components and retained 464 positive density rows. Contact
projection repaired world-boundary ownership but removed much of the
linearized feasibility progress.

## Narrow R47 discriminator

R47 is deliberately a zero-mutation prerequisite. It replays exact R46 and
extracts the already-computed R43 linear metric without another JVP,
workspace, model or nonlinear trial. It checks:

1. exact R46 filter admission and immutable source/trial/owner roots;
2. direct correspondence among source `psi`, predicted reduction and the R43
   linearized residual;
3. finite positive residual versus exact-zero compatibility, without fitting
   a tolerance to the observed value;
4. inherited trust and source-contact tangent ownership;
5. the logical distinction between filter admission, ordinary-step
   compatibility and restoration exit.

Expected scientific routes are:

- `FILTER_NORMAL_STEP_COMPATIBLE_CANDIDATE` if the frozen normal step really
  closes the linearized inequalities and the inherited geometry;
- `RESTORATION_COMPATIBILITY_REQUIRED` if it is filter-acceptable but not a
  compatible normal step;
- fail-closed parent/source/linear/trust/contact/filter/work routes for harness
  or ownership defects.

The latter route does not reject filter-SQP. It selects the correct next
research problem: at the R43 moved state, solve or certify the bounded
contact-cone linearized feasibility problem and determine whether restoration
can exit. A matrix-free primal/dual compatibility certificate should precede
switching and trust-radius implementation.

Frozen contract:
[R47 filter compatibility](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r47-filter-compatibility-contract.md).
