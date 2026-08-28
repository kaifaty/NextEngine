# PS-2 internet-source feasibility and transfer route — 2026-08-28

## Outcome

The bounded review is executable and returns
`ReviewedSourcesCannotCloseV1 / InternetNativeTransferCandidate`. None of the
three reviewed published surfaces closes any of the eight blockers in the
frozen `thin-soda-lime-glass-open-vessel-impact-v1` domain. The existing
domain therefore remains `FallbackOutOfDomain`.

The review does identify one useful next calibration substrate:
`realimpact-normalized-transfer-calibration-v1`. It may calibrate relative
modal frequency, damping and position/listener-dependent participation from
REALIMPACT's force-deconvolved responses. It may not claim absolute amplitude,
exact material composition, exact support, matched cross-tier conditions,
corpus admission, `Pass` or a runtime content role.

## Question and hypotheses

Question: can currently published internet evidence close the eight exact V1
claims without moving a frozen E3 object or requesting local capture?

- **H1 — exact V1 route:** published evidence contains the planned
  soda-lime composition, measured-caliper/mass geometry, foam-annulus support,
  absolute hammer excitation, rim/wall impact, half-metre listener and matched
  repeat lineage. **Falsified within the reviewed source set.**
- **H2 — narrower internet-native route:** published evidence supports a
  useful but explicitly narrower normalized-transfer calibration domain.
  **Supported for REALIMPACT.**
- **H3 — method release fills the lineage:** AV-MSF publishes reusable code or
  independent acquisition data that can close missing claims. **Falsified at
  the frozen review date:** its repository contains only `Code coming soon`.

This is a bounded current-source result, not proof that no future publication
can close V1. The report records exact reconsideration conditions.

## Primary-source findings

### REALIMPACT

The [primary paper](https://ai.stanford.edu/~rhgao/publications/RealImpact.pdf)
documents scanned real objects on a polyester thread mesh, five impact vertices,
a 15-microphone gantry, four listener distances (`0.23`, `0.56`, `0.90`,
`1.23 m`), measured hammer force and force deconvolution. Its material label is
the broad family `glass`, not a composition revision.

The [official repository](https://github.com/samuel-clarke/RealImpact) exposes
preprocessed data and says the raw dataset is still being packaged. The
[raw-data release request](https://github.com/samuel-clarke/RealImpact/issues/3)
remains open. Consequently the published surface supports relative
force-deconvolved transfer, mesh/impact/listener identity and real-object
identity, but not the frozen V1 absolute force-to-amplitude claim or its
support/listener axes.

Disposition: `SelectedTransferCalibration`, zero V1 blockers closed.

### ObjectFolder Real

The [official download page](https://objectfolder.stanford.edu/objectfolder-real-download)
describes 100 real household objects, three meshes per object and 30–50
six-second impact recordings with mesh strike coordinates, contact-force
profiles and video. The [primary benchmark paper](https://ai.stanford.edu/~rhgao/publications/ObjectFolder_CVPR2023.pdf)
states that support varies by object: most use a thin-string platform, light
objects are hung and heavy objects rest on anechoic foam. It does not provide a
reviewed per-object support revision matching the V1 foam annulus, and the
reviewed metadata does not establish the planned half-metre axis/radial pair.

The public force profiles make this a possible later force-calibration source,
but its large batch audio archives and missing exact domain lineage make it a
poor first discriminator.

Disposition: `DeferredForceCalibration`, zero V1 blockers closed.

### AV-MSF

The [project page](https://zisenshao.github.io/AV-MSF/) describes a promising
few-shot modal sound field trained on ObjectFolder Real and REALIMPACT, with
global modal frequencies/damping and a spatial gain field. The official
[repository](https://github.com/ZisenShao/AV-MSF) currently says `Code coming
soon`. The method therefore adds neither executable calibration code nor an
independent acquisition lineage at this checkpoint.

Disposition: `UnavailableCodeOnly`, zero V1 blockers closed.

## Executable contract

`physical-sound-registry source-feasibility` accepts only the frozen three
source profiles and their required primary artifact roles/URLs. It verifies:

1. the exact incomplete domain-claims report and its eight blockers;
2. external-only primary-source snapshots and declared SHA-256 hashes;
3. the complete source set with no duplicate or substituted profile;
4. hard-coded claim-scoped capabilities and limitations;
5. zero V1 blocker closures and a transfer-only next route.

Free-form source claims in the manifest cannot grant capability. Adding a new
source profile or changing a reviewed capability requires a repository code
change and a new report revision.

Command:

```text
cargo run -p xtask -- physical-sound-registry source-feasibility \
  --manifest /home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-source-feasibility-v1/manifest.json \
  --output /home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-source-feasibility-v1/report-a
```

External evidence root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-source-feasibility-v1/
```

Frozen hashes:

| Artifact | SHA-256 |
|---|---|
| Manifest | `52576e5d116393b9a7129efa7efcc222a904f90454ab2a287dc36fc4d8c878ed` |
| Report A | `9a591673fb9567c9a343aeca991e62f1cb299f3a1f6f31e15c38d6050d85b9b1` |
| Report B | `9a591673fb9567c9a343aeca991e62f1cb299f3a1f6f31e15c38d6050d85b9b1` |

The report hash-links the prior exact-domain report
`e6d078bd792ab45f09045bd272df3b39fafa0cb02c3c4e0fddb064aab95f5b60`.
All source snapshots, papers and reports stay external.

## Decision and next action

Preserve the acquisition-shaped V1 profile as fallback-only. Do not invent
metadata and do not widen its domain.

The smallest next implementation package is to preregister
`realimpact-normalized-transfer-calibration-v1` against the existing five
typed REALIMPACT E2 rows, then fit and evaluate relative modal frequency,
damping and spatial participation on object-disjoint development/calibration/
holdout groups. Absolute amplitude, material naturalness and domain admission
remain outside that calibration claim. ObjectFolder Real force calibration is
reconsidered only if the normalized-transfer experiment shows that missing
absolute excitation is the decisive error.
