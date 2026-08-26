# NSR3-B4E2D7R20R63U exact residual-image certificate research

Status: `FROZEN / IMPLEMENTATION_NEXT`.

## Question

R63S rejects both stationary direct-PCG candidates because the safe bound
`||Z||inf ||r||inf/(1-rho)` leaves 66 signs unresolved. R63T then observes
rounded residual corrections of only `4.47e-4/1.09e-3`, but cannot apply them
once subsequent updates fall below center spacing. Are the unchanged R63S
final candidates already certifiably correct when the actual residual image
`Zr` is evaluated exactly instead of replaced by a worst-case product of
norms?

## Exact derivation

For `H* x*=b`, candidate `x`, `r=b-H*x`, approximate inverse `Z` and
`C=I-ZH*`:

```text
e = x*-x,
r = H*e,
Zr = ZH*e = (I-C)e,
e = Zr + C e.
```

R63R proves `rho=||C||inf<1`, so

```text
||e||inf <= ||Zr||inf/(1-rho).
```

This is the same verified-inverse theorem used by R63S, without the extra
submultiplicative relaxation `||Zr||<=||Z|| ||r||`. It is never weaker and can
be much sharper when the residual does not align with the worst inverse row.

With exact common representation

```text
H* = H_num/d,
r = r_num/d,
C = C_num/d,
w = Z r_num,
Zr = w/d,
rho = ||C_num||inf/d,
```

the bound reduces to the exact dyadic fraction

```text
error = ||w||inf / (d-||C_num||inf).
```

No division or rounded residual is needed for the sign comparison.

The construction follows verified residual/inverse methods described in:

- [Rump, Verification methods: rigorous results using floating-point
  arithmetic](https://www.tuhh.de/ti3/rump/intlab/ActaNumerica2010.pdf)
- [Oishi and Rump, Fast verification of solutions of matrix
  equations](https://www.tuhh.de/ti3/paper/rump/OiRu02.pdf)

## Selected discriminator

Reconstruct the two exact R63S iteration-8 candidates, exact common matrix,
exact RHS and R63R binary128 verifier converted exactly to dyadics. For each
candidate independently:

1. form all exact common residual numerators `r_num`;
2. form all exact residual-image numerators
   `w_i=sum_j Z_ij r_num_j`;
3. compute exact `||w||inf` and the positive denominator
   `d-left_inf_num`;
4. classify every component from the strict exact comparison
   `abs(x_i)*(d-left_inf_num)>||w||inf`;
5. bind solution, residual, image, bound, signs, counts and roots.

The current candidates are not updated. R63T projected residuals, rounded
corrections and centers are excluded. R60 solution/signs and stored dense
`H/X` are excluded. Both independently derived final sign vectors must agree.

## Branch after the result

- Both candidates certify: preserve direct tangent PCG as mathematically
  valid for the immutable common RHS. Freeze an earliest-iterate image
  certificate and then a finite/runtime-plausible residual-image enclosure;
  do not integrate dense exact verification.
- Retained certifies but exported does not: isolate exported-factor start and
  recurrence arithmetic with the same exact certificate.
- Neither certifies: compare exact image magnitude with R63T rounded image and
  only then select wider candidate/update arithmetic.
- Apparatus/image failure: repair only exact orientation/counting; retain
  R63S/R63T outcomes.

## Scope ceiling

R63U is an offline correctness certificate for two immutable candidates. It
does not make exact dyadics, dense `Z`, binary128 factors or 16 PCG updates a
runtime policy. It adds no candidate update, sparse realization, timing,
state, following nonlinear transition, GPU or production authority. R64 and
R65 remain blocked.
