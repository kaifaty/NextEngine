# NSR3-B4E2D7R14 sparse AL workspace research

Date: `2026-08-22`

Status: `CLOSED / PASS / SPARSE_AL_WORKSPACE_CANDIDATE`

Contract reclosure: `v2` corrects one pre-implementation label. The value
`5a9d2f67...c07` is D1's complete reference-frame report SHA, while the
decoded frame-zero raw-bit root is `0d567ba5...4d7`. Both bytes are retained;
no topology, formula, route or permitted execution changes.

## Question

Can the D7R13 full private augmented-Lagrangian transaction be expressed over
the already verified canonical neighborhood/CSR infrastructure without any
binary64 result change, before a nominal Dam solve is attempted?

D7R13 establishes a consistent tiny pressure-state transaction, but its dense
oracle enumerates every possible fluid-fluid and fluid-support interaction.
Applying that implementation directly to the selected nominal Dam state would
perform:

```text
17,997,000 fluid-fluid candidates
98,304,000 fluid-support candidates
116,301,000 total candidate checks per evaluation
```

The decoded nominal topology contains only `342,502` canonical pairs,
`611,520` directed records and maximum degree `120`. Sparse traversal therefore
removes about `339.563x` candidate visits before any parallelism, SIMD or GPU
work. This is a structural work ratio, not a timing or production claim.

## Why the existing penalty tape is insufficient

The existing `JointPressureTape` stores the normalized penalty path. For
compression `c = rho/rho0 - 1`, its active coefficient is `kappa*c`. The PHR
augmented-Lagrangian path instead uses:

```text
a = max(0, lambda + kappa*c)
energy   = (a*a - lambda*lambda) / (2*kappa)
gradient = a * dc/dx
HVP      = kappa * (dc/dx dot v) * dc/dx + a * d2c/dx2[v]
```

Consequently the penalty tape cannot be relabelled as AL. D7R14 requires a
separate sparse AL workspace/tape carrying at least radius, kernel first and
second derivatives, constraint and active coefficient. Multiplier ownership
remains per fluid pressure centre.

## Pair topology across a divided trial

D7R13's selected actual reduction is a computational divided difference. A
trial may add or remove a compact-support pair, so evaluating only the current
or only the trial neighborhood is incorrect even though the cubic kernel is
`C2` at the horizon.

The candidate therefore uses the sorted union of current and trial canonical
pairs. A deterministic two-pointer merge emits each pair once, with current-
member and trial-member bits. Missing-side kernel values and derivatives are
the exact zero continuation. Pair order, CSR order and fluid-centre reduction
order remain canonical. This gives `O(P_current + P_trial)` merge work and
retains horizon crossings explicitly.

## Frozen proof sequence

The implementation must prove, in order:

1. sparse AL energy, constraints and gradient are binary64 bit-exact to the
   dense tiny oracle;
2. sparse AL HVP is binary64 bit-exact for every frozen direction;
3. sparse divided actual reduction is binary64 bit-exact for every D7R13
   trial, including explicit add/remove horizon crossings;
4. the complete active and inactive D7R13 transactions reproduce their exact
   roots, routes, work ledgers and rollback;
5. the decoded nominal frame-zero state builds the already selected topology
   root `fb2b8f8...24ba13`, counts `342502/611520/120`, and has zero active
   pressure centres;
6. sparse candidate paths perform zero all-pair candidate calls, keep at most
   two workspaces live and pass forced lifecycle/rollback failures.

The nominal check constructs and audits only the initial workspace. It cannot
run an inner solve, outer solve, macro frame or trajectory.

## Frozen classifications

1. `AL_SPARSE_EVALUATION_MISMATCH`.
2. `AL_SPARSE_HVP_MISMATCH`.
3. `AL_SPARSE_DIVIDED_MISMATCH`.
4. `SPARSE_AL_WORKSPACE_CANDIDATE`.

Precedence is evaluation, HVP, divided, candidate. A mismatch route is a valid
research classification when identity, parents, controls and rollback pass;
it grants no implementation credit beyond locating the first boundary.

## What this stage does not claim

- no wall-clock, CPU utilization or real-time claim;
- no nominal nonlinear solve or trajectory;
- no GPU/CUDA/runtime path;
- no public state, schema, gameplay or physics mutation;
- no production readiness.

A passing sparse reclosure authorizes D7R15 to design a bounded nominal
single-frame shadow solve over the selected topology/cache/owner-gather path.
Only that later stage may measure solver work at nominal scale, subject to the
shared-host performance stop.

## Closure

D7R14 passes; see the
[dated evidence](nonlocal-nsr3b4e2d7r14-sparse-al-workspace-evidence-2026-08-22.md).
All tiny evaluation/HVP/divided/transaction outputs are bit-exact, including
one explicit support crossing. The nominal frame-zero workspace reproduces
`342502` pairs with zero active pressure centres. Proceed to D7R15 contract
research, not directly to a trajectory or production claim.
