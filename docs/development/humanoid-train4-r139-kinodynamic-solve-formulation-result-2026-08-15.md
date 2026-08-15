# TRAIN-4 R139 kinodynamic solve formulation result — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Sole clean report-only fixed-mode kinodynamic solve formulation |
| Status | `R139_COMPLETE_R140_CONFORMANCE_ONLY` |
| Gate decision | `PERMIT_SEPARATE_REPORT_ONLY_R140_KINODYNAMIC_SOLVE_IMPLEMENTATION_CONFORMANCE_ONLY` |
| Claim ceiling | Algorithm and exact-acceptance contract only; no real reconstruction, QP setup/solve, kinodynamic solve, candidate, PhysX or training |

## Immutable result

The sole R139 process ran from clean commit `0a71362`, passed all six frozen
validations and completed without importing a numerical/solver stack. Canonical/
file/profile SHA-256 is
`fc76b3072ca9323a2b5dfea45bb9f5c74de57e485a80ed01a7a8ecd6651fec0c` /
`efd6968fac2a66144b43c8422d798b025a878d0e5588e85d8a3414b745b465de` /
`55326991ba7fc025238a2a6ef3f288984c1dea14533cf8b50f9e6ad897f57853`.
The canonical hash independently reproduces exactly.

R139 reads and hash-closes only the R120, R133 and R138 JSON reports plus their
tracked profile/module/tool identities. Source array payload reads are zero;
the R120 cache is validated from report metadata but is not opened. R136 cache,
acceleration, force and pointwise witnesses remain forbidden inputs. Every real
reconstruction, controller graph, dynamics/Jacobian/residual, system assembly,
QP, factorization, exact trial, optimizer, cache/candidate, PhysX and training
counter is zero.

## Frozen reconstruction and endpoints

A future authorized execution must reproduce ten hash categories before its
first real graph call: R120 report/cache/accepted arrays, R133 report/projected
velocity/applied target/applied effort, R131 mode sequence and R138 graph index/
transition replay. The initial configuration is immutable R120 accepted row
zero and the initial velocity is immutable R133 projected row zero.

Interior R120/R133 state and commands are initialization/tracking references,
not hard equalities. The terminal state is neither byte-fixed nor periodic, but
must satisfy the fixed terminal mode, anchors and every hard graph/descriptor
constraint; q/v terminal tracking receives `16x` weight. Acceleration is
initialized only from reconstructed q/v, while force and impulse start at zero.

## Frozen candidate and acceptance method

The candidate generator is fixed-mode sparse multiple-shooting SCvx/SQP with
OSQP QP subproblems. Each major iteration has three lexicographic passes:

1. minimize maximum normalized elastic violation;
2. hold that optimum within `1e-8` and minimize mean violation;
3. hold both optima within `1e-8` and minimize tracking/regularization.

The normalized trust radius starts at `1`, is bounded by `1/32..2`, and audits
fractions `1..1/64`. Exact funnel decrease must be at least `1e-6` and the
actual/model reduction ratio at least `0.1`; frozen contraction/expansion
thresholds are `0.25/0.75`. Failure at minimum radius is a valid stop, never
permission to tune or restart.

The smooth fixed-branch controller and 32-plane friction cones are candidate-
only. Exact branch equality has zero derivative and is counted. Every trial
discards surrogate effort, rounds commands ties-to-even, replays the complete
R133-order stateful controller and evaluates nonlinear manifold dynamics,
anchors, momentum jumps, individual circular cones and descriptor bounds. The
exact controller and circular cones are the sole acceptance authority.

## Numerical and resource boundary

Evaluation is float64. Contact singular values are null at `<=1e-12`, retained
at `>=1e-10`, and invalid inside the gap; only ranks `0/3/5` are expected and
the known flat-foot force gauge is preserved. Non-finite data, unordered bounds,
extra nullity, cone-classification disagreement and OSQP status/residual
ambiguity stop invalid without restart. QP tolerances remain separate from the
stricter exact residual/controller gates.

The future-only envelope is one clean process/thread, seed zero, at most six
major iterations, `18` QPs, `100000` OSQP iterations per QP, `42` exact trial
audits, four hours and `16 GiB`; restart and manual intervention are forbidden.
R139 grants none of that execution authority.

## Exact stop boundary

The exact transition is `R139_COMPLETE_R140_CONFORMANCE_ONLY`. It authorizes
only a separate report-only R140 implementation/numerical-conformance increment
using synthetic and metadata-only discriminators with zero real work. R140
must prove reconstruction ordering, three-pass QP construction, trust/funnel,
controller exact-versus-surrogate separation, deterministic cone refinement,
numeric invalid cases, hashes, budgets and counters.

Only exact R140 `PASS` plus a later explicit roadmap update may authorize one
bounded R141 execution. R141, contact-semantics changes, candidate publication,
PhysX, corpus admission, visual/exhaustive gate and all learned optimizer or
training work remain unauthorized.
