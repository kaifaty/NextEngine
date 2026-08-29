# NSR3-B4E2D7R19R2 binary64 topology-policy research

Date: `2026-08-23`

Status: `COMPLETE / BINARY64-OWNED POLICY SELECTED / CONTRACT FROZEN NEXT`

## Question

D7R19R1 proves that D7R19's precision failure is confined to the C2
compact-support shell and does not alter any accepted reduction sign. Which
topology policy should a wider-precision acceptance audit use without changing
the binary64 solver objective or promoting runtime state to binary128?

## Ownership distinction

An accepted-step audit contains two different kinds of operations:

```text
discrete operation       pair belongs to current/trial support topology
continuous operations    radius, kernel, density, PHR and inertia arithmetic
```

The current binary64 solver already selected the discrete branch when it built
the current and trial sparse workspaces. Recomputing `r<=h` after promoting
the same coordinates to long double or binary128 does not merely improve the
same arithmetic; it can select a different branch and therefore evaluate a
different piecewise function.

The precision oracle must preserve the primary branch decisions and improve
only the continuous arithmetic. For each state and candidate-union pair:

```text
owned_member = norm_binary64(binary64_positions) <= h_binary64
extended_r   = norm_extended(promote_exact(binary64_positions))

if owned_member:
    evaluate extended W(extended_r) and accumulate in extended precision
else:
    contribute exact zero
```

The binary64 membership mask is frozen per current/trial state. The wider
evaluator still reports whether its independently computed radius would have
selected another branch, but that disagreement is diagnostic and cannot by
itself fail the sign certificate.

## Why this is the selected policy

D7R19R1 supplies the missing safety certificate over the exact first-substep
states:

- all long-double and binary128 disagreements lie inside
  `64*epsilon_binary64*h`;
- maximum observed mismatch `W/W'/W''` is zero;
- the cubic is exactly C2-closed at `h` in all three formats;
- live-extended, binary64-owned and horizon-canonicalized lanes all retain
  resolved positive signs;
- binary128 candidate errors are orders of magnitude below the frozen 5%
  bound.

The selected binary64-owned long-double roots for outer-0 trials `0..=2` are:

```text
11f649ace70101678751f87043d1791047e05c7b72eb406d4964b96fa89dcaa2
e889a80972764d09ccd948cc792889234df9a1f77cf3fd0a36dc84d56adc60c2
4e48d3b67c6262287a7dcd4796faf58a087318a152fc17bed19323770a91665e
```

This policy is also the smallest implementation change: the existing sorted
current/trial superset union remains unchanged, no epsilon is introduced, and
binary64 solver evaluation, HVP, trust ratio, radius update and candidate
trajectory remain bit-exact.

## Rejected alternatives

### Wider precision owns membership

Rejected for this audit. It evaluates another discrete graph and is the exact
mechanism behind D7R19's hard failure. It remains a useful diagnostic lane, as
retained by D7R19R1.

### Horizon epsilon or hysteresis

Rejected. An epsilon changes the effective kernel support and makes topology
depend on an unfrozen tolerance. Hysteresis adds future-affecting state and
would require separate persistence/replay semantics.

### Canonical integer coordinates own private support

Rejected for the nonlinear inner solve. Canonical integer topology is already
correct at durable publication boundaries, where equivalent committed
coordinates must have exact public identity. Applying publication
quantization inside the private solve would change radii, kernel values and
the physical objective.

### Canonicalize every mismatch radius to `h`

Retained only as an oracle control. D7R19R1 shows that it is observationally
equivalent on the frozen targets, but it replaces the independently computed
continuous radius. Binary64-owned membership expresses the actual solver
branch with fewer semantics.

### Runtime long double or binary128 state

Rejected. D7R19R1 proves no need for wider runtime state, and the project has
no portable GPU/runtime binary128 contract.

## Candidate transaction and pre-derived boundary

D7R19's precision flag is accumulated without changing the five accepted
binary64 trial states. After those five trials, a sixth trust solve selects
dimensionless forcing and reaches the frozen 32-HVP per-trust-step cap before
forming another trial. Therefore changing only audit topology cannot change
the following binary64 work:

```text
outer updates                    1
completed accepted/rejected      5 / 0
completed-trial HVPs             85
total budget HVPs                117
dimensionless trust selections   6
workspaces build/release          6 / 6
long-double accepted audits       5
all-pair calls                    0
terminal budget failure           STRUCTURAL_BUDGET_HVP_PER_STEP
```

The candidate is expected to expose the structural watchdog once the certified
topology mismatch no longer hard-fails precision. This is a pre-derived
hypothesis, not permission to relabel D7R19 or enlarge its cap.

## Smallest falsifiable experiment

Freeze D7R19R2 as a separate first-substep shadow:

1. reproduce complete D7R19R1 and D7R19 parent bytes;
2. execute one separate candidate transaction from the same frame-zero
   predictor with binary64-owned precision topology;
3. compare every binary64 current/trial state, candidate reduction, trust
   radius, HVP count and work ledger against the captured D7R19 path;
4. require the three selected binary64-owned long-double roots above and
   retain all membership mismatch counts as diagnostics;
5. require exact policy provenance and prohibit the policy from affecting
   workspace construction, HVP, acceptance or radius decisions;
6. retain every D7R19 finite, mass, boundary, impulse, rollback and route gate;
7. retain the 32-HVP per-step and all other structural caps unchanged;
8. run no second substep, macro, trajectory or timing lane.

Any binary64 trajectory difference, missing shell-parent certificate,
unresolved/negative audit, nonfinite value, lifecycle mismatch or rollback
failure is hard FAIL. Only after those controls pass may the existing nominal
route ladder classify the candidate.

## Authority boundary

A D7R19R2 PASS can reclose this one private first-substep precision policy
only. Even the expected structural-watchdog route would not confirm a solver
state or authorize another substep. The next action would be a bounded
conditioning/Krylov diagnostic at the exact sixth trust solve, not a cap
increase or a performance benchmark.
