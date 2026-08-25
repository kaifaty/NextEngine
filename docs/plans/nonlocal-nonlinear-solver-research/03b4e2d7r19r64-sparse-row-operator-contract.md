# NSR3-B4E2D7R19R64 -- sparse row-operator equivalence contract

Date: `2026-08-25`

Status: `FROZEN / IMPLEMENTATION AUTHORIZED / NO SOLVE`.

Parent: `87a4f2ee`, R63 stdout SHA-256
`38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62`,
semantic `9f456232956a0e080063123acf8a4a0b2dac7a6fd8f2615b270f0958e66a8440`
and route `TANGENTIAL_MASTER_EXPANSION_REQUIRED`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r64-sparse-row-operator-equivalence|v1|parent=87a4f2ee:38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62:9f456232956a0e080063123acf8a4a0b2dac7a6fd8f2615b270f0958e66a8440:TANGENTIAL_MASTER_EXPANSION_REQUIRED|source=r63-state-216c12aeeb47686e823a9911f0aa9dd14ddee53db1701874488dc325a292cea5;position-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;topology-cfcc7ebbeac9ce8d54093d3af109367e5b68b4aceadf11ed516dbfddf5c2d73f;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5;degree44,113,53,5992|representation=all6000rows;directed-slot-offsets-equal-workspace-flat-offsets;slot-pair;slot-participant;unscaled-pair-jacobian;aggregated-entry-offsets;unique-sorted-fluid-particle;coefficient-spacing-times-row-gradient;particle-incidence-offsets;incidence-row-and-entry-backreference;stable-order|action=deterministic-r29-fluid-probe;sparse-directed-slot-order;same-spacing-times-dot-expression;value-and-absolute-sum;fresh-directed-reference|transpose=deterministic-r29-row-probe;sparse-aggregated-row-order;fresh-pair-once-vjp;component-bound=gamma(8*particle-incidence-degree+64)*(reference-absolute+sparse-absolute)|captured=all494-r51rows;full-gradient-reconstruct;component-bound=gamma(8*row-degree+64)*(reference-absolute+sparse-absolute+entry-absolute);direct-diagonal;long-double-diagonal;captured-relative<=1e-12|overlap=particle-to-row-incidence;epoch-tags;dense-scratch6000;touched-rows-only;stable-entry-incidence-order;all494-columns-by6000rows;component-bound=gamma(32*(source-row-degree+target-row-degree)+128)*(reference-absolute+sparse-absolute+overlap-absolute)|classification=all-action-transpose-gradient-diagonal-gram-bits-exact->sparse-row-operator-exact-equivalence-candidate;else-all-predeclared-bounds-pass->sparse-row-operator-bounded-equivalence-candidate;else-arithmetic-alignment-required|controls=parent;source;capture;workspace;topology;slot-structure;entry-structure;incidence-structure;action;transpose;captured-gradient;diagonal;overlap;bounds;mutation-offset;mutation-entry-particle;mutation-incidence-backreference;mutation-epoch-reuse;work;rollback;route-precedence|routes=sparse-row-parent-rejected;sparse-row-source-rejected;sparse-row-capture-rejected;sparse-row-workspace-rejected;sparse-row-topology-rejected;sparse-row-slot-rejected;sparse-row-entry-rejected;sparse-row-incidence-rejected;sparse-row-action-rejected;sparse-row-transpose-rejected;sparse-row-gradient-rejected;sparse-row-diagonal-rejected;sparse-row-overlap-rejected;sparse-row-bound-rejected;sparse-row-controls-rejected;sparse-row-work-rejected;sparse-row-operator-exact-equivalence-candidate;sparse-row-operator-bounded-equivalence-candidate;sparse-row-operator-arithmetic-alignment-required|precedence=parent,source,capture,workspace,topology,slot,entry,incidence,action,transpose,gradient,diagonal,overlap,bound,controls,work,exact,bounded,alignment|work=parent-r63-replays1;new-static-index1;new-r43-workspace1;workspace-release1;sparse-row-builds6000;slot-scans1;entry-builds6000;incidence-builds1;fresh-directed-jvp1;fresh-pair-vjp1;captured-gradient-checks494;captured-diagonal-checks494;incidence-overlap-columns494;column-values2964000;dense-gram-storage0;new-row-vjp0;new-gram-jvp0;new-hildreth0;new-projection0;new-nonlinear-trial0;new-hvp0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58-r59-r60-r61-r62-r63=unchanged;r63-public-output=byte-exact;operator-private-only;dynamic-active-set=none;runtime-state=none;normal-apply=none;tangential-apply=none;filter=none;switching=none;trust-update=none;following-outer=none;tolerance-fit=none;binary128=none;timing=none;runtime=none;production=none|credit=one-private-sparse-row-operator-equivalence-classification-only
```

SHA-256: `a3cd93bb1dc785e7784ff048bc8989b19215e5bc010ffe7a859b3a098f3ffca2`.

## Hard gates

1. Exact R63 public bytes/semantic/route through a passive capture; exact R43
   position/topology, R63 state and R51 master/cache roots.
2. Build all 6000 rows from the same validated flat topology. Slot offsets must
   equal workspace offsets; all slot pair/participant identities are exact.
   Aggregated entries are finite, particle-sorted and unique. Incidence covers
   every entry exactly once and every back-reference is exact.
3. Evaluate the deterministic R29 fluid probe through sparse directed slots
   and one fresh directed JVP. Compare values and absolute sums rowwise.
4. Evaluate the deterministic R29 row probe through aggregated entries and one
   fresh pair-once VJP. Require every component difference inside
   `gamma(8*particle_incidence_degree+64)` times combined absolute work.
5. For all 494 captured R51 rows reconstruct full gradients, compare every
   component under `gamma(8*row_degree+64)`, compare direct/long-double and
   captured diagonals under inherited relative `1e-12`.
6. For every captured row compute all 6000 overlaps using particle incidence,
   epoch tags and touched rows. Compare with the captured Gram column under
   `gamma(32*(source_degree+target_degree)+128)` times combined absolute work.
   Store no dense Gram.
7. Structural controls must reject corrupted offset, entry particle,
   incidence back-reference and stale epoch reuse. Exact/bounded/alignment
   classification is predeclared; no observed tolerance may change it.
8. Exact work/lifecycle/rollback and route precedence. No solver, projection,
   nonlinear trial, state mutation or timing.

Require two clean Release builds and byte-exact outputs. PASS is one private
operator-equivalence classification only; dynamic active-set execution remains
forbidden until exact or bounded equivalence closes.

Rationale:
[R64 research](../../development/nonlocal-nsr3b4e2d7r19r64-sparse-row-operator-research-2026-08-25.md).
