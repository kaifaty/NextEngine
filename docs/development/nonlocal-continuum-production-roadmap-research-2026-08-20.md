# Nonlocal productionization roadmap research — 2026-08-20

Status: `REPORT_ONLY / NPR0_INPUT`

## Question

What must be closed before the retained fast Nonlocal GPU implementation can
be evaluated as the solver for the SPEC-38 basin consumer?

## Decisive local evidence

The fixed-work decision proves an exact standalone 50k GPU workload at about
`3.2 ms` p95/p99 on one Linux RTX 3080. The profile is boundary-free, uses
`dx=0.005 m`, `h=0.015 m`, `dt=0.001 s` and source-shaped coefficients.

The product consumer instead uses a `4×2×1 m` basin, `0.75 m` water depth,
`dx=0.05 m`, mass `0.125 kg`, `h=0.1 m`, `dt=1/240 s`, analytical sealed
geometry and later one moving crate. The nominal `48,000` samples are arranged
`80×15×40` with `+Y` as height. The historical 48k performance lattice is
`80×40×15`; the exact-50k decision lattice is `100×25×20`.

Equal particle counts therefore do not make these the same numerical or
physical workload.

## Primary-source boundary

The 2026 paper's resolution comparison varies spacing only from `0.0070 m` to
`0.0045 m`, keeps smoothing length `h=3dx`, `dt=0.001 s`, and uses fixed
`κ=1`, `λ=0.1`, `μ=0`, `γ=6`. Its Poiseuille example also reports
`dx=0.005 m`, `dt=0.001 s`. The pinned PeriDyno solver computes mass as
`rho0*dx^3` and feeds `dx`, smoothing length and `dt` independently into the
incompressibility, viscosity and surface terms.

These sources support the current source-shaped implementation. They do not
prove invariance under a tenfold spacing change, `dt=1/240`, `h=2dx`, the
SPEC-38 boundary or canonical micrometre publication.

Primary sources:

- [A Nonlocal Unified Variational Framework for Free Surface Flows](https://peridynamics.com/publications/2026-Liu-NUV.pdf), DOI `10.1145/3799902.3811196`;
- [pinned PeriDyno solver](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid/SemiImplicitUnifiedFluidSolver.cu), commit `1aa892bb296fe766d2f9249c881b8605af23a69b`.

## Conclusion

The next productionization work is not PhysX integration and not another
kernel optimization. It is a profile bridge that separates geometry/mass
scale, cadence, support ratio and boundary effects. Coefficients remain
unselected until dimensional analysis plus small physical controls close.

Only after that bridge may a wider independent corpus decide whether Nonlocal
is a physically credible water solver. Only after physical reclosure may an
authority decision fund private runtime integration.

## Rejected shortcuts

- Relabel the 50k benchmark as the basin: same count, different scale/cadence/
  support/boundary.
- Multiply every coordinate and mass while retaining all coefficients: the
  source exposes distinct `dx`, `h`, mass and `dt` factors in different terms.
- Use visual output to fit coefficients: it provides no conservation,
  reference or continuation guarantee.
- Begin GPU authority or public schemas: neither physical validity nor
  cross-target deterministic execution is closed.
