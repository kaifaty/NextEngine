# NSR3-B3D -- reaction-accuracy diagnostic contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / B3_RETRY_BLOCKED`

Parent failure: B3 semantic SHA-256
`9e0eb0baf63c6bf1ae0dd5288cd8809c1722ddf354a1fff86f3d70935f2a56fe`.

The objective, boundary formula, post-solve contact hypothesis, fixtures and
trust model remain unchanged. The report identity is
`boundary-reaction-accuracy-diagnostic-r0`; it grants no trajectory or reaction
authority by itself.

## Replay matrix

Replay both frozen B3 fixtures without aborting on ledger observations:

- adaptive controller candidates through the first failing macroframe;
- fixed `96/192/384` substeps per macroframe through the first failing event;
- one material-inactive ballistic control at each corresponding `h`;
- one captured active pre-contact face state and one active corner-contact
  state from the exact B3 causal trace.

The post-solve sweep remains enabled but every record is classified
`PRE_CONTACT`, `CONTACT` or `POST_CONTACT`. Stable geometry/state hashes must
bind the B3 replay states.

## Required per-step observables

For every replayed step publish vectors and norms for:

```text
J_actual       = sum m (v_s - (v_n+h*g))
J_model        = -h sum grad_y Phi
J_boundary     = -h sum grad_q Phi
D_stationarity = J_actual - J_model
D_translation  = J_model + J_boundary
D_reconstruct  = sum m ((y*-x)/h - (v_n+h*g))
D_contact      = Jf_contact + Jb_contact.
```

Also publish active centres, scaled-displacement residual, gradient norm,
predicted energy reduction, numerical-energy floor, trust radius, stop reason,
outer/reject/HVP counts, contact features and cache epoch.

For each trajectory publish maximum absolute/mixed residual, signed cumulative
vector, cumulative L1 norm and direct terminal momentum closure. No residual
may be omitted because its step later contacts a wall.

## Forward-error bound

Compute, do not fit,

```text
gamma_k = k*epsilon/(1-k*epsilon)
```

for the actual deterministic component path. `k` includes position
subtraction, division, velocity subtraction, mass multiplication and ordered
summation for all fluid samples. The per-step `B_fp` is the componentwise
gamma bound combined by Euclidean norm.

Inactive controls pass only if:

- `||D_reconstruct|| <= B_fp` at every step;
- signed cumulative defect and direct terminal defect are each within the sum
  of their `B_fp` bounds;
- the bound is finite and no larger than `1e-10` of the trajectory momentum
  budget `M*c`.

`B_fp` cannot cover an active `D_stationarity` term.

## Reaction-aware stop counterfactual

Re-run only active smooth steps from identical immutable starts. Retain all B3
trust rules, but replace the ordinary successful convergence exit with:

```text
||D_stationarity|| <= 1e-9 * impulse_scale + B_fp,

impulse_scale = max(
    ||J_actual|| + ||J_model||,
    M*h*||g||,
    1e-12 kg m/s).
```

The ordinary stop is reported but cannot commit an active reaction-aware
state. The counterfactual may continue until the mixed condition passes,
`64` total outer trials, `8` rejected trials, minimum trust radius or the
selected numerical-energy floor. Reaching the energy floor before the mixed
condition emits `REACTION_BELOW_ENERGY_RESOLUTION`; it is not a PASS.

Require:

- translation and contact defects retain B2/B3 limits;
- reaction-aware `D_stationarity` passes on both captured active states;
- position change from the ordinary smooth result is `<=1e-5 dx` and velocity
  change is `<=1e-5 c`;
- HVP calls are at most `2.5x` the ordinary active-step calls plus two;
- no new reject, trust-radius, support-capacity or active-set failure.

## Exit

Two reports must be byte-identical; B3 FAIL, B2 and B1R1 raw reports remain
byte-identical.

- PASS with both inactive forward-error and active reaction-aware gates
  selects `REACTION_AWARE_STOP_CANDIDATE` and authorizes a separately frozen
  B3R retry.
- If inactive certification fails, stop for arithmetic-ledger redesign.
- If the active solve reaches an arithmetic/trust floor or exceeds work,
  select `REACTION_AUTHORITY_TOO_EXPENSIVE_OR_UNRESOLVED` and keep B3 blocked.

No outcome authorizes hydrostatic/dam-break execution, CUDA, runtime schemas,
moving solids or production integration.
