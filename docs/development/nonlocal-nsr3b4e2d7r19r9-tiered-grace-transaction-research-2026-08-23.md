# NSR3-B4E2D7R19R9 tiered-grace transaction research

Date: `2026-08-23`

Status: `COMPLETE / NARROW PRIVATE INTEGRATION SELECTED`

## Question

R8 proves that one online-computable two-tier envelope classifies both known
Krylov completion boundaries and that the later 34-HVP residual-derived model
matches a direct oracle. It does not prove that the policy composes correctly
with trial formation, precision ownership, trust-radius updates and later
solves in the private normalized transaction.

The next falsifiable question is:

> Can the exact R8 tiered envelope replace only the guarded completion branch
> in one bounded private first-substep transaction while preserving ordinary
> direct-model work and every frozen R6/R5/R2 anchor?

## Evidence inherited from R8

The two observed completion paths are materially different:

```text
tier 1: r32 = 1.201853 eta -> HVP 33 converges
tier 2: r32 = 1.891245 eta
        r33 = 1.294362 eta, q33 = 0.684397
        r34 = 0.933412 eta -> converges
```

The later `r_final-g`/direct-`H(step)` comparison passes at
`1.05e-15` L2-relative, `5.83e-14` scaled-component and `2.86e-16`
quadratic/predicted-relative error. Therefore model-image correspondence is
not the remaining uncertainty. Transaction composition is.

## Competing integration choices

### A. Raise every solve cap to 34

Rejected. It erases the online evidence boundary and spends extra work on
solves that neither known tier justifies. It also changes the meaning of the
production cap rather than testing a research policy.

### B. Special-case the captured outer/trial/solve identity

Rejected. That would replay one known answer and provide no reusable solver
rule. Eligibility must depend only on recurrence state available online.

### C. Replace direct model HVPs for all converged solves

Rejected. R6 intentionally proved a narrower ownership rule: ordinary solves
converging within 32 recurrence HVPs retain one direct model HVP. Residual
completion is justified only after an admitted grace path records its final
residual.

### D. Add one explicit tiered completion policy

Selected. The existing direct and one-shot guarded policies remain exact.
One new research-only policy owns:

```text
ordinary convergence <= 32:
    recurrence HVPs + one direct model HVP

tier-1 entry after HVP 32:
    original R5 predicate
    HVP 33 must converge
    residual-derived model, zero model HVP

tier-2 entry after HVP 32:
    R8 wider entry predicate
    HVP 33 must pass R8 continuation predicate if not converged
    HVP 34 must converge
    residual-derived model, zero model HVP
```

## Failure semantics

The policy must distinguish:

- no tier admitted after HVP 32;
- tier 1 admitted but HVP 33 did not converge;
- tier 2 admitted but HVP-33 continuation failed;
- HVP 34 admitted but did not converge;
- ordinary structural budget exhaustion.

These remain structural research failures. None may be hidden as ordinary
solver non-confirmation or retried with an unconditional cap.

## Work ownership

The absolute candidate per-step capacity becomes 34 only for this explicit
private policy. The base recurrence limit stays 32. Total HVP, outer-update,
trial, workspace and precision caps remain `512/16/16/288/64`.

Every formed trial must satisfy exactly one ownership lane:

```text
ordinary: recurrence <= 32, direct model HVP = 1
tier 1:   recurrence = 33, residual model = 1, model HVP = 0
tier 2:   recurrence = 34, residual model = 1, model HVP = 0
```

Denied or nonconverged grace paths form no trial. The sum of recurrence and
direct-model HVPs must equal the structural ledger.

## Non-regression and stopping boundary

The private run must reproduce R8/R7/R6/R5/R2 parent bytes, exact R2 trials
`0..4` and the exact R5 sixth trial. It must expose per-tier attempts,
admissions, continuations, HVP work, convergence and model ownership.

The run stops at the first existing structural, precision or physics route.
It does not need to confirm the substep to pass the discriminator. A new
exact barrier is useful evidence if parent anchors, lifecycle, precision and
route precedence remain valid.

## Scope

R9 authorizes one private first-substep transaction from the same frame-zero
capture. It does not authorize a public commit, a second substep, macro,
trajectory, timing, production policy or performance claim.

## Decision

Freeze R9 around option D. Implement the smallest explicit tiered policy and
one report command. Do not refactor unrelated solver paths or interpret a
deeper terminal boundary as production readiness.
