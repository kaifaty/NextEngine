# NSR3-B4EP10SIRD -- source-local discriminator contract

Status: `CLOSED / PASS / DIRECTED_STRUCTURAL_AUDIT_RESEARCH_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sird-source-local-discriminator|v1|parent=4bd4879fc96f602a3988a9e35fcc184e091509a600bf09dabfe32310e2cec731:62a2205d6ff010ab65d7730c41479305ec2ec15a5dd45c248ccec38e25d10aa3:36461dfb06239f5f2eced51afd75b82fe8d321252cfc3d4cc9d7a294c841c94a|implementation=4946af8abc21320685ed0294045df6692fb92ff3|command=nominal-hydro-split-incoming-phase-timing-8|reduction=directed:evaluation-directed+hvp-directed;setup=evaluation-setup+hvp-setup;compression:hvp-compression;local-scalar:evaluation-pair+evaluation-metadata+evaluation-density+evaluation-center|identity=components-positive;sum-source-local-exact;durations-excluded|runs=3;fresh-processes;serialized;affinity=0-7|gates=sir-semantic-62a2205d6ff010ab65d7730c41479305ec2ec15a5dd45c248ccec38e25d10aa3;share-range<=0.05|route=largest-median-share>=0.20&&lead>=1.20:single-structural-audit;else:no-optimization+narrower-measurement|reference=closed|credit=one-next-mechanical-research-only
```

Identity SHA-256:
`809807a71e2aafa46758b8635d0ba3f366ef4caee40787282fb287dba3b1eaa4`.

## Execution

Run the unchanged command three times in fresh serialized processes on CPUs
`0..7` with the frozen OpenMP environment. Each process must pass, reproduce
B4EP10SIR result `62a2205d...10aa3`, B4EP10SII result `f7b1542f...25fb2`, all
five roots and exact timing/accounting identities.

For every process reconstruct the four groups defined in the identity. Every
component must be positive, all additions checked, and the group sum must
equal reported `source_local_ns`. Divide each group only by
`transaction_total_ns`.

## Stability and route

The absolute range of each group share across the three processes must be at
most `0.05`. If stable, rank median shares:

- largest share `>=0.20` and at least `1.20x` the second authorizes one
  separately frozen structural audit of that group;
- otherwise no optimization is selected and a narrower measurement must be
  designed.

No duration grants performance credit. B4E2, broad corpus, runtime, CUDA/GPU,
schema and production remain blocked.
