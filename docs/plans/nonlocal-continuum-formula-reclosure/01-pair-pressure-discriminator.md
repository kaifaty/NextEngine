# FCR1 — pair enumeration and pressure semantics discriminator

Status: `PASS / FCR2_AUTHORIZED / REPORT_ONLY`

Predecessor: `FCR_ALGEBRA_CANDIDATE`

## Questions

1. Which concrete pair traversal reproduces the directed energy sum without
   hidden factor-of-two errors?
2. Does the operational compression-only rule avoid the tensile attraction
   produced by a literal two-sided density energy at a free surface?
3. What numerical parameter must a source-shaped surface update receive to
   represent the new contract's physical `gamma`?

## Canonical CPU pair rule

The corrected f64 reference enumerates each unordered non-self pair exactly
once in lexicographic `SampleId` order (`i<j`) and accumulates equal/opposite
endpoint contributions in that same order.

For one pair, the full position updates derived from the declared energies
are:

```text
viscosity:
  delta_y_i = -dt*omega/rho0 *
              [2*mu*P_t*delta_ij + lambda*P_n*delta_ij]

surface:
  delta_y_i = -2*gamma*m*dt^2*c(r/r0)*n_ij

delta_y_j = -delta_y_i.
```

An implementation that traverses both directed edges and scatters to both
endpoints must use half of each full coefficient on each visit. Traversing
both directions with the full coefficient must fail the discriminator.

This canonical CPU rule does not prescribe the later CUDA layout. A
directed gather may be proposed in FCR5 only if it reproduces the selected
CPU result within the correspondence gate and keeps its own deterministic
publication contract.

## Pressure cases

Two semantics are compared on identical states:

```text
selected:  Phi = kappa/2 * sum_i max(rho_i/rho0-1,0)^2
comparator: Phi = kappa/2 * sum_i     (rho_i/rho0-1)^2
```

The corpus contains:

1. one underdense two-particle pair;
2. one under-resolved `3x3x3` free-surface patch at product `dx=0.05 m`,
   `h=0.15 m`, `m=0.125 kg`, `rho0=1000 kg/m^3`.

For both cases the selected rule must produce zero pressure force while all
particles remain below rest density. The two-sided comparator must produce a
strictly positive inward corner force. Both variants must conserve total
linear momentum within normalized `1e-12`; this conservation property does
not make the comparator physically acceptable.

## Surface parameter mapping

The contract's full pair update contains `gamma*m`. The discriminator tests
a directed/scattered source-shaped coefficient twice:

- `strength=gamma` must not be accepted as the same physical parameter;
- `strength=gamma*m` must reproduce the declared full pair update at relative
  error `<=1e-12`.

No macroscopic calibration of `gamma` is claimed here; that belongs to
FCR3/FCR4.

## Exit gate

- unique-full and directed-half viscosity/surface accumulation agree with the
  energy-derived update at relative error `<=1e-12`;
- directed-full and unique-half are observably rejected (`>=0.49` relative
  error under the frozen metric);
- compression-only underdense force is `<=1e-12` absolute;
- two-sided patch corner inward force is `>1e-6` and total normalized
  momentum residual is `<=1e-12`;
- `strength=gamma*m` passes and `strength=gamma` is rejected;
- two executions are byte-identical;
- FCR0 and the frozen old-line pass/fail controls remain unchanged.

Passing selects the exact pair and pressure semantics for FCR2. It does not
select physical coefficients or authorize CUDA/runtime work.

The implementation passes. See the
[FCR1 evidence](../../development/nonlocal-continuum-fcr1-pair-pressure-evidence-2026-08-20.md).
