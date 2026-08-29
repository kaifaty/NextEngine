# NSR3-B4EP10SII split incoming integration research -- 2026-08-22

Status: `COMPLETE / OPT_IN_FLOATING_AB_SELECTED`

## Input

B4EP10SID proves exact three-part floating order and B4EP10SICD proves an
independent exact incoming-plan builder. The remaining question is empirical:
does eliminating the serial pressure-active plan repay three construction
regions and 10.67% additional target-side visits?

## Candidate boundary

Add one opt-in research command:

```text
--nominal-hydro-split-incoming-plan-8
```

For each current topology, build the B4EP10SICD incoming plan without also
building a full or active plan. The topology result owns it until evaluation
moves it into the pressure tape; the tape owns it through all HVP calls.
Missing, stale or multiply moved ownership rejects without fallback.

Evaluation and HVP fold every target in exact B4EP10SID order:

```text
active lower incoming
active own source row
active upper incoming
```

The same directed values are read and added in the same order as B4EP10I.
Target logical-work counters remain 749,890,172 retained contributions, while
separate counters expose 454,936,226 full incoming scans and the two exact
374,945,086 retained components.

## Construction scope

This first integrated candidate retains the audited three construction regions
per topology: source/endpoint map, target count and target fill/validation.
It does not yet fuse them with existing topology row count/fill/metadata.
Across the transaction it therefore expects 4,089 total executor regions and
261,696 logical partitions.

This makes the A/B conservative and falsifiable. If it is exact but misses the
speed gate, its internal result can route to construction fusion research; the
gate must not be lowered.

## External A/B

Use one final Release binary pinned to physical CPUs `0..7`, one warmup per
command, then serialized `AB`, `BA`, `AB` pairs. Record monotonic wall and GNU
Time user/system/RSS. Require candidate byte identity and exact physical/work
roots before admitting duration.

PASS requires `3/3` candidate wins, at least `1.05x` median paired speedup,
candidate max/min wall at most `1.10`, and median RSS delta at most 16 MiB.

## Decision

Freeze B4EP10SII before implementation. PASS authorizes only residual timing;
performance failure retains B4EP10I and may authorize a separately frozen
construction-fusion discriminator. B4E2, broad corpus, runtime, GPU, schema
and production remain blocked.
