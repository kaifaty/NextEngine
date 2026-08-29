# Nonlocal NSR3-A2 Hessian coefficient-tape evidence -- 2026-08-20

Status: `PASS / OUTER_STATE_HESSIAN_TAPE_V1_SELECTED / REPORT_ONLY`

## Outcome

`outer-state-hessian-tape-v1` passes every frozen correctness, memory and
performance gate. It assembles pressure-center Jacobians, pressure geometric
radial coefficients, immutable reference-viscosity coefficients and current
surface coefficients once per accepted outer state, then reuses them in every
Krylov Hessian-vector product.

The build cost is charged to both the combined HVP bucket and total solve time.
No dense Hessian, changed arithmetic order, reduced precision, pruning,
parallel reduction or Krylov-vector reuse is included.

| Particles | A1 build+HVP | A2 build+HVP | Speedup | A1 total | A2 total | Speedup |
|---:|---:|---:|---:|---:|---:|---:|
| 512 | `60.182 ms` | `24.789 ms` | `2.428x` | `99.592 ms` | `64.079 ms` | `1.554x` |
| 1000 | `132.037 ms` | `54.602 ms` | `2.418x` | `243.213 ms` | `164.065 ms` | `1.482x` |
| 1728 | `265.900 ms` | `113.847 ms` | `2.336x` | `496.239 ms` | `346.697 ms` | `1.431x` |
| 4096 | `942.607 ms` | `459.092 ms` | `2.053x` | `1810.752 ms` | `1335.068 ms` | `1.356x` |

These are medians from the retained clean campaign pinned to logical CPU 4,
with one warmup per implementation and seven alternating A1/A2 pairs. An
independent immediately preceding campaign also passed: combined speedup was
`2.449x`, `2.461x`, `2.295x`, `2.059x`, and total speedup was `1.558x`,
`1.471x`, `1.422x`, `1.356x` at the same four sizes.

## Exactness and capacity

- tape and A1 HVPs match bit-for-bit on all seven NSR2-B controls;
- every tested outer-state tape HVP matches A1 bit-for-bit;
- final state, objective, stop reason, operation counts and capacity maxima
  match A1 at 512, 1000, 1728 and 4096 particles;
- timing-independent NSR0--NSR2-C2 raw report SHA-256 values remain unchanged;
- the A1 tournament result SHA-256 remains
  `63e49c2f785b6eed8b6015dd6c19b8846208c21cb575a9bde958169ab244281b`;
- A2 timing-independent result SHA-256 is
  `035f0078c1d68ac0c018606f0db4c76cc99f62f2ecd7423613bb05cfab0de3c7`.

| Particles | Actual tape bytes | Frozen cap | Active pressure centers | Directed pressure records | Reference viscosity records | Surface records |
|---:|---:|---:|---:|---:|---:|---:|
| 512 | `3,257,664` | `3,808,000` | 208 | 18,842 | 15,892 | 19,492 |
| 1000 | `7,035,872` | `8,219,648` | 504 | 47,592 | 34,332 | 42,144 |
| 1728 | `12,921,664` | `15,150,336` | 992 | 100,288 | 62,396 | 77,756 |
| 4096 | `33,687,488` | `38,842,112` | 2,736 | 296,064 | 169,812 | 199,572 |

The state roots are unchanged from NSR3-A/A1:

| Particles | State SHA-256 |
|---:|---|
| 512 | `a403336eb5f09d67a25abf580c70ab89f84979cd92a8d66b4c46770ae7d8a0b0` |
| 1000 | `d029671ec131a76098bb9eced9bd3814635996c9c60fd8bd816db78cd3b2e7f0` |
| 1728 | `41d08b3ddd6288a52370510df99ea11d6c1bb15195e982960306832aadccac4c` |
| 4096 | `8d5e2afeb771ccdd696b342889cefda5491c2a7f40b52cea1a1c3e5360708e1d` |

## Retained raw timing arrays

All values are nanoseconds, in execution order.

| Particles | Bucket | A1 | A2 |
|---:|---|---|---|
| 512 | total | `[99173300,104990279,99151580,99591839,99600165,99867598,99116164]` | `[67953153,70725535,63918561,64924787,63671847,64078743,63598999]` |
| 512 | build+HVP | `[60049419,62654535,60101850,60181703,60297748,60366799,60146862]` | `[27450652,28272565,24712274,24978068,24575000,24788750,24571824]` |
| 1000 | total | `[241298391,253727849,244436134,243212939,251333558,238104905,237325477]` | `[183807109,185007842,169268559,164064825,162851690,163388431,161198865]` |
| 1000 | build+HVP | `[131278436,137029664,132036925,132210054,136935488,131153267,130562419]` | `[66158833,66487543,58760897,54170402,54100153,54601892,53450041]` |
| 1728 | total | `[493961004,497785800,494741955,493720400,498787848,497894133,496238765]` | `[346044892,343392866,344291849,347406267,346697412,353127265,361676902]` |
| 1728 | build+HVP | `[264666043,266430972,265666076,264851147,268158780,266543138,265899613]` | `[113846546,112111844,113352140,113739146,114229617,119113947,124613332]` |
| 4096 | total | `[1810179296,1809400749,1810751874,1813782063,1807593304,1855994040,1834206523]` | `[1342447899,1330998117,1332216633,1335067563,1331372281,1446972170,1341689810]` |
| 4096 | build+HVP | `[939484400,938721114,942606896,943085899,939883923,969034922,951759049]` | `[466796083,455567993,453240215,459092161,455958685,533627614,464807169]` |

## Decision

Select `outer-state-hessian-tape-v1` as the report-only CPU research baseline.
The result demonstrates that exact coupled curvature can be reused profitably;
it does not establish physical validity, real-time performance, GPU behavior or
runtime readiness.

Return to NSR3-B0 dimensional/profile derivation. No multi-step physical corpus
may run until coefficients are derived without visual tuning and the boundary
formula is reclosed under its own contract.
