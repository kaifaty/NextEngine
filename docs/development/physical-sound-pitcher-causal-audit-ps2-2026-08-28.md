# REALIMPACT Pitcher causal audit — PS-2 — 2026-08-28

## Outcome

The immutable Pitcher report `8bd5323c…1aea` still rejects the complete
`extractor → scalar surface proxy → Bempp → cooker` protocol. A new read-only
audit, however, proves that the experiment did not admit its observation before
using it to judge mechanics. Runs A/B repeat byte-identically at report
SHA-256 `68c79a37896b776b9e569df31ae868ffdc508a069475e72d4ababd49c7fc5a57`
with decision
`PitcherPhysicalRejectionCausallyConfoundedByObservationAdmissionFailure`.

The audit reuses the already-frozen REALIMPACT transfer V2 holdout thresholds;
it selects no threshold from Pitcher. Four checks pass, but
`decaying_mode_fraction = 0.4375` fails the existing minimum `0.50`. The
original physical protocol nevertheless reports numeric mode coverage as
passing and consumes the modes in its frequency and field comparisons. Eleven
of its sixteen selected peaks (`68.75%`) are also below `500 Hz`, the band in
which the REALIMPACT authors report the less-anechoic room response.

The defensible claim is therefore narrower than the preceding causal wording:

- the combined frozen Pitcher protocol is rejected;
- Bempp convergence is not the observed numerical failure;
- the result does **not** uniquely reject scalar mechanics, elastic shells,
  volumetric FEM or physical sound synthesis, because its target observation
  failed an existing upstream health gate;
- Pitcher remains opened development evidence and cannot be retuned;
- Planter remains sealed and authored clips remain the mandatory fallback.

## Exact audit

| Artifact or observation | Exact result |
| --- | --- |
| Frozen Pitcher parent | `8bd5323cabdb4319f8465631c3c7a54438669ee10afa9cac1f6bb122537b1aea` |
| Audit script | `lab/scripts/physical_sound_pitcher_observation_audit.py`, `8c5c0f259a781b4731ac3a332b386d244de19c65b564906178537dcf7520252e` |
| Bound V2 gate source | `transfer_calibration.rs`, `2624656e…bf80` |
| Bound V2 DSP source | `transfer_calibration/dsp.rs`, `131bbf42…9ca4` |
| Audit A/B report | `68c79a37896b776b9e569df31ae868ffdc508a069475e72d4ababd49c7fc5a57` |
| Selected / persistent | `16 / 12`, checks pass |
| Median tail match error | `21.4241 cents`, check passes `<= 40` |
| Median tail prediction RMSE | `7.7091 dB`, check passes `<= 24` |
| Decaying fraction | `0.4375`, check fails `>= 0.50` |
| Selected frequencies below `500 Hz` | `11/16 = 0.6875` |
| New network/audio/Planter bytes | `0 / 0 / 0` |

The two bound Rust sources are checked by exact hash before publication. The
audit reads only the frozen JSON report; it cannot access the decoded Pitcher
block, geometry, network or Planter payload.

## Bounded research cycle

### H-O — observation/extractor mismatch

**For.** The existing V2 health gate rejects the Pitcher observation, yet the
physical comparison omitted that gate. The [REALIMPACT paper](https://arxiv.org/html/2306.09944v1)
reports RT60 below `0.2 s` above `500 Hz` but longer reverberation below it,
while eleven selected Pitcher peaks are below `500 Hz`. The paper also warns
that generic denoising can shorten real modal tails, so post-hoc denoising is
not a safe repair. The official
[preprocessing code](https://github.com/samuel-clarke/RealImpact/blob/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/preprocess_measurements.py)
deconvolves the hammer recording but does not remove room modes.

**Against.** Twelve peaks have injective tail matches, and four of the five V2
health checks pass. Contamination is established as a serious confound, not as
proof that every selected peak is false.

### H-M — missing thickness/interior and vector elastic mechanics

**For.** The candidate solves a scalar cotangent Laplace-Beltrami problem and
fits one global `frequency = alpha × eigenvalue` scale. It has no vector
displacement, elastic stiffness/mass tensors, wall thickness, curvature-driven
membrane/bending coupling or support condition. The REALIMPACT authors
explicitly list hollow/thin representation, stiffness/damping, element type,
mesh resolution and contact damping among their baseline discrepancies.
[DiffSound](https://hellojxt.github.io/DiffSound/) instead uses high-order
volumetric FEM, and its official
[implementation](https://github.com/TechnetiumMan/DiffSound) includes a
separate hollow-mesh thickness inference experiment. The primary
[Harmonic Shells](https://graphics.stanford.edu/~djames/publication/harmonic-shells-a-practical-nonlinear-sound-model-for-near-rigid-thin-shells/)
work models thin-shell forces and modal coupling rather than a scalar surface
spectrum.

**Against.** Pitcher cannot discriminate this hypothesis until the observation
is admitted. A more expensive shell or volumetric solve against the same
failed target would repeat the causal mistake.

### H-E — impact/support coupling

**For.** Contact damping and finite support can change decay, excitation and
some resonances. REALIMPACT itself lists missing contact damping as a source of
envelope error.

**Against.** The dataset rests objects on a polyester thread mesh specifically
to approximate free vibration. Its force transducer is synchronized with the
microphones, and the published signal is force-deconvolved. This makes an
unmodelled raw hammer profile a poor explanation for the frequency-target
failure, though support remains an exact-domain limitation.

### H-D — listener coordinates or transfer semantics are wrong

**For.** A coordinate-frame or normalization mistake could create very large
spatial errors.

**Against.** The official preprocessing code constructs the same rotated
Cartesian listener positions used by the repository and explicitly publishes
`listenerXYZ.npy`, `vertexXYZ.npy` and force-deconvolved responses. The paper
defines acoustic transfer as the spatial pressure magnitude given vibrational
boundary conditions. This hypothesis is currently disfavoured, not deleted;
a fresh metadata parity check remains mandatory.

### H-A — acoustic transfer representation is insufficient

**For.** REALIMPACT reports that explicit transfer models outperform a
structural-only baseline. Thin geometries can also require more careful
boundary-integral formulations.

**Against.** The Pitcher Bempp solves converge to residuals below `1e-8`, and
the repository's synthetic elastic-FEM/Bempp and full-angular cooker controls
pass. Acoustic transfer cannot repair an upstream set of wrong target
frequencies or wrong structural mode shapes. It remains a downstream
hypothesis only after H-O and H-M are separated.

## Decision and implementation order

Observation mismatch is the cheapest prerequisite and now has direct local
evidence, so it precedes a new mechanics solver. The next development object is
official REALIMPACT `78_CeramicCup`, listed in the frozen upstream
[object roster](https://github.com/samuel-clarke/RealImpact/blob/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/dataset/object_names.txt).
Repository and external experiment inventories contain no prior reference to
that object at this checkpoint.

The next protocol must be frozen before its deconvolved payload is opened:

1. discover and hash only its bounded archive identity, central directory and
   non-audio metadata/geometry;
2. bind one impact's 600-row block, decoder and zero-retry range budget;
3. run the unchanged V2 observation extractor and its complete five-check
   admission gate before any frequency mapping, FEM, Bempp or cooker work;
4. stop with clip fallback if observation admission fails;
5. only on admission, freeze a separate mechanics discriminator comparing the
   retired scalar proxy with dimensionally correct vector elastic shell or
   hollow-volume FEM; do not change the observation extractor in that cycle;
6. use Planter only as a later one-shot holdout after a development candidate
   is frozen and supported.

No automatic `Pass`, exact-domain admission, PS-3, AV-P0D, runtime role or P1
credit is created by this audit.
