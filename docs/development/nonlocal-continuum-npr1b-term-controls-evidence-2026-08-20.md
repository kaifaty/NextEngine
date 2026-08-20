# Nonlocal NPR1-B term-control evidence — 2026-08-20

Status: `NONLOCAL_PRODUCTION_RESEARCH_STOP / REPORT_ONLY / FORMULA_MISMATCH`

## Outcome

The exact NPR0 profile
`nuv-basin-48k-static-support-h3-physical.v4` fails the first mandatory
NPR1-B discriminator. The selected source-shaped SISSM force is not the
directional derivative of the published Nonlocal energy under the declared
product-scale MKS parameterization. Two fresh executions fail byte-identically
with first failure `KERNEL_SOURCE_GRADIENT_MISMATCH`.

This is a physical-definition failure, not CPU/GPU drift. The independent
energy derivatives agree with central finite differences at `6.27e-10` to
`3.45e-9`, while the selected implementation-force errors are `0.60` to
`0.98` against the frozen `1e-7` limit. The contract therefore selects
`NONLOCAL_PRODUCTION_RESEARCH_STOP` before NPR1-C or any long corpus.

## Independent control

Commit `a3da9b14d90cac65fd3fa0712807dd8d5f0579e2` adds a separate strict-f64
`--term-self-test` path to `nonlocal-npr1-canonical`. It does not call the
existing CPU oracle or CUDA pair kernels. It independently evaluates:

- cubic-kernel value and source-gradient goldens;
- the analytical derivative of the published energy and central finite
  differences;
- the physical force reconstructed from the selected Eq. 26 source/matrix
  arithmetic;
- translation invariance and equal/opposite pair closure;
- rigid-translation shear response and a rotating-sphere dissipation control;
- a counterfactual repair probe containing only the missing published
  derivative/coefficient factors.

The report returns nonzero on the selected formula mismatch. This is an
expected successful operation of a failing discriminator, not a crashed test.

## Results

| Control | Energy derivative error | Selected force error | Repair-probe error | Result |
| --- | ---: | ---: | ---: | --- |
| cubic `dW/dr` | `3.44019e-10` | `0.9250000000` | same as energy derivative | FAIL |
| incompressibility, Eq. 7/8 | `1.23537e-9` | `0.9250000001` | `1.23537e-9` | FAIL |
| bulk viscosity, Eq. 10–13 | `3.44458e-9` | `0.9593033631` | `3.44458e-9` | FAIL |
| shear viscosity, Eq. 10–13 | `2.22271e-9` | `0.9796516817` | `2.22271e-9` | FAIL |
| surface tension, Eq. 14/15 | `6.26612e-10` | `0.6000000003` | `6.26612e-10` | FAIL |

All translation-invariance errors are at most `1.73e-14`; every pair closure
error is zero. Rigid translation produces exactly zero shear response. The
source-shaped rotating-sphere control is monotonic in kinetic energy for
`mu=[0,10,100,1000]`, while its angular-momentum ratio falls from `1` to
`0.0525654`, consistently exposing the limitation reported by the paper.
Those local invariants do not override the force/energy failures.

## Bounded diagnosis

The [paper](https://doi.org/10.1145/3799902.3811196) defines `w(r)` as the
first derivative of `W(r,h)` and defines viscosity influence as
`omega(r)=-dW/dr`. The current
[PeriDyno implementation](https://github.com/peridyno/peridyno/tree/1aa892bb296fe766d2f9249c881b8605af23a69b)
instead has the following observable arithmetic:

1. `CubicKernel::weight` uses `q=2r/h`, but `CubicKernel::gradient` returns
   `dW/dq` without the chain factor `2/h`. Its dimensions remain `h^-3`
   instead of derivative dimensions `h^-4`. At product `h=0.15 m`, the exact
   source/energy-gradient ratio is `h/2=0.075`.
2. The viscosity kernel is called through `cuZerothOrder` and therefore uses
   `W`, not the paper's `omega=-dW/dr`. The bulk source coefficient also lacks
   the `1/2` present in the paper's `Lambda_ij`; shear uses the distinct full
   coefficient.
3. The surface source uses `strength*dt^2` with a spline expressed in
   `q=r/r0`. Eq. 14/15 and the chain rule require the corresponding `m/r0`
   factor when `strength` is the paper's `gamma`. For the product
   `m=0.125 kg`, `r0=0.05 m`, the selected/published force ratio is `0.4`.

The repair probe changes only those derivative/influence/coefficient factors
and closes every directional derivative below `3.45e-9`. This localizes the
failure; it does not authorize changing the selected profile.

As of 2026-08-20, PeriDyno `master` is still
`1aa892bb296fe766d2f9249c881b8605af23a69b`. The solver entered upstream in
[PR 94](https://github.com/peridyno/peridyno/pull/94); that PR contains no
review or issue discussion resolving these formula differences. No upstream
erratum was found in the bounded audit.

## Exact artifacts

| Artifact | SHA-256 |
| --- | --- |
| paper PDF | `610047ef32e895026c3661c57999f14ae550d8e2719ba2fcd95742371ad031c1` |
| pinned `Kernel.h` | `0bf941606d13c27ac18d56bf92e92756dabe9c3a0ef9cc651e8d5be61d800ec2` |
| pinned solver `.cu` | `23b78d69690eaf765598d3ccfe923051e5d411d6e94feb1152a86513ba80c516` |
| NPR1-B result root | `069aff09f7919fae33f86b2c86cb4a39d654188a15bd8c14868a9dfd5237e5c9` |
| raw report, both runs | `e8ed6950e33fb9388887c40e304c74b517761698c0c7accbaff9a99899bf2c3a` |
| executable | `3389bf44e070621245d0750ec28657264a8f8298bd2440b7fbc765ed4da795d2` |
| unchanged NPR1-A report | `464a55bc33741f18ddc4b3c1cda6b46b5248bb653789a5e31585482f871ee207` |
| retained CUDA self-test report | `11c4c4857a0efd9c290b5dc1ef493bc2efd0adca33d850f20dfb94baf13cf563` |

Execution used Linux `7.0.0-29-generic`, GCC `15.2.0`, CUDA `13.3` and an
RTX 3080. The term path itself is CPU binary64 with contraction and fast math
disabled.

## Consequence

NPR1-C through NPR1-E, Poiseuille and 16/32/64 convergence runs are
`NOT_RUN_AFTER_FIRST_FAILURE`. NPR2 through NPR8 remain blocked. The old
standalone performance measurements remain valid only for the exact
source-shaped arithmetic they measured; they provide no production-physics
credit.

A future corrected-variational lineage may use `dW/dr`, `omega=-dW/dr`, the
bulk `1/2` and surface `m/r0` factors, but it is a new solver/profile identity.
It must rederive coefficients and restart profile, tiny-physics and
performance reclosure. The failed v4 roots and coefficients cannot be
relabelled or inherited.
