# NSR3-B4E2D7R19R3 -- sixth trust-solve recurrence replay contract

Status: `EXECUTED / PASS / SIXTH_TRUST_FORCING_CONVERGED / REPLAY ONLY / NO STATE`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r3-sixth-trust-recurrence-replay|v1|parent=c8aeac4894d3177610127a86bec2234acd8e3f87:3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0:f806858bba9d3cb5ae781a270ccccbda96c53ce995cca1cc5799e6af36599c95|legacy=d7r19r1-stdoutf77eb3da05b6ddb2da815a228aef1709917a4761af1eea8d1c71a3a2f2cf0aa1;d7r19-stdoutf5811bfc7d5e986d72b9130f8e6cb90c5ae21bd347ce7f0b476c171fe8cff7bb|target=frame0-0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7;transaction-027dc6d474c87172a7848f71c33f30e0527e3915248b7b53ac077e0d946f2145;binary64-trace-6e7a30213e967e910ab3245321850d5633f521327de20ce6fd8f699685179343;outer0;accepted5;failed-trust-index5;current=fifth-trial-position;radius=fifth-radius-after;predicted=same;u0;theta0x3fc5cccccccccccd|solver=exact-r4r2-steihaug;dimensionless-forcing;binary64-owned-membership;live-prefix32;offline-cap128;no-preconditioner|trace=iteration;point-root;residual-root;direction-root;hvp-root;residual-ratio;forcing-eta;curvature;alpha;beta;direction-norm;iterate-norm;candidate-norm;boundary;adjacent-residual-orthogonality;adjacent-a-conjugacy;lanczos-ritz|controls=r19r2-parent-bytes;capture-state-root;live-prefix-offline-replay-exact;static-binding;finite;work;rollback|routes=sixth-trust-nonfinite;sixth-trust-negative-curvature;sixth-trust-boundary;sixth-trust-forcing-converged;sixth-trust-offline-cap-exhausted|precedence=nonfinite,negative,boundary,converged,cap|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;diagnostic-hvp<=128;workspace1;precision-audits0;trial-formation0;acceptance0|candidate-nominal-substeps=0;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;production-cap-change=none|credit=one-replay-only-sixth-trust-recurrence-diagnostic
```

Identity SHA-256:
`b53be768108e166046a7c6cf63d82d4ec18e5fc794392ef465c9b4efa8baacd6`.

## Required command

Add `--nonlocal-al-sixth-trust-recurrence-replay`. It must:

1. reproduce D7R19R2 stdout SHA
   `3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0`
   and semantic result
   `f806858bba9d3cb5ae781a270ccccbda96c53ce995cca1cc5799e6af36599c95`;
2. expose an optional passive recurrence sink through the exact D7R19R2
   candidate path while keeping the public D7R19R2 report byte-exact;
3. identify only outer `0`, failed trust index `5` after exactly five accepted
   trials and derive current position/radius from accepted trial `4`;
4. bind the frame-zero, transaction and complete binary64 trace roots frozen
   in the identity projection, then publish the derived current-state root and
   trust-radius bits;
5. require the live target trace to contain exactly `32` HVP iterations and
   preserve terminal `STRUCTURAL_BUDGET_HVP_PER_STEP`;
6. build one separate static sparse workspace at that state and run the exact
   unpreconditioned R4R2 Steihaug recurrence for at most `128` HVPs with the
   same gradient, radius and frozen dimensionless forcing value;
7. require every offline iteration `0..31` to match the corresponding live
   projection exactly, including point/residual/direction/HVP roots and every
   scalar/predicate;
8. record residual, forcing, curvature, `alpha`, `beta`, norms, trust-boundary,
   adjacent orthogonality/conjugacy and CG-derived Lanczos/Ritz diagnostics;
9. stop before candidate-energy evaluation, trial formation, precision audit,
   trust-radius update or acceptance;
10. retain exact static binding, finite/work accounting, workspace release,
    all-pair-zero and rollback controls;
11. run one fresh process from each of two clean Release builds and emit one
    frozen route under the precedence below.

## Frozen work

The enclosing R2 control retains its historical `117` total HVPs and unchanged
`32`-HVP watchdog. New diagnostic work is limited to:

```text
offline recurrence HVPs       <= 128
offline workspace builds      1
offline workspace releases    1
offline live workspaces        <= 1
precision audits               0
candidate trials formed        0
candidate acceptances          0
all-pair calls                 0
```

Lanczos/Ritz and recurrence metrics must be derived from already computed
vectors/scalars and add no HVP.

## Hard failures

Identity/parent bytes, target derivation/root, live prefix length, first-32
replay equality, forcing value, static binding, any non-terminal nonfinite
trace field, work/lifecycle, all-pair count, rollback, build/process repeat or
route precedence mismatch is hard FAIL.

The terminal nonfinite route is admissible only when all preceding records are
finite and the exact first offending iteration is bound in the result.

## Routes and precedence

1. `SIXTH_TRUST_NONFINITE`;
2. `SIXTH_TRUST_NEGATIVE_CURVATURE`;
3. `SIXTH_TRUST_BOUNDARY`;
4. `SIXTH_TRUST_FORCING_CONVERGED`;
5. `SIXTH_TRUST_OFFLINE_CAP_EXHAUSTED`.

The route describes the offline continuation only. A route is admitted only
after all hard controls, especially exact first-32 equivalence, pass.

## Authority boundary

This contract grants one replay-only diagnostic of the failed sixth private
trust solve. It does not authorize a production cap change, preconditioner,
trial formation/acceptance, another candidate nominal substep, macro,
trajectory, timing, public state mutation, performance claim or
runtime/production authority.

## Result

The replay passes reproducibly and selects
`SIXTH_TRUST_FORCING_CONVERGED`; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r19r3-sixth-trust-recurrence-evidence-2026-08-23.md).
The live/offline first-32 projections are exact, and HVP 33 crosses the frozen
forcing threshold with positive curvature, negligible recurrence error and a
moderate `36.13` Ritz condition estimate. Research/freeze a separate
recurrence-versus-model-HVP cap-policy discriminator next; no cap change or
trial authority is inherited from this replay.
