# NSR3-B4E2D7R20R63ZA finite factor-consumption contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZA` |
| Parent | R63Z semantic `e27ee861...3e4be`; immutable K2 block operator |
| Engineering consumer | Portable start and first-two-update preconditioner actions |
| Claim class | Fixed twofold binary64 triangular-solve correspondence and enclosure |
| Budget | One parent replay; one factor artifact; three complete solves; 30,906 terms; 612 divisions; no PCG/update/timing |

## Exact claim and negation

Represent the frozen exported-binary64 upper factor, inverse scale and each of
three immutable source vectors as canonical twofold binary64 values. Apply
the same permutation and solve `R^T y = scale*P^T rhs`, then `R z = y`, with
fixed twofold operations and three-correction long division.

Every row publishes an outward a-posteriori radius derived from its equation
residual and the exact nonzero binary64 diagonal. Positive resolution requires
all 612 frozen binary128 reference components to lie inside their independent
finite intervals, exact identity/work/control roots and complete execution of
all three subjects. Reference values are absent from the finite signatures.

The negation is the first invalid platform, parent, operator/factor/source
identity, permutation, dimension, normalization, finite operation, nonzero-
diagonal, row equation, containment, work, control or lifecycle boundary.

## Frozen identity

```text
R63Z semantic       e27ee8616c1a34a1a224f65f32d5cebe1cd6dac973ccc2ae89c3fbad8343e4be
R63Z artifact       38cd1871eaf35c2153b469f4e98bba15754baa3853ceb76fdecc40d2d0146de1
R63S semantic       97d992f18912ed9a223a17080820b72d4da54635534d96dba3dc9e5d71454e14
R63S exported lane  5dcc65f74b8769d5a53cf5db2937f4c2013dc4b92f0333bb42a791006bc3c255
exported R           a7a85789364f5af91aaf71e1531759713c95ec085e48b7ca614daa524c3a4f85
permutation          335235cbaf9a93c805a2bfdbd17c593eb3ae44c9a20ea8a752782fa10f15826f
RHS                  64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1
export-wide start    c9e3c16966c9ed31d513af678ae7edec369448d49d600ea301230daa9319db5f
dimension            102
width                2 binary64 words
rounding              FE_TONEAREST, FMA, no fast-math
order                 subject,forward/backward,row,column,component
```

The factor artifact binds all coefficient bits, diagonal signs, permutation,
inverse-scale expansion/radius, parent/operator roots, width and operation
order. A source binds artifact root, role (`start`, `residual0`, `residual1`),
state, component/radius roots and frozen R63S lane root. Any mismatch rejects
before triangular terms.

## Finite operations

- Require canonical normal-or-zero `(hi,lo)`, `abs(lo)<=0.5 ulp(hi)` when
  `hi!=0`, finite normal-or-zero radius and round-to-nearest.
- Use error-free `TwoSum` and FMA `TwoProduct`; renormalize after every add,
  subtract, multiply and division correction.
- Use exactly `q0,q1,q2` long-division corrections and retain two normalized
  words. No data-dependent precision or extra correction.
- Evaluate each row numerator in one fixed flattened component order. Add
  outward source/previous-row radii and the finite reduction error.
- Require exact binary64 factor diagonals to be finite, normal and nonzero.
  The quotient radius is the outward row-equation residual divided by the
  exact diagonal magnitude.
- Partial output is never published. All three immutable subjects execute
  before classification; there is no early successful return.

## Independent audit

Convert all finite components, factor coefficients and frozen binary128
sources/references to exact dyadics. For every row audit the published
equation-residual bound with signed exact comparisons. For every output audit
that the corresponding frozen exported-wide binary128 reference lies in the
finite interval.

The exact oracle receives the finite result only after its root is sealed. It
cannot contribute a quotient correction, expected sign, radius, branch or
route. Require `exact_oracle_classification_uses=0`.

## Fixed work

```text
subjects                       3
forward terms/subject          5,151
backward terms/subject         5,151
divisions/subject              204
total triangular terms         30,906
total divisions                612
reference containments         3 * 204 = 612
row equation audits            3 * 204 = 612
new operator products          0
PCG scalar dots                0
candidate vector updates       0
factor/profile builds          0
timing samples                 0
```

## Controls

1. Exact `2 x 2` upper factor with nonidentity permutation encloses a known
   start and two residual solves.
2. High cancellation produces a nonzero low word and retains containment.
3. Three-correction division closes a case where onefold division does not.
4. Dropping input, row or quotient radii rejects the independent reference.
5. Zero/near-zero diagonal, subnormal, NaN, Inf, overflow and negative radius
   reject before partial publication.
6. Transposition/orientation changes factor and output roots.
7. Stale operator, factor, permutation, inverse scale, lane, role or state
   rejects before terms.
8. Mutating one high, low, radius or root changes/rejects the artifact.
9. Exact-oracle mutation changes containment but never the finite result.
10. Classifier precedence covers every route; R63B--R63Z regressions remain
    byte-exact.

## Routes

1. `FACTOR_CONSUMPTION_APPARATUS_REJECTED`.
2. `FACTOR_CONSUMPTION_IDENTITY_REJECTED`.
3. `TWOFOLD_TRIANGULAR_ARITHMETIC_REJECTED`.
4. `START_FACTOR_CONSUMPTION_REJECTED`.
5. `INITIAL_RESIDUAL_FACTOR_CONSUMPTION_REJECTED`.
6. `ITERATION1_RESIDUAL_FACTOR_CONSUMPTION_REJECTED`.
7. `TWOFOLD_EXPORTED_FACTOR_CONSUMPTION_CANDIDATE`.

## Stop and ceiling

Success authorizes only R63ZB fixed state-`0..2` twofold recurrence research.
It does not authorize a portable factor builder, runtime state-2 stop, broader
corpus, timing, GPU, runtime or production.

Failure does not authorize fitted tolerances, adaptive width, post-run third
word, native binary128 runtime, rank reduction, row deletion, regularization
or a different Krylov method without a new research/contract boundary.
