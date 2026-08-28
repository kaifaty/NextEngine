# Ceramic Cup adaptive-decay counterfactual preflight — PS-2 — 2026-08-28

## Outcome

The adaptive estimator's first real-data counterfactual is hash-closed before
payload analysis. Manifest
`48001fb7e9d6d799b12141f3b755f954bca501e7729b78f42a561330a00fff9a`
binds runner
`e50bec233b037b8ed14f62a96faeca3fd8adda946f63222d41428e604ca3056b`,
the successful synthetic control, rejected fixed-window Ceramic result, exact
decoded identity, fixed rows and seven decision checks. Two preflights emit
byte-identical report
`4358d4da602e5d4cf430f6c6db708a0c0dddc14cf37b87765937ee54137e220f`
with decision `CeramicCupAdaptiveDecayCounterfactualFrozen`.

Preflight reads zero real-payload bytes. It validates parent reports and file
length, while full block rehash is deferred to each analysis. Network,
additional payload, physics and Planter counts are zero.

## Fixed input and candidate

- existing `78_CeramicCup`, impact ordinal `0`;
- decoded block `3405843a…e6ca`, `600 × 208895` `<f4` samples;
- unchanged rows `0..14` at angle/distance `0°/0 mm`;
- unchanged spatial frequency selection;
- adaptive algorithm and constants from synthetic runner `fe59b3d6…9c20`;
- rejected fixed-window fraction `0.1875` as comparator; and
- no row selection, cutoff, filtering change, denoising or threshold tuning.

The runner independently implements the same bounded algorithm for the shorter
real record: `20 Hz` causal bandpass per output, summed spatial power,
`eta=0.01 s` RMS envelope, per-mode peak, final-10% noise floor and the frozen
`90%..10%` dynamic-range interval.

## Frozen support checks

- selected modes at least `6`;
- persistent recall at least `0.50`;
- median repeated-tail frequency error at most `40 cents`;
- valid adaptive fits for at least `0.75` of modes;
- decaying-mode fraction at least `0.50`;
- median fit `R²` at least `0.95`; and
- improvement over fixed-window decay fraction at least `0.25`.

Any failure rejects the counterfactual. A full pass is still opened-development
method evidence, not admission: an independent unopened object must validate
the method before mechanics can be considered.

## Next action

Commit and transfer this checkpoint, then analyze twice. Each run must rehash
the full decoded block before mapping the fixed rows. Publish the outcome once;
do not tune, fetch, run mechanics or access Planter.

