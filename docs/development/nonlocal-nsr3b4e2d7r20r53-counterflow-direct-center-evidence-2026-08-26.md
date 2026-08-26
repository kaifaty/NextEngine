# NSR3-B4E2D7R20R53 counterflow direct-center evidence

Status: `PASS / COUNTERFLOW_DIRECT_CENTERED_SLOPE_CANDIDATE`.

## Identity and reproduction

- semantic: `5418141a96d4a2c2d89d875450aeaf2164a66811b01a510a30e64625b91740a4`;
- exact R51 counterflow case/final step and R52 absence-of-audit boundary;
- final direct tuple selected by execution order after 622 observed direct
  solutions: transition 86, dimension/support 66/66;
- matrix/factor roots `3b0b5b6b...a98a` / `23f1bbb2...10b7`;
- RHS/direct-solution roots `698106e6...6d3a` / `820c3d59...3ab7`.

## Certificate result

The ordinary cheap direction error is `8.8529495396e-3`. The explicitly invoked
existing verified-inverse passes on the same factor with 66 columns,
`rho=2.7205185568e-25` and refined error `1.0931517281e-24`.

The independent depth-16 exact/Dot2 centered certificate also passes:

- left `rho_bound=3.7879390202e-28`;
- uniform error `1.9113931473e-33`;
- 66 positive, zero negative/unresolved;
- minimum separation `2.0083968664e-2`;
- certificate root `2dc5e558...17dd`.

The frozen work is exactly 66 inverse columns, 5,676 compensated dots and
376,002 dot input pairs, with zero new factorization.

## Shadow slope

The slope is recomputed around the certified center, not the old direct value:

| quantity | inherited | certified-center shadow |
|---|---:|---:|
| nominal slope | `1.8493164062012005e-19` | `1.8493164062012005e-19` |
| ordered bound | `2.0123909084417865e-14` | `1.9450129301444759e-40` |
| lower slope | negative | `1.8493164062012005e-19` |

The shadow direction is therefore strictly certified. Shadow slope root is
`154f23e3...3f91`; no center, direction or trial was applied.

## Regression and interpretation

Independent repeats are byte-identical. R52/R50/R51 preserve semantic hashes
`86d2dcd4...17a6`, `190ac441...d86e` and `48df3b26...adca`.

R53 proves that counterflow's remaining rejection is a verifier-placement
problem, not poor conditioning or an invalid direction. It also reveals a
smaller candidate than centered refinement: the already implemented legacy
verified-inverse passes by roughly 22 orders over the cheap error and may alone
close the slope. Test that minimal remedy report-only before authorizing a
centered trajectory.
