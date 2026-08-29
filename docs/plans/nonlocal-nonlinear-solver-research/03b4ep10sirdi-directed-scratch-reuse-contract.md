# NSR3-B4EP10SIRDI -- directed scratch reuse implementation/A-B contract

Status: `CLOSED / PASS / CANDIDATE_RESIDUAL_ATTRIBUTION_RESEARCH_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdi-directed-scratch-reuse|v1|parent=4ae408eb091ee617fa74325a8a2b53652407dd80ba07533b0e6fc8e190c95743:944f00de623cb681eb1846e52612a133e4e84bfce8df01d988fd6ef2b3bf089a:ec199f0da2dda5764bc67e0c9a587e79561f75b6b4650000fc56458b5f93b1f6|implementation=28e0fcf25999124c54a34f956e00ed6f6c6f5565|commands=baseline:nominal-hydro-split-incoming-plan-8,candidate:nominal-hydro-directed-scratch-reuse-8|ownership=transaction-local;parallel-trace-adjacent;high-water;release-all-exits|scope=directed-vec3-only;evaluation+hvp;compression+target-unchanged|work=calls685;full-init454936226;growth670229;active-writes374945086;payload16085496|semantics=sirda-liveness;source-order-exact;target-fold-exact;physics-roots-exact;old-commands-exact|capacity=67108864;fail-closed|timing=external-monotonic+gnu-time;one-warmup-each;three-pairs=AB,BA,AB;serialized;affinity=0-7|gates=candidate-exact-3of3;wins3of3;median-paired-speedup>=1.05;candidate-range-ratio<=1.10;rss-delta-kib<=16384;median-total-cpu-ratio<=1.02|failure=retain-b4ep10sii|reference=closed|credit=candidate-residual-attribution-research-only
```

Identity SHA-256:
`35a1d41b78d132429334a34d8c99e6d2870b2b8a68ee949beb5a3c69375dff10`.

## Implementation boundary

Add only:

```text
--nominal-hydro-directed-scratch-reuse-8
```

The command is B4EP10SII plus transaction-local reuse of the directed `Vec3`
temporary in evaluation and HVP. It may not reuse compression/target buffers,
fuse loops, change partitioning, retain storage after transaction exit or run
the SIRDA shadow bitmap in timed work.

Grow the buffer only to a larger directed size and address only the current
prefix. Capacity above 67,108,864 bytes fails before use. Active source rows
overwrite their slots exactly as in B4EP10SII. Every return path releases the
buffer and reports zero live storage.

## Functional gate

One candidate process must reproduce B4EP10SII result `f7b1542f...25fb2`, all
five roots, correspondence `1e4bedbb...08a35d`, 226 evaluations, 459 HVPs,
4,089 regions and 261,696 partitions. Require exactly 685 reuse calls,
454,936,226 requested full slots, 374,945,086 active overwrites, 670,229
growth slots, 16,085,496 peak bytes, one release, zero live storage and zero
capacity/liveness failures. Old B4EP10SII, B4EP10SIR and B4EP10SIRDA commands
remain exact.

## External A/B

After one unmeasured warmup per command, run serialized `AB`, `BA`, `AB` pairs
on CPUs `0..7` with exact OpenMP placement. All candidate outputs must be
byte-identical and exact; stderr must be empty. Require candidate wins in all
three pairs, median paired speedup `>=1.05`, candidate wall range ratio
`<=1.10`, median RSS delta `<=16,384 KiB`, and candidate/baseline median total
CPU ratio `<=1.02`.

PASS selects directed scratch reuse only for the nominal research path and
authorizes candidate residual-attribution research. Failure retains B4EP10SII.
B4E2, broad corpus, runtime, CUDA/GPU, schema and production remain blocked.
