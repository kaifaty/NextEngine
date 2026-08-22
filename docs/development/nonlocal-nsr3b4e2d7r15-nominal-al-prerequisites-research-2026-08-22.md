# NSR3-B4E2D7R15 nominal AL prerequisites research

Date: `2026-08-22`

Status: `RESULT_CLOSED / PASS / SHARED_HOST_PERFORMANCE_STOP`

## Question

What must be reclosed before the exact D7R14 sparse augmented-Lagrangian
solver can execute even one meaningful nominal Dam physical substep?

D7R14 removes the dense pair traversal, but a direct nominal solve would still
combine three unverified assumptions. Running it now would conflate temporal
scaling, precision certification and static-support rebuild cost with the
nonlinear solver result.

## Finding 1: the solver time step is not yet explicit

All D7R13/D7R14 inertia, inner gradient/HVP, stationarity normalization,
direct reduction and divided reduction use the global `TIME_STEP = 1/240`.
The already verified Dam step one, however, accepts `78` physical substeps per
macro frame. Its aligned substep is:

```text
macro dt bits       0x3f71111111111111  (1/240)
aligned substep dt  0x3f0c01c01c01c01c  (1/(240*78))
```

Using macro `dt` inside a nominal substep would underweight inertia by
`78^2 = 6084` relative to the aligned substep. That is a different objective,
not a performance approximation. D7R15 must thread one explicit positive
binary64 `dt` through every time-dependent candidate formula while preserving
the complete D7R14 bytes at the legacy value.

The `78` count is an alignment witness inherited from the existing accepted
Dam step-one schedule. It does not yet select AL's production temporal policy.

## Finding 2: the sign oracle would reintroduce dense work

D7R14's sparse transaction deliberately retains D7R13's independent binary128
audit for candidate-effect acceptances. That audit still enumerates every
fluid-fluid and fluid-support candidate. On nominal Dam it would recreate the
`116,301,000`-candidate path in binary128.

D7R15 therefore needs a sparse offline binary128 audit. It traverses a
canonical current/trial union drawn from the existing `0.04h` conservative
superset, recomputes compact-support membership in binary128 and accumulates
the same fixed/compensated energy certificate. Runtime positions,
multipliers and accepted state remain binary64; binary128 cannot become
continuation state.

The new oracle must reproduce all four D7R13 candidate-effect audit roots:

```text
3f6b74612d212d48ee40ba1bc2fffd27a23a20cc1d2c390cf2e1b59194a4a8fc
a11e56103bc1ea0b3a88b4b82354a0e66240db3e9057f98a1a0f987951cfe0aa
e06edef57dda0cf7cabc75de35365ea57432f19d4706879fc7086afb40865e51
b0db278089eb109d19879de3433fb2d69c6160f755fee572f75fdda86c584312
```

## Finding 3: fixed support must not be rebuilt per trial

The D7R14 proof workspace uses the correct canonical neighborhood, but its
standalone builder canonicalizes and sorts all `16,384` static boundary
samples for each workspace. The established nominal path already owns
`JointStaticSupportIndex`, an identity-bound binding and flat CSR construction.

D7R15 must split AL tape construction from topology construction and add a
static-support-bound builder. One static index serves the complete command;
each current/trial workspace contains only dynamic fluid topology and AL
coefficients. A binding identity mutation must fail before evaluation.

Topology skin-cache reuse and owner-parallel AL kernels remain later
optimizations. This stage only removes the obviously invariant support rebuild
without making a timing claim.

## Frozen proof sequence

1. reproduce the complete D7R14 stdout and both tiny transaction roots;
2. make `dt` explicit through every candidate-only time-dependent formula and
   reproduce D7R14 bit-for-bit at `1/240`;
3. compare dense/sparse explicit-dt evaluation, gradient, HVP and divided
   reduction at `1/(240*78)` on frozen tiny controls;
4. reproduce all four dense binary128 audit roots through the sparse superset
   oracle and prove exact pair membership;
5. reproduce tiny and nominal topology/evaluation through one identity-bound
   static support index, with a forced binding mutation rejection;
6. keep all-pair candidate calls zero, at most two live workspaces and exact
   build/release accounting;
7. build the nominal decoded frame-zero workspace only. Do not predict, solve,
   integrate or publish a nominal state.

## Frozen classifications

1. `EXPLICIT_DT_MISMATCH`.
2. `SPARSE_BINARY128_MISMATCH`.
3. `STATIC_SUPPORT_BINDING_MISMATCH`.
4. `NOMINAL_AL_PREREQUISITES_CONFIRMED`.

Precedence is time step, binary128 oracle, support binding, confirmed. A
mismatch is a valid bounded research result if identity, parent bytes,
lifecycle and rollback controls pass.

## Rejected next actions

- A full `1/240` nominal AL step: wrong objective for the selected substep
  schedule.
- A complete 78-substep macro: temporal policy and per-substep correctness are
  not yet established, and the shared-host long-run stop remains active.
- Dense binary128 auditing at nominal scale: defeats D7R14 structurally.
- Disabling the oracle to gain speed: removes the acceptance certificate.
- Parallel/GPU work now: would optimize an incompletely parameterized path.

A PASS authorizes D7R16 to freeze one aligned nominal Dam substep shadow with
predeclared nonlinear work, residual, conservation, rollback and watchdog
gates. It still does not authorize a macro trajectory or production path.

## Result

D7R15 passes as `NOMINAL_AL_PREREQUISITES_CONFIRMED`; see the
[dated evidence](nonlocal-nsr3b4e2d7r15-nominal-al-prerequisites-evidence-2026-08-22.md).
The explicit aligned `dt`, four sparse binary128 audit roots and identity-bound
static support are exact. Research/freeze D7R16 before executing one nominal
substep.
