# PS-2 REALIMPACT multi-object spatial-axis extension — 2026-08-28

## Decision

`FixedVerticalCandidateTwoObjectAxisStratificationRejected`.

The previously selected `vertical-rbf-sigma052-ridge001-v1` remains supported
for its original Green/Shell/Skull fixed-condition pilot, and it passes the new
Glass Goblet object rule. It fails the separately preregistered Blue Bowl rule.
It must therefore remain a conditional vertical-line profile rather than a
generic formula for arbitrary glass bodies.

This experiment stratifies the unchanged vertical interpolator over declared
angle/distance conditions. It does not fit or test interpolation between angles
or distances and does not establish a three-dimensional radiation field,
material identity, naturalness, admission, validator `Pass` or runtime use.

## Frozen lineage

The prerequisite vertical calibration report is
`abc13a9cbdcaac6118197267f55264a03088130c362758e1212960dbce57389d`;
the transfer-V2 report remains `01346767…c444`. The RBF candidate, nine anchor
microphones, six held microphones, microphone-7 normalization, DSP window and
all condition gates are unchanged.

Before any new development audio was opened, manifest
`c90194c809dd55afc7ab54e9fe94e09f6527f36ba5d21af081ebf03c484645fd`
froze six Green Goblet condition blocks:

- angle `0°`, distance offsets `0`, `333`, `666`, `1000 mm`;
- distance offset `0`, angles `20°` and `40°`.

Each condition contains the same 15-microphone vertical line. The selected
audio rows are `0..74` and `120..134`; row/condition identity is checked against
the exact published angle, distance, microphone, vertex and listener arrays.

Two Green development runs are byte-identical. All `6/6` condition blocks pass:
median of condition median errors is `3.8384 dB`, median condition ratio to the
microphone-7 constant baseline is `0.6856`, and worst condition p90 is
`13.2101 dB`. The development report is
`5fdaacd1a93c5af13d85ffe966889be482a544cf062306be0db58c70def3a1ed`;
the selected 90-row block is `c0f6e0cc…3121`.

## Evaluation rule frozen before holdout access

Evaluation manifest
`dbc958bd4aacec9ff5359fe434bfa6ab0fd680c8f0aa6048e98286aec7296912`
was written after the Green repeat and before Blue Bowl or Glass Goblet rows
`1..134` were opened. It declares prior row-0 exposure and fixes both objects as
equally required holdouts. No candidate selection or threshold change occurs
between objects.

An object passes only when:

- the base `0° / 0 mm` block passes every original condition gate;
- at least `4/6` condition blocks pass every original condition gate;
- median of condition median errors is at most `7 dB`;
- median condition ratio to the constant baseline is at most `0.9`;
- worst condition p90 is at most `24 dB`.

The study supports the fixed-candidate extension only if both objects pass.

## Holdout result

| Object | Base | Passed blocks | Median of medians | Median ratio | Worst p90 | Object result |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `6_Bowl` | fail | `3/6` | `4.3001 dB` | `0.8991` | `12.9808 dB` | reject |
| `94_GlassGoblet` | pass | `4/6` | `4.5303 dB` | `0.8526` | `14.6430 dB` | pass |

Blue Bowl's base block is the decisive counterexample. Its median error is not
large (`5.0251 dB`), but RBF improves only `0.4375` of components and its median
error ratio to constant is `0.9923`; the model adds almost no value over a
spatially constant response. The `1000 mm` block is worse than constant
(`1.0219`), while `333 mm` narrowly misses the frozen ratio gate (`0.9101`).

Glass Goblet passes the object rule. Its failures are `666 mm` on improved-mode
fraction (`0.4375`) and `40° / 0 mm`, where ratio is `1.3298` and improved-mode
fraction `0.375`. This independently demonstrates that angle/distance strata
cannot be treated as automatically interchangeable even on a vessel geometry.

The aggregate negative decision is not threshold-sensitive on Blue: both the
required base condition and required block count fail. Retuning either after
inspection would invalidate the holdout.

## Reproducibility and bounded access

Two complete evaluation runs are byte-identical:

- report: `77a1f9e9df0ca02a29912ce7c925fc7e090803b19b58c4c4fddbc08ec288356a`;
- Blue 128 MiB compressed prefix: `3b3ca43e…c6a8`;
- Blue selected 90-row block: `60f26294…6da6`;
- Glass 128 MiB compressed prefix: `66950a80…90f3`;
- Glass selected 90-row block: `52bea7e9…1e82`.

Blue transfers `134,903,456` validated range bytes (`5.6263%` of its archive);
Glass transfers `134,784,127` (`5.8279%`). The command validates archive HTTP
identity, EOCD, central directory, entry metadata, all small annotation/mesh
members, condition coordinates and each finite selected row. Raw prefixes,
blocks and reports remain external and are not repository content.

The package also adds a typed bounded `glass-goblet-row-0-v1` archive profile.
Its independent row-0 run reproduces the historical payload
`15c87b87…325b` and audition WAV `4574ad6f…a2b8` from a fixed 1 MiB prefix.

## Interpretation and next discriminator

The falsified hypothesis is that one shape-agnostic vertical RBF can serve as a
general participation formula for these glass/shell bodies. Preserve the
negative report and do not tune sigma, anchors or gates on Blue/Glass.

The next evidence-backed hypothesis is shape-conditioned spatial structure:
listener coordinate alone is insufficient, so the formula may need object
geometry/mode-shape descriptors and an explicit angle/distance kernel. That
hypothesis requires a new object-disjoint split from still-unopened REALIMPACT
objects before fitting. Blue and Glass remain immutable evaluation evidence,
not future development data.

The smallest next action is a bounded research/preregistration package that:

1. inventories new official objects without reading their audio rows;
2. freezes development/calibration/holdout identities and body-geometry
   descriptors;
3. compares a coordinate-only control against one shape-conditioned model;
4. retains per-object or clip fallback whenever the selective gate fails.

All eight exact-domain claims, calibrated domain/OOD/shadow false-pass risk,
material/perceptual quality, source-model search, contact projection,
whole-mixer cost and P1 integration remain open.

## Primary sources

- [REALIMPACT project page](https://samuelpclarke.com/realimpact/)
- [Frozen REALIMPACT repository](https://github.com/samuel-clarke/RealImpact)
- [REALIMPACT CVPR paper](https://jiajunwu.com/papers/realimpact_cvpr.pdf)
- [Blue Bowl archive](https://downloads.cs.stanford.edu/viscam/RealImpact/6_Bowl.zip)
- [Glass Goblet archive](https://downloads.cs.stanford.edu/viscam/RealImpact/94_GlassGoblet.zip)
