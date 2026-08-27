# Physical sound steel residual v4 — 2026-08-27

| Field | Result |
| --- | --- |
| Scope | External real-metal expansion, interpretable diagnostics and one frozen offline steel counterfactual |
| Status | `MEASURED / V4_REJECTED / FALLBACK_OUT_OF_DOMAIN` |
| Generator | `nextengine-steel-stochastic-residual.v4` |
| Automatic decision | Keep authored/current fallback; no demo, runtime, content or public-contract promotion |
| Claim limit | Material-identity diagnostics only; no calibrated risk, force/location fidelity or subjective-naturalness claim |

## Question and boundary

The v3 roughness winner passed BEATs for all three impact positions but failed
PANNs for all three. This cycle asks two bounded questions:

1. Which interpretable statistics separate v3 from more than one real-metal
   object family?
2. Does one pre-frozen decaying broadband residual close that measured gap
   without regressing the independent BEATs head?

This remains P0 evidence under Proposed SPEC-45. Raw recordings, derived WAVs,
feature matrices, model weights and reports stay in the external experiment
store. Current steel/wood/glass defaults, the demo profile, authored clip
fallback, public contracts and gameplay authority are unchanged.

## Real-metal expansion

The external benchmark adds four two-second windows from the public
[YCB-impact sounds dataset](https://osf.io/4tcp6/), associated with the
[IROS 2022 paper](https://www.iri.upc.edu/files/scidoc/2619-Recognizing-object-surface-material-from-impact-sounds-for-robot-manipulation.pdf).
The paper describes more than 3,000 impacts over 75 YCB objects and distinct
material labels. The bounded subset adds two object families absent from the
Kronland material corpus:

- one thin aluminium container at the two published robot-speed folders;
- one rigid steel skillet at the same two speed folders.

Each published five-second 48 kHz stereo IEEE-float RIFF/WAVE source was
downmixed and cropped to a fixed two-second PCM16 window beginning 50 ms before
the dominant isolated onset. Exact source IDs, URLs, hashes, offsets and derived
hashes are frozen in the external `corpus-record.json`.

The OSF node declares no license. Its status remains `NOASSERTION /
no_repository_or_distribution`; this scientific diagnostic use does not place
recordings or derived audio in Git or a distributed product. Measurement scope
is truthfully `material_identity_only`: impact location, listener pose and
calibrated force are not inferred from folder names.

With these four windows added to the real development gallery, BEATs classifies
all ten leave-object-out real recordings correctly. PANNs classifies all six
real-metal recordings correctly but maps the two real-wood recordings to glass;
the new gallery is therefore useful independent metal evidence, not a calibrated
general-material authority.

## Measured v3 gap

A hash-frozen deterministic diagnostic uses one common two-second post-onset
window, 2,048-sample Hann STFT and 256-sample hop. Values below are medians over
five Kronland metal recordings, four YCB windows and the three v3 impact
positions:

| Statistic | Kronland real metal | YCB real metal | v3 winner |
| --- | ---: | ---: | ---: |
| Aggregate spectral flatness | `−15.916 dB` | `−17.315 dB` | `−53.336 dB` |
| Active-bin fraction within 40 dB | `0.522` | `0.535` | `0.133` |
| Local peak density | `5.221/kHz` | `3.940/kHz` | `1.155/kHz` |
| Normalized spectral flux | `0.0483` | `0.1310` | `0.00641` |
| Adjacent-frame spectral cosine change | `0.0137` | `0.0688` | `0.00188` |
| Tail high/air energy fraction | `0.699` | `0.0465` | `0.000256` |

The cross-corpus agreement is the important result. The absolute tail balance
differs because the objects and acquisition paths differ, but both real sources
are much denser, flatter and less stationary than v3. Critical-band sidebands
made v3 rough enough for BEATs; they did not reproduce broadband, time-varying
metal structure seen by PANNs.

Primary prior art is consistent with this diagnosis. Aramaki et al. identify
rich/broad/dissonant spectra and roughness as important metal cues. Cornell's
[Harmonic Shells](https://www.cs.cornell.edu/projects/HarmonicShells/)
demonstrates that nonlinear thin-shell coupling adds richness beyond linear
modal models for trash cans, sheet metal and cymbals. The result does not prove
that nonlinear shells are the next Next Engine implementation; it falsifies
another damping-only explanation.

## Frozen v4 counterfactual

The search grid was fixed before any v4 feature extraction:

- one byte-exact v3 control: `f650 / d4000 / r350 / i900`;
- one deterministic decaying-white residual family;
- Q15 residual gain `{128, 256, 512, 1024}`;
- amplitude T20 `{300, 600, 1200} ms`;
- center, edge and corner with the same per-position seeds as v3.

This yields 13 profiles and 39 WAVs. The residual uses a bounded xorshift stream
and a Q30 per-sample decay coefficient inside the heap-backed offline search
voice. Zero gain disables the branch exactly, so the control remains byte-equal
to v3. Repeated generation is byte-identical, all outputs are non-silent and
unclipped, and the accepted wood/Q30 controls remain outside this code path.
The revised command also regenerates the complete v3 directory byte-for-byte,
including report SHA-256 `c689d7b2…6e0db6f0d`.

The counterfactual tests one proposition only: whether a weak stochastic tail is
the missing structure. It is not an authored residual, learned model, nonlinear
shell solver or runtime proposal.

## Automatic results

All candidates stay in `calibration`; real `holdout` and `shadow` entries never
select a profile. The original Kronland-only development gallery gives:

| Head | Metal renders | Best relevant result |
| --- | ---: | --- |
| Classical AV-P0B | `0/39` | all map to wood |
| BEATs | `7/39` | v3 control remains `3/3`; no residual profile reaches `3/3` |
| PANNs CNN14 | `0/39` | `g1024/t1200` improves worst margin from v3 `−0.2274` to `−0.0555`, but maps to wood/glass |

Against the expanded Kronland+YCB real gallery, classical reaches `16/39`,
BEATs remains `7/39`, and PANNs reaches `4/39`. The PANNs-best
`g1024/t1200` is only `2/3` with worst margin `−0.00725`. No profile passes all
positions under both learned heads. Adding development anchors changes nearest
neighbors but does not repair head agreement.

The strongest residual also exposes why a scalar flatness target is unsafe:

| Statistic | Kronland real | YCB real | `g1024/t1200` |
| --- | ---: | ---: | ---: |
| Spectral flatness | `−15.916 dB` | `−17.315 dB` | `−14.737 dB` |
| Active-bin fraction within 40 dB | `0.522` | `0.535` | `1.000` |
| Spectral flux | `0.0483` | `0.1310` | `0.0157` |
| Tail flatness | `−35.146 dB` | `−21.507 dB` | `−0.706 dB` |

It reaches the aggregate flatness range by filling every bin with an almost
white, overly stationary tail. PANNs moves toward metal but still abstains;
BEATs moves from metal toward glass. Averaging those errors would reward metric
gaming rather than improve material identity.

## Decision

The v4 result is `FallbackOutOfDomain` and the residual family is rejected as a
steel profile:

- admit no v4 candidate to the demo, runtime or content;
- keep the current steel profile and authored clip fallback unchanged;
- retain v3 roughness and v4 residual as causal negative evidence;
- add `stationary-white-residual` to the validator's mutation/counterexample
  families rather than continuing its gain/T20 search;
- do not start a v5 coefficient grid from these same heads.

The next smallest evidence-backed work is validator-first: add grouped metal
families and a temporal-spectral dynamics specialist/mutation ladder that can
separate real evolving modal density from white-noise occupancy. Only after that
head generalizes across object/source groups should one new source-model
counterfactual be considered. Plausible future candidates are a dense
frequency-dependent residual fitted from recordings or bounded nonlinear mode
coupling; neither is selected here.

## Frozen external evidence

Experiment root:
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/av-p0b-kronland-material-v1`

| Artifact | SHA-256 |
| --- | --- |
| YCB corpus record | `814217a61db443cf98b964d169326e5355701bcbced68f7789556bd27c28a399` |
| v3 + YCB diagnostic report | `eaf8bd243bb0b5163e929edd74cb3a1c1e3495cb4637ccfd346e75dc921d239b` |
| v4 generation report | `8a1df2c44f13cab32c7cffa24a3d9c18b22bc2bc8ec82e34ba53e9fc8a3d1687` |
| v4 original ensemble manifest | `4015a00bc2daf1c25a4529d490045420ae60a784f46e5965526aea6aae890b3f` |
| v4 BEATs matrix | `1b91ab71deac850240f52b6e615da385e20fe94e0d727e05977bf843e45e55f4` |
| v4 PANNs matrix | `239b5dcc05c5db854a946d152645ca5c04ad728b0bc844c0a85dc5057e41b9fe` |
| v4 original ensemble report | `4f1f1d7021f8c42281c3f3d4821bc5120c7f39fd407b1d6a0041514c76d643a1` |
| v4 + YCB ensemble manifest | `02680609de2b252b8c555bde2cdbe828abae1c256ea459f4143f5c3d272bb67d` |
| v4 + YCB BEATs matrix | `8868502b637771fb4af1cf63762f51f57e3aede15800038e335ea62fc6f50c45` |
| v4 + YCB PANNs matrix | `bf88165345020efefcfc6f77f8bad17edbde3666f9185f5ac196aa220898c9a6` |
| v4 + YCB ensemble report | `c592dd68e3caef176561594a725c6655f63f4edac1b25397aa2cdf7c725b9ab7` |
| v4 + YCB diagnostic report | `32734ad57ea575bcc1996cc41494e3cfa57417ff1241db4436d2628cc5d5e87b` |

External extraction uses PyTorch `2.8.0+cpu`, the existing frozen BEATs and
PANNs checkpoints, `panns-inference 0.1.1`, FFmpeg `8.0.1-3ubuntu2` and the
hash-frozen diagnostic script `3bc702e6…cc3ed49e`. No external artifact enters
the repository.
