# NSR3-B4EP10CTD -- current-topology reverse-plan audit contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10ctd-current-topology-plan-audit|v1|parent=5c96bd69d2a370a080482bb3284d44ee46a832aec3e551ddf0f6d517283a1943:42794aa55ca69ea7431324fb0a1a17457a025c3040591f9abaeb3f7e3ce3410a:3774b48864dcd9a23c6dcd78b386606096ded9bebeb590a256a44916fe4f5a90|implementation=6d1652f60ca6955e4389f9b2955927b004541e93|command=nominal-hydro-current-topology-plan-audit|baseline=serial-active-plan|candidate=current-topology-full-plan;compression-sentinel-positive|relation=source-by-slot-exact;active-target-row-stable-subsequence|audit=evaluation226;hvp459;plans226|work=current-full-scans-measured;retained749890172;ratio<=1.20|capacity=combined-added<=67108864|negatives=source;target|runs=2;stdout-byte-exact;stderr-empty|timing=none;old-commands-exact|reference=closed|credit=b4ep10ct-construction-research-only
```

Identity SHA-256:
`d5d457d09b116496185e8eb11f97f80e27959757b22e6de5b674b12437ca7259`.

## Implementation boundary

Add only:

```text
--nominal-hydro-current-topology-plan-audit
```

The returned transaction remains the B4EP10I serial active-plan path. For
each evaluation, build a separate full current-topology plan using the
existing validated serial builder and an all-positive compression sentinel.
Do not use that plan for floating evaluation or HVP. Old commands must not
construct, retain or scan it.

The audit may retain the full plan with the evaluation tape solely so each HVP
call can charge its projected full target-row scan. Its pointer/storage must
remain transaction-local and cannot survive tape/workspace release.

## Exact oracle and work

Across 226 plans require exact `source_by_slot` and exact stable-subsequence
target rows after filtering full entries by the real source compression.
Require 226 evaluation audits and 459 HVP projections. Retained evaluation
plus HVP entries must total exactly 749,890,172, matching the selected active
path. Measure full current-topology entries separately for evaluation and HVP;
their combined ratio to retained entries must be finite and at most `1.20`.

Require checked target/source/participant coverage, strictly increasing full
rows, zero fallback and no partial successful plan. Dedicated copies must
reject one invalid source mapping and one invalid target slot. Combined active
plan, full plan and owner scratch peak must not exceed 67,108,864 bytes.

## Runs and regressions

Run two fresh Release processes pinned to CPUs `0..7` with fixed OpenMP
placement and dynamic teams off. Require zero exit, empty stderr and
byte-identical stdout. No internal or external duration receives evidence
credit.

The final binary must preserve exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10PCI stdout
`e4559fcd5762cbf53a8ed5cdf57899dd2df1c364d0287525bcf55bef3eac8062`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

## Exit

PASS authorizes only separately frozen current-topology construction-dataflow
research. Failure retains B4EP10I and both previous plan candidates as
negative evidence. B4E2, broad corpus, runtime/GPU/schema and production
remain blocked.
