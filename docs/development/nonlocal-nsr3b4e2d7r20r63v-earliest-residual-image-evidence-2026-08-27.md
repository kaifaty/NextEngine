# NSR3-B4E2D7R20R63V earliest residual-image evidence

Status: `PASS / TWO_LANE_EARLIEST_IMAGE_CANDIDATE`.

Claim status: `PROVED` over the frozen finite two-lane state ladders `0..8`.
Evidence classes: `EXACT_CERTIFICATE`, `CORRESPONDENCE`, `NUMERICAL`.

## Reproducible result

Implementation commit: `4a08cca8`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-earliest-residual-image
```

Two fresh executions are byte-identical:

```text
stdout sha256   62e1c37617017c169bdd0ab05793c91f542cc0582eec610cf17fd763d1202e44
semantic sha256 905ce406943303ac51b5f33a1580ddc438ad8e6adb6d38ed2b8eb179954acad8
route           TWO_LANE_EARLIEST_IMAGE_CANDIDATE
controls root   93344133290a6f04f8a2b07ba7a221df36c420af268edea1646f86f78cd8c6dc
retained lane   22d3b114dcec5845183fa72982203242f8c608fdeb0b6c1f6cbbf37ccc75d8a0
exported lane   1f6a7041734089ed0a07bfc5fb6ecce51ea742376fa31b2f8efe1353520407d6
```

The exact R63U parent semantic, both R63S lane roots, state-8 certificate
roots, controls, snapshot correspondence, fixed work and workspace lifecycle
all close.

## Exact boundary

Both independent lanes first resolve all 102 signs at PCG state 2:

```text
state  retained unresolved/error       exported unresolved/error
0      66 / 4.1143780882e15            66 / 4.0865762286e15
1      66 / 1.6828894130e15            66 / 3.4664220929e15
2       0 / 4.4386063132e-4             0 / 1.0104478022e-3
```

Every later state `3..8` also resolves exactly
`24 positive / 78 negative / 0 unresolved`. State 2 is therefore the earliest
stable exact residual-image certificate for both frozen recurrences. No
expected iteration or sign vector entered the classifier.

The unchanged state-8 roots reproduce R63U exactly:

```text
retained  38bd22c1b779801d807b525acb8a50a4515b7afb16dc84cf0301619358987adf
exported  f1937129df5e6daf0c6b5c2ecd810c8dbf1d52688e7c701ce108f573e18fe29f
signs     89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094
```

## Independence and work

The replay completes all eight updates before the R63V observer consumes its
snapshots. Snapshot vectors are excluded from the old R63S serialization and
are independently matched to the already frozen per-state solution roots.
Certificate values, pass bits and first-pass state cannot alter recurrence,
factor work or termination.

The two candidate lanes retain 16 factor solves and 18 direct rectangular
products. The new observer performs 18 certificates: 1,836 solution
conversions, 187,272 exact residual products, 187,272 exact image products
and 1,836 sign comparisons. It performs zero certificate-driven updates and
zero early stops.

All R63B--R63U stdout hashes remain byte-identical. Build and `git diff
--check` pass. No timing was sampled.

## Meaning and ceiling

State 2 is now the mathematical correctness baseline for this one immutable
dimension-102 common-operator RHS. This does not select a runtime stop rule:
the proof still depends on dense exact dyadic residual/image evaluation, a
binary128 factor and one offline verified inverse.

The next gate must reproduce states `0..2` with a finite outward enclosure
whose production-facing checker consumes no exact dyadics. Sparse realization,
corpus generalization, timing, runtime/GPU authority and production promotion
remain later independent gates.
