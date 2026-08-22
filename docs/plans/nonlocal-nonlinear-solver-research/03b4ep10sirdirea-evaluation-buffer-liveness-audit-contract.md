# NSR3-B4EP10SIRDIREA -- evaluation buffer liveness audit contract

Status: `CLOSED / PASS / EVALUATION_BUFFER_REUSE_CONTRACT_RESEARCH_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdirea-evaluation-buffer-audit|v1|parent=a018e4d47080b76bb56166a5a6a7c5ba892e23687249cb519956951248ca3974:f445395e1443d71f30827c3ea7372ad1bc2d066d96b7c859ed6be862675118a6:b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9|implementation=7f5bc93d118f0dab33e62d3584c9362b1e2f2ce0|command=nominal-hydro-directed-scratch-evaluation-buffer-audit|candidate=unchanged-sirdi-return;shadow-only|buffers=gradient:total:vec3;density:fluid:f64;radius:pair:f64;compression:fluid:f64;hvp-gradient:pair:f64;hvp-second:pair:f64;density-contribution:pair:f64|proof=full-write-before-publication;density-contribution-read-after-write;density-read-after-write;workspace-acquire-release-exact;ephemeral-acquire-release-exact|projection=two-workspace-lanes;one-ephemeral-lane;per-role-high-water-growth|calls=evaluation226;retention42;workspace-acquire226;workspace-release226|max-live=workspace2;ephemeral1|runs=2;fresh-processes;stdout-byte-exact|capacity=audit-shadow<=8388608;projected-pool<=67108864|negatives=missing-write;duplicate-release|timing=none|route=full-coverage&&workspace-max2&&ephemeral-max1&&projected-growth/repeated<=0.02:reuse-contract-research;else:stop|reference=closed|credit=evaluation-buffer-reuse-implementation-contract-research-only
```

Identity SHA-256:
`dfc1bb3d154f9e406c89d0fde2304983c837d9f27d43888be542f6e0e5697e1d`.

## Implementation boundary

Add only:

```text
--nominal-hydro-directed-scratch-evaluation-buffer-audit
```

It executes the unchanged, uninstrumented B4EP10SIRDI candidate. Shadow state,
lane receipts and counters cannot enter physical arithmetic, active-set or
accept/reject branches, roots or returned workspaces outside this command. No
buffer is reused and every existing vector construction remains unchanged.

## Per-buffer proof

Audit exactly seven roles: gradient at total-participant extent; density and
compression at fluid extent; radius, HVP gradient, HVP second and density
contribution at pair extent. For each of 226 evaluations report requested,
written and maximum slots plus initialization bytes. Require each requested
index to be written before publication. Density-contribution reads must occur
only after the pair phase writes the referenced index; density reads must occur
only after the density phase writes the center.

The audit owns no floating values. One missing-write corruption must fail with
the exact coverage class before the positive audit can pass.

## Lifetime and high-water proof

Successful workspace publication acquires one audit receipt. Every existing
workspace release consumes exactly that receipt; moves and accepted retention
must preserve it without acquiring another. Require 226 acquires, 226 releases,
zero double/unknown releases, zero final live receipts and exactly two maximum
simultaneous receipts. A duplicate-release corruption must reject.

The builder-local density-contribution shadow owns a separate ephemeral
receipt guarded across all returns. Require 226 acquires/releases, maximum one
live receipt and zero final live receipts.

Map each real workspace receipt to the first free one of two projected lanes.
For six retained roles record only capacity growth above that lane's prior
high-water; record density contribution against one independent lane. Require
checked byte accounting, maximum shadow payload no more than 8,388,608 bytes,
maximum projected reusable capacity no more than 67,108,864 bytes and
projected growth bytes / repeated initialization bytes no more than `0.02`.

## Exactness and execution

Run two fresh serialized processes on CPUs `0..7`. Both must be byte-identical,
exit zero with empty stderr and reproduce B4EP10SIRDI result
`b4f847cb...777e9`, B4EP10SII result, correspondence, five physics roots,
query/work/reuse/retention counts and duration-free semantics exactly. Timing
is forbidden and the audit grants no speed credit.

## Exit

PASS authorizes research and freezing of one opt-in evaluation-buffer reuse
implementation/A-B contract. Any coverage, receipt, capacity, parent semantic
or negative-control mismatch stops this path. B4E2, broad corpus,
runtime/GPU/schema and production remain blocked.
