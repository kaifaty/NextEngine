# NSR3-B4E2D7R20R37 exact inverse-residual evidence

Status: `PASS / ARITHMETIC_ENCLOSURE_DOMINATES`.

Implementation `9d01cf30` emits reproducible semantic:

```text
e9e61c01d6ae9ebe07d7ef707dfb5121f352b1e2df85f8e2a5160d9015a598ff
```

The exact R35 torsion case/step and R36 inverse/norm roots all reproduce.
Observer metadata exposes two already existing verified-inverse calls with
`63 + 65 = 128` columns; the unique selected second root is the frozen failed
audit `5fe4d71e...5e7c`. No factorization or right-hand-side solve is added.

| item | value |
|---|---|
| matrix root | `45e2c92b...9bef` |
| Cholesky factor root | `1c159edb...bbf0` |
| represented inverse root | `8ec902ee...5803` |
| exact residual root | `d6bd19f5...dcfa` |
| exact audit root | `eb9d2fb3...fefe` |
| exact worst row | `25` |
| exact dyadic norm | `239600336344559834894212622125514564501876763146687114968282451981 * 2^-225` |
| outward binary128 norm | `4.443635206380081e-3` |
| inherited norm enclosure | `3.753859524925144e+1` |
| inherited / exact-outward ratio | `8.447722080191076e+3` |

Thus the represented inverse candidate satisfies `||I-AX||_inf < 1` by a
large margin. The torsion matrix is not rejected by this candidate; the R36
failure comes from the arithmetic enclosure used to evaluate the residual.
This selects compensated verified-dot-product research and rejects immediate
scaling/factorization/support changes.

The exact apparatus passes binary128 round-trip including a subnormal,
identity/power-of-two/cancellation controls, a noncontractive negative and a
permutation repeat. Two result repeats are exact. R36 remains
`448a9b8b...b90f`.

The first capture apparatus emitted invalid semantic `cb73accd...db1` because
it assumed one internal verified-inverse call. Parent roots and the selected
audit were already exact, but the observer stopped before the dyadic audit.
The amended capture requires both existing call roots and a unique target;
the invalid result receives no scientific credit.

R37 is an offline certificate discriminator. Its arbitrary-size integer
oracle and binary128 path have no runtime or production authority.
