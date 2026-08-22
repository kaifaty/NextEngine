# NSR3-B4EP10SID -- split self/incoming plan audit contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sid-split-self-incoming-audit|v1|parent=d5d457d09b116496185e8eb11f97f80e27959757b22e6de5b674b12437ca7259:64e0898ac28ffa5061e1e743064f707ff0b970a562bc0c44163d3d722d0c85bb:e6abe4bffc499c00cdd5ade4f5712ca50162bb0ee6d07b155df46a70a036d7ca|implementation=88adf275d9b37de870c15dad5ab4583e76c0d607|command=nominal-hydro-split-incoming-plan-audit|representation=source-row-self;participant-reverse-incoming|fold=lower-incoming,active-own-row,upper-incoming|audit=plans226;evaluation226;hvp459;slot-sequence-exact|work=incoming-full454936226;own-retained374945086;visits829881312;baseline749890172;ratio=1.1066704738730726;limit<=1.15|capacity=combined-added<=67108864|negatives=own-boundary;incoming-target|runs=2;stdout-byte-exact;stderr-empty|timing=none;old-commands-exact|reference=closed|credit=b4ep10sic-construction-research-only
```

Identity SHA-256:
`5b730b2e84676f8b47aa6b16ca2e519ae8b7dff43e935a1fc2ba642bed976b4c`.

## Implementation boundary

Add only:

```text
--nominal-hydro-split-incoming-plan-audit
```

The returned evaluation and HVP remain the B4EP10I serial active-plan path.
Construct an audit-only current-topology full plan as in B4EP10CTD, then view
as incoming only those target entries whose source differs from the target.
Do not execute the candidate floating fold.

For fluid target `t`, reconstruct the candidate slot sequence in exact order:

1. active incoming slots below `flat_offsets[t]`;
2. every own-row slot when `compression[t] > 0`;
3. active incoming slots at or above `flat_offsets[t + 1]`.

Any incoming slot inside the own interval, own slot outside it, invalid source,
target or non-increasing row rejects. Support targets use incoming only.

## Exact work and capacity

Across 226 plans, 226 evaluations and 459 HVP projections require:

- 454,936,226 full incoming entries;
- 374,945,086 retained incoming entries;
- 374,945,086 retained own-row entries;
- 829,881,312 total projected visits versus 749,890,172 active target
  entries, exact ratio `1.1066704738730726` and at most `1.15`;
- byte-identical reconstructed and selected active target rows;
- zero order, coverage or fallback mismatch.

Incoming-plan capacity is source-by-slot plus one incoming target slot per
directed slot and target offsets. Its maximum payload plus existing owner
scratch must not exceed 67,108,864 bytes. Dedicated copies must reject one
shifted own interval and one invalid incoming target slot.

## Runs and regressions

Run two fresh Release processes pinned to CPUs `0..7` with fixed OpenMP
placement and dynamic teams off. Require zero exit, empty stderr and
byte-identical stdout. No duration receives evidence credit.

The final binary must preserve exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10CTD stdout
`fbfc07a9f0ee7fb3b3cb495e4b5c76d73c8967f5ba84aa70c4c75c1ffd0c8663`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

## Exit

PASS authorizes only separately frozen B4EP10SIC topology-construction
research. Failure retains B4EP10I and all previous plan candidates as
negative evidence. B4E2, broad corpus, runtime/GPU/schema and production
remain blocked.
