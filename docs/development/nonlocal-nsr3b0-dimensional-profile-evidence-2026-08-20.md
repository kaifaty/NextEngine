# Nonlocal NSR3-B0 dimensional/profile evidence -- 2026-08-20

Status: `PASS / FORMULA_RECLOSURE_REQUIRED / PHYSICAL_EXECUTION_BLOCKED`

## Outcome

The dimensional discriminator passes twice byte-identically and rejects
`nuv-variational-fcr1` as a physical-profile identity. The reason is not a
solver failure or an unsuitable material coefficient: the declared raw cubic
density kernel has only one eighth of unit continuum mass and produces only
`0.125224338*rho0` on the canonical `H=3dx` reference lattice.

The released PeriDyno implementation computes a fixed lattice scaling factor
for this kernel family. Reconstructing that operation gives
`sK=7.985668078772472`. It makes the reference lattice density exactly `rho0`
and gives continuum integral `0.9982085098465513`.

| Control | Result | Gate |
|---|---:|---:|
| raw continuum integral | `0.12499999999999904` | analytic `1/8`, relative error `<=1e-12` |
| raw `H=3dx` lattice density ratio | `0.12522433816880058` | frozen value, current identity outside `[0.99,1.01]` |
| selected fixed normalization | `7.985668078772472` | reciprocal lattice ratio |
| normalized lattice density ratio | `1.0` | relative error `<=1e-12` |
| normalized continuum integral | `0.9982085098465513` | `[0.995,1.005]` |
| normalized `dW/dr` finite difference | `9.54e-12` | `<=1e-7` |
| cubic radial moment `I2` | `0.2652382611878 m` | analytic/numeric relative error `2.09e-16` |
| surface half-space constant | `1459*pi/210 = 21.826588959940516` | analytic/numeric relative error `7.00e-15` |

Diagnostic result SHA-256 is
`ae2d923db990069a31965960ec17e86f2a9e66274ada92d690e5acc492126051`.
Both raw reports have SHA-256
`57b3f0dc0e7c0152be4c8591c8adb079d12db6bc9d5180b65e124571dd11919a`.
All eight retained FCR1/NSR0--NSR2-C2 raw report hashes remain byte-identical.

## What the coefficients mean

The objective fixes the coefficient dimensions:

| Coefficient | FCR dimension | Continuum interpretation |
|---|---|---|
| `kappa` | joule | `K_eff = kappa*rho0/m` |
| `lambda`, `mu` | `kg m/s` | kernel- and resolution-dependent bond dissipation |
| `gamma` | `m/(kg s^2)` | `sigma_eff=(1459*pi/210) gamma rho0^2 dx^5` for a flat interface |

Consequences are material:

- copying paper `kappa=1` from `dx=0.005 m` to `dx=0.05 m` changes the
  implied bulk modulus from `8 MPa` to `8 kPa`; preserving the same modulus
  requires the discrete energy coefficient to scale with particle volume;
- the stopped product `kappa=9196.875` implies `73.575 MPa` under the corrected
  map, but remains a historically tuned value, not a selected material;
- product-scale `gamma=1000` would imply `6820.81 N/m`; ordinary water instead
  maps near `0.01066` at `dx=0.05 m`;
- positive `mu` dissipates rigid rotation, as the paper itself notes. It
  cannot be called objective Newtonian shear viscosity. Physical water and
  Poiseuille controls must use `mu=0` and derive the remaining simple-shear
  response from `lambda`.

These conclusions follow from the FCR energy and pair convention. Published
Table 1 strengths use the authors' source-shaped derivative, lattice scaling
and mass conventions, so their numerical values cannot be imported into the
corrected energy.

## Algebra anchors, not profiles

For `rho0=1000 kg/m^3`, `dx=0.05 m`, `H=0.15 m`, `m=0.125 kg`,
`dt=1/240 s`, one metre hydro head and allowed density strain `1e-3`, the
derived anchors are:

| Property target | Derived discrete value |
|---|---:|
| bulk modulus `9.81 MPa` | `kappa=1226.25 J` |
| nominal water viscosity `1e-3 Pa s`, `mu=0` | `lambda=1.413823172873555e-5 kg m/s` |
| IAPWS water surface tension `0.07274 N/m` at `20 C` | `gamma=0.01066442403928581 m/(kg s^2)` |

The corresponding groups are `H/dx=3`, `m/(rho0 dx^3)=1`,
`Pi_g=0.00340625`, `Pi_kappa=68.125`,
`Pi_lambda=9.42549e-6`, `Pi_mu=0`, and
`Pi_gamma=4.62866e-7`.

They remain algebra controls until a normalized formula passes exact
derivative/HVP correspondence and the later Poiseuille and flat-interface
validations. No visual tuning or old-profile inheritance is allowed.

## Primary-source boundary

The 2026 [Nonlocal paper](https://doi.org/10.1145/3799902.3811196) defines the
unified energy but does not publish an SI/resolution transform for Table 1.
The pinned [PeriDyno code](https://github.com/peridyno/peridyno/tree/1aa892bb296fe766d2f9249c881b8605af23a69b)
explicitly computes a lattice scaling factor. The surface target comes from
the [IAPWS release](https://www.iapws.org/relguide/Surf-H2O.html), which gives
`72.74 mN/m` at `20 C`.

## Decision

Freeze `lattice-normalized-cubic-v1` under a new
`nuv-variational-fcr2` identity. Apply the same immutable scale to `W`,
`dW/dr` and `d2W/dr2`, then repeat formula, pair, dense-Hessian/HVP and trust
controls before NSR3-B1. FCR1 and all of its evidence remain historical and
unchanged. Physical trajectories, CUDA and runtime authority stay blocked.

