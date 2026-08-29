# NSR3-B4E2D7R20R63X composed affine-image research

Status: `FROZEN / IMPLEMENTATION_NEXT`.

## Question

R63W encloses every exact residual and image entry but loses 18 orders when
independent residual radii pass through `|Z|`. Can the same finite arithmetic
certify state 2 if correlation is preserved by evaluating the mathematically
equivalent affine image `Zb-(ZH*)x` as one compensated dot per row?

## Derivation

For exact common `H*=H_num/d`, binary128 verifier `Z`, RHS `b` and candidate
`x`:

```text
g_i       = sum_j Z_ij b_j
M_num_ik = sum_j Z_ij H_num_jk
Zr_i     = g_i - sum_k (M_num_ik/d) x_k.
```

The identity is exact and changes no verifier theorem:

```text
e = Zr + (I-ZH*)e,
||e||inf <= ||Zr||inf/(1-rho).
```

Unlike R63W, no independently widened residual vector exists. Cancellation
between `Zb` and `(ZH*)x` remains inside one compensated Dot2. This is the
standard error-free-transform use case covered by
[Ogita, Rump and Oishi](https://ogilab.w.waseda.jp/ogita/math/doc/2005_OgRuOi.pdf).

## Selected discriminator

An exact-only builder constructs `g` and `M=ZH*`, then emits binary128 nearest
centers and outward nonnegative radii for every entry plus the unchanged
`rho_upper`. The finite certificate receives only those binary128 arrays and
candidate `x`.

For every row:

```text
center_i = Dot2([g_center_i, M_center_i,:], [1,-x])
radius_i = dot_bound
         + g_radius_i
         + sum_k M_radius_ik |x_k|.
```

The maximum `|center_i|+radius_i`, divided outward by the lower
`1-rho_upper`, classifies signs. Exact R63V image values audit containment but
cannot enter the finite center, radius or sign path.

Replay states `0..2` on both lanes. Require state 0/1 rejection, state-2
`24+/78-/0?`, all 612 exact image containments, exact parent roots and fixed
work. No residual interval from R63W is available to the candidate.

## Interpretation boundary

This is an alternative representation/mechanism, not a retry of R63W and not
a precision increase. Success proves that correlation-preserving finite
arithmetic is possible for one frozen dense system. A later gate must recover
the same bound without materializing exact/dense `ZH*`, for example through a
factorized affine expansion or sparse block application.

Failure localizes either affine-profile representation radius or compensated
cancellation as insufficient. It does not authorize fitted radii, wider
runtime precision, sparse work, timing, GPU or production claims.
