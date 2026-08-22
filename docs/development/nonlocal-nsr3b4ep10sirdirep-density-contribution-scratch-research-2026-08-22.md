# NSR3-B4EP10SIRDIREP density-contribution scratch research -- 2026-08-22

Status: `COMPLETE / EPHEMERAL_HIGH_WATER_REUSE_SELECTED`

## Question

Can one mechanically remove the largest safe subset of repeated setup
initialization after SIRDIREI failed, without changing any returned workspace
representation or default path?

## Selected boundary

`density_contribution` is the only audited pair-sized setup buffer that is
strictly builder-local. It is written once for every current pair, read only in
the later density fold and destroyed before the workspace is returned.
SIRDIREA proves exactly one live ephemeral lane and full write-before-read.

Keep one ordinary `std::vector<double>` inside the transaction's parallel trace.
For each of 226 evaluation calls:

1. grow it only if `pair_count` exceeds the current high-water size;
2. never shrink it inside the transaction;
3. write and read only indices `[0, pair_count)`;
4. report payload from current pair extent, not high-water size;
5. release storage through a transaction guard on every exit.

This preserves standard vector semantics and changes no `Evaluation`, tape,
gradient, returned-workspace size, HVP coefficient, plan, directed scratch or
physical arithmetic. The projected work changes 85,716,150 repeated initialized
slots to 380,511 growth slots / 3,044,088 bytes (`0.0044391984474337681x`).

## Measurement design

Use an opt-in command against the unmodified SIRDI command in one binary. Run
one warmup per command, then balanced serialized `AB`, `BA`, `AB` pairs on
cores `0..7`. Require exact candidate semantics and all lifetime/work counts.

The prior allocator failure shows that relative A/B is insufficient when a
shared implementation change corrupts the baseline. Add explicit baseline
health gates: median baseline wall no more than 4.72 s (under 1.10x the accepted
4.290430589 s median) and baseline range ratio no more than 1.10. Candidate
must win all three pairs with median speedup at least `1.02`, range ratio at
most 1.10, RSS delta at most 8 MiB and total CPU ratio at most 1.02.

The 2% gate is lower than earlier broad candidates because this isolated role
accounts for only one of four pair-sized `double` initialization streams; it is
predeclared from SIRDIREA work counts and SIRDIRE timing, not from candidate
A/B results.

## Decision

Freeze B4EP10SIRDIREP as one density-contribution scratch reuse path only.
SIRDI remains the immutable baseline. Do not change returned storage, allocators,
other setup vectors or arithmetic. PASS selects residual attribution research;
FAIL retains SIRDI and stops this buffer-init branch.
