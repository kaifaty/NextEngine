# NSR3-B4E2D7R10 private divided-reduction inner evidence

Date: `2026-08-22`

Status: `PASS / PRECISION_CERTIFICATE_REQUIRED / PRIVATE_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,990,600-byte executables at
SHA `81a86d4e1276955804357eea8d59a71392a8d04696991207cf210891abb7ba2a`
and Build ID `1f733f3d19b24ecccbe94b53d7fffed2644a0d2e`.

Both D7R10 processes exit zero with empty stderr and byte-identical 4,806-byte
stdout reports at SHA
`ee7b1d4eb0b5eb37334415fa38a1ee2c2a716fa9ab5b628985e7fed06c1c710d`.
The semantic result is
`48b498487db1265d28123dc2ae2edca92a66bddc2cee7714b3f6ffb77216e77f`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r10.0V7sd2`.

Both builds retain the exact D7R9 FAIL and D7R9R1 PASS stdout hashes:

```text
D7R9    2c45e93d4153edf714b0f490be3f3d760b14a1ead1e8ed8ae9528d2555ec91f0
D7R9R1  2f9a935e4a8f24bc60b8b146424d73c98b5d8dc47756bcf6a70b367f03f47e64
```

The two in-process runs are exact. All values are finite, model/HVP/radius
policy and accepted-sign ledgers pass, and public/input states roll back.

## Private convergence result

Every formerly failing inner converges with one candidate acceptance, no
rejects and two HVP calls:

| Request | Initial stationarity | Final stationarity | Predicted reduction | Divided reduction | Ratio | Extended sign |
|---|---:|---:|---:|---:|---:|---|
| `1e-8` | `1.197642e-8` | `2.021660e-14` | `1.727430e-15` | `1.727433e-15` | `1.000002` | resolved positive |
| `1e-9` | `1.179692e-9` | `1.664317e-14` | `1.676030e-17` | `1.676000e-17` | `0.999982` | resolved positive |
| `1e-10` | `1.534119e-10` | `1.045977e-14` | `2.834331e-19` | `2.834613e-19` | `1.000100` | unresolved positive observation |

All three accepted trials are inherited-raw rejections, so the selected
change is exercised. The first two cross 192 boundary support/kernel segments
and nevertheless agree with the quadratic model and extended oracle. The
tight trial changes no kernel segment or PHR branch.

There are three accepted trials, zero rejected trials and six total HVP calls.
Two accepted signs are resolved positive, none is resolved negative, and one
is unresolved by the frozen long-double threshold.

## Remaining precision boundary

For the tight accepted trial:

```text
divided binary64 reduction       +2.8346132192522592e-19
quadratic predicted reduction    +2.8343305480444868e-19
long-double compensated result   +1.9820570965750628e-19
long-double ULP ratio            +468
```

The binary64 candidate and analytic model agree to about `1e-4` relative,
but D7R8 intentionally requires at least 1024 long-double ULPs and therefore
does not certify this sign. Calling it positive from appearance alone would
violate the frozen evidence rule.

## Decision

Select `PRECISION_CERTIFICATE_REQUIRED`. The divided numerator removes the
observed inner convergence failure, but outer integration remains blocked.
Freeze D7R11 as an offline IEEE binary128/libquadmath energy oracle over
exactly the three D7R10 accepted pairs. It must use independently recomputed
energy, resolve signs well above binary128 ULP scale, compare the candidate
magnitude and authorize no runtime precision change.

