# NSR3-B4E2D7R20R54 counterflow legacy-slope evidence

Status: `PASS / COUNTERFLOW_LEGACY_SLOPE_CANDIDATE`.

## Result

- semantic: `cce59025c9d4b606662d436f82463faef6682c8d07e7cb795c8113cdc00c389e`;
- unchanged direct solution/direction roots `820c3d59...3ab7` /
  `f09a8af6...821e`;
- existing verified-inverse root `e49c0000...3fc6`, 66 columns,
  `rho=2.7205185568e-25`, error `1.0931517281e-24`;
- zero factorization and zero compensated dots.

The old and refined nominal slope are byte-identical at
`1.8493164062012005077558791263e-19`. Only the enclosure changes:

| quantity | inherited cheap error | existing verified-inverse |
|---|---:|---:|
| direction error | `8.8529495396e-3` | `1.0931517281e-24` |
| direction bound term | `2.0123909084e-14` | `2.4848764689e-36` |
| total slope bound | `2.0123909084e-14` | `2.4850709658e-36` |
| slope lower bound | negative | `1.8493164062e-19` |

The minimal legacy certificate therefore closes the slope by roughly 17
orders of margin without moving the solution.

## Reproduction and scope

The result repeats byte-identically. R53/R52/R50/R51 retain exact semantics
`5418141a...40a4`, `86d2dcd4...17a6`, `190ac441...d86e` and
`48df3b26...adca`.

This authorizes only a default-off trajectory discriminator at an exact/KKT
direction rejected solely by the slope enclosure. It does not justify
unconditional inverse work, default policy, timing, runtime or production.
