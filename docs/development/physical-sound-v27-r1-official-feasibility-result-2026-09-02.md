# Physical sound V27 R1 — official compact-representation feasibility result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `RUN_A_RESOURCE_TIMEOUT / NO_CANONICAL_OUTPUT / RUN_B_NOT_STARTED / QUALITY_UNOBSERVED / M0B_SPENT` |
| Roadmap package | V27 `R1` |
| Protocol | P0a `m0b-v1.1`, SHA-256 `54522c26eb3db62ad316ea6001f9380651f760329f0db8556b441ad286b649e8` |
| Implementation | commit `db85c799c44b8a8d17fd0afe5442b46fd208a4a9`, root `d013ec35f6499cfc17f5beeec3db8d245f2a296b88a104cc80047b2ece04f456` |
| Official manifest | External `/tmp/nextengine-v27-r1-official-GMCIdk/manifest.json`, SHA-256 `40a98ab9f289cd3b3b8359fb503fc45bdaaa7554cdaa468527c795ed1b4e4962` |
| Product effect | None; authored clips remain authoritative |

## Question and result

Can the frozen compact physical representation complete its official CPU
training/evaluation inside `1,800 s`, then answer the neural-versus-control
quality question reproducibly?

**No result was obtained.** Run A remained CPU-active until the external
`1,800 s` wall timeout terminated it. No canonical output was atomically
published, no terminal quality metrics or decision exist, and run B was not
started. This is a resource reject, not evidence for or against neural quality.

## Frozen execution

The manifest bound:

- combined V3 `c43ba8e…70bb`;
- T0 evidence `884da56f…7ed`;
- X0 lineage `e4f6bb11…34f`;
- protocol `54522c26…49e8` and implementation root `d013ec35…f456`;
- official profile, seed `3101`, CPU/one-thread, `1500 + 500` steps;
- unchanged model, losses, controls, ablations, roles and gates;
- offline/frozen/no-sync environment with cgroup `IPAddressDeny=any`,
  `MemoryMax=4 GiB`, `MemorySwapMax=0` and external `1,800 s` timeout.

Run A used one process throughout. Read-only health checks observed about
`101%` CPU and RSS between `1,383,644` and `1,564,012 KiB`; the process remained
well below the memory ceiling until the wall timer. These observations are not
an exact peak-RSS certificate: the `/usr/bin/time` record stayed empty because
the outer timeout terminated the timed process before it could flush.

## Publication audit

| Evidence | Result |
| --- | --- |
| Published `run-a/` directory | absent |
| Canonical weights/predictions/freeze | absent |
| Control/holdout/disclosed-real/final reports | absent |
| MLflow directory | absent |
| stdout / stderr | empty / empty |
| Resource record | empty; timeout wrapper prevented final flush |
| Run B | `NOT_STARTED_BY_STOP_RULE` |

One interrupted staging directory remains external. It contains only the
value-independent `surface-query-report.json`, `1,164` bytes, SHA-256
`4f3db4c91dd138789247608c0dd38abbeb4ecec519fe3b92e9e1636776375f31`.
There is no candidate freeze, model tensor, prediction or partial published
output. The residue is diagnostic evidence that the current signal handler
does not remove staging after an external `SIGTERM`; it grants no quality or
publication authority.

## Cost shape, without opened metrics

The frozen runner statically requests five complete variants: neural A, neural
B, no-geometry, no-contact and no-residual. Each performs `2,000` optimizer
steps, for `10,000` total steps before evaluation. Every real-adaptation step
also evaluates three transfer contexts, two recording losses and one synthetic
replay batch. This explains a large fixed CPU surface, but the terminated run
published no step cursor or profile, so it does not prove which component was
the bottleneck and cannot select a code change.

## Decision

R1 closes as `RESOURCE_REJECT`. M0b is spent. Do not rerun A, start B, shorten
the schedule, drop an ablation, enable compilation/GPU, change a loss or inspect
partial internal values under the same protocol.

Roadmap V27's R2A branch requires an R1 pass and is not authorized. R2B requires
a representation reject and is also not authorized by a resource timeout.
Signal-blind Q0 validator/source inventory may continue, but no validator
release, Metal candidate, cooker or demo may claim readiness from this result.

The next generator attempt needs a new falsifiable resource hypothesis and a
new protocol/roadmap. It must first measure a value-independent successful
control that predicts official cost, include flushable resource evidence and
prove external signal cleanup, before opening any fresh model role.
