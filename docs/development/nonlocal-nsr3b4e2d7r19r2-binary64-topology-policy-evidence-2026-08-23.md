# NSR3-B4E2D7R19R2 binary64 topology-policy reclosure evidence

Date: `2026-08-23`

Status: `PASS / NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED / NO STATE`

## Outcome

D7R19R2 closes D7R19's precision boundary without changing its binary64
solver path. Wider-precision accepted-step audits reuse the exact binary64
current/trial support masks, while independently recomputing radii, kernels,
density and energy in long double or binary128. Membership disagreements
remain visible diagnostics but no longer redefine the audited objective.

The candidate passes every topology-policy, precision, binary64 trajectory,
work, static-binding and rollback control. The next exact boundary is the
unchanged structural watchdog: after five accepted trials, the sixth
dimensionless-forcing trust solve consumes all `32` permitted HVPs before it
can produce a trial.

No solver state is confirmed or selected. This result does not authorize a
larger cap or another nominal substep.

## Exact parent correspondence

D7R19R1 and D7R19 remain exact:

- D7R19R1 stdout SHA-256:
  `f77eb3da05b6ddb2da815a228aef1709917a4761af1eea8d1c71a3a2f2cf0aa1`;
- D7R19R1 semantic result:
  `c5cc2c128104e006e491afd40b2ebb474508aa80d97a21c0750f3bec4758cd7e`;
- D7R19 stdout SHA-256:
  `f5811bfc7d5e986d72b9130f8e6cb90c5ae21bd347ce7f0b476c171fe8cff7bb`;
- D7R19 semantic result:
  `bcc6f588010999f23664209037a0daae961606ca29fdcc0d7a31e661902e185b`.

The complete binary64 trial trace is identical between D7R19 and the R2
candidate:

```text
parent trace     6e7a30213e967e910ab3245321850d5633f521327de20ce6fd8f699685179343
candidate trace  6e7a30213e967e910ab3245321850d5633f521327de20ce6fd8f699685179343
```

The trace binds every current/trial position root, stationarity, trust radius,
step norm, predicted/raw/divided reduction, ratio, radius owner, acceptance,
curvature flag and per-trial HVP count. Frame zero remains
`0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7`.

## Precision-policy result

The selected private policy is exactly:

```text
owner             binary64
predicate         norm_binary64(position_i-position_j) <= h
extended radius   recomputed from exactly promoted binary64 inputs
mismatch           diagnostic only
epsilon            none
hysteresis         none
canonical coords   not used in the private solve
runtime wide state none
```

All five accepted trials remain finite and long-double-resolved positive;
there are zero resolved-negative or unresolved audits. The three mismatched
trials retain `10,989` repeated membership observations and reproduce the
D7R19R1-selected binary64-owned roots:

```text
11f649ace70101678751f87043d1791047e05c7b72eb406d4964b96fa89dcaa2
e889a80972764d09ccd948cc792889234df9a1f77cf3fd0a36dc84d56adc60c2
4e48d3b67c6262287a7dcd4796faf58a087318a152fc17bed19323770a91665e
```

The candidate component transaction root is
`027dc6d474c87172a7848f71c33f30e0527e3915248b7b53ac077e0d946f2145`.
Its internal component-report stdout SHA-256 is
`57e66a3ba73b0087c8251d356653196d3f834d2be73d5355a54301f9aa856fd0`.

## Structural boundary

Candidate work is bit-for-bit/counter-for-counter identical to D7R19:

```text
outer updates                    1
completed accepted/rejected      5 / 0
completed-trial HVPs             85
total budget HVPs                117
dimensionless trust selections   6
inherited trust selections       0
workspaces build/release/maxlive 6 / 6 / 2
precision audits                 5 long double / 0 binary128
all-pair candidate calls         0
per-trust-step HVP cap           32
terminal failure                 INNER:STRUCTURAL_BUDGET_HVP_PER_STEP
```

The difference between `85` completed-trial HVPs and `117` total HVPs is the
exact `32` HVPs consumed by the failed in-flight sixth trust solve. No trial
position, precision audit or acceptance exists for that solve.

This is now the first admissible route because the topology policy closes the
preceding precision hard-fail. It does not imply that `32` is the wrong cap;
the recurrence mechanism is not yet measured.

## Reproducibility

Implementation commit:
`158c5a20` (`research: reclose binary64 topology policy`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r2-a.Qq1dk1`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r2-b.8WyHpd`.

Both binaries are `5,768,512` bytes, have SHA-256
`d1556d0bbcdb212e688beca738659d895186ed12252232a7e5dce4331e7420d2`
and GNU build ID `b4e980aff318f3d21808d8f6d6135c5d865a4a65`.

Both fresh processes exit `0`, emit empty stderr and reproduce:

- stdout-with-LF bytes: `2,657`;
- stdout SHA-256:
  `3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0`;
- semantic result:
  `f806858bba9d3cb5ae781a270ccccbda96c53ce995cca1cc5799e6af36599c95`;
- route: `NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED`.

Raw outputs are under:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r2-a.DyW1pE`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r2-b.ppV5Qp`.

## Decision

Retain binary64-owned topology for this private precision-policy lineage and
preserve D7R19/D7R19R1 as historical evidence. Do not add runtime wide state,
an epsilon, hysteresis or private canonical-coordinate topology.

Research and freeze D7R19R3 as a replay-only diagnostic of the exact failed
sixth trust solve. It must start from the fifth accepted candidate state and
original trust radius, reproduce the first 32 HVP recurrence exactly, accept
no trial, and distinguish:

1. negative/nonfinite curvature;
2. trust-boundary truncation;
3. residual stagnation or loss of conjugacy;
4. ordinary slow convergence from an ill-conditioned positive operator;
5. a small additional iteration tail.

Any continuation beyond 32 HVP is offline diagnostic work under a separately
frozen cap. It cannot change the R2 watchdog or become candidate work.
