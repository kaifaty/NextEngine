# NSR3-B4EP10SIRDIR -- directed scratch residual timing contract

Status: `FROZEN / IMPLEMENTATION_PENDING`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdir-directed-scratch-residual-timing|v1|parent=35a1d41b78d132429334a34d8c99e6d2870b2b8a68ee949beb5a3c69375dff10:b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9:1e4bedbb2c3ed7512ee0a31e879c9d7ba7887dca1fb5e833c89f0c45b108a35d|implementation=f33bf3aa68283a4391d91c74329a8ae9349927d2|command=nominal-hydro-directed-scratch-phase-timing-8|candidate=split-incoming+directed-high-water-reuse|instrumentation=steady-clock;hierarchical-stage+23-subphase+worker-active;durations-excluded|categories=topology-total;source-local=evaluation-setup+pair+metadata+density+center+directed+hvp-setup+compression+directed;target-fold=evaluation-target+hvp-target;control=remainder|calls=transaction1;topology226;evaluation226;hvp459;reuse685;regions4089;partitions261696|runs=3;fresh-processes;serialized;affinity=0-7|gates=sirdi-result-b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9;candidate-work-exact;reuse-exact;stage-component-capacity-identities;top-level-share-range<=0.05;category-share-range<=0.05|route=orchestration-median>=0.15:persistent-region;imbalance-median>=0.15:partition-balance;else-largest-category-share>=0.20&&lead>=1.20:single-discriminator;else:no-optimization|reference=closed|credit=next-mechanical-research-only
```

Identity SHA-256:
`e620072432ce93e98a3f58005b0bc428f8a12b975d5af34143ddde55c415b374`.

## Implementation boundary

Add only:

```text
--nominal-hydro-directed-scratch-phase-timing-8
```

It executes the unchanged B4EP10SIRDI candidate with the existing B4EP10R1
steady-clock and worker-active instrumentation enabled. All old commands stay
uninstrumented. Timing values cannot enter a physical branch, root, failure
classification or semantic result.

## Exactness and timing identities

Each of three fresh serialized processes must reproduce:

- B4EP10SIRDI semantic result
  `b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9`;
- B4EP10SII result `f7b1542f...25fb2`, correspondence
  `1e4bedbb...08a35d` and all five frozen physics roots;
- one transaction, 226 topology, 226 evaluation and 459 HVP calls;
- 4,089 executor regions and 261,696 logical partitions;
- 685 reuse calls, 454,936,226 requested slots, 374,945,086 active writes,
  670,229 growth slots, 16,085,496 peak bytes, one release, zero live buffers
  and zero reuse failures;
- every topology/evaluation subphase called 226 times and every HVP subphase
  called 459 times;
- component sums no greater than their enclosing stage, stage sums no greater
  than transaction total, and active <= maximum-active <= wall capacity.

For each process reconstruct the four categories exactly as frozen in the
identity projection. Checked arithmetic must prove that they are positive
where work exists and sum exactly to the transaction duration.

## Stability and routing

Across three processes require the range of every top-level stage share and
every four-category share to be at most `0.05` absolute.

Route exactly one next research target:

1. median executor orchestration share `>=0.15`: persistent-region research;
2. otherwise median imbalance share `>=0.15`: partition-balance research;
3. otherwise, if the largest four-category median share is `>=0.20` and at
   least `1.20x` the second, freeze one discriminator for that category;
4. otherwise select no optimization and design a narrower measurement.

No duration creates speed credit; B4EP10SIRDI retains only its external A/B
result.

## Exit

PASS authorizes only the routed mechanical research/design stage. Any SIRDI
semantic, scratch-lifetime or timing-accounting mismatch rejects the
instrumentation. B4E2, broad corpus, runtime/GPU/schema and production remain
blocked.
