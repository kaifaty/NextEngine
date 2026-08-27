# NSR3-B4E2D7R20R63ZG certificate-detail decomposition research

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_NEXT`.

## Question

Reviewed R63ZF proves that both final binary128 solutions have the same 102
nonzero raw signs, while the R63Y certificate changes from `24+/78-/0?` to
`12+/24-/66?` and its global `error_upper` grows by about `3.06e16`.

R63Y constructs that error as:

```text
image_i = v_i - M_i x
image_infinity_upper = max_i (abs(image_center_i) + image_radius_i)
denominator_lower    = down(1 - rho_upper)
error_upper          = up(image_infinity_upper / denominator_lower)
```

The frozen twofold profile already reports:

```text
rho_upper             0.00906152187947077
denominator_lower     about 0.9909384781205292
1/denominator_lower   about 1.0091443839143854
```

The contraction multiplier is identical for both endpoints and only about
`1.009`. It cannot by itself explain a `3.06e16` relative error gap. The open
question is therefore whether the common endpoint produces a large affine
image center, a large enclosure radius, or an interaction with componentwise
solution margins.

## Minimal discriminator

Expose one private detail DTO around the unchanged R63Y certificate. It carries
the already computed image centers/radii, solution expansion centers/local
radii, denominator, error and immutable certificate identity. It does not
change R63Y arithmetic or roots.

For each tangent/common endpoint derive outward budgets:

```text
U = max_i (abs(image_center_i) + image_radius_i)  native image budget
C = max_i abs(image_center_i)                    center-only budget
R = max_i image_radius_i                         radius-only budget

G  = up(U / denominator_lower)                   native global error
GC = up(C / denominator_lower)                   center-only global error
GR = up(R / denominator_lower)                   radius-only global error
```

Then rerun only the final sign-bound comparison over an identity-sealed table:

```text
solution representation: tangent / common
budget origin:          tangent / common
budget mode:            U / GC / GR / G
```

`U` is deliberately unamplified; comparing it with `G` detects whether the
small contraction multiplier crosses an actual sign boundary. Two local-only
cells use zero global budget. Native `(tangent,G_t)` and `(common,G_c)` cells
must reproduce the immutable certificates exactly.

## Competing hypotheses

| ID | Hypothesis | Decisive observation |
|---|---|---|
| H0 | affine-image center residual is sufficient | common `GC` reproduces rejection for both solution representations while common `GR` does not |
| H1 | representation radius is sufficient | common `GR` reproduces rejection for both solutions while common `GC` does not |
| H2 | both components are independently sufficient | both common `GC` and `GR` reject both solution representations |
| H3 | mixed rowwise image bound is required | common native `G` rejects both, but neither `GC` nor `GR` does |
| H4 | solution-margin interaction remains | tangent/common outcomes differ under the same sealed global budget |
| H5 | contraction amplification changes the boundary | a native-origin `U` cell and corresponding `G` cell classify differently |
| H6 | detail correspondence is defective | native roots, recomputed maxima/error, work or native sign cells fail |

H0--H3 are admitted only after the native two-by-two global-budget columns are
solution-independent. Otherwise H4 has precedence. H5 is reported separately
and cannot be inferred from the scalar amplification value alone.

## Private detail surface

Add `FormulaProbeCertificateDetail` and
`formula_probe_certificate_detail(fixture, solution)` to the private research
API. The detail must bind:

- the immutable minimal certificate and profile root;
- width/dimension and exact operation counts;
- `image_infinity_upper`, `denominator_lower`, `error_upper` and minimum
  separation bit identities;
- all image center/radius and solution center/local-radius vectors;
- component, radius and detail roots.

The wrapper may recompute only the 102 width-two solution-center Dot2 values
needed for synthetic sign cells. It must reuse the unchanged R63Y profile,
solution conversion and certificate producer. Parent cache bytes and all R63ZF
outputs must remain identical.

## Boundaries

R63ZG is a certificate diagnostic, not a new certificate. Synthetic budget
cells have no proof or runtime authority; they only localize the observed
native boundary. No operator, recurrence, endpoint, profile, radius, rho,
tolerance, precision, iteration or nonlinear state changes. No timing, corpus,
CPU/GPU runtime or production claim is admitted.
