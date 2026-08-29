# NSR3-B4E2D7R20R63T common-residual recoverability research

Status: `FROZEN / IMPLEMENTATION_NEXT`.

## Question

R63S proves that both binary128 direct tangent-PCG lanes reach a stationary
finite representation residual, while their exact common-operator residuals
still leave 66 signs unresolved. Is the frozen exact common system
mathematically recoverable from those final candidates by ordinary residual
correction, or does the binary128 correction/update arithmetic itself fail at
the weak-mode scale?

This is the minimum diagnostic before redesigning the factor or finite
operator representation. It deliberately uses the R63R dense inverse `Z` as
an offline correction provider. A pass cannot authorize that representation
in runtime.

## Analytic basis

For common `H* x*=b`, candidate `x`, residual `r=b-H*x` and R63R approximate
inverse `Z`, one standard refinement transaction is

```text
delta = Z r
x_new = x + delta.
```

In exact arithmetic its error propagation is

```text
x* - x_new = (I-ZH*) (x*-x).
```

R63R proves `||I-ZH*||inf=0.00906152187947...<1`. Thus the ideal transaction
is contractive. R63S's two final certified error bounds give report-only ideal
frontiers:

```text
depth    retained upper     exported upper
1        5.15e12            4.65e12
4        3.83e6             3.46e6
6        3.14e2             2.84e2
7        2.85               2.57
8        2.58e-2            2.33e-2
12       1.74e-10           1.57e-10
```

These values omit residual projection, matrix-vector rounding and center
update rounding. They choose only a conservative fixed budget and cannot pass
a checkpoint.

This separation follows standard iterative-refinement analysis: residual,
correction solve and update arithmetic have distinct precision requirements,
and convergence depends on the correction equation being sufficiently
accurate rather than on a small self-residual alone.

Primary sources:

- [Carson and Higham, A New Analysis of Iterative Refinement and its
  Application to Accurate Solution of Ill-Conditioned Sparse Linear
  Systems](https://eprints.maths.manchester.ac.uk/2604/3/17m1122918.pdf)
- [Rump, Verification methods: rigorous results using floating-point
  arithmetic](https://www.tuhh.de/ti3/rump/intlab/ActaNumerica2010.pdf)
- [Oishi and Rump, Fast verification of solutions of matrix
  equations](https://www.tuhh.de/ti3/paper/rump/OiRu02.pdf)

## Selected experiment

Replay R63S exactly and retain the two iteration-8 candidates. For each lane
and each of 16 fixed updates:

1. convert the current binary128 center to exact dyadics;
2. compute all 102 exact common residual numerators
   `r_num=b*d-H_num*x`;
3. project `r_num/d` once to binary128 and bind the exact projection error;
4. compute `delta=Z*r` by fixed-order binary128 Dot2 rows;
5. compute the updated center by 102 binary128 Dot2 sums;
6. independently recompute the exact common residual and R63R sign
   certificate for the actual updated center.

The correction residual and the independent certificate residual are
separate computations. No certificate value enters the next recurrence. Every
iteration `1..16` is executed after a pass, and later certificates must remain
valid. Both final sign vectors must agree without consulting R60 signs.

## Why not more PCG or componentwise relabeling

R63S iterations 6--8 have identical exact certificate roots. More unchanged
PCG work therefore has no observed mechanism to leave the finite tangent
fixed point. The 66 unresolved components are also not zeros: unresolved means
only that the common solution interval crosses zero. Importing the historical
R60 pattern would destroy the independent common semantics established in
R63P--R63R.

## Branch after the result

- Both lanes certify: freeze factor-based common-residual refinement with the
  same 16-update and exact-certificate schedule, replacing dense `Z*r` by the
  retained/exported triangular solves.
- Retained certifies but exported does not: preserve retained-wide correction
  and isolate exported-factor arithmetic before any sparse work.
- Neither certifies but residuals contract before a stable floor: research a
  wider correction/update lane; do not extend the fitted iteration budget.
- Residuals do not contract as predicted: audit residual orientation,
  projection and update arithmetic before changing representation.
- Apparatus failure: repair only the apparatus and retain R63S.

## Scope ceiling

R63T proves or rejects mathematical recoverability only. Dense `Z`, exact
`H*` residual generation and binary128 are offline instruments. No public
state replacement, following nonlinear transition, sparse realization,
timing, runtime stop, GPU or production claim is admitted. R64 and R65 remain
blocked.
