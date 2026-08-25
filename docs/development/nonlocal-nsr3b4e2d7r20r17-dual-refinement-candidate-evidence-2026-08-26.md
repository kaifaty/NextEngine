# NSR3-B4E2D7R20R17 dual-refinement candidate evidence

Status: `PASS / V3_DUAL_REFINEMENT_UNRESOLVED`.

Implementation `9546e852` first reproduces default R8 semantic exactly:

```text
afca1c777053172169e227bcf3625ad7c828892520340db5eb45c98d53b4374a
```

The opt-in 12-case candidate emits:

```text
d224bc8c0f451e1a07aa4cbdf6cc194c85f6341ca3be5a1d74e2a7d97cfe6ab8
```

Eleven cases certify. All eight v2 cases and the two previously successful v3
cases retain their old roots and consume no dual refinement.

| repaired case | refinement work | outcome |
|---|---:|---|
| v3 corner | one fallback, 133 inverse columns | strict KKT in 10 accepted steps; primal `4.332e-33` |
| v3 shear | one fallback, 125 inverse columns | no rejection, but reaches cap 32; primal/dual mapping `1.745e-7` |

The candidate therefore repairs the structural ratio failure and fully closes
corner. It does not close the frozen all-twelve criterion: shear moves from an
active-set rejection after 10 accepted steps to a finite monotone-looking
32-step trajectory, but remains about fourteen orders of magnitude above the
strict certificate scale.

Blind generalization and development-candidate authority remain false. Raising
the cap without inspecting contraction/globalization would be post-result
tuning and is not authorized.

