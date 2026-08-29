# NSR3-B0 -- dimensional and profile-eligibility contract

Status: `FROZEN / DIAGNOSTIC_FIRST / PHYSICAL_EXECUTION_BLOCKED`

Identity under diagnosis: `nuv-newton-krylov-r0` over
`nuv-variational-fcr1`.

This stage determines whether the corrected objective can represent a
dimensionally coherent material profile. It does not tune coefficients and
does not execute a trajectory. A solver optimization pass cannot compensate
for a density kernel or coefficient map with the wrong physical scale.

## Primary inputs

- [Nonlocal Unified Variational Framework](https://doi.org/10.1145/3799902.3811196),
  equations 3--15 and Table 1; pinned PDF SHA-256
  `610047ef32e895026c3661c57999f14ae550d8e2719ba2fcd95742371ad031c1`;
- [pinned PeriDyno solver](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid/SemiImplicitUnifiedFluidSolver.cu)
  and its `ParticleApproximation::calculateScalingFactor` implementation;
- [IAPWS surface-tension release](https://www.iapws.org/relguide/Surf-H2O.html)
  for the `20 C` water anchor `sigma=72.74 mN/m`;
- the verified FCR0 energy, pair convention and physical-distance derivative.

Publication Table 1 values are numerical strengths, not SI material
constants: the paper gives no unit or resolution transformation for them, and
the released code uses a lattice scaling factor plus source-shaped derivative
and mass conventions that differ from FCR0. Their numbers are comparison
inputs only.

## Dimensional ledger

Use base dimensions mass `M`, length `L`, time `T`.

| Quantity | Dimension |
|---|---|
| `x`, `y`, `dx`, horizon `H` | `L` |
| particle mass `m` | `M` |
| rest density `rho0` | `M L^-3` |
| time step `dt` | `T` |
| normalized cubic `W` | `L^-3` |
| `w=dW/dr`, `omega=-w` | `L^-4` |
| total objective and `kappa` | `M L^2 T^-2` |
| `lambda`, `mu` in the FCR pair potential | `M L T^-1` |
| FCR surface coefficient `gamma` | `M^-1 L T^-2` |

The last three rows are discrete-energy coefficients. They cannot be populated
with a continuum viscosity or surface tension by copying the same numerical
value.

## Mandatory kernel discriminator

For the FCR cubic definition, `q=2r/H` and
`alpha=3/(2*pi*H^3)`, prove independently that

```text
integral_R3 W_raw dV = 1/8.
```

For an infinite simple-cubic reference lattice with `H=3dx`, `m=rho0 dx^3`,
sum all integer offsets inside support and report

```text
R_raw = dx^3 * sum_k W_raw(|k| dx).
```

The current physical identity is ineligible when `R_raw` is outside
`[0.99,1.01]`. No change to `rho0`, `m` or `kappa` may hide this failure.

The only authorized remediation candidate is
`lattice-normalized-cubic-v1`:

```text
sK(dx,H) = 1 / R_raw
W = sK W_raw
dW/dr = sK dW_raw/dr
d2W/dr2 = sK d2W_raw/dr2.
```

The factor is computed once from the immutable reference `H/dx`, includes the
self sample, and remains fixed while particles move. It must make reference
lattice density equal to `rho0` within `1e-12` relative and the continuum
integral lie within `[0.995,1.005]` at `H/dx=3`.

This remediation changes density, active pressure sets, gradients and
curvature. PASS therefore requires a new objective identity
`nuv-variational-fcr2`; it cannot silently modify FCR1 or inherit its hashes.

## Dimensionless groups

Normalize length by `dx` and energy by `m dx^2/dt^2`. Every future profile
must publish at least

```text
Pi_H      = H/dx
Pi_mass   = m/(rho0 dx^3)
Pi_g      = |g| dt^2/dx
Pi_kappa  = kappa dt^2/(m dx^2)
Pi_lambda = lambda dt/(m dx)
Pi_mu     = mu dt/(m dx)
Pi_gamma  = gamma m dt^2/dx.
```

Equal raw coefficients at another resolution do not preserve these groups.
For fixed physical density, particle mass scales as `dx^3`.

## Derived material mappings

Let `Vp=m/rho0` and, for the normalized FCR cubic,

```text
I2 = integral r^2 (-dW/dr) dV = sK * (31/140) * H.
```

### Compression

Matching `kappa/2 sum_i (J_i-1)^2` to continuum bulk energy gives

```text
K_eff = kappa/Vp = kappa*rho0/m
kappa = K_target*Vp.
```

For a declared maximum hydrostatic head `Lh` and allowed small-strain density
error `epsilon_rho`, use the predeclared lower bound

```text
K_target = rho0*|g|*Lh/epsilon_rho.
```

### Viscous response

The bond model is not a general objective Newtonian viscosity. For the
manufactured simple shear `v=(s*y,0,0)`, its equivalent scalar Rayleigh
coefficient is

```text
eta_shear = (I2/Vp) * (lambda/30 + 4*mu/15).
```

For rigid rotation it dissipates

```text
R_rotation/volume = mu*I2/(6*Vp) * ||Omega||_F^2.
```

Therefore physically objective water and Poiseuille controls require `mu=0`.
`mu>0` may later be studied as a named non-objective numerical material, but
cannot be called physical Newtonian shear viscosity. With `mu=0`, derive
`lambda=30*eta_target*Vp/I2` and validate it once against Poiseuille; do not
select it from a sweep.

### Flat-interface surface energy

For the FCR directed-pair convention and its continuous compact spline,
half-space missing-bond integration gives

```text
aC = -pi * integral_0^3 q^3 C_hat(q) dq = 1459*pi/210
sigma_eff = aC * gamma * rho0^2 * dx^5
gamma = sigma_target/(aC*rho0^2*dx^5).
```

This is the flat-interface continuum-limit map. NSR3-B1 must validate it with
a resolution-paired planar or droplet control before it becomes profile
evidence.

## Frozen product-scale anchors

These are algebra-control outputs, not selected runtime profiles:

```text
rho0 = 1000 kg/m^3
dx = 0.05 m
H = 0.15 m
m = 0.125 kg
dt = 1/240 s
|g| = 9.81 m/s^2
maximum hydro head = 1.0 m
allowed hydro density strain = 1e-3
nominal water viscosity = 1e-3 Pa s
water surface tension at 20 C = 0.07274 N/m
```

Expected derived values, with binary64 output retained by the discriminator:

```text
R_raw ~= 0.12522433816880058
sK ~= 7.985668078772472
I2 ~= 0.2652382611878 m
kappa ~= 1226.25 J
lambda_water ~= 1.4138231728735551e-5 kg m/s
mu_water = 0
gamma_water ~= 0.010664424039285813 m/(kg s^2).
```

## Exit and stop rules

The diagnostic passes only if unit exponents, closed-form moments, lattice
sum, normalized density, derivative finite difference, material mappings and
dimensionless groups all meet their frozen checks.

Expected disposition for the current identity is
`FORMULA_RECLOSURE_REQUIRED`. It blocks NSR3-B1 and all physical trajectories.
The next implementation step is a separate normalization reclosure contract
that reruns FCR0/FCR1 and the exact HVP/trust controls under FCR2. If any old
hash changes before the new identity is explicitly selected, stop.

