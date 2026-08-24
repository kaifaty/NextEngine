# NSR3-B4E2D7R19R41 support-crossing and contact research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Question left by R40

R40 has three independent negative admission facts, but frozen precedence
reports topology first. Before designing a composite normal/tangential step we
must determine whether the changed graph represents a physical kernel change
or only an identity change at the exact compact-support boundary, and whether
contact failure is a new collision or inward motion at an already active box
face.

## Exact compact-support boundary

The closed B2 cubic kernel uses pair membership `r<=H` and already proves

```text
W(H)   = 0
W'(H)  = 0
W''(H) = 0
```

exactly in binary64. A pair can therefore disappear from the computational
graph without changing density, gradient or Hessian if its source and trial
kernel values and derivatives are all exactly zero. Net pair count alone is
insufficient: entered and lost pairs must be merged by ordered identity and
evaluated at both states.

R41 uses no fitted shell epsilon. It records binary64 radius bits, signed
`H-r` margin, positive-value ULP distance from `H`, kernel value/first/second
derivative and density contribution for every symmetric-difference record.
A crossing is materially zero only if all six kernel quantities and the
complete per-center crossing-density ledger are exactly zero.

An exact-horizon-to-outside dense control must take this branch. A separate
strictly interior-to-outside control must retain a nonzero classification.

## Contact ownership

For every fluid particle, axis and lower/upper box face, compute current and
trial penetration and the signed normal component of the frozen R40 physical
step. The audit partitions face records into:

- already penetrating and worsened;
- newly penetrating;
- resolved;
- unchanged/nonpenetrating.

It roots the complete ordered ledger and reports the worst owner. No contact
projection, clipping or alternate trial is permitted.

## Scientific routes

After hard parent/source/work controls, precedence is:

1. any nonzero kernel crossing:
   `NONZERO_SUPPORT_CROSSING_REQUIRES_RELINEARIZATION`;
2. materially zero shell but a new penetrating face:
   `NEW_CONTACT_CROSSING_REQUIRES_CONTACT_SOLVE`;
3. no new face, but inward normal motion worsens existing penetration:
   `CONTACT_TANGENT_NORMAL_STEP_REQUIRED`;
4. zero-material shell and unchanged contact:
   `MATERIAL_TOPOLOGY_EQUIVALENCE_CANDIDATE`.

The last route changes neither R40 nor production policy. It would only allow
a later separately frozen proposal to distinguish material support from exact
zero-weight graph identity. The merit rejection remains independently open.

## Work and continuation

R41 replays R40 once and rebuilds exactly its two workspaces. One sorted merge
audits pair crossings; `6000*3*2=36000` direct box-face tests audit contact.
There are zero new JVP, VJP, HVP, trial, precision, model or outer operations
and no timing.

- nonzero support: relinearize/globalize before any composite design;
- new contact: integrate a contact-feasible normal subproblem first;
- worsened existing contact: research projection into the box tangent cone,
  then revisit full merit composition;
- material-equivalent topology and unchanged contact: preserve the exact
  evidence for a separate topology-policy proposal, then address merit.

Frozen contract:
[R41 crossing/contact audit](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r41-support-crossing-contact-contract.md).
