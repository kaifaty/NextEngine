# NSR3-B4C3L -- compensated-ledger normalization discriminator

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / B4C3A2_BLOCKED`

Parent B4C3TAR is the preserved FAIL with JSON-without-final-LF SHA-256
`b5ea40b96a812fdf090e7982d036ac3a893f13e6d8720e5381d131d4a02d0d50`
and semantic SHA-256
`5a87dde2b546df086124abe2d1597bda5e8d0a651088f233cea91dbb53acc4b1`.
B4C3A1 remains exact at JSON-without-final-LF
`107e56e5be49ebb157b0662767c934ced6addf6f3a7054d696772d6699ee1e90`.

## Identity

```text
identity  canonical-compensated-ledger-normalization-r0
policy    b1136c2c3dc7970cbee0ce67b0d129b849ed8957268bb7281063af9e60200e2f
formula   nextengine.nonlocal.canonical-ledger-policy|v2|
          kkt-sum-scale-gated|max-scale-diagnostic|threshold=1e-9
```

The canonical representation profile and P1/P2 scenario hashes do not change.
This policy hash identifies evidence/admission semantics only and is not a
runtime or serialization profile.

## Per-entry definitions

For the physical solver transition before canonical publication define:

```text
DeltaP = momentum(v_solve) - momentum(v_input)
E      = |I_gravity| + |R_support| + |R_contact|
L_kkt  = DeltaP - I_gravity + R_support + R_contact
S_kkt  = max(|DeltaP| + E, 1e-30)
S_max  = max(|DeltaP|, E, 1e-30)
```

The balanced publisher records direct numerical impulse `I_q`. Require:

```text
L_raw  = momentum(v_published) - momentum(v_input)
         - I_gravity + R_support + R_contact
L_comp = L_raw - I_q
closure = |L_comp - L_kkt|
r_kkt  = |L_comp| / S_kkt
r_max  = |L_comp| / S_max
```

Retain the existing forward-error closure allowance. Gate `r_kkt<=1e-9` and
require its difference from the source KKT residual to remain within the
derived division/norm forward bound. Report `r_max` without gating it. Continue
to report and separately label the raw-published residual; neither raw nor
strict-max residual may be called the physical KKT residual.

Require finite positive scales, `S_max<=S_kkt<=2*S_max` outside the common
`1e-30` floor, and `1<=r_max/r_kkt<=2` whenever both residuals are positive and
the floor is inactive. Exact zero remains exact zero.

## Synthetic controls

Use exact fixed vector/scalar controls, independent of the observed P1 value:

1. equal `d=e` with nonzero ledger gives normalization ratio exactly two;
2. choose `|L|=1.5e-9*S_max` and `S_kkt=2*S_max`, so KKT-scale admission passes
   at `0.75e-9` while strict-max diagnostic exceeds `1e-9`;
3. one dominant side approaches ratio one;
4. exact zero with the common scale floor remains finite and passes;
5. corrupt `L_comp` beyond the closure allowance, set `r_kkt>1e-9`, inject
   nonfinite input and an invalid scale; all four negatives reject atomically.

No tolerance is derived from B4C3TAR.

## Real-state controls

1. Recompute both residuals for every B4C3A1 one-frame P1/P2 ledger entry.
   Canonical frames, roots, direct impulse, energy decomposition, contacts and
   all non-ledger gates remain exact. Both old and candidate gates pass.
2. Recreate the exact B4C3TAR P1 frame-seven committed state. Execute
   `16/32/64/128` from that same state and global offset. Require the frozen
   strict legacy status pattern `PASS/FAIL/PASS/FAIL` and strict residuals at
   the preserved diagnostic values.
3. Under candidate admission all four ledger sequences pass, compensated
   closure remains at the preserved zero/near-zero values and every
   `r_kkt<=1e-9`. The unchanged embedded 16/32 gate must then be evaluated and
   reported, without committing either level.
4. Repeat the complete discriminator byte-identically and preserve B4C3TAR,
   B4C3A1, B4C3Q, B4C3A and B4C2T historical report hashes.

## Decision boundary

PASS selects `CANONICAL_KKT_SCALE_LEDGER_CANDIDATE` and authorizes only B4C3A2
one-frame selected-policy stage/ledger revalidation under a new evidence
identity. FAIL preserves B4C3A1 and B4C3TAR FAIL. It does not authorize raising
the threshold, complete adaptive replay, fixed reference, nominal, CUDA,
runtime, schema or production work.
