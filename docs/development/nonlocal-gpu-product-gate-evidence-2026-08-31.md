# NCGP6 product trajectory gate — 2026-08-31

## Verdict

`REFUTED_BOUNDED / PERFORMANCE NOT_RUN`.

The frozen NCGP6 implementation and apparatus reproduce exactly, but the
first 4k long-trajectory gate fails before the 50k stages. Hydrostatic hold
completes 112 GPU/CPU steps and then reaches a nearest-rank position p99 of
`2.576882866 mm`, above the frozen `2.5 mm` limit. Position RMSE remains
`0.780996279 mm`; the diagnostic maximum is `20.810782528 mm` at SampleId
`28200`, component `y`.

This is not a same-state formula failure. The first CPU/GPU step starts from
identical canonical bytes and has position RMSE `0.040051 um`, maximum
`0.113995 um` and an exact pressure-active signature. Corrected and stable-ID
permuted GPU executions remain exact through the stopped trajectory. Density,
compression, momentum, energy, containment, finiteness, capacity and solver
failure gates remain within their frozen limits.

The failure therefore says that at least 41 of the 4,000 stable-ID samples
have independently evolved CPU/GPU positions above `2.5 mm` at step 112. It
does not by itself establish whether the macroscopic water state has diverged.
Dam-break, orifice, 16k/50k correctness, 50k sealed-basin correctness and the
complete-step timing are `NOT_RUN`, as required by the frozen stop order.

## Frozen identity and reproduction

- source commit:
  `1066ef593d41918c570921a3447e0a48af9ea090`
- source tree:
  `e00a8ed2a46691a2db712f4e4c18bb3ad53c4854`
- aggregate NCGP6 contract root:
  `c6c021a20f8ad7141a000464b7427faf28d2e2e8205692e8f6b1052d00a753ed`
- NCGP6 leaf contract SHA-256 before implementation:
  `2a675b16bbb71891faa61737df3d14bfb683d9a787d22e14a23de36dbe5b1594`
- source root:
  `a4d52574c193926380b77be1a7de9da49fc10356634e4006e4db2853867b1af4`
- two clean Release binaries, byte-identical SHA-256:
  `9e3e1f072b3a9ced7f5e2e42a8e35a457a8ee59c5d1ad158c879a7a53af3c2c0`
- host: NVIDIA GeForce RTX 3080, SM 8.6, CUDA runtime/driver 13.3
- allocated device memory: `137251397` bytes
- flags: C++ `-O3 -Wall -Wextra -Wpedantic -Werror -ffp-contract=off
  -fno-fast-math`; CUDA `-O3 --fmad=false --prec-div=true --prec-sqrt=true
  --ftz=false`; `SM=86`

The exact gate command is:

```text
nonlocal-corrected-cuda-product-gate --product-gate-self-test
```

Both clean builds report PASS with byte-identical stdout SHA-256
`4380351cfd22e0afe502891aa3a7d9ddde0702288db33167fbbcf3798a3a41bd` and
result root
`f8a09ab731c537e7e85ce170eacf4df2b9a83f5e6bdc799f724f552ca4d72f33`.
The self-test fixes nearest-rank p99 index `3959`, proves that one large tail
remains diagnostic-only, proves that 41 samples above `2.5 mm` reject through
p99 while RMSE still passes, and rejects rank, sample, work and omitted-p99
mutations.

The exact physical command is:

```text
nonlocal-corrected-cuda-product-gate \
  --correspondence-4k-product hydrostatic-hold 240 128
```

Both clean processes exit `37` with byte-identical stdout SHA-256
`5ff5d205078534ff62e3e98cb46bfe5dd79a1b9c05c8d02cc24e701e10dbc31a`.
The result root is
`4e72a97b1add42e7c58f839fc66dd918879b4325c23f75e5e5f1cf696ea57009`,
the final state root is
`0f5e8ed895ef2bf9f65f76e615fa2ff0adf467de64d126be4171c04f41b41d9b`,
the position receipt root is
`001ec35478bf5b6fc77f06488ac176274187d76380bfcd786d6dcd123ba35d63`,
and the position-work root is
`ac7eccce01ca780b66089df03abe513abd67bada6024974d295f7e81784422b3`.

Raw JSON remains outside Git for this host session at
`/tmp/ncgp6-exact-a-gate.json`, `/tmp/ncgp6-exact-b-gate.json`,
`/tmp/ncgp6-exact-a-hydro240.json` and
`/tmp/ncgp6-exact-b-hydro240.json`. The hashes, embedded identities and exact
commands above are the durable locators.

## First-failure metrics

| Observable | Frozen limit | Observed maximum through step 112 | Result |
| --- | ---: | ---: | --- |
| position RMSE | `2.5 mm` | `0.780996279 mm` | PASS |
| position p99 | `2.5 mm` | `2.576882866 mm` | **FAIL** |
| position maximum | diagnostic | `20.810782528 mm` | warning |
| same-state first-step maximum | `5 um` | `0.113995347 um` | PASS |
| density correspondence RMSE | `5%` | `0.205559%` | PASS |
| density correspondence maximum | `10%` | `2.756778%` | PASS |
| compression-density RMSE | `5%` | `0.062021%` | PASS |
| compression-density maximum | `10%` | `1.449365%` | PASS |
| normalized momentum residual | `1%` | `0.238287%` | PASS |
| positive energy excess | `1%` | `0%` | PASS |
| basin penetration | `2.5 mm` | `0 mm` | PASS |
| permutation mismatch | exact zero | `0 steps` | PASS |

The primary and permuted GPU routes report no failure and use at most 125 HVP
of the frozen 128 budget. The CPU route reports no failure and uses at most
110 HVP. The pressure-active signatures of independently evolved CPU/GPU
states differ on 62 steps after the exact first step; the first difference is
step 8 at a density straddling the pressure threshold. This is sealed
diagnostic evidence of trajectory separation, not a same-state active-set
failure.

## Retained apparatus controls

The exact build-A retained controls all pass:

| Control | Raw stdout SHA-256 | Result |
| --- | --- | --- |
| profile | `b61d98e3c80387f6c667117e300f6d4dc1c3e13fa6a7a964a4f9e4d17c7ac97d` | PASS |
| graph | `1d2bfeddb9aeb22b7a0c7438ff03d4b28a642d7e7acc0859f15414191062454e` | PASS |
| boundary | `8eeef64cc4b7f7ea0a6308a8c1e2a6cafc3be8f1ed9161ee3eae4a1a3bf1c5e6` | PASS |
| transaction | `55923c2204115c8626d2b728d52c8ae93d29350274b8e914fe59395a5badee3c` | PASS |
| physics | `f510b13b6ca310c45135d0fb915b62d0974045d4e2b558545317d6909eb93d39` | PASS |

Compute Sanitizer and independent review are `NOT_RUN`: the positive route
stopped at the first physical gate, so neither is required to support a
positive claim. They remain mandatory if a successor reaches a final PASS.

## Bounded research conclusion

NASA's CFD verification guidance distinguishes implementation verification
from the accuracy of a calculation and recommends conservation checks plus
comparisons to highly accurate solutions. It also states that accuracy
requirements depend on the engineering quantity being used. The SPHERIC
oscillating-drop benchmark evaluates long-duration particle methods using
drop axes, energy history, volume conservation and field diagnostics; its
3-D dam-break benchmark is defined by free-surface evolution and experimental
data. These primary sources support testing macroscopic quantities of interest
rather than assuming that a stable-ID particle correspondence is itself the
product quantity:

- <https://www.grc.nasa.gov/www/wind/valid/tutorial/verassess.html>
- <https://www.grc.nasa.gov/www/wind/valid/tutorial/overview.html>
- <https://www.spheric-sph.org/tests/test-01>
- <https://www.spheric-sph.org/tests/test-02>

This evidence does not waive the failed NCGP6 gate. It motivates one bounded
successor diagnostic with competing hypotheses:

- **H7A — Lagrangian identity separation:** stable-ID p99 fails, but fixed-grid
  mass/density, free-surface and integral observables remain within a
  pre-frozen error budget;
- **H7B — macroscopic divergence:** the same step-112 witness also fails one or
  more fixed-grid or free-surface observables;
- **H7C — comparator sensitivity:** field errors materially change with voxel
  resolution, so the apparatus cannot yet support a product decision.

The smallest discriminator is to replay the exact step-112 witness and compare
CPU/GPU state on a canonical fixed Eulerian grid, with exact mass, momentum,
energy and containment retained. This must be a new frozen diagnostic; it
cannot silently reinterpret NCGP6 or admit performance. A product gate and
50k timing may resume only after the diagnostic is reviewed and the user
explicitly accepts the resulting quantity-of-interest contract.

## Decision

Close NCGP6 `REFUTED_BOUNDED`. Do not raise the p99 threshold, delete tail
samples, change particle identity, alter physics, increase HVP above 128 or
time the stopped candidate. Preserve NCGP4 and NCGP6 as separate honest
negative results. The next action is a root-closed Eulerian field diagnostic,
not another tolerance adjustment.
