# B4C4CM flat-adjacency timing research

Status: `DESIGN FROZEN / EXECUTION AUTHORIZED`

Date: `2026-08-21`

## Question

Does B4C4C's structural reduction improve the actual serial CPU construction
path when allocator, cache, evaluation and tape-transfer costs are included?

Work counts alone cannot answer this. B4C4C removes nested allocations,
directed-record duplication and row sorts, but it also validates flat rows and
recovers participants through pair indices during the initial gradient.

## Timed scope

For every pre-recorded canonical fluid state, both variants use the same
already-built immutable B4C4B support index. Time exactly:

```text
joint neighborhood discovery + pair sort
adjacency construction
pressure evaluation
pressure tape construction / ownership transfer
deterministic checksum consumption
```

The legacy path builds nested participant rows and reconstructs tape CSR. The
candidate builds flat pair-index CSR and transfers it. Static-index creation,
source transaction, corpus hashing, exact output comparison and reporting are
outside timed regions.

## Corpus and protocol

Reuse the B4C4BM state-capture mechanism, now guarded by the exact B4C4C
parent. It records the complete retained one-macro query sequence:

- P1: `264` canonical 48-fluid states with 544 immutable supports;
- P2: `9` canonical 27-fluid states with 1,216 immutable supports.

Preflight every state for exact pairs, row order, evaluation, tape and
checksum. Run three untabulated warmups, then 21 paired measurements with
alternating legacy/candidate order. Use `steady_clock` nanoseconds and report
all raw samples, min/median/max/MAD, paired wins and median ratio.

## Decision rule

PASS means only that the benchmark protocol and exactness checks are valid.
Classify observed timing as `CANDIDATE_FASTER` or `CANDIDATE_NOT_FASTER` but
apply no fitted ratio or win-count threshold. Require three independent
processes to repeat the deterministic corpus/checksum result; raw timing bytes
are expected to differ.

Complete-lane rollout may be designed only after the measurement is recorded.
No whole-solver, runtime, GPU or production claim follows from this benchmark.

## Decision

Execute the frozen
[B4C4CM contract](../plans/nonlocal-nonlinear-solver-research/03b4c4cm-flat-adjacency-timing-contract.md).
B4C4C remains the isolated correctness candidate while B4D stays blocked.
