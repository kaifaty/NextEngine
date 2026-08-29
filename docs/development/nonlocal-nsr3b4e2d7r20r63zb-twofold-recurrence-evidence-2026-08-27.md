# NSR3-B4E2D7R20R63ZB twofold recurrence evidence

| Field | Result |
|---|---|
| Evidence status | `SUPPORTED_BOUNDED / REVIEWED_NEGATIVE` |
| Producer status | `FAIL` |
| Route | `TWOFOLD_RECURRENCE_LADDER_REJECTED` |
| Result SHA-256 | `acfe6bafe9569eba101ed1e352cc01e149023e295bd8c082bdac58e49a709ebf` |
| Stdout SHA-256, two author runs plus reviewer run | `a10567198c7abc54e6702724de65ec80c5fc06b14d6164e4fc3374f1308c1176` |
| Source SHA-256 | `4ad9a6f30aed337594291602a6e7b83738d6100cff4850b4f82adbecd3a8f5a5` |
| Combined tracked/untracked diff SHA-256 | `2a5d86bf49dfc015aa535f722105d7b057885071f8bc5770db080093e56a4899` |
| Release binary SHA-256 | `a6fa477f03b9f514c2bfef4ea381f9a2e8f569f351fee0f283b88e164a3c3638` |
| Identity root | `21061745ade97a5a295a087f139fe2bb4a189a2d52109f10e34e8104a2ebc324` |
| Transaction root | `7aea732146e20ee40a77fc06c0311911429687503ddaadf07ce5ab666249bf96` |
| Controls root | `1185325452bd029b5e5f09a4fc995bc7a4d6ffc8f818f7ab668c7a16f6067fe5` |
| Authority | one frozen offline arithmetic discriminator only |

## Strongest supported conclusion

The exact sealed combination of the R63Z common-block K2 artifact, R63ZA
exported-factor centers, frozen RHS and fixed two-update PCG recurrence does
not reproduce the contracted R63Y reject/reject/pass ladder. All three
produced states remain `12+/24-/66?`; state 2 therefore has 66 unresolved
signs instead of the required `24+/78-/0?`.

This is not a refutation of PCG, twofold arithmetic, a different operator or
factor construction, a different number of updates, the Nonlocal formulation,
or production feasibility. It is a reviewed negative for one immutable
artifact/factor/RHS transaction.

## Finite recurrence result

| State | Positive | Negative | Unresolved | Affine error upper |
|---:|---:|---:|---:|---:|
| `x0` | 12 | 24 | 66 | `4.0865762286298115e15` |
| `x1` | 12 | 24 | 66 | `2.6192562645932580e15` |
| `x2` | 12 | 24 | 66 | `1.0719463174851391e15` |

The error decreases across the finite states but remains many orders above
the frozen state-2 sign margin. Positivity, arithmetic, state sealing and
fixed-work gates all close; only the required certificate ladder rejects.

The complete work ledger is exact:

```text
certificates                   3
operator products             3
operator entries         31,212
factor solves                 3
factor terms             30,906
factor divisions            612
rho dots                      2
denominator dots              2
dot terms                   408
scalar divisions              3
solution updates            204
residual updates            204
direction updates           102
adaptive stops                0
```

No exact/binary128 candidate, audit radius, expected sign, passing iteration
or adaptive stop enters the finite producer or verifier signature.

## Comparator ceiling

An observational binary128 replay uses the same finite common-block and
exported-factor artifacts. It also leaves all three states at `12+/24-/66?`.
Its maximum distance from the finite K2 states is about `6.89e-3`,
`2.12e15` and `2.53e15`; its distance from the earlier tangent-product parent
lineage is about `8.47e-5`, `1.28e15` and `1.47e15`.

This replay is useful only for localization. It shares the R63Y affine
verifier and is explicitly reported as
`shared_r63y_verifier=true / independent_end_to_end=false`. It cannot
independently validate that verifier or generalize the negative result.
Mutating only its parent-distance input changes the comparator root while the
finite states, certificates, finite differences, route and producer semantic
remain unchanged.

## Controls and independent review

The final apparatus executes the same recurrence transaction for its decisive
negative paths:

- dimension mismatch rejects as arithmetic before products, solves or states;
- `H=I, M=I` reaches zero `rho0` and rejects before Krylov updates or states;
- `H=-I, M=I` reaches a negative first denominator and rejects before Krylov
  updates or states.

Platform/arithmetic apparatus controls are separate from live identity
controls. Frozen parent, operator, profile, factor, permutation, inverse
scale, RHS, lane, dimension, width and recurrence order are self-rehashed and
sealed. The shared router preserves apparatus, identity, arithmetic,
positivity, verifier and ladder precedence.

Compilation and author runs were not treated as sufficient. Three independent
review passes found and forced repair of:

1. incompletely sealed parent/profile/factor payloads;
2. synthetic positivity and comparator-isolation controls that did not execute
   the code they claimed to test;
3. top-level route wiring that collapsed live identity/arithmetic/positivity
   failures into apparatus rejection.

Every material repair invalidated the preceding review snapshot. A fresh
review of the final source and binary found no remaining formula, count,
state, sealing, comparator or route defect and independently reproduced the
final stdout hash. Earlier snapshots have diagnostic value only and no
scientific route credit.

## Regressions and authority

Post-change direct parent outputs remain byte exact:

```text
R63Y  7427ecdb3c4c7c95900bc6046545d298a674be42865936bad543c7e0191f4372
R63Z  de22f5f638fa434ca61857f09b5b9f6988d40f33a5e7fcec4f43a4b9b21bda54
R63ZA ed2511615eb0b488b4337133860971f86a0d7ad48c757db49c5fcda32b325f72
```

No timing, GPU, runtime, nonlinear-state commit or production claim is
admitted. SPEC-38 remains Proposed and the CPU DFSPH V1 candidate remains the
production baseline.

## Next discriminator

Do not retry the same common-block/exported-factor K2 combination, fit a
tolerance, add a third word after observing this result, or change Krylov
method in the same gate. R63S and the binary128 comparator already consume the
same exported factor; they differ in operator representation and projection of
the RHS/inverse scale. The next experiment must therefore run a fixed `2x2`
factorial: tangent versus K2 common-block product crossed with original versus
projected inputs, under the same recurrence and R63Y certificate. Only after
that comparison may the roadmap select an operator, input-projection or
recurrence-arithmetic repair.
