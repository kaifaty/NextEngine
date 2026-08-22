# NSR3-B4E2D7R18R2 full normalized private transaction research

Date: `2026-08-22`

Status: `CLOSED / PASS / NORMALIZED_INNER_POLICY_STILL_INSUFFICIENT`

## Question

Does the directly normalized AL representation remain scale-invariant through
the complete nonlinear transaction, including Steihaug iterations, divided
acceptance, outer dual updates, confirmation, holdout and rollback?

D7R18R1 proves the normalized objective, gradient, HVP and divided reduction
at one fixed current/trial pair. It does not prove that repeated binary64
branches select the same nonlinear history. D7R17 therefore remains blocked
even though the isolated formula gate passes.

## Mathematical boundary

For positive `M`, `dt` and `kappa`, multiplying the dimensional objective by
`dt^2/M` preserves the exact minimizer, Newton equation and trust ratio:

```text
u     = lambda / kappa
theta = kappa * dt^2 / M

Ebar  = 1/2 ||y-y_hat||^2
      + theta/2 sum(max(0,u+c)^2-u^2)

gbar  = y-y_hat + theta sum(max(0,u+c) J)
Hbarp = p + theta sum((Jp)J + max(0,u+c) Hc p)
```

That equivalence does not imply byte equality with the dimensional D7R13
history. Binary64 CG computes residual norms, curvature, `alpha` and `beta`
after a different accumulation order. Requiring the old transaction root from
the new arithmetic would therefore confuse representation correspondence with
rounding identity.

The correct full-transaction oracle is instead:

1. preserve D7R13 as a byte-exact legacy regression;
2. require the normalized active fixture to reproduce D7R13's semantic
   provisional/confirmation/holdout sequence at outer `11/12/13`;
3. require the inactive fixture to remain exactly still at `0/1/2`;
4. derive the reference and aligned `theta` independently from their exact
   `{dt,kappa}` inputs and require the two complete normalized transaction
   roots to be byte-identical.

The third requirement tests inactivity. The fourth is the decisive scaling
test: the same geometry, prediction, initial `u` and exact `theta` must produce
the same nonlinear branches, precision roots, final private state and work
ledger at both physical scales.

## Admission and outer state

Use only normalized state for decisions:

| Gate | Frozen normalized form |
|---|---:|
| primal | `max(max(c,0)) <= 1e-8` |
| stationarity | `max(norm(gbar))/dx <= 1e-10` |
| dual update | `max(abs(delta u)) <= 0x3da1eed347666340` |
| complementarity | `max(abs(u_next*c)) <= 0x3d6cb1520bd70533` |
| position | `rms(delta y)/dx <= 1e-8` |
| feasibility | `u_next >= 0`, finite and primal-monotone |

`equivalent_pressure_change` may be reconstructed for diagnosis, but it does
not participate in admission. It scales with `kappa` and is not an invariant
state gate. The nominal pressure/contact/support impulse ledger remains a
later D7R19 requirement.

## Accepted-sign precision

Every accepted binary64 trial receives a direct normalized long-double audit
with naive and compensated accumulation. Pair membership must be exact; a
resolved negative reduction at `1024` long-double ULPs is a contradiction.
An unresolved ordinary acceptance is reported but does not change the legacy
acceptance policy.

Every candidate-effect acceptance -- divided reduction accepts while raw
subtraction would reject -- additionally receives a direct normalized
binary128 audit. Naive and compensated signs must agree, positive reduction
must resolve by at least `4096` binary128 ULPs, pair membership must be exact
and divided-reduction relative error must be at most `5%`. Binary128 remains an
offline oracle and cannot become runtime state or choose an iterate.

Both precision paths use the static-bound current/trial `0.04h` pair union.
Dense all-pair candidate enumeration is forbidden even on the tiny fixture so
the tested transaction remains structurally eligible for a later nominal run.

## Bounded experiment

Run five private transactions:

```text
reference active
reference active repeat
reference inactive
aligned active
aligned inactive
```

Each transaction has at most 64 outer updates. Each inner retains at most 64
trials, eight rejects, minimum radius `1e-14`, unchanged radius ownership and
at most `3N+1` HVP calls per trial. For eight particles, the complete frozen
upper bounds are 320 outer updates, 20,480 inner trials and 512,000 HVP calls.
These are safety bounds, not expected work or performance claims.

All state remains private. Active and inactive public roots are captured
before the run and must remain exact after forced rollback. No nominal
substep, macro, trajectory, timing, parallel or GPU path is admitted.

## Falsifiable classifications

1. `NORMALIZED_ACCEPTED_SIGN_CONTRADICTION` -- a selected accepted trial has
   independently resolved negative reduction.
2. `NORMALIZED_ORACLE_OR_REDUCTION_BOUND_REQUIRED` -- candidate-effect
   acceptance lacks the frozen positive binary128 certificate.
3. `FULL_NORMALIZED_PRIVATE_STATE_CONFIRMED` -- active and inactive
   confirmation/holdout semantics pass and both physical profiles have one
   byte-exact normalized transaction identity.
4. `NORMALIZED_INNER_POLICY_STILL_INSUFFICIENT` -- an inner solve fails before
   the required confirmation.
5. `NORMALIZED_OUTER_STATE_FORMULATION_REQUIRED` -- inners remain valid but
   normalized outer confirmation does not reproduce the frozen sequence.

Hard-control failures do not route around identity, legacy bytes, invalid
prework, pair membership, work/lifecycle accounting, cross-profile equality or
rollback.

## Decision

Freeze D7R18R2 as the smallest complete normalized transaction discriminator.
A PASS may authorize research/freeze of D7R19, one aligned nominal Dam substep
under the normalized representation. It does not itself authorize that run,
another substep, a macro, trajectory, timing, public state or production use.

## Closure

R2 executes reproducibly and classifies the active path at outer `1` as
`MINIMUM_TRUST_RADIUS`; see the
[dated evidence](nonlocal-nsr3b4e2d7r18r2-full-normalized-private-transaction-evidence-2026-08-22.md).
Reference/aligned roots are exact, but the normalized divided reduction loses
the positive outer-1/trial-2 signal by roughly three orders of magnitude.
Research a replay-only normalized precancellation discriminator next. Do not
retry the full transaction unchanged.
