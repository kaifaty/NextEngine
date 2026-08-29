# NSR3-B4EP10S owner-parallel scaling evidence -- 2026-08-22

Status: `PASS / OWNER_PARALLEL_8_SCALING_CANDIDATE / HOST_SPECIFIC`

## Result

The frozen physical-core experiment selects 8 workers:

- serial B4EP7I median wall: 7.313827799 s;
- 8-worker median wall: 5.912456915 s;
- same-round speedups: `1.2370170854362037x`,
  `1.2370200585216442x`, `1.2370210553510015x`;
- median same-round speedup: `1.2370200585216442x`;
- worker-1/worker-8 median ratio: `1.9309754210699395x`;
- worker-8 median effective utilization: `6.4213636330761599`
  CPU cores.

All three worker-8 runs beat their same-round serial run, every condition's
range ratio is below 1.10, and all 18 measured processes preserve exact
stdout with empty stderr.

Worker 16 has the fastest median at 5.813467075 s, but worker 8 is only
`1.0170276770682494x` slower, inside the frozen 3% knee. It therefore wins as
the smallest near-fastest count while using a median 6.42 rather than 12.49
effective cores.

## Scaling table

| Condition | Median wall (s) | Min--max wall (s) | Range ratio | Median effective cores | Median RSS (KiB) |
|---:|---:|---:|---:|---:|---:|
| serial | 7.313827799 | 7.313444488--7.314597359 | 1.0001576372 | 0.9953748160 | 64,052 |
| 1 | 11.416808981 | 11.217073283--11.516239470 | 1.0266706100 | 0.9958034256 | 95,704 |
| 2 | 7.914412059 | 7.814595841--7.914580567 | 1.0127946125 | 1.8307982182 | 95,704 |
| 4 | 6.113523009 | 6.112445419--6.113659556 | 1.0001986336 | 3.3695792049 | 95,860 |
| 8 | 5.912456915 | 5.912161258--5.913074258 | 1.0001544275 | 6.4213636331 | 95,612 |
| 16 | 5.813467075 | 5.713124436--5.813813506 | 1.0176241689 | 12.4875006612 | 95,652 |

The owner-computes path is slower than the old serial path at one worker.
Its extra transpose plans, scratch payload and 3,411 OpenMP regions cost about
4.10 s before useful scaling. Eight workers recover that overhead and improve
whole-macro wall by 23.70%, but the result is far below the idealized B4EP9
Amdahl potential. This is direct evidence that region launch/barrier cost,
serial plan/prefix work and/or added memory traffic remain material.

The parallel path also raises nominal process RSS by about 31 MiB. That is
consistent with the B4EP10D/B4EP10I owner-plan and scratch boundary, and must
remain part of later capacity work.

## Admission and host

- CPU: AMD Ryzen 9 3950X, 16 physical cores/32 SMT threads, one NUMA node;
- governor/EPP: `performance`; boost enabled; no host policy changed;
- serial affinity: CPU 0;
- worker N affinity: distinct physical CPUs `0..N-1`;
- OpenMP: `OMP_PLACES=threads`, `OMP_PROC_BIND=close`, dynamic teams off;
- initial load: 0.03; initial available memory: 23,253,299,200 bytes;
- two unmeasured warmups and 18 measured fresh processes;
- measured execution was strictly serialized.

All three repetitions for each condition have the frozen stdout SHA-256:

| Condition | stdout SHA-256 |
|---:|---|
| serial | `8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095` |
| 1 | `bd3a55db1dad1cd2f9173b259ede5e32c5d337e3d0c13c032dd8318a37b1ff0f` |
| 2 | `8231d6099b1cfb730ffb37cd14f6a116821a4f00bd13053e3783933f69fa77bf` |
| 4 | `f6a5ee62152971b0689b40b68102e2e1dfa983d40677004933474c66c1265255` |
| 8 | `c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3` |
| 16 | `11aa81d704172a820eadcf941ce9b690567ab25c9a90601b98154c645e6c407f` |

Every parallel report retains common correspondence SHA-256
`917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca`
and zero team, coverage or worker mismatch.

## Reproducibility

- contract identity:
  `f034be427e9ca744391df7848e8c1237d6bda6485126dd1444e4286f6392ea03`;
- implementation commit:
  `abb7a06bdc16c3a906ce3b0e5d85f19a250dd9bd`;
- executable SHA-256:
  `b5bc2619f58c6eff8d219016b1a3d145add1ca63b4470e28e0b08be4dd325c46`;
- raw `metrics.tsv` SHA-256:
  `282d4062d3f9cdec6a2725b77088eec3465a5c2662f60190859997bfaa7f5fd5`;
- derived `stats.tsv` SHA-256:
  `c599288e7f3e97f953a703138d6f72b42d87bcdf0f0506fe11f249f0e41f9b7c`;
- paired-speedup table SHA-256:
  `1ed0ae73321db92620bbf94dc5b0aecc07a227d1ca186e8d727f36f7bfca2740`;
- CPU map SHA-256:
  `0cdf0f8a194bb0d28ff4a9ac105c4c89cf1e4e2913989d5d5697fd7712da9646`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10s.n6Toxm`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10s-evidence|v1|identity=f034be427e9ca744391df7848e8c1237d6bda6485126dd1444e4286f6392ea03|implementation=abb7a06bdc16c3a906ce3b0e5d85f19a250dd9bd|binary=b5bc2619f58c6eff8d219016b1a3d145add1ca63b4470e28e0b08be4dd325c46|metrics=282d4062d3f9cdec6a2725b77088eec3465a5c2662f60190859997bfaa7f5fd5|stats=c599288e7f3e97f953a703138d6f72b42d87bcdf0f0506fe11f249f0e41f9b7c|pairs=1ed0ae73321db92620bbf94dc5b0aecc07a227d1ca186e8d727f36f7bfca2740|medians-ns=S:7313827799,1:11416808981,2:7914412059,4:6113523009,8:5912456915,16:5813467075|range-ratios=S:1.0001576372121086,1:1.0266706100113834,2:1.0127946125473848,4:1.0001986335937212,8:1.0001544274521885,16:1.017624168898813|effective-cores=S:0.99537481604302658,1:0.99580342556276769,2:1.8307982182171905,4:3.3695792049320481,8:6.4213636330761599,16:12.487500661153819|selected=8;fastest=16;knee-ratio=1.0170276770682494|speedups-s8=1.2370170854362037,1.2370200585216442,1.2370210553510015;median=1.2370200585216442|worker1-over8=1.9309754210699395|exact=18of18;stderr=0|decision=b4ep10r-profile-research
```

SHA-256:
`839fe1bb6fedadfafd5fe44866723026096124a89bf824ac2fd2c839787f5167`.

## Decision

Retain `OWNER_PARALLEL_8_SCALING_CANDIDATE` for this host and nominal
research transaction. B4EP10R may now research and freeze an exact
selected-count residual attribution experiment. The 5.91-second one-macro
result is not close to production or real-time and does not authorize B4E2,
broad-corpus, runtime, GPU or public-schema work.
