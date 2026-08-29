# NSR3-B3D2 -- finite-precision merit discriminator contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / B3_RETRY_BLOCKED`

Parent status: `NSR3B3D1_FAIL / REACTION_BELOW_ENERGY_RESOLUTION`, D1 semantic
SHA-256 `f9ed6a83170eff4c965b8c573eac0dd6f308beb41a6f7fcc365f8ad2b11a2ec7`.

This is a six-state diagnostic. It must not continue a physical trajectory or
change any parent solver decision.

## Inputs and replay

Replay face and corner D1 trajectories at fixed `96/192/384` substeps per
frame and capture the state, owned displacement and proposed trust step at the
first `REACTION_BELOW_ENERGY_RESOLUTION` exit. Require the exact parent raw
JSON-without-newline SHA-256
`6dede55270ca21c583ba83f5eebb740b0c0adf0e4b2a61cd00eb941906e4d259`.

Every captured substep index, predicted decrease, inherited floor, reaction
defect and reaction limit must equal D1. Capture state hashes are published.

## Objective-difference oracle

For the existing double-precision endpoint compressions `c0,c1`, correction
`p`, owned `delta` and `delta*`, evaluate:

```text
dPhi = 0.5*kappa * sum (c0-c1)*(c0+c1)
dI   = -0.5*M/h^2 * sum p*(2*(delta-delta*)+p)
dF   = dPhi + dI.
```

Evaluate the same factored expression in `long double` from the exact stored
double inputs. The binary64 implementation publishes a forward error bound:

- local potential terms use `gamma_4` times
  `0.5*|kappa|*(|c0|+|c1|)^2`;
- local inertia components use `gamma_5` times
  `0.5*M/h^2*|p|*(2|delta-delta*|+|p|)`;
- deterministic accumulation adds `gamma_(4N-1)` times the sum of absolute
  term magnitudes.

Require the double result to lie inside its bound from the extended-precision
result. A positive energy sign is certified only when `dF-bound > 0` and the
extended-precision sign agrees. An ambiguous sign remains ambiguous.

## Representation and stationarity probes

Publish for each state:

- correction norm in `dx` and changed materialized position components;
- minimum/max correction-to-position-ULP ratio for nonzero components;
- pressure active-center and pair-count correspondence at the current/trial
  endpoints;
- maximum density and compression change;
- current and trial reaction defect using owned displacement;
- unchanged reaction limit and residual reduction ratio.

Classify one next candidate only when all six states agree:

1. `FACTORED_OBJECTIVE_DIFFERENCE_CANDIDATE` if every factored decrease has a
   certified positive sign and extended-precision correspondence;
2. otherwise `FLOOR_STATIONARITY_MERIT_CANDIDATE` if topology is unchanged,
   every trial residual is finite, strictly lower, and at or below the
   unchanged reaction limit;
3. otherwise `LOCAL_GEOMETRY_OWNERSHIP_REQUIRED` if any nonzero correction is
   not materialized or the pressure endpoint cannot represent its change;
4. otherwise `NUMERICAL_GLOBALIZATION_STOP`.

No threshold may be fitted after observing the states. D2 PASS means the
diagnostic and classification are valid, not that the classified solver
candidate passes.

## Exit and regressions

Two D2 reports must be byte-identical. D1, B3D, B3 and B2 reports remain
byte-exact. Report identity is `finite-precision-merit-discriminator-r0`.

No full trajectory, B3 retry, physical corpus, CUDA, performance, runtime or
production authority is granted.
