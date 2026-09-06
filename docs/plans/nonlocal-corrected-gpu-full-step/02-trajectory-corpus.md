# NCGP1 exact trajectory corpus — revision 3

| Field | Value |
| --- | --- |
| Research ID | `NCGP1` revision 3 |
| Status | `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY` |
| Parent | revisions 1 and 2; this document only resolves previously unspecified corpus duration and initial conditions |
| Selection time | before CPU trajectory solver or trajectory comparator implementation |

Revision 3 changes no coefficient, arithmetic profile, physical tolerance,
work ceiling, performance window or stop rule. It makes the fixed corpora in
revision 1 executable rather than allowing scenario duration to be selected
after observing correspondence.

## Tiny and invariance cases

- `compressed-pair` and `combined-tetrahedron` are the immutable NCGA5
  fixtures and profiles translated by `(0.75,0.75,0.75) m`; one converged
  implicit substep is compared to the independent CPU solve. Translation does
  not alter their graph or objective. The historical unpreconditioned policy
  is used and their `5 micrometres`/active-set gates are unchanged.
- `free-fall` is one isolated sample at `(0.75,0.75,0.75) m`, no ghosts, for
  32 steps. Its analytic position and velocity are evaluated at every step.
- translation/rotation invariance uses a `4x4x4` compact block, no ghosts, for
  8 steps. The translated case adds `(0.5,0.25,0.25) m`; the rotated case uses
  a positive 90-degree rotation around gravity through the block centroid.
  Initial velocities receive the same transform. No case reaches the box.
- viscosity decay uses the retained two-sample tangential counterflow at the
  translated interior position for 32 steps. Total kinetic energy must be
  monotone non-increasing up to a binary64 relative allowance of `1e-6`.
- surface relaxation uses the retained four-sample tetrahedron with zero
  gravity and pressure inactive for 32 steps. Its independent CPU energy must
  decrease and the GPU state remains subject to the common `5 micrometres`
  tiny correspondence gate.

## 4k CPU-oracle trajectories

Each trajectory has exactly 16 accepted substeps (`1/15 s`) and uses the
three-layer basin ghost shell:

1. `hydrostatic-hold`: `20x20x10`, zero initial velocity and normal gravity.
2. `dam-break`: `10x20x20`, zero initial velocity and normal gravity.
3. `orifice-jet`: `20x20x10`; samples inside the cylinder
   `(y-0.75)^2+(z-0.25)^2 <= 0.15^2` start with `v_x=1.5 m/s`, all others
   start at rest. The name denotes a frozen jet initial condition; there is no
   internal baffle or open outer wall, and the analytical outer basin remains
   sealed.

Both solvers start from identical binary32-representable state bytes. At every
accepted step compare positions and densities by ascending SampleId. The
published RMSE/max are maxima over all 16 snapshots. A failure or differing
sample/active-set cardinality stops the corpus before the next scenario.

## 50k trajectory and metrics

The primary `50x40x25` coherent state executes exactly 240 accepted substeps.
No CPU 50k oracle is claimed. Every step must use the selected smallest passing
HVP ceiling from `{32,64,128}` and an identical work route for equal semantic
inputs. Coherent/permuted identity is checked for the first, 120th and 240th
accepted states; the advected capacity state is a one-step admission/control.

Density percentage errors are candidate-minus-CPU errors normalized by
`rho0`. Momentum residual is

```text
||P_new-P_old-dt*(M*g+F_ghost)-I_contact||
-------------------------------------------------
max(||P_old||, dt*M*||g||, spacing*M/dt)
```

where `F_ghost` is the separately reduced pressure force exerted by fixed
ghost centers and `I_contact` is the declared impulse from swept analytical
projection. Internal pressure, viscosity and surface forces are not inserted
as an external correction. Positive energy excess and reversible drift use
the initial complete mechanical energy as denominator, floored by `1` joule.

The 240-step run records every state scalar needed by these metrics but emits
full vectors only at the three permutation checkpoints. The parent physical
thresholds and the rule that correctness precedes timing remain unchanged.
