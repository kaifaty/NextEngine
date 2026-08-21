# NSR3-B4C3MAG canonical topology evidence

Status: `PASS / CANONICAL_TOPOLOGY_ADAPTIVE_MACRO_TRANSACTION_CANDIDATE`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --canonical-topology-self-test
```

Two parent-gated reports executed concurrently and are byte-identical:

```text
status                 PASS / TOPOLOGY_RESEARCH_ONLY
raw JSON + LF          c9d96f0bfa65c2aea0bda4c99753e078a54ae8f6e0a7045b68ad5527911a926e
raw JSON without LF    5823054bdf6ee4a9f3624f68f9577cba5b5acf18824b542787551a315f45fe28
semantic result        052f7a3f481c7f2b09781bdbc5991b1640cdbd3616c6cc38eb22301e54fc5e08
wall time              67.19 s / 66.96 s
CPU utilization        301% / 301%
maximum RSS            14,432 KiB / 13,740 KiB
B4C3MA negative exact  true
B4C3PE1 positive exact true
```

The isolated probe passes at raw-with-LF
`ec572e7c6ba6b703fbcba88f450cea6831fc929b1e31bd72baa1c16af4e76de6`,
raw-without-LF
`3d0eea8b3d2b716583d5c52469556194d0d8829d236f4822536dd12699df6fc0`
and the same semantic result.

## Topology discriminator

P1 deliberately reproduces the raw binary mismatch: 64 private exact-equality
features become 40 after decode, with 24 raw features lost. Canonical integer
identity maps both sets to all 64 KKT terminal features exactly. Geometry is
valid, every decoded integer coordinate is inside the closed box, no raw
penetration occurs and the maximum raw boundary shift remains only
`2.7755575615628914e-17 m`.

P2 has no boundary features and passes both raw and canonical identities.
Controls prove the decimal raw mismatch, exact canonical equivalence, rejection
at one integer unit, sample/face identity binding, collapsed-geometry rejection
and outside-box rejection.

## Transaction result

| Case | Accepted / attempted / discarded | Nonlinear / spectral HVP | Position admission | Velocity admission |
|---|---:|---:|---|---|
| P1 | `42 / 63 / 21` | `412 / 48` | temporal `0.633564` | temporal `0.001401` |
| P2 | `2 / 3 / 1` | `0 / 0` | temporal `0.020587` | absolute `0` |

All exact recovery grammar, adjacent selection, mixed-budget, macro ledger,
root, workspace and prepublication rollback gates pass. P1's macro KKT-scale
ledger residual is `2.330888e-11`; P2's is zero.

## Decision

Select `CANONICAL_TOPOLOGY_ADAPTIVE_MACRO_TRANSACTION_CANDIDATE`. Durable
contact topology is exact in canonical integer coordinates; raw binary equality
remains a diagnostic and is not an epsilon gate.

Authorize only design of a complete P1/P2 adaptive macro recovery replay.
Adaptive-versus-fixed comparison, nominal corpus, B4C4/B4D, CUDA, runtime/
schema and production remain blocked.
