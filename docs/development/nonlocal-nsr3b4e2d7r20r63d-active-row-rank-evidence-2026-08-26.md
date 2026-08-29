# NSR3-B4E2D7R20R63D active-row rank evidence

Status: `PASS / PROJECTOR_METRIC_NUMERICAL_RANK_LOSS`.

## Resolution

The frozen R63D discriminator separates the source constraint geometry from
the projected NNQP Gram block:

```text
102 original source rows B
    exact represented rank: 102
    binary64 normalized pivot rank: 102
                    |
                    | active projector derivative J
                    v
projected block H = B J B^T
    exact represented rank: 102
    binary64 normalized pivot rank: 101
    binary128 normalized pivot rank: 101
```

There is no literal duplicate source row. Both predeclared prime fields prove
full represented row rank for `B` and `H`. The projected matrix is therefore
not exactly singular as represented, but one direction is so weak that both
fixed numerical pivot profiles classify it below their precision-scaled
thresholds.

This selects a projector-range/nullspace research branch. It does **not**
authorize dropping a constraint or treating numerical rank 101 as a physical
rank.

## Frozen subject and correspondence

| Item | Observed |
|---|---:|
| Parent case | `v5-filled-oblique-jet-twist-3x5x7` |
| Particles / scalar coordinates | `105 / 315` |
| Selected passive rows | `102` |
| Parent inverse audits observed | `125` |
| Matrix-selected audits | `1` |
| Selected identity columns | `102` |
| Selected matrix root | `aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d` |
| Source row root | `6788422f80d918f39abfb7f9eb3f839e07f4598e408127fce67303314fcf5635` |
| Source value root | `3b6167b55b5e32f5b4a83ae35bbd3c31610422d70a5b741745d0f856994205f2` |
| Sparse entries / scalar writes | `4,245 / 12,735` |
| Nonzero source scalars | `8,792` |
| Duplicate coordinate writes / rows | `0 / 0` |

The observer stores `source_rows[indices[local]]` at the same verified-inverse
call selected by the immutable matrix root. R63B and R63C public outputs remain
byte-identical, so adding this private capture does not change their solver
lineage.

## Exact represented-rank proof

Each finite dyadic coefficient was mapped to two fixed odd prime fields before
elimination. A full row rank under either prime is sufficient; both agree:

| Matrix | Rank mod `2^61-1` | Rank mod `2^31-1` | Exact-full proof |
|---|---:|---:|---:|
| Source `B`, `102 x 315` | `102` | `102` | yes |
| Projected `H`, `102 x 102` | `102` | `102` | yes |

The two source eliminations each map 32,130 coefficients and select 102
pivots. The two projected eliminations each map 10,404 coefficients and select
102 pivots. All dyadic mapping and field-arithmetic controls pass, including
normal/subnormal binary128 values and literal full/duplicate-rank matrices.

## Numerical rank profiles

Both Gram matrices were independently diagonal-normalized. The binary64
diagnostic uses the frozen `n*2^-53*max(diagonal)` threshold; binary128 uses
the already frozen complete-pivoted diagnostic.

| Profile | Source `B B^T` | Projected `H` |
|---|---:|---:|
| binary64 rank | `102` | `101` |
| binary64 threshold | `1.1324274851176597e-14` | `1.1324274851176597e-14` |
| binary64 minimum accepted pivot | `4.2112699298986272e-4` | `8.6891059380584221e-7` |
| binary64 rejected pivot | `0` | `-5.5908135514416735e-16` |
| binary128 rank | `102` | `101` |
| binary128 threshold | `1.0274913290503679e-27` | `1.0274913290503679e-27` |
| binary128 minimum accepted pivot | `4.8766927674329809e-4` | `7.7352967042956946e-7` |
| binary128 rejected pivot | `0` | `1.4266648943795972e-30` |

The small negative binary64 rejected pivot lies well inside its positive
threshold and is consistent with roundoff around a near-null mode; it is not a
certified negative eigenvalue. The binary128 profile sees the same rank split
with a tiny positive rejected pivot, three orders below its threshold. This
cross-precision agreement makes a mere binary64 factor-construction artifact
implausible.

## Interpretation

R63A showed extreme conditioning in the projected matrix. R63B/R63C showed
that ordinary binary64 inverse-factor constructions cannot certify the needed
orientation. R63D now localizes the origin:

- the original constraint rows are algebraically independent and numerically
  full-rank under the frozen binary64 source profile;
- the active projector metric suppresses one independent combination to a
  near-null direction;
- the projected block remains algebraically full-rank, so automatic row
  deletion would change the represented problem rather than remove a proven
  duplicate;
- more accumulation-order or inverse-orientation variants cannot address this
  geometric conditioning mechanism.

The smallest next research question is therefore: which combination of the
102 rows lies near the nullspace, and is it explained by clamped coordinates,
the active ball tangent, or their composition? That requires a separately
frozen projector range/nullspace witness. No RHS solve is justified yet.

## Reproduction

Build:

```bash
cmake --build /tmp/nextengine-r20r4-build \
  --target nonlocal-formula-reclosure -j 8
```

Run:

```bash
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-active-row-rank
```

Observed semantic hash:

```text
f9a5f0d807818eab61759b81d1b1df99265909c249fd8348308e614d06a45b8d
```

Two independent stdout hashes:

```text
11b8ddd92e5b4163f6122227e482aa64ec56a9926c6993c1293d03e74015e619
11b8ddd92e5b4163f6122227e482aa64ec56a9926c6993c1293d03e74015e619
```

Parent regressions after the private capture extension:

```text
R63B stdout 320d6ea1842d49fef30b6821586b189dc923ef5bcfe9e3c22e86f4a38bc24216
R63C stdout 0ace54f4baa595a931db4a7da14464fca22b1cdfe68d4869212b50c39b56581b
```

Implementation commit: `9702a65a`.

## Authority ceiling

Exactly one immutable passive block was diagnosed. There were zero new NNQP
RHS solves, row drops, centers, replacements, state updates or timing claims.
The result gives no universal rank theorem and no runtime, GPU or production
authority. R64 and R65 remain blocked.
