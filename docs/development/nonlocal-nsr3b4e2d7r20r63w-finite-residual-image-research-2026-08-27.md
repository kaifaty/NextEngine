# NSR3-B4E2D7R20R63W finite residual-image research

Status: `FROZEN / IMPLEMENTATION_NEXT`.

## Question

R63V proves that both direct-PCG lanes first certify at state 2, but its exact
dyadic dense certificate is an offline oracle. Can a bounded binary128
checker enclose the same exact common residual and residual image, reject
states 0--1 and certify state 2 without consuming exact arithmetic in its
production-facing path?

## Primary-source basis

[Ogita, Rump and Oishi, Accurate Sum and Dot Product](https://ogilab.w.waseda.jp/ogita/math/doc/2005_OgRuOi.pdf)
derives error-free `TwoSum`/`TwoProduct` transformations and the compensated
`Dot2` algorithm with rigorous error estimates. The existing R38 apparatus
implements that fixed-order construction in binary128 and has exact dyadic
positive/negative controls.

[Rump, Verification methods: rigorous results using floating-point
arithmetic](https://www.tuhh.de/ti3/rump/intlab/ActaNumerica2010.pdf)
provides the verified-residual/inverse framework used by R63R--R63U. R63W
keeps the same theorem `e=Zr+Ce`, `||C||inf<1`; it changes only how the two
finite products and their outward error are enclosed.

## Selected finite enclosure

Build immutable binary128 center/radius data from the exact common matrix once:

```text
H* in [Hc - Hr, Hc + Hr]
rho = ||I-ZH*||inf <= rho_up < 1.
```

The builder is checked against the exact R63Q/R63R oracle, but the certificate
function receives only `Hc`, nonnegative `Hr`, binary128 `Z`, `rho_up`, `b`
and candidate `x`.

For each row, one compensated dot encloses the residual center and an outward
absolute propagation encloses representation error:

```text
rc_i = Dot2([b_i,Hc_i,:], [1,-x])
rr_i = dot_bound_i + sum_j Hr_ij |x_j|
r_i in [rc_i-rr_i, rc_i+rr_i].
```

The image uses a second compensated dot and interval propagation:

```text
wc_i = Dot2(Z_i,:, rc)
wr_i = dot_bound_i + sum_j |Z_ij| rr_j
(Zr)_i in [wc_i-wr_i, wc_i+wr_i].
```

Then

```text
image_up = max_i (|wc_i|+wr_i)
error_up = image_up / down(1-rho_up).
```

Every addition, multiplication and division used for a radius is rounded
outward with the already audited R38 helpers. Normal-or-zero intermediates are
mandatory; underflow, nonfinite data, negative radius or `rho_up>=1` fails
closed.

## Selected discriminator

Replay both immutable state ladders only through state 2. For every state:

1. run the finite checker with no dyadic input;
2. independently run the R63U exact oracle;
3. require all 102 exact residuals and all 102 exact images to lie inside the
   finite intervals;
4. require finite states 0--1 to reject and state 2 to resolve the exact
   `24+/78-` signs;
5. bind centers, radii, Dot2 roots, containment counts, signs and work.

The exact oracle validates correspondence only. No exact residual, image,
sign or bound may repair the finite result.

## Alternatives and ceiling

- `||Z|| ||r||` is rejected because R63S already proves it loses 17--18
  orders on this weak-mode residual.
- Directed-rounding mode changes are deferred: the existing fixed
  round-to-nearest `Dot2` path already has audited error-free transforms and
  avoids changing the process floating environment.
- Sparse/direct tangent evaluation is deferred until the dense finite
  enclosure itself is shown adequate; otherwise sparse work would hide the
  first arithmetic boundary.

Success proves only a bounded finite checker for one frozen RHS and three
states per lane. It does not select binary128 as runtime arithmetic, a state-2
production stop, sparse storage, timing, GPU or production authority.
