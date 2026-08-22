# NSR3-B4EP10SIRDIRE -- evaluation setup timing contract

Status: `FROZEN / IMPLEMENTATION_PENDING`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdire-evaluation-setup-discriminator|v1|parent=e620072432ce93e98a3f58005b0bc428f8a12b975d5af34143ddde55c415b374:1f66ab3c1bffa2199759c66e6297273e4777902777cedd1634dbad7e0895992e:b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9|implementation=225af12de50282734bfaee1c267031d07e96bd28|command=nominal-hydro-directed-scratch-setup-timing-8|instrumentation=parent-23-phase+evaluation-setup-validation+capacity+buffer;durations-excluded|segments=validation:entry-through-flat-source;capacity:checked-payload;buffer:seven-vector-prepare;residual:setup-minus-segments|calls=transaction1;setup226;validation226;capacity226;buffer226;reuse685;regions4089;partitions261696|runs=3;fresh-processes;serialized;affinity=0-7|gates=sirdir-result-1f66ab3c1bffa2199759c66e6297273e4777902777cedd1634dbad7e0895992e;sirdi-result-b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9;segments-positive;sum<=setup;share-range<=0.05|route=largest-median-setup-share>=0.40&&lead>=1.20:segment-structural-audit;else:no-optimization+narrower-measurement|reference=closed|credit=one-next-mechanical-research-only
```

Identity SHA-256:
`a018e4d47080b76bb56166a5a6a7c5ba892e23687249cb519956951248ca3974`.

## Implementation boundary

Add only:

```text
--nominal-hydro-directed-scratch-setup-timing-8
```

It executes the unchanged B4EP10SIRDIR candidate and enables three additional
nested timers inside `evaluation_setup`. All existing commands retain their
instrumentation state and output. Durations cannot enter any physical branch,
root, failure class or semantic result.

## Exactness and accounting

Each of three fresh processes must reproduce B4EP10SIRDIR duration-free result
`1f66ab3c...5992e`, B4EP10SIRDI result `b4f847cb...777e9`, B4EP10SII result,
correspondence, five physics roots, all work/reuse counts and existing phase/
executor identities.

Require exactly 226 calls for each new segment. Every duration is positive.
Checked addition reconstructs:

```text
segment_sum = validation + capacity + buffer
residual = evaluation_setup - segment_sum
```

`segment_sum <= evaluation_setup`, residual is positive and all four values
sum exactly to `evaluation_setup`. For every run report both nanoseconds and
shares of `evaluation_setup`.

## Stability and routing

Across three processes, each of the four setup-share ranges must be at most
`0.05` absolute. Route exactly one next research target only when the largest
median share is at least `0.40` and leads the second by at least `1.20x`:

- validation selects one structural certificate/ownership audit;
- buffer selects one write-before-read, lifetime and high-water audit;
- capacity/control selects no implementation and requires a narrower audit;
- no stable leader selects no optimization and a narrower measurement.

No duration changes B4EP10SIRDI speed credit.

## Exit

PASS authorizes only the routed structural audit/research. Any parent semantic
or timer-accounting mismatch rejects the discriminator. B4E2, broad corpus,
runtime/GPU/schema and production remain blocked.
