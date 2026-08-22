# NSR3-B4EP10SIR -- split incoming residual timing contract

Status: `CLOSED / PASS / SOURCE_LOCAL_DISCRIMINATOR_RESEARCH_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sir-split-incoming-residual-timing|v1|parent=9a496e6129ce4669af31aa056446f3743404bfe38ebf458edff1f6cb7b816774:5c8ca71e2c6d7ebe77e3868b9077254694a6ac640300755d9ba06e293ff168cd:a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b|implementation=e7dec71bee66a72a497ef02315688995b675e345|command=nominal-hydro-split-incoming-phase-timing-8|instrumentation=steady-clock;hierarchical-stage+23-subphase+worker-active;durations-excluded|categories=topology-total;source-local=evaluation-setup+pair+metadata+density+center+directed+hvp-setup+compression+directed;target-fold=evaluation-target+hvp-target;control=remainder|calls=transaction1;topology226;evaluation226;hvp459;regions4089;partitions261696|runs=3;fresh-processes;serialized;affinity=0-7|gates=semantic-result-f7b1542f30fef20a08da57c83bb200878cdf426d87b627445e422c9a72825fb2;candidate-work-exact;stage-component-capacity-identities;top-level-share-range<=0.05;category-share-range<=0.05|route=orchestration-median>=0.15:persistent-region;imbalance-median>=0.15:partition-balance;else-largest-category-share>=0.20&&lead>=1.20:single-discriminator;else:no-optimization|reference=closed|credit=next-mechanical-research-only
```

Identity SHA-256:
`4bd4879fc96f602a3988a9e35fcc184e091509a600bf09dabfe32310e2cec731`.

## Implementation boundary

Add only:

```text
--nominal-hydro-split-incoming-phase-timing-8
```

It executes the unchanged B4EP10SII candidate with the existing B4EP10R1
steady-clock and worker-active instrumentation enabled. All old commands stay
uninstrumented. Timing values cannot enter a physics branch, root, failure
classification or semantic result.

## Exactness and timing identities

Each of three fresh serialized processes must reproduce B4EP10SII semantic
result
`f7b1542f30fef20a08da57c83bb200878cdf426d87b627445e422c9a72825fb2`,
all frozen roots and candidate work, and exactly:

- one transaction, 226 topology, 226 evaluation and 459 HVP calls;
- 4,089 executor regions and 261,696 logical partitions;
- every topology/evaluation subphase called 226 times and every HVP subphase
  called 459 times;
- component sums no greater than their enclosing stage, stage sums no greater
  than transaction total, and active <= maximum-active <= wall capacity.

For each process reconstruct disjoint durations:

```text
topology = topology_total
source_local = evaluation setup + pair + metadata + density + center
             + directed + HVP setup + compression + directed
target_fold = evaluation target + HVP target
control = transaction_total - topology - source_local - target_fold
```

All terms must be positive where the frozen command performs work; subtraction
must be checked and the four categories must sum exactly to transaction total.

## Stability and routing

Across three processes require the range of each top-level stage share and
each four-category share to be at most `0.05` absolute.

Route exactly one next research target:

1. median executor orchestration share `>=0.15`: persistent-region research;
2. otherwise median imbalance share `>=0.15`: partition-balance research;
3. otherwise, if the largest four-category median share is `>=0.20` and at
   least `1.20x` the second, freeze one discriminator for that category;
4. otherwise select no optimization and design a narrower measurement.

No duration grants B4EP10SII speed credit; that credit remains solely in its
external A/B evidence.

## Exit

PASS authorizes only the routed mechanical research/design stage. Any semantic
or accounting mismatch rejects the instrumentation. B4E2, broad corpus,
runtime/GPU/schema and production remain blocked.
