# NSR3-B4E2D7R20R46 two-refinement trajectory evidence

Status: `PASS / TWO_REFINEMENT_LATER_INVERSE_BOUNDARY`.

Implementation `fb1c8bfb` emits reproducible semantic:

```text
e1c904c730afcdad36a24a4c0e430687b65f77ae81bef59bd8a4529e574b1ec5
```

The default-off hook is called three times. It matches and applies the frozen
original depth-eight and second depth-16 certificates exactly once each, then
rejects one unrecognized tuple without retry. The practical certificate roots
are `0d3e1914...6849` and `2a271988...472`; their radii are respectively
`4.5753008642e-14` and `3.1658780995e-14`, with signs `24/41/0` and
`29/36/0` and strictly positive separation.

The unchanged NNQP advances from R43's `600` principal solves/transitions to
`602`, completes seven accepted iterations and stops at a third
`VERIFIED_INVERSE_AUDIT_REJECTED` boundary. Its frozen identities are:

| object | SHA-256 |
|---|---|
| final case | `83aacc9a34e1265f8c6c9c6f2f9323ba59770e3fe5c211b212fa360ea0ec3a8c` |
| final step | `3e94e52d7d8fe8eb710465bf995bf47dae2d4101cda9da6669df4619ae7acb87` |
| target tuple | `06d5be834503a36963f0750618f11bee4b2d46e5a9efcaf909d581ecd1250e62` |
| matrix | `79244e371e1c6c05d7d4a1b409be19cdb6b45b5256668519db485e318df0d477` |
| inverse | `be9cb7827cad6fdf05ed09e3f9553a25d723ae116c7ca70bb3cddc8315c68d26` |
| RHS | `d4e62af1044f89d2a7e77cb5e24d9fd5d65d997c5bd5cba18a2b34a428ea4397` |
| solution | `fb226f2c44921d7d5017dd703001fc3e9b15634467854ac554c09bb9fa19e1d0` |

Two R46 executions are exact. R42--R45 retain their frozen semantic hashes.
No production path, parameter, trial, factorization policy, counterflow state
or timing changed.
