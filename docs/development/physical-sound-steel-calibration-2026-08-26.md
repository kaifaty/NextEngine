# Physical sound steel calibration — 2026-08-26

| Field | Value |
| --- | --- |
| Status | `BOUNDED_REFERENCE_SCREEN_PASS / FITTED_CANDIDATE_D / HUMAN_AUDITION_OPEN` |
| Question | Can the failed heuristic steel impact be moved toward an ordinary plate-like metal hit without another blind coefficient-tuning cycle? |
| Result | Yes as an external P0 candidate: a 12-mode, short-decay steel profile halves median spectral error against a small CC0 metal-impact screen and removes the multi-second ringing failure; modal agreement remains mixed, so human audition is still required |
| Scope | Isolated PresentationOnly laboratory and feature-gated demo; no public schema, controlled RealImpact claim, P1 promotion or shipping default |

## Reference screen and provenance

The bounded screen used the seven WAV files from BMacZero's
[Metal Impact Sounds](https://opengameart.org/content/metal-impact-sounds),
published under CC0. The source page describes them as a small variety of
metallic impact sounds and tags the pack with metal, hammer, impact, hit,
clank and clang. Files stayed under
`/tmp/nextengine-steel-calibration-20260826`; no audio entered the repository.

| File | SHA-256 |
| --- | --- |
| `bing1.wav` | `d14941ce27e6b409e0cd941eaad7201d613070a4c3eb9c3627fec4cf3ae686e4` |
| `bong1.wav` | `a6558d186f2843aefa212d567963b0ad79a6f78614bcf650f519620fc1549a6d` |
| `clink1.wav` | `0d439cd2b1f36f501b94c39661e39f91d160affe25c79f77dff7a2cd0ed7eadd` |
| `clink2.wav` | `ecbf5a221f2aff32376b8584eea875d0b7d9963d22f6c16e237f5e7b63f51dc2` |
| `clink3.wav` | `dd07516e51e9092efc36d13668de7d29d096955a437894cedeb5064933e2650f` |
| `thud2.wav` | `5fb5c1b37a95cb6716d0bb529a0b9fe1ae9f3e2228b8f30c5c3dcbb1b570a3af` |
| `thud3.wav` | `3aa636679ffdc9602d27655ca59a7c7342ae137b68e54c61aad0fb3c414506ca` |

All seven inputs are 44.1 kHz mono PCM S16. Their filenames do not provide
controlled object geometry, strike position, force trace, microphone pose or
take identity. They are therefore a broad timbral screen and human-audition
anchor, not a calibrated matched corpus or relational-physics dataset.

The RealImpact repository was also checked. It exposes a controlled metal
subset in principle, but the single `67_IronPlate.zip` object archive reports
`2,310,122,028` bytes and the repository's MIT license is stated for software;
the exact recording redistribution terms still require review. That archive
was not downloaded for this bounded cycle.

## Falsifiable hypothesis

The failed steel preset was dominated by too few low/narrow resonances and
excessively slow damping, not by missing signal plumbing. Its centre impact
had spectral centroid `440.12 Hz`, flatness `-68.28 dB`, a two-second window
and a broadband T20 estimate of `3,210 ms`; high-band decay was censored far
beyond the capture. The reference screen spans approximately `1.06–3.46 kHz`
spectral centroid and mostly `55–340 ms` per-band T20, with one intentionally
longer low `bong` outlier.

The smallest repair was therefore:

- expand only steel from five to twelve fixed-point modes sampled from the
  external pack's recurring modal regions (`215–5,696 Hz`);
- preserve frequencies across centre/edge/corner while changing only signed
  modal participation;
- shorten frequency-dependent T20 to `60–270 ms` and the voice window from
  two seconds to `500 ms`;
- extend the deterministic strike component from `3 ms` to `20 ms` and raise
  it only enough to restore broadband contact energy;
- keep wood/glass bytes, authority isolation and the ordinary clip fallback
  unchanged.

## Bounded candidate comparison

Every row used the same seven frozen reference hashes and evaluator profile
`e1e57d9b598b1a0797fd5120608ccf0d775ec568b8ea15c6f72807c106366af1`.
The retained report uses blind seed `260826`. The table reports medians; no
weighted scalar selected a winner.

| Candidate | Median spectrum RMSE dB | Median modal cost | Median absolute mean-T20 delta ms | Conclusion |
| --- | ---: | ---: | ---: | --- |
| Failed baseline | `44.5951` | `1.04514` | `6606.82` | Too narrow and multi-second ringing |
| A: new modes, long damping | `32.5171` | `0.96580` | `658.17` | Better modes but still audibly risky ringing |
| B: bright participation | `22.7489` | `1.21708` | `711.73` | Correct spectral region, decay still censored |
| C: aggressively short decay | `22.5541` | `1.40836` | `66.71` | Clean but likely too dry |
| D: intermediate decay | `21.9281` | `1.36658` | `49.17` | Selected Pareto candidate; human audition required |

Candidate D centre moved spectral centroid to `1,680.46 Hz`, flatness to
`-35.67 dB`, temporal centroid to `27.54 ms` and mean reported T20 to
`89.03 ms`, with no clipping or Q0 failure tag. Its centre WAV SHA-256 is
`3b873244f6fb825318238bd98f70fa0c922b4541032281084ef7c0e291313883`;
the new six-second demo-sequence WAV SHA-256 is
`09e82f99555b102d3c7567cb8168b786367fbe9b1639caeeb0a9770f20204599`.
The full seven-pair report SHA-256 is
`fe003ca663964741517abfdc28b2f828e583f0d30b73bcf30a902ac3d4f15418`.

The modal median became worse than the failed baseline because the seven
references describe several different metal bodies. Candidate D is retained
as a Pareto point, not declared globally superior: spectrum and decay improve,
while object-specific modal identity and onset still disagree. The edge
position also receives `DECAY_WINDOW_CENSORED` from the current band-regression
diagnostic despite its bounded 500 ms voice, so it remains a human-audit item.

## Decision and next action

Use candidate D in the off-by-default laboratory/demo and preserve all prior
negative-baseline hashes as external evidence. Do not generalize the profile
to “all steel” or promote it to content/runtime contracts.

The next action is one blind product-owner audition of old versus D and D
versus the representative CC0 anchors. If D is still perceptually wrong,
record the named artifact before changing coefficients. A controlled small
steel-object recording or licensed RealImpact subset remains necessary before
relational fitting, held-out ranking or autonomous Q2 search can begin.
