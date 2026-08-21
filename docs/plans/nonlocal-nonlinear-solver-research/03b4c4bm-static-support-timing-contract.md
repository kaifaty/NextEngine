# NSR3-B4C4BM -- static-support timing discriminator

Status: `FROZEN / MEASUREMENT_AUTHORIZED / ROLLOUT_BLOCKED`

Parent B4C4B isolated probe passes with JSON-without-final-LF SHA-256
`187cff865ea739f96c1440f3041e7fd1f6de1a8c95436ae810f604850d80c90e`
and semantic SHA-256
`40f181c646d2bbc208a46915f09ae9c49dfcd29f22f7969b21940e26f6e2720b`.

## Identity

```text
sha256  eaed90deca8ca29ff07fb1b7df3ecbb3c1ca868181a4835c28caecc25c379758
text    nextengine.nonlocal.static-support-timing|v1|parent=b4c4b-probe|corpus=recorded-retained-one-macro-p1:264,p2:9|warmup=3|rounds=21|order=alternating-ab-ba|clock=steady-ns|thresholds=none
```

## Corpus and preflight

Capture, outside timing, the canonical tagged fluid input of each workspace in
the selected retained one-macro P1/P2 transactions. Require exact counts
`264/9`, passing source transactions, zero live ownership and deterministic
ordered corpus hashes.

For every state, build legacy combined and candidate split neighborhoods
outside timing. Require full exact equality and matching pair hash. Build and
bind the immutable support index once per fixture before any timed candidate
pass. The index build is deliberately excluded: B4C4B amortizes it over the
transaction.

## Measurement

- clock: `std::chrono::steady_clock`, reported integer nanoseconds;
- execution: P1 then P2, single-threaded within each case;
- warmup: three full corpus passes per path, excluded from samples;
- measured rounds: 21 paired passes;
- order: AB on even rounds, BA on odd rounds;
- timed scope: neighborhood builder calls plus deterministic numeric checksum
  accumulation only;
- allocations remain part of the builder under test;
- no solver, tape, evaluation, hashing, comparison or serialization is timed.

For both paths require passing neighborhoods and the same round checksum. The
checksum folds pair count, fluid/support pair counts, distance tests, maximum
degree and first/last pair identities for every corpus state.

## Statistics and disposition

Report all 21 samples per path, median, minimum, maximum, median absolute
deviation, paired candidate-win count and `legacy_median/candidate_median`.
Classify each case from median ordering only:

```text
candidate median < legacy median  -> CANDIDATE_FASTER
otherwise                         -> CANDIDATE_NOT_FASTER
```

No ratio is a pass/fail threshold. PASS means only valid measurement. Three
independent reports must reproduce parent, corpus, checksums and non-timing
semantic result; raw JSON and timing statistics are expected to differ.

## Gate and authority

A valid diagnostic may authorize a separately frozen rollout or lookup
redesign decision. It cannot select B4C4C, B4D, nominal corpus, CUDA,
runtime/schema or production, and it makes no whole-solver speedup claim.
