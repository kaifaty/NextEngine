# Physical sound validator PS-1 — amplitude-envelope specialist

Date: 2026-08-27
Status: `PS_1_EXIT_MET / SELECTIVE_PASS_DISABLED / PS_2_NEXT`

## Question

Can a deterministic amplitude-envelope specialist close the two frozen
shuffled-envelope false passes from AV-P0C without changing the source model,
corpus, partitions, mutation parents or provisional threshold-selection rule,
and without losing the stationary/frozen negative controls?

## Frozen inputs and authority

- Input manifest SHA-256:
  `98f5489aa40841f2cb0a67409d454cb40e983a957f36fa077a9d426375750281`.
- Corpus remains 19 real entries plus 36 controlled mutations across the same
  development/calibration/holdout/shadow partitions.
- Mutation families remain stationary white, stationary coloured, frozen
  spectral envelope and shuffled temporal envelope.
- Scores are normalized only from real development entries. The provisional
  threshold still maximizes grouped calibration balanced accuracy, with the
  lower threshold winning ties.
- The benchmark remains diagnostic-only and cannot emit registry `Pass` or
  promote a formula family. Unknown or uncovered cases retain authored clip
  fallback.

The recordings and generated mutation WAVs remain external under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/`; no corpus audio or
model artifact was added to the repository.

## Implementation

Evaluator profile
`nextengine.experimental-physical-sound-corpus-benchmark.ps-1-specialists.v3`
adds two current-only built-in profiles:

1. `amplitude-envelope-ps-1-v1` — 11 bounded coordinates from frame log-RMS
   slope and curvature, positive-slope/turning behavior, early/middle/late
   energy, early-to-late energy and energy/spectral-change coupling.
2. `temporal-amplitude-consensus-ps-1-v1` — the concatenated temporal and
   amplitude coordinates evaluated by the unchanged standardized-distance and
   threshold-selection rule.

Frame RMS is collected inside the existing 1,024-sample, 256-hop STFT pass.
The specialist therefore adds linear scalar work and bounded frame storage,
not a second FFT pass. The report schema is bumped to current-only V4 and now
publishes the feature-set identity, specialist identity, provisional outcome
and explicit `REAL_COVERAGE_REJECT` or
`CONTROLLED_MUTATION_FALSE_PASS` failure tags.

## Result

| Profile | Calibration coverage / false-pass groups | Holdout coverage / false-pass groups | Shadow coverage / false-pass groups |
| --- | --- | --- | --- |
| Frozen temporal AV-P0C | `1/3`, `0/3` | `1/3`, `1/3` | `0/3`, `1/3` |
| Amplitude specialist alone | `2/3`, `1/3` | `1/3`, `1/3` | `2/3`, `1/3` |
| Temporal + amplitude consensus | `2/3`, `0/3` | `1/3`, `0/3` | `2/3`, `0/3` |

The consensus provisional threshold is `0.9305864784564901`. Every stationary
white, stationary coloured, frozen-spectrum and shuffled-envelope mutation is
rejected in calibration, holdout and shadow.

The two frozen target counterexamples now have positive rejection margin:

| Entry | Partition | Score | Margin above threshold |
| --- | --- | ---: | ---: |
| `holdout-wood-b4-recorded-shuffled-envelope` | holdout | `1.3964302905470751` | `0.4658438120905850` |
| `shadow-wood-b5-recorded-shuffled-envelope` | shadow | `2.0111879848906530` | `1.0806015064341630` |

The closest correctly rejected controlled counterexample is
`shadow-metal-m10-recorded-stationary-colored` at `1.035801350086878`, only
`0.105214871630388` above the threshold. That margin is the first regression
sentinel for later specialist changes.

Four real entries remain outside provisional coverage:

- calibration metal M8, score `1.690793904684979`;
- holdout glass V5, score `1.1116544979221399`;
- holdout metal M9, score `3.1466402451243036`;
- shadow metal M10, score `1.035871148872412`.

Observed grouped false-pass risk is zero in each measured partition, but there
are still only three reject parent groups per partition. The 95% Wilson upper
bound is therefore `0.5614970317550455`, far too weak for automatic admission.
This is a blind-spot closure result, not a quality or safety bound.

## Reproducibility

Two complete external reports are byte-identical:

- primary:
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps1-amplitude-envelope-v1-final/report.json`;
- repeat:
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps1-amplitude-envelope-v1-final-repeat/report.json`;
- report SHA-256:
  `f8e25750e1c3ebb85c808ef3b3a818729b9910e2e28818b3b3f27ef975247785`;
- evaluator-profile SHA-256:
  `953578f2974103b01968c1008887c93ee28a96b39e8d81ef70ce43dd6787e20f`.

Verification passed:

- `cargo test -p xtask physical_sound --no-fail-fast` — 40 focused tests;
- `cargo clippy -p xtask --all-targets -- -D warnings`;
- `cargo fmt --all -- --check`;
- `cargo run -q -p xtask -- boundary-scan`;
- two full benchmark executions plus byte comparison.

## Decision and next action

PS-1 exit is met. The amplitude specialist closes the known shuffled-envelope
blind spot, the consensus retains all earlier controlled rejects and improves
calibration/shadow real coverage relative to temporal-only AV-P0C.

`Pass` remains disabled. PS-2 must broaden independent real object/family
evidence, replace material-only `unspecified` axes with exact acquisition
metadata where available, and pre-register numeric false-pass/coverage policy
before opening a new shadow or beginning AV-P0D source search.
