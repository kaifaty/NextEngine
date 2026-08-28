# Ceramic Cup multi-output counterfactual preflight — PS-2 — 2026-08-28

## Outcome

The first real-data reuse after the synthetic multi-output control is
hash-closed before analysis. External manifest
`add0017b6350bcfd5a500d1c0e776bb3b56733c1e329001d72d0f8239b1625e1`
binds runner
`b5635c4297c00e67d83dfab094f859b3f7b76cf800b95786224b08f92116fb7a`,
the exact rejected Ceramic observation and causal diagnostic, decoded-block
identity, successful synthetic control, fixed rows, unchanged V2 thresholds
and one spatial-power candidate. Two clean preflights emit byte-identical
report
`7bf54ec89b837864a9d79bc16ecc9e1275995d945e29fd411759df2dfeb1e0e1`
with decision `CeramicCupMultiOutputCounterfactualFrozen`.

Preflight reads no real payload bytes. It validates the small parent reports,
checks that the external decoded file exists with the frozen byte length, and
defers the full decoded-block hash check until analysis. It performs no network
request, additional acquisition, physics run or Planter access.

## Fixed input

- object `78_CeramicCup`, opened-development role;
- existing impact ordinal `0` only;
- decoded block SHA-256 `3405843a…e6ca`, `501,348,000` bytes,
  shape `600 × 208895`, dtype `<f4`;
- rows `0..14`, all microphone IDs at angle/distance `0°/0 mm`;
- unchanged reference row `7`, whose frozen decaying-mode fraction is `0.25`;
  and
- no value-dependent row selection, filtering, denoising or threshold change.

The analysis must rehash the full decoded block before mapping these rows. It
then reads `12,533,700` existing payload bytes into the 15-output matrix. Those
bytes are existing opened-development evidence, not a new acquisition.

## Candidate and frozen decision

`spatial-modal-power-15-v1` is the exact source
`f0483c397843994d23793a4f3392dd49d9ce5d90ec218caa1b10f2a846a2068e`
that passed the synthetic control. It sums per-microphone FFT power before the
unchanged V2 peak selection and three-bin decay tracking. It fits no
cross-channel phase or spatial parameter.

Support requires all six conjunctive checks:

- selected modes at least `6`;
- persistent recall at least `0.50`;
- median repeated-tail frequency error at most `40 cents`;
- decaying-mode fraction at least `0.50`;
- median tail-prediction RMSE at most `24 dB`; and
- decaying-mode-fraction improvement over row 7 at least `0.25`.

Any failure publishes `CeramicCupMultiOutputCounterfactualRejected`. A complete
pass publishes `CeramicCupMultiOutputCounterfactualSupported`, but still grants
no admission, perceptual quality, mechanical causality or runtime credit.

## Next action

Commit and transfer this preflight checkpoint, then execute the immutable
counterfactual twice. Publish the outcome once. Do not select microphones,
weaken a gate, fetch data, run mechanics or access Planter after seeing it.

