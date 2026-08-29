# NSR3-B4E2D7R19R3 sixth trust-solve recurrence research

Date: `2026-08-23`

Status: `COMPLETE / REPLAY-ONLY DIAGNOSTIC SELECTED / CONTRACT FROZEN NEXT`

## Question

D7R19R2 removes the precision-topology hard failure without changing any
binary64 state or work. Why does the exact sixth Newton-CG trust solve then
consume all `32` permitted HVPs: is it a near miss, a trust-boundary event,
negative curvature, numerical loss of the CG recurrence, or ordinary slow
convergence of an ill-conditioned positive operator?

This question must be answered before considering a preconditioner or any
change to the production watchdog.

## Exact target

The failed solve is uniquely reconstructible from the R2 capture:

```text
outer update             0
completed trials         5 accepted / 0 rejected
current position         trial_position of accepted trial 4
trust radius             radius_after of accepted trial 4
predicted position       unchanged frame-zero predictor
normalized dual u        zero
theta bits               0x3fc5cccccccccccd
forcing policy           dimensionless stationarity, frozen per solve
workspace topology       binary64 sparse topology at current position
```

The enclosing candidate transaction is
`027dc6d474c87172a7848f71c33f30e0527e3915248b7b53ac077e0d946f2145`;
its complete binary64 trace is
`6e7a30213e967e910ab3245321850d5633f521327de20ce6fd8f699685179343`.
The derivation above is preferable to copying coordinates into a new fixture:
it keeps the replay attached to the exact accepted trajectory and makes any
future parent drift a hard failure.

## Competing mechanisms

The existing Steihaug recurrence solves `H p = -g` from `p=0`, with
`r_0=g`, `d_0=-r_0` and the stopping test
`||r_k|| <= eta ||r_0||`. The following mechanisms make different observable
predictions:

1. **Small tail beyond the watchdog.** Curvature stays positive, the iterate
   remains inside the trust ball and the forcing test succeeds shortly after
   iteration 32.
2. **Trust-boundary truncation.** A later unconstrained CG candidate crosses
   the radius and Steihaug returns the boundary intersection.
3. **Negative or nonfinite curvature.** `d^T H d` becomes nonpositive or an
   HVP/scalar becomes nonfinite.
4. **Ill-conditioned positive operator.** Curvature stays positive and the
   residual keeps decreasing, but the inferred Ritz interval is wide enough
   that 32 iterations are structurally insufficient.
5. **Finite-precision recurrence damage.** Residual decrease stagnates or
   reverses while adjacent residual orthogonality / direction
   `H`-conjugacy deteriorates.

A single terminal iteration count cannot distinguish mechanisms 4 and 5.
The replay therefore needs the recurrence, not merely a larger cap.

## Selected observations

For each HVP iteration, record exact binary64 roots of the point, residual,
direction and HVP image, plus:

- residual norm and ratio to `||r_0||`;
- frozen forcing `eta` and the forcing threshold;
- curvature, `alpha`, `beta` and direction norm;
- current/candidate point norm and trust-boundary predicate;
- normalized adjacent residual inner product;
- normalized adjacent `H`-conjugacy error;
- the CG-derived Lanczos tridiagonal coefficients.

For a positive interior prefix, derive smallest/largest Ritz values and a
condition estimate from the same scalars. This adds no HVP. Exact vector roots
are required because scalar agreement alone could hide a different Krylov
state.

## Prefix control

Instrumentation must be passive. The exact R2 transaction is run once with an
optional trace sink attached to the failed sixth solve; its existing report
bytes, work counters and `32`-HVP failure remain exact. A separately executed
offline replay starts from the captured target and must reproduce all first
32 iteration projections byte-for-byte.

This two-lane construction checks both directions:

```text
live R2 solve, cap 32       proves the traced prefix is the historical solve
offline replay, cap 128     proves continuation does not alter that prefix
```

If prefix equivalence fails, no continuation result is admissible.

## Why the offline cap is 128

`128 = 4 * 32` is large enough to separate a small tail from persistent slow
convergence while remaining a bounded diagnostic over one frozen 18,000-DOF
system. It is tiny relative to the solver's generic `3*n` loop bound and does
not authorize `128` as a runtime setting.

The replay stops earlier on forcing convergence, trust-boundary truncation,
negative curvature or nonfinite data. It cannot evaluate a candidate energy,
form a trial, run a precision audit or update trust radius/state.

## Rejected experiments

### Increase D7R19R2's watchdog and rerun the transaction

Rejected. That mixes diagnosis with policy, can create a new accepted state
before the mechanism is understood and destroys exact R2 work
correspondence.

### Add a preconditioner immediately

Rejected for this step. Without an unpreconditioned spectral/recurrence
baseline, a preconditioner result cannot identify what it fixed or whether it
changed the operator path.

### Run a second nominal substep or macro case

Rejected. There is no confirmed first-substep state, and the shared-host
performance stop still forbids macro/trajectory/timing work.

### Use timing as the discriminator

Rejected. This is a convergence-mechanism question. Wall time on the shared
host neither identifies conditioning nor grants production performance
credit.

## Smallest falsifiable experiment

Freeze D7R19R3 with these controls:

1. reproduce exact D7R19R2 stdout and semantic bytes;
2. derive the failed solve only from R2's fifth accepted trial and bind its
   state/radius roots;
3. passively capture the live 32-HVP recurrence without changing R2 bytes or
   work;
4. rebuild one static sparse workspace and replay at most 128 HVPs;
5. require the offline first-32 projection to equal the live prefix exactly;
6. emit one mutually exclusive terminal route: nonfinite, negative curvature,
   trust boundary, forcing convergence or offline-cap exhaustion;
7. publish recurrence and Ritz diagnostics without choosing or testing a
   preconditioner;
8. form zero trials, accept zero state, run zero precision audits and retain
   exact rollback;
9. run no candidate nominal substep, macro, trajectory or timing lane.

## Decision boundary

- Positive interior convergence only a few iterations after 32 would support
  a later cap-policy study, but cannot itself change the cap.
- A wide positive Ritz interval with continuing residual decrease would
  authorize research/freeze of a preconditioner discriminator.
- Strong orthogonality/conjugacy loss or residual stagnation would instead
  authorize a numerically stabilized Krylov-recurrence study.
- Boundary or negative curvature would redirect work to trust-region policy,
  not preconditioning.
- Offline-cap exhaustion without a clear mechanism requires a narrower
  follow-up diagnostic, not an unbounded continuation.

## Authority boundary

D7R19R3 can classify exactly one failed private trust solve. It cannot change
D7R19R2, increase a production cap, accept a trial, confirm/publish state,
claim performance or create runtime/production authority.
