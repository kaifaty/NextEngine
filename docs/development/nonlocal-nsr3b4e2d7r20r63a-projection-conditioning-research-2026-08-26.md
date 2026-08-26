# NSR3-B4E2D7R20R63A projection and conditioning research

Status: `FROZEN / EXACT_DECOMPOSITION_IMPLEMENTATION_NEXT`.

## Question

Why does the R60 inverse cease to be an inverse after independent binary64
projection: loss in the matrix, loss in the inverse, their interaction, or a
scale/conditioning barrier that makes the unscaled representation unsuitable?

R63A is a no-solve diagnostic. It must identify the source before any
equilibration or new inverse algorithm is selected.

## Mathematical decomposition

Let `Aq,Xq` be the exact represented binary128 capture and `Ad,Xd` the exact
dyadic values of their binary64 projections. For the right defect:

```text
Rqq = I - Aq Xq
Rdq = I - Ad Xq
Rqd = I - Aq Xd
Rdd = I - Ad Xd

Rdd = Rqq + (Rdq-Rqq) + (Rdd-Rdq)
     = Rqq + (Rqd-Rqq) + (Rdd-Rqd)
```

The same four combinations are evaluated in left order. Every equality is
checked componentwise with exact dyadic integers. Norms of the individual
differences distinguish matrix projection, inverse projection and interaction
without attributing a non-additive matrix norm to a guessed cause.

## Conditioning facts, not conclusions

R63A also records exact/outward infinity norms of `Aq,Ad,Xq,Xd`, their
projection differences, nonzero entry ranges, diagonal/row ranges and
symmetry. The product `||A||inf*||X||inf` is reported only as an inverse-norm
conditioning proxy; it is not relabelled as an exact condition number.

The primary literature supports diagonal equilibration as a way to reduce
entry/row/column range and often improve conditioning, but not as a guarantee.
LAPACK's own `xGEEQU` documentation explicitly makes this limitation. Ruiz's
iterative infinity-norm method is interesting for a later symmetric scaling
candidate because it preserves symmetry.

Sources:

- [LAPACK xGEEQU purpose and limitation](https://www.netlib.org/lapack/explore-html/d5/d8a/group__geequ_gaed5e4c5d914951627451f0e0f06136e9.html)
- [Daniel Ruiz, A scaling algorithm to equilibrate both rows and columns norms in matrices](https://cds.cern.ch/record/585592)
- [Rump, Verification methods: Rigorous results using floating-point arithmetic](https://www.tuhh.de/ti3/rump/intlab/ActaNumerica2010.pdf)

## Frozen classification

Using factor 16 only as a coarse, predeclared discriminator:

- inverse projection dominates when its direct contribution is at least 16x
  the matrix contribution on both left and right decompositions;
- matrix projection dominates under the symmetric condition;
- otherwise the result is mixed;
- failure of original two-sided contraction or an exact identity is an
  apparatus/parent rejection, not a projection classification.

The classification selects no scaling formula. If projection loss is
scale-driven and the original matrix is symmetric, research may next compare a
fixed symmetry-preserving power-of-two equilibration against an unscaled
control. If it is not, the next candidate must instead address rank/conditioning
or stop the binary64 route.

## Ceiling

R63A authorizes no factorization, inverse construction, refinement, centered
iteration, trajectory, timing, runtime/GPU work or production conclusion.

## Result pointer

The exact experiment classifies the failure as mixed and exposes a genuine
conditioning barrier; see the
[R63A evidence](nonlocal-nsr3b4e2d7r20r63a-projection-conditioning-evidence-2026-08-26.md).
