# Physical sound AV-P0C controlled-mutation checkpoint, 2026-08-27

## Status

`CONTROLLED_MUTATIONS_MEASURED / SELECTIVE_PASS_DISABLED`

This checkpoint asks whether the deterministic AV-P0C temporal descriptor can
separate grouped real rigid-impact recordings from controlled temporal failure
modes without using holdout or shadow data for threshold selection. It is
external research evidence only. It does not admit a formula family, replace an
authored clip or create P1 authority.

## Frozen implementation and artifacts

The repository tool now provides:

- `xtask physical-sound-mutations`, which copies only real corpus entries into
  a self-contained external pack and deterministically derives four
  `expected_validator_outcome: reject` mutation families;
- explicit distinction between controlled negative mutations and existing
  published resynthesis/tuning mutations whose quality expectation remains
  `unspecified`;
- a temporal selective-risk report normalized from real development entries
  only, with calibration-only threshold selection and grouped holdout/shadow
  measurement;
- conservative parent grouping: a parent object is a false pass if any of its
  expected-reject mutation families passes;
- a two-sided 95% Wilson upper bound reported for the observed grouped
  false-pass proportion. The bound is diagnostic and does not define a product
  policy.

The controlled pack contains 10 real development entries and three real parent
objects in each of calibration, holdout and shadow. Each non-development real
parent has these deterministic mutation families:

1. stationary white-noise tail under the original RMS envelope;
2. stationary coloured-noise tail under the original RMS envelope;
3. repeated/frozen spectral-envelope carrier under the original RMS envelope;
4. shuffled block-RMS temporal envelope with the source carrier retained.

Exact external evidence identities:

| Artifact | SHA-256 |
|---|---|
| Source AV-P0B/YCB manifest | `e238fc42de974315bd30e6729c80c71a3f9a6f13e0b1c8bbe975b67e4bade1db` |
| Controlled-mutation manifest | `98f5489aa40841f2cb0a67409d454cb40e983a957f36fa077a9d426375750281` |
| Mutation-pack report | `50c2fd16852573b888d2e3ead9a50b540af04fe4472aa314c0466a98f0b8ecb8` |
| Evaluator profile | `f9e7553045fa65de3217f21ef84c9c30f6f135be270e167ae68d14b9014c4cf5` |
| Benchmark report | `604411196c2bd90ed6d6561971fc5e7dcb005751fe2bdfc2bc0b9bf41dca4291` |

Recordings, derived WAVs and reports remain outside the repository under the
existing non-distribution research boundary.

## Method

The quality score is the RMS standardized Euclidean distance from an entry's
10-dimensional temporal descriptor to the nearest real development anchor of
the declared material. Per-dimension mean and population standard deviation
come only from the real development gallery; the standard-deviation floor is
`1e-9`.

Calibration considers real entries as expected accepts and controlled
mutations as expected rejects. Mutations are grouped by parent entry using the
minimum child score, because acceptance of any child is a group false pass. The
provisional diagnostic threshold maximizes grouped balanced accuracy on
calibration, with the lower threshold winning ties. Holdout and shadow never
select the threshold.

## Results

The selected calibration threshold is `0.40490598982279524`.

| Partition | Real coverage | Grouped false pass | Observed risk | 95% Wilson upper |
|---|---:|---:|---:|---:|
| calibration | `1/3` | `0/3` | `0.000` | `0.5615` |
| holdout | `1/3` | `1/3` | `0.3333` | `0.7923` |
| shadow | `0/3` | `1/3` | `0.3333` | `0.7923` |

All stationary-white, stationary-coloured and frozen-spectrum entries reject
at this threshold in the exact measured pack. The two group false passes are
the shuffled-envelope mutations of holdout wood-B4 (`0.2829292805`) and shadow
wood-B5 (`0.3016100644`). Both are closer to the development temporal manifold
than their real parent scores. This is a direct counterexample to treating the
current spectral-evolution descriptor as a complete temporal-quality head.

The same descriptor is also the wrong replacement for the classical material
head: on the broader steel-v4/YCB manifest, temporal leave-family-out material
accuracy is `5/10`, versus `8/10` for the frozen classical AV-P0B profile.
Temporal dynamics therefore remains a specialist signal rather than a material
classifier.

## Conclusion and next discriminator

AV-P0C now has a reproducible mutation and grouped-risk substrate, but it has no
admission authority. The exact evidence rejects automatic `Pass` for two
independent reasons:

- current coverage rejects most real objects;
- three parent objects per partition cannot bound false-pass risk, and the
  amplitude-envelope mutation already produces holdout and shadow false
  passes.

The current material-identity AV-P0B/YCB manifests also cannot populate honest
SPEC-45 acoustic-domain rows: they do not identify exact geometry, support,
excitation range or listener/radiation conditions. Registry V1 remains
intentionally unpopulated for those sources instead of encoding invented domain
precision; the controlled Q30 geometry corpus is the next eligible registry
seed after it gains matched real/validator evidence.

The smallest next validator-only experiment is to keep this mutation pack and
all splits frozen, then add an amplitude-envelope trajectory specialist:
frame log-RMS slope/curvature, monotonicity violations, early/mid/late energy
ratios and coupling between energy change and spectral change. It must first
reject the two frozen shuffled-envelope counterexamples without reducing the
existing stationary/frozen controls to a trivial signal gate. A later corpus
revision must add independent real object families before any numeric
risk/coverage policy can be pre-registered. Do not tune a new source model or
enable registry `Pass` during this validator release experiment.
