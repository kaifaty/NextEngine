# NSR3-B4E2D7R19R6 guarded-residual transaction research

Date: `2026-08-23`

Status: `COMPLETE / FULL PRIVATE TRANSACTION SELECTED / CONTRACT FROZEN NEXT`

## Question

D7R19R5 proves one exact guarded sixth trial is accepted. How should that
completion policy enter the full private first-substep transaction without
changing the five ordinary trials already proven by D7R19R2?

## Narrow integration rule

Residual-derived model reuse is not made the default. The trust completion
policy has two explicit paths:

```text
ordinary solve
  recurrence converges within base 32 HVP
  -> direct H(step) model HVP
  -> historical behavior and bytes

guarded solve
  recurrence reaches base 32 without convergence
  -> evaluate frozen near-convergence guard
  -> at most one HVP 33
  -> require forcing convergence
  -> model image = r_final - g
  -> no direct candidate model HVP
```

This preserves the first five trials exactly while admitting only the R5-
certified sixth completion. Any later solve can use the guarded path only by
satisfying the same predicate independently.

## Budget semantics

The existing `maximum_hvp_per_trust_step` combines recurrence and direct model
HVPs. R6 must make ownership explicit in the candidate trace:

```text
base recurrence HVPs     32
guard recurrence HVPs     1
absolute per-step HVPs    33
ordinary model HVPs        1 after recurrence convergence
guarded model HVPs         0
total transaction HVPs   <= 512
```

The absolute value `33` is not an unconditional recurrence cap. At attempted
HVP 33, the solver must first prove the frozen R5 guard. If the guard fails or
HVP 33 does not converge, the solve terminates structurally without a trial.

Ordinary solves that converge within 32 retain their direct model HVP; their
combined recurrence+model work must also fit the absolute 33-HVP cap.

## Candidate solver result

Refactor trust completion to return, privately:

```text
step
model image or direct-image requirement
recurrence HVP count
model HVP count
guard eligibility/use
termination owner
```

The legacy policy remains the default and must reproduce R2/R3/R4/R5 bytes.
The guarded-residual policy is passed explicitly only to one separate R6
candidate transaction using binary64-owned precision membership.

## Required trajectory anchors

Before later transaction behavior is interpreted:

1. trials `0..4` must reproduce the complete R2 binary64 trial projection,
   acceptance, radius, direct-model HVP and precision roots exactly;
2. trial `5` must reproduce R5 current/step/trial roots, predicted/divided
   bits, ratio, precision root, acceptance and unchanged radius exactly;
3. candidate work through trial `5` must be exactly five historical ordinary
   completions plus one guarded residual completion;
4. no trial can use residual-derived model image without recorded guard use.

Only subsequent trials/outer updates are new research observations.

## Why a full transaction is now justified

R3 isolated recurrence convergence, R4 model image, and R5 trial acceptance.
The remaining unknown is dynamical: after the sixth accepted position becomes
the current private iterate, does the inner solve converge, hit another
guarded boundary, reject, exhaust a structural cap or expose a new precision
issue?

That cannot be answered by another disconnected replay. A single bounded full
transaction is now the smallest experiment preserving causal order.

## Rejected integrations

### Residual-derived model on every trial

Rejected. It would alter the first five model-image arithmetic paths before
they have lane-specific correspondence evidence.

### Global unguarded cap 33

Rejected. Absolute capacity and authorization are separate: HVP 33 requires
the frozen convergence-state predicate.

### Commit a successful transaction publicly

Rejected. R6 is still a private first-substep candidate. Even a confirmed
solver state must pass the existing boundary/impulse ledger and a later
explicit public integration contract.

### Run a second substep or timing lane

Rejected. The first-substep result is not known yet, and shared-host
performance work remains stopped.

## Smallest falsifiable experiment

Freeze D7R19R6 to:

1. reproduce exact R5/R4/R3/R2 parent bytes;
2. retain the legacy direct-model policy as default and byte-exact;
3. run one separate guarded-residual candidate from the same frame-zero
   predictor, zero normalized dual and binary64-owned precision topology;
4. enforce base `32`, one guarded recurrence HVP, absolute per-step `33` and
   unchanged total/outer/trial/workspace/precision caps;
5. require exact R2 trials `0..4` and exact R5 trial `5` before later work is
   admissible;
6. record ordinary/guarded recurrence and model-HVP ownership separately;
7. retain all finite, divided-repeat, precision, static binding, lifecycle,
   mass, boundary, impulse, rollback and route-precedence controls;
8. classify the full transaction under the existing nominal route ladder plus
   explicit precision/structural failure precedence;
9. run no second substep, macro, trajectory or timing lane and publish no
   state.

## Authority boundary

D7R19R6 can select or reject one private first-substep transaction policy. It
cannot mutate public physics state, generalize the cap to production, run
another substep, claim performance or create runtime/production authority.
