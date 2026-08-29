# NSR3-B4E2D7R16 -- sparse precision transaction contract

Status: `PASS / NOMINAL_TRANSACTION_BACKEND_CONFIRMED / SHARED_HOST_PERFORMANCE_STOP`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r16-sparse-precision-transaction|v1|parent=4d717a336e7e888f328581da18216f17699e9c8d:a207d671c41648ebcc4c89d73dc86acd2569b6ca8b150faa6584b850534c2858:dda8399f35556ada9bf38cfb1445c324f6f5b4c0193a59e2d4c9c4ea5f57f5cf|legacy=d7r15-complete-bytes;d7r13-active4d6b8c58c2db672bea5fcae0a4d5fb077cc8587398540128bcb09b6a75400ef8;inactive6df37c6ae522de96d3a300abefcbf035af55e04d9f6d7bf77b6cedbea5f1ab1e|hidden-dense=accepted-long-double-audit;116301000*4=465204000-candidate-checks-per-acceptance|long-double=sparse-current-trial-union;skin0.04h;canonical-order;naive+compensated;half-horizon+horizon-margin;membership;sign-fields-exact|transaction=joint-static-support-binding-every-workspace;sparse-long-double-every-accepted;sparse-binary128-candidate-effect;explicit-positive-dt;flat-csr|oracle=d7r13-active+repeat+inactive-roots;all-inner-roots;all-long-double-fields;all-binary128-roots;d7r15-complete-bytes|work=candidate-all-pair-calls0;workspace-live<=2;release-exact;static-index-builds1;support-canonicalizations1;timing=none|nominal=read-only-predictor;dt0x3f0c01c01c01c01c;lower-y-clamped400;free5600;solve-none;contact-impulse-ledger-required-next|routes=sparse-long-double-mismatch;static-transaction-binding-mismatch;sparse-precision-integration-mismatch;nominal-transaction-backend-confirmed|precedence=long-double,static-binding,precision-integration,confirmed|runs=2-release-builds;2-processes;byte-exact|trajectory=none;nominal-solve=none;public-commit=none;physics-mutation=none;runtime-wide-precision=none|credit=one-nominal-transaction-backend-only
```

Identity SHA-256: `0376082e71d477a20647e1797a318221fe7fdd026fd5a988eb0693aa51a5445f`.

## Required command

Add `--nonlocal-al-sparse-precision-transaction`. It must:

1. reproduce complete D7R15 stdout bytes;
2. implement a sparse long-double current/trial audit over the canonical
   `0.04h` union, including exact half-horizon/horizon margins, membership,
   naive/compensated totals and sign fields;
3. route every accepted inner audit through sparse long double and every
   candidate-effect audit through sparse binary128;
4. use one identity-bound `JointStaticSupportIndex` for every workspace in a
   complete active, active-repeat and inactive sparse AL transaction;
5. reproduce all D7R13 transaction, inner and precision roots exactly at
   legacy `dt`, while retaining explicit `dt` in the integrated path;
6. prove zero candidate all-pair calls, maximum two live workspaces, exact
   releases, one static-index build and one support canonicalization;
7. reconstruct only the aligned nominal predictor/contact facts: exactly
   `400` lower-y clamps and `5,600` free samples, with finite separate gravity
   and predictor-contact impulses;
8. execute no nominal AL solve, integration, publication, trajectory or
   timing measurement;
9. execute twice in each of two clean Release builds and emit exactly one
   route under the frozen precedence.

## Routes

1. `SPARSE_LONG_DOUBLE_MISMATCH`.
2. `STATIC_TRANSACTION_BINDING_MISMATCH`.
3. `SPARSE_PRECISION_INTEGRATION_MISMATCH`.
4. `NOMINAL_TRANSACTION_BACKEND_CONFIRMED`.

Identity, parent bytes, ordering, membership, precision-root, binding
mutation, lifecycle, all-pair-call, predictor-count, impulse-ledger,
rollback, build/process repeat or route-precedence mismatch is hard FAIL.

A confirmed backend authorizes D7R17 research/freeze and one aligned nominal
Dam substep shadow only. It grants no macro frame, trajectory, timing,
runtime-wide precision, projected-contact redesign, public state, physics
mutation or production authority.
