# NSR3-B4E2D7R17 -- nominal substep shadow contract

Status: `CLOSED / PASS_CLASSIFICATION / NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED / SOLVER_NOT_CONFIRMED / SHARED_HOST_PERFORMANCE_STOP`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r17-nominal-substep-shadow|v1|parent=1297a1f83a3dee629c9e7987de9d159f7c6795e3:47976f826fa5c7c16209f5e3e8a0829e443ec90d3bd225fb61e73c1d506ff156:4c537f706dee3941808f0c44c1b2db30dd79254bcbbd42c9ac0dc92aee3f8cd5|alignment=frame0-0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7;dt0x3f0c01c01c01c01c;particles6000;lower-y-clamps400;free5600|predictor=v+dt*g;box-clamp;contact-impulse-separate|solver=d7r16-static-bound-sparse-precision;eta1e-10;kappa1226.25;private-only;confirmed+warm-holdout|required-budget=outer-updates16;inner-trials16;hvp-per-step32;hvp-total512;workspace-builds288;accepted-audits64;prework-check;release-exact|ledger=gravity;predictor-contact;kinematic-pressure;fixed-support-reaction;fluid-momentum-identity;pressure-support-residual|gates=finite;mass-exact;dual-feasible;primal<=1e-8;stationarity<=1e-10;complementarity<=1e-9;penetration<=1e-12;impulse-closure<=1e-10-scaled;all-pair-calls0;live-workspaces<=2|routes=nominal-structural-watchdog-exhausted;nominal-solver-not-confirmed;nominal-boundary-penetration;nominal-impulse-ledger-mismatch;nominal-substep-shadow-confirmed|precedence=watchdog,solver,boundary,ledger,confirmed|runs=2-clean-release-builds;1-process-each;byte-exact;timing=none|trajectory=none;nominal-substeps=1;macro=none;public-commit=none;physics-mutation=none;projected-contact=none;runtime-wide-precision=none|credit=one-private-nominal-substep-only
```

Identity SHA-256: `4d11030cab2be075fcb703e1f3d377a3d3e331c71906784656ab0b2e725305f5`.

## Required command

Add `--nonlocal-al-nominal-substep-shadow`. It must:

1. reproduce complete D7R16 stdout bytes;
2. reject invalid budget or static identity before nonlinear work;
3. enforce all frozen work limits inside the outer, inner, CG, workspace and
   precision loops rather than checking only after completion;
4. execute exactly one aligned, box-clamped nominal Dam predictor and private
   AL solve from zero multiplier;
5. require confirmed consecutive admissible outer updates and one admissible
   warm holdout, selecting the confirmed state rather than the holdout state;
6. report exact work counters, final residuals, density range, penetration,
   private root and separate gravity/contact/pressure/support impulses;
7. pass the two scaled impulse closures at `1e-10` and every inherited
   nonlinear admissibility gate;
8. preserve decoded/public state bytes and perform no publication, second
   substep, macro, trajectory or timing measurement;
9. execute once in each of two clean Release builds and emit one route under
   the frozen precedence.

## Routes

1. `NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED`.
2. `NOMINAL_SOLVER_NOT_CONFIRMED`.
3. `NOMINAL_BOUNDARY_PENETRATION`.
4. `NOMINAL_IMPULSE_LEDGER_MISMATCH`.
5. `NOMINAL_SUBSTEP_SHADOW_CONFIRMED`.

Identity, parent bytes, alignment, non-finite state, mass, lifecycle,
all-pair-call, rollback, build/process repeat or route-precedence mismatch is
hard FAIL.

The executed shadow exhausts the outer-update watchdog with one accepted
trial and two HVPs per outer update. Inner stationarity is already far inside
its gate while primal progress remains slow. This authorizes only an
explicit-`kappa` and dimensionless `dt`--`kappa` scaling prerequisite before
another nominal solve; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r17-nominal-substep-shadow-evidence-2026-08-22.md).
It grants no macro, trajectory, public state, projected-contact redesign,
runtime integration, parallel/GPU path or production authority.
