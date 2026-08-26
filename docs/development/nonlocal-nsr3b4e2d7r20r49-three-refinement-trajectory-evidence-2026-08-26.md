# NSR3-B4E2D7R20R49 three-refinement trajectory evidence

Status: `PASS / THREE_REFINEMENT_LATER_INVERSE_BOUNDARY`.

Implementation `e065274a` emits reproducible semantic:

```text
719e0d5067dadc7ab09374e923f3f764ae84e1f509784b0b11339024493df3cc
```

The default-off hook applies the three frozen certificates exactly once. Their
practical roots are `0d3e1914...6849`, `2a271988...472` and
`dc4449ec...e3a4`; all are finite, no-underflow, contractive, fully signed and
match their frozen depth/radius values.

Torsion advances from R46's `602` to `708` principal solves/transitions and
from seven to eight accepted iterations. It then stops at a fourth distinct
`VERIFIED_INVERSE_AUDIT_REJECTED` target:

| object | SHA-256 |
|---|---|
| final case | `f940e0ade20c1a5cc33ee75fe4e7293a016a55763b6229939a01f48faad39724` |
| final step | `7ea9c684ae5aadece2d429e00da495af01d08a3bf3a838d07f6e60cbe0256320` |
| target tuple | `8fe9bf8afd7f632a570bfb8b8f8b723b6a985a6a04f2d132eb5ce4334b357f6f` |
| matrix | `e24fa32ed6b787c8afc5544c470d8f32f306e2d4360e45f58e109281d1a64054` |
| inverse | `b583ab4674ce6c87623d3351691a6b548a07c3ebe387315053cdd52fa07316d6` |
| RHS | `a9d5748646c3c4dd0eed55db9f1992e4ac031920386afccf55d1e2c2ea50543c` |
| solution | `819d8ce96231fa71f19d418b17d1b061f07f6e8573452b0899e48f34e96e42b9` |

Two R49 runs and R48--R46 regressions are exact. Four distinct target tuples
now establish a repeated certificate-path defect. Per the frozen exit rule,
no fourth target-specific inverse/center audit will be added; the next stage is
a root-agnostic compensated verifier with a structural work cap.
