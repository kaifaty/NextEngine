# NSR3-B4E2D7R19R65 composed dual-merit research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Why the filter result changes the next question

v8 proves that normal-safe density progress exists, but only after relaxing
strict intermediate primal-inertia descent. A full filter is one possible
globalization. It is not automatically the correct mechanism inside R65,
because R65 is still solving the fixed convex projection TRQP, whereas the
filter-SQP papers govern nonlinear outer iteration acceptance.

The projection subproblem already has an exact concave Lagrange dual. Testing
that dual is therefore prior to inventing an inner filter lifecycle.

## Derivation

In normalized step coordinates, let

```text
minimize_s  F(s) = 0.5 ||s-t||^2
subject to  c + A s <= 0
            s in D,
```

where `D` is the exact joint contact-box/trust-ball set. With multipliers
`lambda>=0`,

```text
L(s,lambda) = F(s) + lambda^T(c+A s).
```

For fixed `lambda`,

```text
s(lambda) = P_D(t-A^T lambda).
```

Thus the exact composed dual value is

```text
d(lambda) = F(s(lambda))
          + lambda^T(c+A s(lambda)).
```

Completing the square gives an independent correspondence form. For
`a=A^T lambda` and `z=t-a`,

```text
d(lambda) = 0.5 ||s-z||^2
          + lambda^T c
          + a^T t
          - 0.5 ||a||^2.
```

The current v6 baseline and every projected candidate already compute
`s(lambda)`. v8 additionally owns a fresh directed `raw=c+A s`, so both dual
forms can be evaluated without new sparse operators or projections.

The physical R63 inertia is `SPACING^2*F`; the dual audit stays in normalized
units to match the Hildreth/PCG multipliers and R64 operator. Mixing physical
inertia with normalized `lambda^T raw` is forbidden.

## Interpretation boundary

Dykstra's projection method is equivalent to a primal-dual row-action/dual
coordinate method, and Hildreth is its halfspace/polyhedral specialization:

- Bregman, Censor and Reich,
  [JCA 1999](https://math.haifa.ac.il/yair/Dykstra.jca99.pdf);
- Tibshirani,
  [arXiv:1705.04768](https://arxiv.org/abs/1705.04768).

The analytic derivation above is specific to the current convex TRQP. It does
not transfer to the outer nonlinear Nonlocal problem without relinearization,
nor does it prove convergence of an inexact accelerated block.

## Hypotheses

| ID | Hypothesis | Prediction | Falsifier |
|---|---|---|---|
| M1 | strict intermediate inertia is the wrong inner merit | every blocked outer has a normal-safe candidate with strict positive exact composed-dual change | any blocked outer has none |
| M2 | filter globalization is required even for this convex inner path | no blocked outer has positive composed-dual change despite v8 feasibility progress | any blocked outer has dual ascent |
| M3 | merit ownership changes by phase | only a strict subset of blocked outers has composed-dual ascent | uniform all or uniform none |
| M4 | direct formula/sign/scaling is wrong | completed-square form or scalar controls disagree | bounded agreement and controls |

## Predeclared resolution

```text
15/15 blocked outers have safe dual ascent
    -> COMPOSED_DUAL_PATH_CANDIDATE

0/15 have safe dual ascent
    -> FILTER_CONTROLLER_REQUIRED

strict nonempty subset
    -> PHASE_DEPENDENT_DUAL_FILTER_REQUIRED

identity/correspondence/control/work failure
    -> COMPOSED_DUAL_REFERENCE_RETAINED
```

This measurement does not apply the best dual candidate. A later stage must
separately freeze candidate selection, sufficient numerical margin, state
transition and terminal KKT comparison.

Frozen contract:
[composed dual-merit discriminator](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r65-composed-dual-merit-contract.md).
