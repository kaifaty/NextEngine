# NSR3-B4C4CM -- flat-adjacency construction timing

Status: `PASS / COMPLETE-LANE DESIGN AUTHORIZED`

Parent B4C4C passes at isolated JSON-without-final-LF SHA-256
`45f340a34f2e970179bf30358c126dc1ea30302c8bcca70c1c190715836f5a7b`
and semantic SHA-256
`22007ffd36f7189ea4aca544c93f7a0b3193ec6962438c626be0a03179dabb8f`.

## Identity

```text
sha256  1ebff56a6509b8f2b3fa2fbdf90f48abfa58abfc1db8d63d0f0c4982d28a4e51
text    nextengine.nonlocal.flat-adjacency-timing|v1|parent=b4c4c:45f340a34f2e970179bf30358c126dc1ea30302c8bcca70c1c190715836f5a7b|corpus=p1:264,p2:9|scope=neighborhood+evaluation+tape|warmup=3|rounds=21|order=abba|thresholds=none
```

## Corpus and exactness

Capture all canonical fluid states from the retained one-macro P1/P2 query
sequence. Require exactly `264/9` states and repeat the B4C4BM corpus identity
for the same case name/index binding. Build one immutable support index per
case outside timing.

For every state, require exact legacy/candidate pairs, participant rows,
evaluation, untaped semantics, final pressure tape and deterministic checksum
before accepting any timing sample.

## Measurement

Time neighborhood + evaluation + tape construction and a bounded checksum
consumer. Exclude source transaction, corpus/index creation, preflight and
reporting. Execute:

1. three warmup pairs;
2. 21 measured pairs;
3. legacy first on even rounds, candidate first on odd rounds.

Report raw nanoseconds, min/median/max/MAD, candidate wins, median speedup and
classification. `steady_clock` must be steady; elapsed samples and checksum
must be valid.

## Gate and authority

Apply no timing threshold. Three sequential processes must reproduce the same
deterministic result hash over parent/corpus/checksum/protocol while raw timing
reports may differ. PASS is measurement-only and may authorize B4C4C1
complete-lane design. B4D, nominal corpus, CUDA, runtime/schema and production
remain blocked.
