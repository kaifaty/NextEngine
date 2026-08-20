# NSR3-B4BK -- box-contact KKT discriminator contract

Status: `EXECUTED_FAIL / SIDE_FACE_HYPOTHESIS / B4B_RETRY_BLOCKED`

Parent B4B status is `FAIL / P1_REFERENCE_RUN`; semantic SHA-256 is
`59d7f2436f8bbbea974b47f4109d038ed09251012340accd3b0e0664b6ee2b16`
and JSON-without-final-LF SHA-256 must equal
`fef0035a7e0d005358d08d16ed134a656d73127c5ba215d2663f9feb95bdb5e3`.

## Identity and scope

```text
box-contact-kkt-discriminator-r0
```

This is a one-substep mathematical discriminator. It does not continue B4B,
change the pressure objective, select a general collision model or authorize
runtime work.

## Frozen states

Use the exact B4B P1 initial 48-fluid / 544-support state with zero velocity
and the exact static box. Evaluate one substep for each fixed schedule:

```text
h48  = (1/240)/48
h96  = (1/240)/96
h192 = (1/240)/192.
```

Replay the selected B3R post-solve candidate without changing it. Require:

- fixed 48 completes;
- fixed 96/192 stop at substep zero with
  `REACTION_BELOW_ENERGY_RESOLUTION`;
- their reaction defects, limits, predicted reductions, floors, step norms,
  residual ratios and topology-change flags equal B4B exactly.

Also use the exact initial B4B P2 block at `h48` as a detached negative.

## Bound-constrained candidate

For owned displacement solve

```text
min F(delta) = M/(2h^2)||delta-delta*||^2 + Phi(x+delta;q)
s.t. low-x <= delta <= high-x.
```

All constants, support samples, pressure branch and analytic HVP are
unchanged. Start from `clamp(delta*,low-x,high-x)`. Every accepted iterate is
feasible. Classify a scalar lower coordinate active only at exact equality
with total gradient `g>=0`, and an upper coordinate active only at equality
with `g<=0`; all other coordinates are free.

Use a projected active/free Steihaug trust step. HVP directions are zero on
the current active set; trial displacement is projected to the box. Accept
only finite strict objective decrease with the inherited trust ratio. At the
inherited binary64 energy floor, a trial may instead be accepted only when
the squared **projected KKT impulse residual** strictly decreases, active
multipliers remain nonnegative, and the complete ledger improves. At most
four such accepts and 64 outer trials are allowed. No coefficient, tolerance,
line search or best-of-run choice is permitted.

## KKT and physical gates

For each P1 step size require:

- exact feasibility and finite state;
- nonnegative lower/upper multipliers and exact complementarity;
- free-coordinate gradient and wrong-sign bound gradient represented in a
  projected KKT residual;
- `h*||r_KKT||` no greater than the existing mixed reaction limit;
- pressure support translation closure `<=1e-10`;
- contact fluid/reaction impulse closure `<=1e-12 N s`;
- complete normalized momentum ledger `<=1e-9` and penetration
  `<=1e-12 m`;
- the lower-y multiplier set is nonempty; no x/z or upper multiplier is
  active;
- velocity is reconstructed exactly as `delta/h`;
- objective does not exceed the feasible projected predictor by more than its
  computed binary64 forward bound.

Across `48/96/192`, report position, velocity, multiplier and impulse
differences. No fitted convergence-order gate is assigned to this
one-substep nonsmooth transition.

For detached P2 at `h48`, require the constrained path to be bit-identical to
the owned pressure-inactive semi-implicit gravity recurrence, with zero
pressure-active centres, zero multipliers, zero pressure/contact reaction and
zero penetration.

## Work, repeatability and exit

Report objective/gradient/HVP evaluations, projected trials, active-set
changes, floor-merit trials/accepts and first failure. Two reports must be
byte-identical. B4B, B4A, B3R, D5, original B3 and B2 raw outputs remain
byte-exact.

PASS selects `BOX_CONTACT_KKT_CANDIDATE` and authorizes only a separately
frozen `tiny-pressure-water-corpus-r1-contact-kkt` contract. FAIL rejects the
exact-hard-box candidate and authorizes research of a new SAM/IPC-style
contact-potential formula identity. Neither outcome authorizes P2 continuation,
joint neighborhoods, nominal water, viscosity, surface tension, CUDA,
runtime or production integration.

Execution is recorded in the
[dated B4BK evidence](../../development/nonlocal-nsr3b4bk-contact-kkt-evidence-2026-08-21.md).
The KKT and detached gates themselves pass; r0 is rejected because its frozen
claim that no x/z multiplier may activate is geometrically false for a column
that fills the box cross-section. The r0 identity is not amended.
