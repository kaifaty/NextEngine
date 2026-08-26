# Physical sound wood/glass calibration — 2026-08-26

## Status and bounded claim

`WOOD-B_ACCEPTED / GLASS-D-F_REJECTED / GLASS-G_PARTIAL_ACCEPT / GLASS-H_AUDITION_REQUIRED / NO_P1_PROMOTION`

This report records one frozen P0 screen for two concrete experimental targets:

- a dry, solid hardwood block struck once;
- a deliberately short clink from a small glass object struck once without
  fracture.

The accepted 12-mode wood profile, partially accepted hybrid Glass-G and
four-mode Glass-H counterfactual exist only inside the off-by-default
laboratory and feature-gated reference demo.
They do not establish universal material sounds, calibrate the quality
evaluator, close SPEC-45 P1, or alter the shipped clip fallback.

## Frozen external reference screen

All source and decoded audio stayed outside the repository. OpenGameArt lists
each source page below as CC0:

| Source | Archive SHA-256 | Bounded use |
| --- | --- | --- |
| [Wood and Metal Sound Effects: Volume 2](https://opengameart.org/content/wood-and-metal-sound-effects-volume-2) | `91fd8fdac3d4f900223b96595e48610a6f3934c05cfa2bf5a4c9030f43a6f985` | three short wood-impact anchors |
| [100 CC0 SFX](https://opengameart.org/content/100-cc0-sfx) | `a5c135878c132f1c59cca54e60061c296cd0ac27ad031ca2c41b8cd5cab3c706` | two glass-impact anchors |
| [100 CC0 SFX #2](https://opengameart.org/content/100-cc0-sfx-2) | `0fc61b4494e2e893c0c015ced4877b3f689c7d84a48cb61daecd7ddb52db797b` | one wood and three glass anchors |

The decoded mono PCM-S16 48 kHz inputs were hash-frozen before candidate
search:

| Target | File | Decoded WAV SHA-256 |
| --- | --- | --- |
| wood | `wood2.wav` | `d2edc973bf7edfbaabd50c5637b6efd1e9343a519826b237b659614e38bb9a33` |
| wood | `wood14.wav` | `c6952d60f3638ababd891b3579cb78ec77355678b7b2e4eeb5e881e4eddb8007` |
| wood | `wood16.wav` | `b6f1c68a13631c8855553b744f68acdd196ecc26e56a4b684254d455f799a37a` |
| wood | `sfx100v2_wood_hit_02.wav` | `3f40643f4ecc5c978fe91b7a4a65620310a3004502864bfed38761745362f900` |
| glass | `glass_01.wav` | `5d57807a099de42aab8553022ac97ec5531dabd9105608249d869fbe81e7e346` |
| glass | `glass_02.wav` | `04537063471be425bf973a10d89289a13dbc60a8364d3266fed9d5bef983bf0e` |
| glass | `sfx100v2_glass_01.wav` | `eeea23b1c4dea9e94105ce18330a1139bf9831a909d2c40a3c3eeda745834c18` |
| glass | `sfx100v2_glass_04.wav` | `307441804d7b859d853ac0f04893ad1e5a4cdc2a0b57adc19333318db36371f8` |
| glass | `sfx100v2_glass_05.wav` | `b9a61e7a2b92ff187bc271c6cd180bfae476fdbcb7d8cc7aea5b14b6da32ad79` |

These are heterogeneous recordings and bodies. They are a broad timbral
screen, not a controlled matched corpus. The target labels above are therefore
engineering hypotheses rather than source metadata or ground truth.

## Frozen evaluator and method

The existing classical evaluator profile SHA-256 was
`e1e57d9b598b1a0797fd5120608ccf0d775ec568b8ea15c6f72807c106366af1`.
Every comparison used the same medium-force center render, onset rule,
gain-normalized multiresolution spectrum, modal assignment and per-band decay
analysis. Raw level remained separate. The original candidate selection used
Pareto evidence and target relations. The corrective cycle adds a frozen CLAP
screen only as secondary disagreement evidence; neither it nor a weighted
scalar is treated as a listening or quality oracle.

The baseline report SHA-256 was
`e5e309ea241d40a22adaf5e378fdee925786b81c9e44880103abe4fc7f5f5733`.
The selected manifest SHA-256 is
`3d3b0f7730fa2ce09e2ad6a7eeed658b2ba26fa87436aa986c3c0d7b63235ebb`;
its report SHA-256 is
`adbccf32d3ab47c77f769c4599236c95114083c1cb0dedf733b28e02303aba46`.
The report still says `NeedsHumanAudit` for every pair, as required.

## Wood candidate search

| Candidate | Median spectrum RMSE | Median modal cost | Median absolute mean-T20 delta | Temporal centroid |
| --- | ---: | ---: | ---: | ---: |
| old five-mode baseline | `35.66129 dB` | `1.073954` | `194.81427 ms` | `133.967 ms` |
| A: 12 short modes, low-heavy | `23.59795 dB` | `1.708984` | `21.19001 ms` | `20.528 ms` |
| B: harder mid-band balance, selected | `21.11627 dB` | `0.847192` | `33.18396 ms` | `18.087 ms` |

Candidate B improved spectrum and modal assignment against the baseline while
moving the energy envelope from a long drone to the 18–29 ms range of the
selected anchors. Its spectral centroid moved from `180.041 Hz` to
`325.253 Hz`, broadband T20 from `240.649 ms` to `74.331 ms`, and flatness from
`-66.388 dB` to `-41.050 dB`. The final profile uses fixed frequencies from
140–4,100 Hz, decreasing T20 from 90–25 ms, a 10 ms deterministic strike and a
bounded 350 ms voice.

## Glass candidate search

| Candidate | Median spectrum RMSE | Median modal cost | Median absolute mean-T20 delta | Temporal centroid |
| --- | ---: | ---: | ---: | ---: |
| old five-mode baseline | `31.87924 dB` | `1.860082` | `55.33225 ms` | `61.788 ms` |
| A: fitted modes, weak onset | `34.45369 dB` | `0.591736` | `242.22209 ms` | `40.798 ms` |
| B: stronger broadband onset | `30.24545 dB` | `0.591749` | `87.87219 ms` | `37.701 ms` |
| C: flatness-matched onset | `28.61521 dB` | `0.592003` | `83.47988 ms` | `28.712 ms` |
| D: 25% longer tail, later rejected | `28.96005 dB` | `0.671596` | `61.45113 ms` | `33.773 ms` |

Candidate D deliberately accepts a small spectrum/modal regression from C to
recover a more plate-like tail. Against the old baseline it still improves
median spectrum RMSE and modal cost substantially; its decay disagreement is
recorded rather than averaged away. Spectral centroid moved from `917.129 Hz`
to `3,386.563 Hz`, broadband T20 from `149.571 ms` to `115.078 ms`, and
flatness from `-64.784 dB` to `-44.161 dB`, close to the selected anchors. The
profile uses fixed inharmonic modes from 1,172–8,875 Hz, T20 from 350–112.5 ms,
a 10 ms deterministic strike and a bounded 700 ms voice.

## Product-owner result and glass correction

The product-owner audition accepted wood-B as normal and rejected Glass-D and
Glass-F as strongly metal-like. Those judgements outrank every descriptor and
CLAP gain above.
Published listening work explains why the miss was plausible: glass and steel
form a frequently confused hard-material group, while glass is associated with
higher signal frequencies in fine-grained identification. Frequency-specific
decay remains useful, but long-term spectral content also affects material
categorization. Sources: [Giordano and McAdams
2006](https://www.mcgill.ca/mpcl/files/mpcl/blg_smc_2006_jasa.pdf) and
[Hjortkjær and McAdams
2016](https://orbit.dtu.dk/en/publications/spectral-and-temporal-cues-for-perception-of-material-and-action-/).

The first correction therefore froze wood-B, retained D as a rejected
metal-like control, and narrowed F to the four short-clink anchors.
The long-ring `sfx100v2_glass_05.wav` remains frozen evidence but is excluded
from this discriminative split; results below must not be compared as though
the five-anchor and four-anchor medians were the same evaluation split.

| Short-clink candidate | Median spectrum RMSE | Median modal cost | Median absolute mean-T20 delta | Temporal centroid |
| --- | ---: | ---: | ---: | ---: |
| E: eight high modes, 300 ms voice | `25.91974 dB` | `0.603817` | `45.01008 ms` | `10.511 ms` |
| F: six high modes, 200 ms voice, selected for audition | `20.15047 dB` | `0.562628` | `66.49024 ms` | `6.301 ms` |

Glass-F wins the spectrum/modal comparison on the same four anchors and is
farther from the frozen steel control in modal assignment (`2.049772`). Its
six modes span 2,760–8,875 Hz with T20 values of 75–30 ms, a 4 ms deterministic
strike and a 200 ms bounded voice. Its centroid is `4,309.584 Hz` and broadband
T20 is `46.391 ms`; the deliberately short envelope is the counterfactual to
D's dense long-lived metal-like tail.

As a secondary screen, Apache-2.0
[CLAP HTSAT](https://huggingface.co/laion/clap-htsat-unfused) was run externally
with four fixed prompts per class. Glass-F scored glass `0.573800`, metal
`0.018618`, click `0.115041`; E scored `0.489794`, `0.008015`, `0.288352`.
Because the same screen labelled two real glass anchors primarily ceramic and
the synthesized steel primarily click, this evidence only breaks the E/F tie;
it does not override audition or establish calibrated autonomous ranking.
The discriminative manifest/report hashes are `2fd6b068…534e` and
`492a43a1…a9f`; the CLAP report hash is `cd1be134…0f1a`.

Glass-F then failed the product-owner audition too. This falsifies the bounded
strategy of selecting another ordinary modal bank from the current automatic
screens; it does not falsify modal synthesis for accepted steel/wood or every
possible glass object.

## Model-family escalation and Glass-G

Three competing explanations were checked before another implementation:

| Hypothesis | Evidence | Decision |
| --- | --- | --- |
| F still encodes metal-like spectral roughness | Aramaki et al. report that glass often has only a few distinct spectral components, while metal's dissonant aspect and roughness help separate it; onset alone does not define material | Leading: replace six simultaneous modes and a 4 ms full-gain broadband strike with three sparse partials and one fused, low-gain micro-contact onset |
| A thick/free plate is the wrong glass object hypothesis | Giordano and McAdams found real steel and glass plates perceptually equivalent inside the hard-material group; listeners instead followed size/frequency | Leading: target a small-object clink, not a universal glass plate |
| Recognizability requires audible shard/breaking pulses | Warren and Verbrugge show that asynchronous multiple pulse trains strongly identify the *breaking action* even with fixed spectra | Rejected for this experiment: it would change the event and violate the impact-only/no-fracture scope |

Sources: [Aramaki et al. 2011](https://doi.org/10.1109/TASL.2010.2047755),
[Giordano and McAdams
2006](https://www.mcgill.ca/mpcl/files/mpcl/blg_smc_2006_jasa.pdf),
[Warren and Verbrugge
1984](https://cspeech.ucd.ie/Fred/docs/warrenandverbrugge.pdf), and
[Hjortkjær and McAdams
2016](https://www.mcgill.ca/mpcl/files/mpcl/hjortkjaer_2016_jasa.pdf).

Glass-G is exactly one counterfactual: partials at 4,320/6,480/8,640 Hz with
T20 values of 45/32/22 ms, followed by no audible fracture train. Its
non-modal residual is three high-passed deterministic microbursts ending at
1.375 ms, short enough to fuse into one contact onset. Center descriptors are
spectral centroid `4,438.190 Hz`, broadband T20 `36.687 ms`, flatness
`-47.249 dB` and temporal centroid `7.387 ms`. They diagnose the intended
sparse/short relation but confer no perceptual pass.

The product-owner reported that G was more glass-like, reversing the D/F
metal-like verdict without accepting it as final. That result is evidence for
the small-object clink plus fused-onset model family and permits one
conservative spectral-balance refinement; it is not evidence for a universal
glass profile or an autonomous evaluator.

Glass-H preserves G's fused-onset pulse definition byte-for-byte. It reduces
the 4,320 Hz partial, shortens the three established partials to T20 values of
38/26/17 ms, and adds a low-gain 10,800 Hz partial with a 10 ms T20. Its
center spectral centroid is `4,845.789 Hz`, bandwidth `1,128.729 Hz`, broadband T20
`27.903 ms`, flatness `-42.616 dB` and temporal centroid `5.016 ms`, versus
G's `4,438.190 Hz`, `591.971 Hz`, `36.687 ms`, `-47.249 dB` and `7.387 ms`.
These relations confirm only that H is brighter, wider and shorter as intended;
they do not rank H above G. The center Q0 report retains
`DECAY_WINDOW_CENSORED` because its low/mid tail falls below the finite decay
fit window. That limitation is recorded rather than treated as a sound-quality
failure or hidden in a scalar.

## Exact selected outputs

| Output | WAV SHA-256 |
| --- | --- |
| `wood-center.wav` | `bff56a532b254fff1e18e7d5896643a92f0be4308d3979969c76d81a38d47fc9` |
| `wood-edge.wav` | `d10538c2b77dd5a9a47a60bebe4833fc1e376c278780dc675aeff1d676317f62` |
| `wood-corner.wav` | `9bdb766f77359a376ee01e3cf9e78d3b27cab0decca1bf865be499345e1e0242` |
| `glass-center.wav` | `12c81bce61938e89904717461d95031755a7919cbb0f817ea6d0a0457e5d4c09` |
| `glass-edge.wav` | `8fdea1c5eb41d81336fdc670eff174d73fcd1fdc7b84b62ab42c260192c22104` |
| `glass-corner.wav` | `8aa5dbceeb0b979a3b4fd1c29f0c7337ba4b46b31d2787f7da3eb0aa1a4f0e77` |
| `demo-sequence.wav` | `3cfbef1609ce57c39b52d25f3a9b6b6e53dcc172cdb0939e4ad137aabb66e2d2` |

The center raw interleaved PCM hashes are pinned in focused Rust tests. Steel's
existing exact PCM hash remains unchanged. Center/edge/corner share frequencies
and damping and differ only in modal participation, preserving the required
position relation.

## Decision and remaining uncertainty

Keep accepted wood-B bit-exact, preserve Glass-G as the positive control and
expose Glass-H as the sole new demo counterfactual. Keep rejected D/F,
reference audio, blind bundles, model weights and evaluator artifacts external.
Preserve the ordinary clip mixer as the shipping/fault fallback.

The next discriminator is a direct product-owner audition of Glass-H against
the partially accepted Glass-G. H may be preferred, rejected in favor of G or
motivate a bounded blend; the descriptors cannot make that decision. A
controlled recording of one known glass body at repeated positions/forces is
still required before held-out ranking, autonomous quality iteration, a generic
material claim or any P1 promotion.
