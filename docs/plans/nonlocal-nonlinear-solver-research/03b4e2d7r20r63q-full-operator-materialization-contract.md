# NSR3-B4E2D7R20R63Q full-operator materialization contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63Q` |
| Architecture snapshot | R63P semantic `b7cae513...e3e1`; rank-one explanation rejected, tangent weak-form error smaller |
| Engineering consumer | Decide which finite operator representation may receive the next RHS certificate |
| Claim class | Exhaustive exact-rational full-matrix correspondence and error norms |
| Claim status target | apparatus/oracle boundary, tangent rejection or tangent full-operator candidate |
| Budget | One capture-only replay; 5,253 exact upper-triangle source/tangent dots; 10,404 ordered error audits; no solve or timing |

## Exact parent

Require:

```text
R63P semantic       b7cae5137a2e9c3cd9acdd678044dab5fcf820172db15ea13981b443c765e3e1
R63P observation    a8e4311989c3c8d1e80d6f7d8b0fb840d2e35775de4f9198fad7e24ab124e5a5
R63P arbitration    c535acfe3323cdb83b5dfe30d572136138388d3efda2e32b3c108f3775d66196
R63E full matrix    aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d
R63F tangent values 114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1
```

Reconstruct R63P exactly, including its rejected dominance gate and
`common_oracle_selects_tangent=true`. All source rows, projector data, dense
entries and tangent coefficients are immutable.

## Exact oracle construction

Convert each binary128 source/tangent/dense/projector/scale value once to exact
dyadic form. For each row compute exact `g_i` over the 174 free coordinates.
For each upper-triangle pair `(i,j)` compute:

```text
c_ij       = exact sum over 174 free source products
h_num,ij   = sigma (c_ij d - g_i g_j)
k_ij       = sigma exact_sum_over_315(T_i,l T_j,l).
```

Mirror all three values to `(j,i)`. Require exact source/tangent symmetry and
10,404 complete ordered values. `H*_ij=h_num,ij/d`; the denominator remains
symbolic and positive.

## Exact error metrics

For each ordered entry construct `eH_ij` and `eK_ij` as specified in the
research note. Count strict dense wins, strict tangent wins and exact ties.
Compute:

```text
max_H = max eH_ij / d
max_K = max eK_ij / d
inf_H = max_i sum_j eH_ij / d
inf_K = max_i sum_j eK_ij / d
fro2_H = sum_ij eH_ij^2 / d^2
fro2_K = sum_ij eK_ij^2 / d^2
oracle_inf = max_i sum_j abs(h_num,ij) / d.
```

All max, sum and strict comparisons use exact integers/exponents. Publish
binary128 outward projections only as diagnostics.

Also construct the exact ideal semantic defect numerator

```text
delta_num,ij = sigma (s-d) g_i g_j,
delta_ij = delta_num,ij / d^2,
```

and its exact infinity norm. Reproduce the complete R63P weak observation and
roots; no new witness or fitted direction is permitted.

## Candidate gate

Require:

1. exact parent, conversions, symmetry, counts and positive denominator;
2. exact `inf_K < inf_H`;
3. exact `fro2_K < fro2_H`;
4. exact R63P weak result and `common_oracle_selects_tangent=true`.

Entry winner counts, maximum errors, oracle norm and ideal semantic-defect norm
are published but do not add a fitted acceptance threshold.

## Fixed work

- source/tangent conversion: 32,130 values each; dense conversion: 10,404;
- radial row dots: `102 * 174 = 17,748` products;
- upper source Gram: `5,253 * 174 = 914,022` products;
- upper tangent Gram: `5,253 * 315 = 1,654,695` products;
- mirrors per symmetric matrix: 5,151;
- two errors, two row-sum terms and two Frobenius terms per ordered entry;
- zero RHS/inverse/factor/triangular/PCG/sparse/timing/state work.

Bind all counts and roots. Exact verification work is not a performance claim.

## Controls

1. A small dyadic `A,y,d,sigma` has an independently hand-derived exact `H*`.
2. Perturbing one dense entry while leaving exact tangent coefficients selects
   tangent in both global norms.
3. Perturbing one tangent coefficient enough to reverse a global norm selects
   tangent rejection.
4. Upper-triangle mirroring, transpose reversal and asymmetry are observable.
5. Mutating any oracle/error entry or norm changes the appropriate root.
6. Classifier precedence covers every route.
7. R63B--R63P byte regressions pass.

## Resolution precedence

1. `FULL_OPERATOR_MATERIALIZATION_APPARATUS_REJECTED`.
2. `EXACT_COMMON_OPERATOR_CONSTRUCTION_REJECTED`.
3. `TANGENT_GRAM_FULL_OPERATOR_REJECTED`.
4. `TANGENT_GRAM_FULL_OPERATOR_CANDIDATE`.

## Does not count

A binary128-only oracle; random probes; witness-only evidence; a relative
tolerance; a required entry-win percentage; patching either matrix; solving an
RHS; selecting PCG convergence; sparse/precision changes; timing, state,
runtime/GPU or production inference.

## Stop and reconsider

- Candidate: freeze a structured RHS/certificate stage against the exact
  common operator, preserving stored dense `H` only as a historical comparator.
- Tangent rejection: evaluate direct source/JVP/source and a wider dense
  construction against the same exact matrix before any solve.
- Oracle/apparatus rejection: locate the first conversion, symmetry, identity
  or count failure; do not weaken a norm.
- R64 and R65 remain blocked for every route.

