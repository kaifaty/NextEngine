# NSR3-B4EP6 coefficient residual-attribution research -- 2026-08-22

Status: `COMPLETE / PASS / WORKSPACE_EVALUATION_BASE_TAPE_SELECTED`

## Question

After B4EP5 reduces the already cached nominal Hydro transaction from median
10.61 s to 8.50 s, does exact HVP application still dominate, or has the
serial residual become balanced enough that statistical gprof attribution is
insufficient to select another change?

## Competing hypotheses

The physical schedule is unchanged: 226 state queries, 459 HVPs, 42 attempted
substeps and 221 outer trials. B4EP5 removes repeated kernel-function work but
adds two state-local coefficient arrays. Four hypotheses remain:

1. **HVP arithmetic/traversal still dominates.** It continues to traverse
   every active directed record twice for every HVP.
2. **Complete workspace construction now dominates.** Every state still
   filters cached topology, materializes CSR, evaluates the objective and
   builds both pressure and coefficient tapes.
3. **Coefficient construction is itself material.** It evaluates two scalar
   kernels and writes two doubles for every admitted pair, including pairs
   that may serve few HVPs.
4. **The residual is balanced.** HVP, workspace and solver control may be too
   close for sampled function attribution to justify another implementation.

The B4EP4 profile is obsolete for ranking these hypotheses: it contains the
971,831,424 kernel calls that B4EP5 deliberately removed from HVP execution.

## Measurement design

Use one external GCC 15.2 `-O3 -DNDEBUG -g -pg -ffp-contract=off
-fno-fast-math` build of implementation
`7263d8929491b66ea94eb74713fab4c136efe5be`. Run exactly once:

```text
nonlocal-formula-reclosure --nominal-hydro-hvp-coefficient-ablation
```

Admit samples only if stdout remains byte-identical to B4EP5. Attribute
top-level inclusive CPU to HVP, complete workspace and residual solver/control
work. Within workspace, separate topology/filter/CSR, evaluation/base tape and
coefficient-tape population. Keep instrumented wall/RSS descriptive only.

## Frozen routing

- Select a top-level next design only if the leader is at least `1.20x` the
  runner-up.
- If workspace wins, apply the same rule among its three subcategories.
- If no leader clears the rule, authorize scoped internal phase timing and no
  optimization.
- Never combine a workspace and HVP change from the same profile.
- Solver-policy, parallel, GPU, B4E2, runtime and production work stay out of
  scope.

## Decision

Freeze one B4EP6 exact-output profile. It may route to one B4EP7 design or to
internal phase timing; it cannot itself authorize an implementation.

The exact profile selects evaluation/base tape by the frozen two-level rule;
see the [dated evidence](nonlocal-nsr3b4ep6-coefficient-residual-attribution-evidence-2026-08-22.md).
