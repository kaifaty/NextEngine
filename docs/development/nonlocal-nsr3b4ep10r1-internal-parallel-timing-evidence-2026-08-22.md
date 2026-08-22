# NSR3-B4EP10R1 internal parallel timing evidence -- 2026-08-22

Status: `PASS / EVALUATION_PLAN_RESEARCH_SELECTED / TIMING_ONLY`

## Result

Three fresh 8-worker processes pass every frozen correspondence, counter and
capacity gate. All reproduce semantic result SHA-256
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`
and B4EP10I common correspondence SHA-256
`917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca`.
Program stderr is empty.

The executor-capacity result rejects both initial overhead hypotheses:

| Capacity share | Three values | Median | Range |
|---|---|---:|---:|
| orchestration | 0.0146384, 0.0110778, 0.0107156 | 0.0110778 | 0.00392284 |
| imbalance | 0.0655452, 0.0668413, 0.0628240 | 0.0655452 | 0.00401730 |
| active | 0.919816, 0.922081, 0.926460 | 0.922081 | 0.00664397 |

OpenMP-region orchestration is about 1.1% of available worker capacity and
partition imbalance about 6.6%. Neither reaches its frozen 20% routing
threshold. Persistent-team amortization and partition rebalancing therefore
receive no optimization credit from this experiment.

## Hierarchical attribution

All top-level shares are stable well inside the maximum 0.05 range:

| Stage | Median transaction share | Range |
|---|---:|---:|
| topology | 0.101901 | 0.000259850 |
| evaluation | 0.546301 | 0.000605284 |
| HVP | 0.284999 | 0.00140587 |
| residual | 0.0665950 | 0.000745059 |

Evaluation leads HVP by `1.9168506974796429x`, above the frozen `1.20x`
route. Its leading subphase is `evaluation_plan`:

| Evaluation subphase | Median ns | Median evaluation share |
|---|---:|---:|
| owner plan plus canonical energy fold | 1,427,089,685 | 0.423316 |
| directed values | 1,026,179,815 | 0.306543 |
| setup | 426,292,677 | 0.126788 |
| target gather | 231,705,323 | 0.0690040 |

The remaining five evaluation subphases are each below 4.4%. The selected
next question is therefore the construction and ownership of the active
target-gather plan, including the small serial canonical energy fold currently
timed with it. This timing does not yet identify a safe transformation.

## Exact work and regressions

Every run reports exactly one transaction, 226 topology calls, 226 evaluation
calls, 459 HVP calls, 3,411 executor regions and 218,304 logical partitions.
Per-region capacity identity holds and all 23 subphase call counts are exact.

The final binary also preserves all prior receipts:

| Command | Preserved SHA-256 or semantic result |
|---|---|
| owner parallel 1 | `bd3a55db1dad1cd2f9173b259ede5e32c5d337e3d0c13c032dd8318a37b1ff0f` |
| owner parallel 2 | `8231d6099b1cfb730ffb37cd14f6a116821a4f00bd13053e3783933f69fa77bf` |
| owner parallel 4 | `f6a5ee62152971b0689b40b68102e2e1dfa983d40677004933474c66c1265255` |
| owner parallel 8 | `c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3` |
| owner parallel 16 | `11aa81d704172a820eadcf941ce9b690567ab25c9a90601b98154c645e6c407f` |
| fused serial | `8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095` |
| fused phase semantic result | `44e93e6e4ee24dcc623fe36d4c99ad4456f482fe1b47d18410ffb08204f9cd72` |
| owner-dataflow audit | `22ed1368916e01cb10e701123c7185c2f1702a4a485ccbcfede9109d0fd1155c` |

## Reproducibility

- contract identity:
  `ab9f3e0bca369c80e0185d6c73cb8334ff4baa6a9aa8728758b6fe162975d638`;
- implementation commit:
  `e6ea63381437a3d5e0179bc164475e6983314836`;
- executable SHA-256:
  `5b79a0166dd85c7bddb89d07b0629c68a0b891b0bac3510c4e5abb2eabfb9587`;
- raw run SHA-256 values:
  `857057c4166e2665080b38579576fc66dee0ac2fc1e01a007e177263fbe0053a`,
  `07c25bcfe9aae1904aaa1a79dddb46d8106a7321a309a2d3863045977d9d02d7`,
  `528fe00c373c705d54e467eedb918b606957a9838c8c93794f014b1015b162f7`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10r1.KBsVYt`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10r1-evidence|v1|identity=ab9f3e0bca369c80e0185d6c73cb8334ff4baa6a9aa8728758b6fe162975d638|implementation=e6ea63381437a3d5e0179bc164475e6983314836|binary=5b79a0166dd85c7bddb89d07b0629c68a0b891b0bac3510c4e5abb2eabfb9587|runs=857057c4166e2665080b38579576fc66dee0ac2fc1e01a007e177263fbe0053a,07c25bcfe9aae1904aaa1a79dddb46d8106a7321a309a2d3863045977d9d02d7,528fe00c373c705d54e467eedb918b606957a9838c8c93794f014b1015b162f7|semantic=a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b|correspondence=917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca|calls=transaction:1,topology:226,evaluation:226,hvp:459,regions:3411,partitions:218304|executor-medians=orchestration:0.011077760464830021,imbalance:0.065545153423310784,active:0.92208091877289078;ranges=0.003922842833479004,0.0040172980650664825,0.006643973559577132|stage-medians=topology:0.1019005239775314,evaluation:0.5463009172916731,hvp:0.2849992010384392,residual:0.06659503211530525;ranges=0.00025985020157522076,0.0006052838071242128,0.0014058676780261958,0.0007450592463777855|leader=evaluation;runner=hvp;ratio=1.9168506974796429|subphase=evaluation-plan;median-ns=1427089685;stage-share=0.4233158357706572|old=owner1:bd3a55db1dad1cd2f9173b259ede5e32c5d337e3d0c13c032dd8318a37b1ff0f,owner2:8231d6099b1cfb730ffb37cd14f6a116821a4f00bd13053e3783933f69fa77bf,owner4:f6a5ee62152971b0689b40b68102e2e1dfa983d40677004933474c66c1265255,owner8:c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3,owner16:11aa81d704172a820eadcf941ce9b690567ab25c9a90601b98154c645e6c407f,fused:8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095,phase:44e93e6e4ee24dcc623fe36d4c99ad4456f482fe1b47d18410ffb08204f9cd72,audit:22ed1368916e01cb10e701123c7185c2f1702a4a485ccbcfede9109d0fd1155c|decision=b4ep10p-evaluation-plan-research
```

SHA-256:
`ad54e7ab063e88c45dae8ee518a94870c585bc4c5879b2f9d6ae770afe385232`.

## Decision

Close B4EP10R1 as PASS and authorize only B4EP10P evaluation-plan
architecture research. The instrumented durations are not throughput or
speedup evidence. B4E2, broad corpus, runtime, GPU, schema and production
remain blocked.
