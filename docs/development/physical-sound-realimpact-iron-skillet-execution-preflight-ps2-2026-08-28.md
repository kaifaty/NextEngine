# REALIMPACT Iron Skillet observation execution preflight — PS-2 — 2026-08-28

## Outcome

The complete four-stage Iron Skillet runner is hash-closed before observation
payload access. Manifest
`2c98971ace0861475209e103b5bcb9e873c1c4a3dd59085fea329e85d4b3d8ee`
binds runner
`0c53553a5288fabd6beb14edc18cb13c1a5494bf5da07ef3ace1049da8be0988`,
the discovery/protocol lineage, exact four ranges, decode shape and row count,
source-derived selector, variable-length adaptive estimator, diagnostic top-16
comparator, dependency versions, gates and immutable stop rule. Two preflights
emit byte-identical report
`b9927f175fad4cb3b598dde587bd5994fa1b759c63e9465584d124484853672b`
with decision `IndependentObservationExecutionPreflightSupported`.

The preflights make zero network requests and read zero observation or Planter
payload bytes. They grant no material, mechanics, quality, admission, runtime
or `Pass` credit.

## Frozen stages

1. `preflight` validates every source and parent hash without payload access;
2. `acquire` performs exactly three 1360-byte condition-metadata ranges and one
   512 MiB observation-prefix range, once;
3. `decode` validates local ZIP records/CRC32, NPY shape and impact-zero row
   identity, then emits exactly 600 rows; and
4. `analyze` reads rows `0..14` as 15 synchronized microphones, detects onset,
   applies the unchanged salience/persistence selector at three scalar gains,
   fits variable-length adaptive envelopes and publishes all eight gates.

The fixed top-16 path is reported only as a diagnostic comparator. It cannot
affect the admission decision.

## Implementation controls

- all outputs and payload artifacts must remain outside the repository;
- range responses require exact `206`, URL, length, ETag, Last-Modified and
  `Content-Range` identities;
- download streams are hashed and bounded without buffering the 512 MiB prefix;
- decode stops after exactly 553317600 row bytes and validates the 128-byte NPY
  header for shape `(3000, 230549)` and dtype `<f4`;
- impact-zero metadata must contain one vertex, 10 angles, four distances, 40
  condition pairs and microphone order `0..14` per condition;
- adaptive filtering derives working length from the decoded recording rather
  than the 384000-sample synthetic fixture; and
- acquisition failure publishes an immutable rejection with no retry or prefix
  growth.

A local 230549-sample truncation of the frozen synthetic control exercises that
variable-length path without real payload: onset remains 240, all 16 truth
modes are selected, adaptive valid/decaying fractions are `1.0/1.0`, median
`R²` is `0.952464` and scale-bin invariance passes.

## Frozen decision boundary

The candidate must pass all protocol gates already published in
[the discovery/protocol evidence](physical-sound-realimpact-iron-skillet-discovery-and-observation-protocol-ps2-2026-08-28.md).
A pass is one independent-object method-transfer result only. It cannot establish
metal identity, physical parameter calibration, perceptual quality, exact-domain
admission or mechanics validity.

## Next action

Status is
`IRON_SKILLET_EXECUTION_PREFLIGHT_SUPPORTED / FOUR_REQUEST_ACQUISITION_AUTHORIZED /
OBSERVATION_NOT_OPENED / NO_RETRY_OR_PREFIX_GROWTH / MECHANICS_BLOCKED /
PLANTER_SEALED`.

Commit and transfer this checkpoint, then execute the four-request acquisition
once. On any failure stop. On success decode once and run two byte-identical
offline analyses; do not tune or substitute another object after seeing data.
