# NSR3-B3D reaction-accuracy research -- 2026-08-21

Status: `COMPLETE / TWO_DEFECT_CLASSES / DIAGNOSTIC_REQUIRED`

## Problem restatement

B3 did not establish whether post-solve contact composes correctly because
its first face failure occurred before contact. The immediate question is
narrower:

> When may a position-accurate smooth solve publish a boundary reaction, and
> how much additional nonlinear work is required to certify it?

The B3 traces show two different numerical regimes that must not share an
unqualified tolerance.

## Defect decomposition

For one smooth substep define:

```text
v* = v_n + h g
v_s = (y_s - x) / h
J_actual  = sum_i m (v_s_i - v*_i)
J_model   = -h sum_i grad_y_i Phi
J_boundary = -h sum_b grad_q_b Phi
```

Then publish three independent defects:

```text
D_stationarity = J_actual - J_model
D_translation  = J_model + J_boundary
D_reconstruct  = sum_i m ((y*_i-x_i)/h - v*_i)
```

`D_translation` is the already passed B2 virtual-gradient identity.
`D_stationarity` measures whether the nonlinear stop is sufficient for
reaction authority. `D_reconstruct` isolates binary64 position-to-velocity
roundoff in a material-inactive step.

Contact adds a fourth independent identity only after those pass:

```text
D_contact = Jf_contact + Jb_contact.
```

## Competing hypotheses

### H1 -- active support needs a reaction-aware stop

Evidence for: the corner `/192` failing step has one active centre and accepts
the selected scale-aware stop with zero HVP calls, while its relative ledger
defect is `5.52e-3`. The displacement criterion answers a state-accuracy
question and is not algebraically equivalent to an impulse criterion.

Falsifier: continuing the unchanged trust solve cannot reduce
`D_stationarity` before the selected numerical-energy floor or trust-radius
floor.

### H2 -- the inactive failures are only floating-point reconstruction floor

Evidence for: inactive `/384` rows have no material HVP, absolute defects near
`2e-12 kg m/s`, and only fail because the denominator shrinks with `h`.

Falsifier: the measured inactive defect exceeds a conservative forward-error
bound or accumulates coherently beyond the global momentum budget.

### H3 -- contact splitting is the leading cause

Evidence against: the exact first face failure has zero contact events.

Disposition: rejected as the explanation of the current boundary. Contact
composition remains untested rather than failed.

### H4 -- loosen the B3 relative ledger

This would mix a real active stationarity defect with inactive arithmetic
roundoff and could make an incorrect reaction look valid.

Disposition: rejected. Any mixed certificate must add an independently
computed floating-point bound and may waive only the covered arithmetic part.

## Binary64 certificate

Use the standard forward-error factor

```text
gamma_k = k*epsilon / (1-k*epsilon)
```

with an operation count derived from the actual component-wise subtraction,
division, mass multiplication and deterministic summation path. The inactive
certificate is

```text
||D_reconstruct|| <= B_fp
```

where `B_fp` is computed from magnitudes of the exact inputs and reported per
step. It is not a fitted epsilon and cannot cover `D_stationarity` when
pressure is active.

Signed cumulative defect, cumulative L1 defect and direct final-minus-initial
momentum closure are all required. Signed cancellation alone is insufficient.

## Reaction-aware counterfactual

Keep the exact B3 objective, HVP, trust model, acceptance ratio and arithmetic
floor. Change only convergence ownership:

- inactive material retains the exact fast path and binary64 certificate;
- active support may stop only when the mixed impulse residual
  `||D_stationarity|| <= 1e-9*impulse_scale + B_fp` passes;
- if the numerical-energy floor is reached first, report
  `REACTION_BELOW_ENERGY_RESOLUTION` and reject reaction authority;
- all extra outer trials, HVP calls and rejected trials are charged.

This is a diagnostic counterfactual, not a selected solver until it passes
accuracy and cost gates.

## Decision

Execute the frozen
[B3D contract](../plans/nonlocal-nonlinear-solver-research/03b3d-reaction-accuracy-contract.md).
Do not retry B3, change contact ordering or relax its ledger before B3D
selects an evidence-backed reaction certificate.
