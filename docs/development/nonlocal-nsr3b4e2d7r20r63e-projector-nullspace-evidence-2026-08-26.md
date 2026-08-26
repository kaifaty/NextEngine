# NSR3-B4E2D7R20R63E projector nullspace evidence

Status: `PASS / CLAMP_METRIC_NUMERICAL_RANK_LOSS`.

## Resolution

The inherited near-null row combination is created by the active clamp mask,
not by the trust-ball tangent projection.

All three represented Gram blocks remain algebraically full-rank 102 under
both fixed prime fields. Numerically, however, the rank falls to 101 as soon as
the 141 clamped coordinates are removed:

```text
source B B^T            binary128 rank 102
       |
       | Q: zero 141 clamped coordinates
       v
clamp B Q B^T           binary128 rank 101
       |
       | remove active-ball radial component
       v
tangent B T B^T         binary128 rank 101
       |
       | scale by 1/(1+eta)
       v
full B J B^T            binary128 rank 101
```

The selected R63D pivot witness makes the attribution much stronger: almost
all of its source-vector energy lies in coordinates which the clamp derivative
sets to zero. The ball-radial component is negligible by comparison.

## Projector state and correspondence

| Item | Observed |
|---|---:|
| Projector root | `bcbb5a0d2088b17424c8d6427803e3f09f3174f099c78cf11ec8d9b0a3d58f22` |
| Ball active | yes |
| `eta` | `1.7679394766303986` |
| Free components | `174` |
| Lower-clamped components | `70` |
| Upper-clamped components | `71` |
| Fixed components | `0` |
| Captured/reconstructed full matrix root | `aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d` |
| Projector JVP correspondence | bit-exact for all 102 rows |
| Full Gram correspondence | bit-exact for all 10,404 entries |

The context slot was armed only around the existing NNQP call and was clear
before and after replay. It observed the inherited 125 inverse audits, selected
one immutable matrix call and copied no state into public solver roots.

## Rank profiles

Every block has modular rank 102 under both `2^61-1` and `2^31-1`. Thus none
of the rank-101 profile results is an exact-dependency claim.

| Block | binary64 rank | binary128 rank | binary128 rejected pivot |
|---|---:|---:|---:|
| Clamp `G_Q` | `101` | `101` | `2.1761047086342890e-30` |
| Tangent `G_T` | `101` | `101` | `1.4278194292758434e-30` |
| Full `G_J` | `101` | `101` | `1.4266648943795972e-30` |

The binary64 threshold is `1.1324274851176597e-14`; the binary128 threshold is
`1.0274913290503679e-27`. The clamp step alone crosses both pre-frozen
thresholds. The ball and uniform scale preserve rather than create that
classification.

## Near-null witness energy

The deterministic normalized-pivot witness was mapped back to the original
row coordinates without renormalization:

| Quantity | Squared norm / energy |
|---|---:|
| Source combination `||v||^2` | `2.0521189788970652e+1` |
| Clamped component `||c||^2` | `2.0521189788970652e+1` |
| Ball-radial component `||r||^2` | `2.4896241251651714e-32` |
| Retained tangent `||t||^2` | `3.9495781330250565e-30` |
| Full derivative image `||Jv||^2` | `5.1551051769618936e-31` |
| Gram energy `alpha^T G_J alpha` | `1.4266103580831735e-30` |
| Tangent energy `||t||^2/(1+eta)` | `1.4269019125494562e-30` |

The free radial-plus-tangent energy is roughly 31 orders of magnitude smaller
than the source energy. Within that already tiny free remainder, the ball
removes only about `2.49e-32`. This is a clamp-nullspace alignment, not a
ball-radial alignment.

All frozen consistency gates pass:

- `v=c+r+t` maximum residual `2.14e-50`;
- `Jv` agrees with the production-form JVP;
- `G_J alpha` agrees with `B(Jv)`, maximum residual `1.57e-35`;
- the Gram/tangent energy identity closes inside the predeclared gamma bound.

## Architectural consequence

The 102 source constraints are neither duplicate nor exactly redundant. Their
dual representation becomes nearly non-unique on this projector face because
one independent source-row combination acts almost exclusively on coordinates
that are locally immobile under the clamp derivative.

Consequently:

- another inverse accumulation order cannot fix the cause;
- wider precision can certify a solve but would preserve a badly chosen dual
  representative and is not the first architectural remedy;
- deleting one row based on numerical rank would change the represented
  constraints without proving primal equivalence;
- the next research target is a projection-equivalent dual representative (or
  an explicitly range-space formulation) that preserves the same primal
  projected state while avoiding the near-null dual combination.

The 2026 extreme-point correction result for polyhedral projection is directly
relevant as a research analogue, but our simultaneous active Euclidean ball
requires an independent equivalence derivation.

## Reproduction

```bash
cmake --build /tmp/nextengine-r20r4-build \
  --target nonlocal-formula-reclosure -j 8

/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-projector-nullspace
```

Semantic hash:

```text
2c9e75c42c140cb828b5421cb8727e2371746e5a0eb65e75698cc4e634b16002
```

Two independent stdout hashes:

```text
db3e61ad2955ccbedb1e98aaf0258a7aab0e8c5696f26c87f0ef09e0ba3c0710
db3e61ad2955ccbedb1e98aaf0258a7aab0e8c5696f26c87f0ef09e0ba3c0710
```

Parent regressions after private projector capture/witness retention:

```text
R63B stdout 320d6ea1842d49fef30b6821586b189dc923ef5bcfe9e3c22e86f4a38bc24216
R63C stdout 0ace54f4baa595a931db4a7da14464fca22b1cdfe68d4869212b50c39b56581b
R63D stdout 11b8ddd92e5b4163f6122227e482aa64ec56a9926c6993c1293d03e74015e619
```

Implementation commit: `eb85ddd4`.

## Authority ceiling

No NNQP RHS, rank action, row drop, center, replacement, state update or timing
was executed. The result is one immutable local face and does not select a
production representative policy, runtime arithmetic or GPU path. R64 and R65
remain blocked.
