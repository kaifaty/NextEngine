# NSR3-B4E2D7R19R20 shadow outer-7 execution research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

## Question

Can the exact R19 one-use grant execute one more nonlinear outer update under
the remaining epoch-1 slice budget without changing the physics result or work
semantics relative to an independent unsliced cumulative-budget oracle?

## Bound source

R20 starts from the exact non-admissible R18 successor and R19 ownership
objects:

```text
R18 state             5ad2f99d356c9f5f7553d84189e1e279dcd360bdb9a588370a874b2c5fe707bc
R18 receipt           5a029f24a82b60117492f1929a2d63489d2321b63286a9260282101b74ef8a64
R19 state             37d1556efa98ceb68db566a6f553d82295fc6b233fcbfb0aa6d393a96a4ef6b5
outer-7 grant         cb1b13d53754e7b1408d66eeb0a00f7dc02a697ccd338d8d77834a6f8b1a6776
grant receipt         28b5b27673dc9ed247957c159f7c4ac071a074902c918a273327a0d0a40b513a
active owner          f88ea265a2e2544b76ae335770bb548da6aef76c7a3883ff7ea87e62e6f92ac4
position              494dc8d29030f06b79b7047a82ce83eb994e863541becc37f60d3343f08ff29f
dual                  9ec049a088270e211435a5cc16b4fea23826c261145209cb28748b8dec92c778
history               a36fa9c02821f1eac2785aa0ec35632f088810b3be625303271d2b9750b8a5b1
epoch / next outer    1 / 7
slice / cumulative    50 / 573 HVP
```

The active owner with flags changed from `1` to `3` has pre-derived root
`94df43022a9b8fd28e00f926db066e065bd8fa17ff65598b01dfe48e11e47f8a`.

## Experimental design

Execute the same outer-update function twice from independently cloned
position, dual, predicted position, normalized `theta`, static support index,
formula, topology, forcing and trust-completion policies:

| Lane | Starting total | Hard maximum | New HVP available |
|---|---:|---:|---:|
| candidate | epoch slice `50` | `512` | `462` |
| oracle | cumulative `573` | `8704` | `8131` |

Both lanes execute outer index `7` once. The candidate may not reset the epoch
or reinterpret its nonzero starting total. The oracle is comparison-only and
never supplies state when the candidate fails.

The exact baseline used ledger is:

```text
7,2,25,1,50,573,37,23,552,21,2,23,0
```

The per-update bounds are one outer, at most 16 trials, at most 462 new
candidate HVPs, 251 new workspaces and 41 new precision audits. The unchanged
per-trust-step cap is 34 HVPs.

## Required equivalence

The two lanes must match bit-for-bit in:

- complete outer update and trial trace;
- resulting position, normalized dual, support energy/density/constraint,
  gradient and topology counts;
- update and work roots;
- HVP/workspace/precision deltas and completion-policy accounting;
- failure/admissibility state and all resource counters after subtracting the
  different starting totals.

This is stronger than comparing only the final residual. A budget offset must
not be observable to the mathematical update before a real bound is hit.

## Transaction and outputs

After exact equivalence only, consume the R19 active owner and emit a private
outer-execution receipt plus committed shadow state:

```text
receipt  NEALOER1  schema 1  body 404  total 420 bytes
state    NEALOES1  schema 1  body 196  total 212 bytes
```

The receipt binds the R19 state/grant, consumed owner, update, position, dual,
successor history, limits and complete successor used ledger. The state binds
the source state, grant, grant receipt, consumed owner and execution receipt.
All objects are prepared and canonical-roundtripped before one copy-on-write
commit. Every failure preserves the original bytes; duplicate replay is
prework and idempotent.

## Fail-closed interpretation

- exact lanes plus admissible state: research a separate private convergence
  and publication-boundary certificate; do not publish from R20;
- exact lanes plus non-admissible state with slice capacity remaining:
  research another one-use outer grant;
- candidate slice exhaustion while the oracle can continue: stop at a budget
  boundary and research an epoch transition; never adopt the oracle state;
- candidate/oracle mismatch: preserve R19 and diagnose budget leakage before
  any retry;
- common finite solver failure: record the first exact failure without tuning
  coefficients or caps.

## Scope

R20 authorizes one private candidate outer 7 and one private oracle outer 7
only after its contract is frozen. It authorizes no second substep, macro,
trajectory, timing, public/world commit, runtime integration, durable or
concurrent CAS, live budget-code change or production policy.
