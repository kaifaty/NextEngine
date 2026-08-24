# NSR3-B4E2D7R19R42 stable-superset relinearization research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Question left by R41

R41 proves that the R40 trial crosses compact-support pairs with nonzero
kernel curvature. Reusing the source active-pair graph is therefore invalid,
but rebuilding an unrelated graph and requiring identical pair IDs is also
too strong: active compact-support membership is expected to change under a
nonlinear step.

The missing abstraction is a stable *ownership* domain whose entries are
masked by exact state-local compact support. This is the classical purpose of
a Verlet neighbor list: a list radius larger than the physical cutoff permits
reuse while the interaction itself retains the original cutoff. The original
primary reference is Loup Verlet, 1967,
[`10.1103/PhysRev.159.98`](https://doi.org/10.1103/PhysRev.159.98).

## Existing mechanism and analytic coverage

The private CPU research path already owns a canonical B4EP3 superset:

```text
physical horizon H = 0.15
skin s             = 0.04 H = 0.006
list radius        = H + s = 0.156
reuse certificate = 4 d_max^2 <= s^2 (1 - 2e-12)
```

The factor four follows directly from the triangle inequality: if each fluid
particle moves by at most `s/2`, no pair that is within `H` at the new state
could have been farther than `H+s` at the anchor. Static support has only one
moving endpoint and is covered by the same conservative certificate.

R40 reports a *global* physical step L2 norm of
`6.141425261566961e-8`. Every individual displacement is bounded by that
global norm, which is far below `s/2 = 0.003`; therefore the source-anchored
superset must geometrically cover both exact R40 graphs. R42 still computes
and roots the actual maximum displacement and performs an ordered inclusion
audit rather than treating the bound as sufficient implementation evidence.

The skin is not physical support. `H`, `W`, `W'` and `W''` remain unchanged;
pairs with state-local `r>H` contribute exact zero and are excluded from the
active mask.

## Selected experiment

Build the canonical `H+s` superset once at the exact R40 source. From that
same sorted pair vector:

1. filter exact `r<=H` current and trial neighborhoods;
2. compare both, including order/CSR/radii/kernel coefficients/constraints,
   with independent canonical current and trial normalized workspaces;
3. prove ordered inclusion of both R41 pair ledgers in the superset and root
   the two state masks;
4. build the trial feasibility operator with freshly evaluated trial radii
   and `W'`, not source coefficients;
5. validate pair-once versus directed JVP, centered nonlinear
   superset finite difference, JVP/VJP adjoint identity and fixed-support
   translation behavior under the inherited R29 bounds;
6. report the trial hinge objective, violation, active rows and zero-origin
   projected mapping `||A_trial^T max(c_trial,0)||` without selecting a
   stationarity tolerance;
7. compare one deterministic current/trial JVP to establish whether the
   operator changed; this is attribution only, not an acceptance gate.

Centered finite differences evaluate all superset entries through exact
compact support at the perturbed positions. This deliberately permits pairs
to cross `H`; the closed kernel is `C2` there, so a correct first derivative
must remain consistent without a shell epsilon.

## Scientific routes

After parent/source/dense and exact work controls, precedence is:

1. certificate failure: `STABLE_SUPERSET_CERTIFICATE_REJECTED`;
2. missing current/trial active pair: `STABLE_SUPERSET_COVERAGE_REJECTED`;
3. canonical/masked mismatch: `STABLE_SUPERSET_CORRESPONDENCE_REJECTED`;
4. trial JVP/finite-difference/adjoint/translation failure:
   `TRIAL_RELINEARIZATION_REJECTED`;
5. all controls pass:
   `STABLE_SUPERSET_RELINEARIZATION_CANDIDATE`.

The last route authorizes only a later topology-admission design. It does not
retroactively accept R40: contact tangent feasibility and negative complete
merit remain open.

## Rejected alternatives

- expanding physical `H` to `H+s`;
- treating tiny `W` or density contributions as exact zero;
- freezing the current active graph or current Jacobian coefficients;
- building a union from the observed current/trial pair IDs as production
  policy, because it has no forward coverage certificate;
- fitting a support epsilon or stationarity tolerance;
- projecting contact, changing the normal step or accepting the R40 trial in
  this stage;
- wall/CPU timing on the shared host.

Frozen contract:
[R42 stable-superset relinearization](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r42-stable-superset-relinearization-contract.md).
