# NSR3-B4E2D7R20 binary128 oracle evidence

Status: `PASS / ORACLE_UNRESOLVED / CANDIDATE NOT EXECUTED`.

## Immutable inputs

The offline oracle ran once from implementation `5be5b059` against manifest
semantic `420031730236445859426960bc5540ae8b8f9522d4b9263fe5e86cf5f13f85b4`
and operator-preflight semantic
`16a24ef3040411ef0bd29294f0887b29a1cce7690c7d8a8bee511ea58880b1f6`.
The executable checked the exact eight preflight problem roots before any
oracle cycle. Linux x86-64 GCC `__float128` had a 113-bit mantissa.

The frozen oracle used stable density-halfspace, contact-box and trust-ball
Dykstra projections; checkpoints were `2^8, ..., 2^18`; the simultaneous KKT
and primal-dual threshold was `2^-70`. No wall timing was admitted.

## Result

The harness passed all source, precision, lifecycle and finite gates, but the
terminal route is `ORACLE_UNRESOLVED`. Result semantic:

```text
885dc7bdb628e4d435b3cd6966c87b6866dd9ca33fec3d8b9718b8477328c60f
```

Four quiet transfer/control cases certified at cycle zero. Three excited
cases reached the full fixed cap:

| case | cycle | scaled primal | projected dual | complementarity | stationarity | scaled gap |
|---|---:|---:|---:|---:|---:|---:|
| supported column | 262144 | `1.328e-33` | `1.085e-32` | `6.531e-34` | `2.312e-17` | `4.216e-18` |
| blind filled edge | 262144 | `8.240e-7` | `3.530e-1` | `7.977e-3` | `1.384e-1` | `1.344e-1` |
| blind filled corner | 262144 | `2.087e-5` | `4.359e-1` | `2.490e-2` | `1.349e-1` | `5.049e-2` |

The supported case is feasible to the oracle's primal scale and misses only
the deliberately much stricter reference stationarity/gap threshold. The two
blind cases remain many orders of magnitude away in several independent KKT
components. Treating them as merely needing one more depth extension is not
supported.

Exact structural work for the three nonzero-cycle cases was 45,088,768
density projections, 786,432 box projections, 786,432 ball projections and
4,798,808,064 sparse coefficient terms. This is offline reference work, not a
simulation or production-performance measurement.

## Interpretation and boundary

The result neither validates nor refutes the R65 composed-dual candidate on a
general corpus, because the frozen contract forbids running that candidate
after an unresolved oracle. It instead exposes a corpus-admission gap: input
excitation (`P_D(t)` violates density) does not prove that the density,
contact and trust sets have a nonempty intersection.

Post-run review closes the feasibility question without another solve. Every
case has `source_positive=0`, the contact bounds contain the zero step, and the
trust ball contains zero. Thus `s=0` is an explicit feasible witness for every
materialized TRQP. The two blind outcomes prove that cyclic primal Dykstra is
an inadequate high-accuracy reference for these geometries at the frozen cap;
they do not expose infeasible corpus inputs.

R20R1 phase-I was consequently withdrawn before implementation or execution.
The next experiment must use a globally coupled, independently implemented
oracle rather than extend cyclic Dykstra. Tolerance changes, blind-case
geometry changes and candidate execution remain forbidden until that oracle
closes.

This evidence grants no runtime, GPU, nonlinear-step or production authority.
