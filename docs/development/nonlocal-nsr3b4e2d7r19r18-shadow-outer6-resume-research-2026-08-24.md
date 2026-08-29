# NSR3-B4E2D7R19R18 shadow outer-6 resume research

Date: `2026-08-24`

Status: `PASS / RESUME EQUIVALENT / OUTER6 NOT ADMISSIBLE / R19 NEXT`

## Question

Can the R17 epoch-1 grant continue exactly one outer update without the
soft-budget boundary changing any physical or nonlinear-solver result?

R18 is the first stage allowed to execute outer 6. It remains a private
shadow continuation and cannot commit world state.

## Selected experiment

Run the same `al_normalized_private_outer_update` twice from independent
copies of the exact R16/R17 payload:

```text
candidate                                oracle
epoch 1 slice budget                     unsliced cumulative budget
total_hvp = 0 / max 512                  total_hvp = 523 / max 8704
                  \                     /
                   same outer 6 inputs
                   same solver policies
                   same static identity
                            |
                    exact result compare
```

The candidate represents the resumed epoch. The oracle represents the same
work as though no soft slice boundary had occurred. Both use the selected
precancelled divided reduction, dimensionless forcing, binary64-owned
membership and tiered residual-completion policy.

This is stronger than comparing against a new implementation: the only
intentional difference is budget representation. Position, dual, predicted
position, theta, static support, formula, solver and completion identities are
identical.

## Starting resource state

The active grant carries:

```text
limits  16,16,34,512,288,64,8704
used     6,2,25,1,0,523,33,21,504,19,2,21,0
```

Before both executions, beginning outer 6 increments the outer counter from
6 to 7 and resets the current inner-trial counter. Candidate slice HVP begins
at zero; oracle total HVP begins at 523. Maximum bounded deltas are one outer,
16 inner trials, 512 HVP, 255 workspace builds and 43 precision audits. The
candidate's updated cumulative HVP is `523 + candidate_delta` and must remain
at or below 8704.

Actual deltas are deliberately not predeclared. They are valid only if the
two independent executions produce exact outer-update, position, dual,
trial, precision, work and policy-trace roots and identical delta ledgers.
This avoids fitting the contract to an observed output.

## Active-owner transaction

R17 active-owner bytes/root are revalidated before work. The positive path
changes its flags from active/unconsumed `1` to active/consumed `3`; the
precomputed consumed-owner root is
`abbef5a38a947f21945d9a96e50df2597a4ac4647a2d733c911ff55cb74eaa74`.

Candidate/oracle outputs are prepared in local copies. Only after exact
equivalence and resource validation may one copy-on-write commit atomically:

1. consumed active owner;
2. canonical shadow-resume receipt;
3. canonical shadow-resume state root.

The `NEALRSM1` receipt has a `404`-byte body and `420` total bytes. It binds
flags, epoch, outer index, source/grant/input-state/outer-update/position/dual/
successor-history roots, seven limits and thirteen successor used counters.

The `NEALRST1` state has a `196`-byte body and `212` total bytes. It binds the
source envelope, consumed source owner, grant, R17 transition receipt,
consumed active owner and R18 resume receipt roots.

Result-dependent roots are derived only after the independent executions;
the frozen projection and exact oracle comparison define their bytes.

## Failure and idempotence controls

Source/grant/owner/duplicate/stale/resource controls fail before physics work.
Candidate or oracle failure, an injected oracle-root mismatch and an injected
precommit abort preserve the exact R17 input state. Duplicate replay after a
successful shadow commit fails before work and preserves the exact R18 state.

Negative controls after physical execution reuse the already captured local
candidate/oracle results; they do not rerun outer 6.

## Authority boundary and disposition

R18 may execute exactly one candidate outer 6 and one independent oracle
outer 6. It may commit one private shadow receipt only. It may not execute
outer 7, a second substep, macro or trajectory; mutate public/world physics;
change live budget code or production policy; publish a schema; or run timing.

If exact equivalence fails, the result is research evidence against current
resume semantics and the active owner remains unconsumed. If it passes, a
later separately frozen stage may decide whether outer 6 is merely progressing,
admissible, or a new suspension boundary; R18 itself grants no further work.

The executable gate is frozen by the
[D7R19R18 contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r18-shadow-outer6-resume-contract.md).

The candidate and oracle pass exactly as recorded in the
[D7R19R18 evidence](nonlocal-nsr3b4e2d7r19r18-shadow-outer6-resume-evidence-2026-08-24.md).
