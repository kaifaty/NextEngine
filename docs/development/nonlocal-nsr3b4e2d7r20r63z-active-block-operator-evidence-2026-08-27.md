# NSR3-B4E2D7R20R63Z active-block operator evidence

| Field | Result |
|---|---|
| Status | `PASS` |
| Route | `TWOFOLD_BLOCK_DENSE_OPERATOR_CANDIDATE` |
| Semantic SHA-256 | `e27ee8616c1a34a1a224f65f32d5cebe1cd6dac973ccc2ae89c3fbad8343e4be` |
| Stdout SHA-256, both final executions | `de22f5f638fa434ca61857f09b5b9f6988d40f33a5e7fcec4f43a4b9b21bda54` |
| Parent R63Y semantic | `ad3d84abaecc5142f05d369d0e653533c5e152cac2a9112efb2ad8277d70fb9d` |
| Parent R63Y stdout | `7427ecdb3c4c7c95900bc6046545d298a674be42865936bad543c7e0191f4372` |
| Controls root | `d5032a6ba4b73ced5c64b384f784ce7523586ac1f520839b536ff044788b048f` |
| Authority | offline arithmetic research only |

## Observation

R63Z materialized the selected exact common operator `H*` as one immutable
`102 x 102` profile. Every coefficient is represented by two binary64 words
plus an outward binary64 radius. All `10,404` coefficients have a nonzero low
word; the maximum remaining coefficient radius is
`6.548326479487695e-35`.

The retained and exported candidate ladders each executed states `0..2`.
Every row product is one fixed-order `Dot2Err` of length `408`, followed by
the coefficient/candidate/cross representation radii. The finite signature
contains no exact numerator, denominator, oracle sign or expected product.

## Exact containment

| Lane | State 0 maximum radius | State 1 maximum radius | State 2 maximum radius | Contained |
|---|---:|---:|---:|---:|
| retained-wide | `6.6115943069409402e-14` | `6.6729157544103484e-14` | `6.7687115645031691e-14` | `306/306` |
| exported-factor | `6.6216111086973449e-14` | `6.5500406784991171e-14` | `6.7681701320280227e-14` | `306/306` |

The independent exact rational oracle recomputed `62,424` products and
confirmed all `612/612` coordinate containments. Finite work was exactly
`612` row dots, `249,696` component pair products and `62,424` radius
triplets. All six products execute regardless of an earlier lane result.

## Structural selection

The frozen tangent is `102 x 315` with `17,748/32,130` nonzeros. A two-stage
CSR/CSC action `T(T^T x)` therefore visits `35,496` stored coefficients per
vector. The complete local common block visits `10,404`, a structural ratio
of `3.4117647058823528`. This selects block-dense storage for this active
dimension-102 transaction only; the world representation remains sparse and
block structured. No elapsed-time or throughput claim follows from this
count.

## Controls and regression

All disclosed controls pass: identity, positive product, cancellation,
nonzero coefficient/candidate radii, dropped-radius rejection, nonsymmetric
orientation, subnormal/NaN/Inf/negative-radius rejection, finite-operation
overflow rejection, stale parent/common/tangent/width/order/lane/state
identity, high/low/radius/root mutation, exact-oracle independence and route
precedence. The candidate binds the exact immutable operator-artifact root;
even a self-rehashed mutation is rejected before the first dot.

The focused target builds successfully. Both final R63Z executions are
byte-identical. R63Y was replayed after the final build and retained stdout
`7427ecdb...f4372`; the earlier sequential R63B--R63X regression set remains
unchanged. No CPU/wall performance A/B was run on the shared host.

## Conclusion and next boundary

R63Z closes the first portable binary64 common-operator product boundary.
It does not generate a new candidate, apply a preconditioner, execute PCG,
choose a nonlinear stopping rule or establish runtime/GPU/production
readiness.

The next gate must consume this exact immutable artifact in a fixed finite
factor/preconditioner recurrence. It must keep candidate production separate
from the R63Y affine verifier, bind every recurrence scalar and state to the
same operator identity, and use exact arithmetic only as an independent
offline correspondence oracle. Corpus, timing and integration remain blocked
until that recurrence closes.
