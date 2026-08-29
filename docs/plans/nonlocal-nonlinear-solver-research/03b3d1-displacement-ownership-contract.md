# NSR3-B3D1 -- substep displacement-ownership contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / B3_RETRY_BLOCKED`

Parent disposition: `INACTIVE_FORWARD_ERROR_CERTIFICATE_REJECTED`, B3D
semantic SHA-256
`e99cda02f945e8402e6f4ca30742853c1fa29f1c845ed5c8a24db180877fa3b1`.

The continuum objective, trust model, reaction-aware active stop, boundary
support and post-solve contact order remain unchanged. The only candidate is
the transient numerical identity `owned-substep-displacement-r0`.

## Candidate arithmetic

For every substep:

```text
v*      = v_n + h*g
delta*  = h*v*
y*      = x + delta*
delta_0 = delta*

for every accepted trust step p:
    delta <- delta + p
    y     <- x + delta

for each accepted lower-face contact:
    delta_axis <- (wall_axis + R) - x_axis

y_next = x + delta
v_next = delta / h.
```

Rejected trust steps cannot modify `delta`. Trial coordinates use a temporary
`delta+p`. Contact edits only the final accepted displacement. Position is
rebuilt from the same displacement after each accepted smooth/contact update;
no later `(y-x)` reconstruction is allowed on the candidate path.

The report publishes bit-exact hashes for `delta`, position and velocity at
every captured step. It also evaluates the old world-subtraction velocity as
a non-authoritative counterfactual.

## Replay and bound gates

Reuse all B3D face/corner fixed `96/192/384` traces and captured active states.
Require:

- inactive displacement reconstruction uses a newly derived forward bound
  containing `|delta|/h` and `|v*|`, never `|x|/h`;
- every inactive per-step defect is inside that bound;
- cumulative bound `<=1e-10*M*c` at every ladder level;
- signed cumulative and direct terminal defects are within cumulative bound;
- translation/contact defects retain B3D limits;
- position consistency `||y_next-(x+delta)||` is bit-zero;
- velocity consistency `||v_next-delta/h||` is bit-zero.

The candidate's final positions and velocities must differ from the old
non-aborting B3D traces by at most `1e-5 dx` and `1e-5 c`. This is a numerical-
ownership test, not permission to change the physical trajectory.

## Active reaction and cost gates

Re-run the two B3D reaction-aware captures through displacement ownership.
Require the same mixed stationarity limit, position/velocity change gates and
`2.5x+2` HVP cap. Displacement storage is exactly one transient `Vec3` per
fluid sample (`24N` bytes in the strict-f64 oracle); no neighbor or Hessian
capacity changes.

## Exit

Two reports must be byte-identical. B3D, B3, B2 and B1R1 historical outputs
remain exact.

PASS selects `DISPLACEMENT_OWNED_REACTION_CANDIDATE` and authorizes only a
frozen B3R retry. Failure preserves B3 FAIL and selects either
`DISPLACEMENT_CERTIFICATE_REJECTED` or
`REACTION_AWARE_DISPLACEMENT_REJECTED` at the exact first gate.

No runtime/public state, hydrostatic/dam-break execution, CUDA, performance,
moving-solid or production authority is granted.
