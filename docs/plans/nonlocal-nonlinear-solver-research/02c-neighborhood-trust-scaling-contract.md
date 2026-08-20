# NSR2-C -- neighborhood trust-solver scaling contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

Selected inputs: `canonical-cell-neighborhood-v1`, unpreconditioned
Steihaug--Toint trust-region Newton-CG and the NSR2-A1 scale-aware stop.

## Implementation boundary

- Build the immutable reference-position viscosity pair list once.
- At each accepted/current outer state, build one current pair list and sorted
  pressure adjacency; reuse both for every inner HVP and model-reduction HVP.
- Each valid trial builds its own current pair list before objective/gradient
  evaluation. A rejected trial publishes neither pairs nor state.
- On acceptance the trial pair list becomes the next current list; do not
  rebuild it solely because the outer iteration advanced.
- The exact NSR1 trust policy and NSR2-A1 stopping criterion remain unchanged.

## Correspondence controls

For the 8/27/64 NSR2-A1 fixtures, compare the neighborhood solver to the
all-pairs unpreconditioned solver. Final position, objective, gradient,
accepted/rejected counts, HVP count, active-set changes and convergence stop
must be bit-for-bit identical.

## Scaling controls

Use centered cubic lattices with side counts `5`, `8` and `10` (125, 512 and
1000 particles), placement pitch and material spacing `0.05 m`, horizon
`0.15 m`, the NSR2-A1 coefficients/velocity field, and
`rho0=max_i rho_i(y_star)/1.05`.

The report records initial/final pair counts, maximum neighbors, outer/accepted/
rejected trials, objective evaluations, HVP calls, pair-build count, maximum
pair count, active-set changes, objective and scale-aware residual. Wall time
is deliberately excluded from the exact report; NSR3 owns performance.

## Gates

- every correspondence control is bit-exact;
- every scale case converges with finite state, monotonically accepted
  objective and momentum residual `<=1e-12`;
- final scaled displacement residual `<=1e-8`;
- outer trials `<=32`, rejected trials `<=8`, HVP calls `<=128`;
- maximum neighbor count `<=160` and maximum unique pair count `<=80*N`;
- pair count/capacity is validated before each evaluation;
- two reports are byte-identical and all historical hashes remain unchanged.

PASS selects `NSR_NEIGHBORHOOD_TRUST_CANDIDATE` and authorizes NSR3 measured
CPU optimization plus broader physical controls. A second correspondence
mismatch stops the lineage. A scale-work failure records the exact dimension
and motivates a separately frozen global preconditioner; it does not reopen
the rejected local block metric.

