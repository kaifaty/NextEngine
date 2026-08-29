# NSR3-B4C3MA adaptive macro transaction evidence

Status: `FAIL / CANONICAL_TOPOLOGY_RECLOSURE_REQUIRED`

Date: `2026-08-21`

## Reproducible result

Isolated command:

```text
nonlocal-formula-reclosure --macro-adaptive-transaction-probe
```

Two reports are byte-identical:

```text
status                 FAIL / TRANSACTION_RESEARCH_ONLY
first failure          p1-supported-adaptive-macro-transaction:ADAPTIVE_MACRO_TRANSACTION_GATE
raw JSON + LF          b95c9fd26053ebbf55616ca12ec7d6209590254ec6da835eaa273fad067c6377
raw JSON without LF    8def13d3b0f54fdffaa451846219d9a842acda416e2270464349aff6831847a1
semantic result        ee8b80a04e0b251c95697f22a8fc07bd7b4101cf171c3c14a287f3d42e70eb8f
wall time              approximately 0.60 s
```

The isolated stop rule prevents the expensive parent-gated replay. B4C3PE1
remains the last selected positive boundary.

## What passed

- P1 derives `21` initial substeps, selects the adjacent `21/42` pair and
  accounts `42/63/21` accepted/attempted/discarded substeps.
- P2 derives `1`, selects `1/2` and accounts `2/3/1`.
- Both adjacent embedded gates, both position/velocity mixed admissions,
  balanced publication, macro KKT ledger, roots and workspaces pass.
- P1 publication uses only `0.633564` of its selected temporal position budget
  and `0.001401` of its selected temporal velocity budget.
- P2 position uses `0.020587` of its temporal budget; its exactly uniform
  velocity uses the absolute branch at zero utilization.
- Exact failure grammar, non-adjacent/exhaustion policy negatives and forced
  prepublication rollback all pass.

## Blocking topology result

The selected private P1 endpoint has 64 exact boundary memberships, equal to
its KKT terminal contact features. A raw binary64 equality check sees only 40
after canonical decode and reports 24 lost upper-face memberships. It adds no
features, creates zero penetration and moves each affected coordinate by at
most `2.7755575615628914e-17 m`.

The discrepancy is the representation identity of the same decimal geometry:
the binary fixture computes upper bounds such as `0.2 - 0.025`, while canonical
decode represents `0.175` on the micrometre lattice. It is not a micrometre
quantization displacement and not a physical contact change. Nevertheless,
B4C3MA's raw `double == double` topology gate is false and the frozen stage
correctly remains FAIL.

## Decision

Do not introduce an epsilon or weaken contact identity. Design a separate
canonical-topology discriminator that compares both particle and boundary
coordinates through `canonical::quantize_position`, yielding exact integer
feature identity. Keep raw binary membership, maximum coordinate shift and
penetration as mandatory diagnostics. Re-execute the complete one-frame
transaction only under a new policy identity.

Complete adaptive replay, adaptive-versus-fixed comparison, nominal corpus,
CUDA, runtime/schema and production remain blocked.
