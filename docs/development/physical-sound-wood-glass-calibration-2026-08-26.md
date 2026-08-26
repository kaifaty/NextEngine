# Physical sound wood/glass calibration — 2026-08-26

## Status and bounded claim

`WOOD-B_ACCEPTED / GLASS-D_REJECTED / GLASS-F_AUDITION_REQUIRED / NO_P1_PROMOTION`

This report records one frozen P0 screen for two concrete experimental targets:

- a dry, solid hardwood block struck once;
- a deliberately short glass clink struck once without fracture.

The selected 12-mode wood and six-mode Glass-F profiles replace the failed
five-mode presets only inside the off-by-default laboratory and feature-gated
reference demo.
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

The product-owner audition accepted wood-B as normal and rejected Glass-D as
strongly metal-like. That judgement outranks the descriptor gains above.
Published listening work explains why the miss was plausible: glass and steel
form a frequently confused hard-material group, while glass is associated with
higher signal frequencies in fine-grained identification. Frequency-specific
decay remains useful, but long-term spectral content also affects material
categorization. Sources: [Giordano and McAdams
2006](https://www.mcgill.ca/mpcl/files/mpcl/blg_smc_2006_jasa.pdf) and
[Hjortkjær and McAdams
2016](https://orbit.dtu.dk/en/publications/spectral-and-temporal-cues-for-perception-of-material-and-action-/).

The correction therefore freezes wood-B, retains D only as a rejected
metal-like control, and narrows the new target to the four short-clink anchors.
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

## Exact selected outputs

| Output | WAV SHA-256 |
| --- | --- |
| `wood-center.wav` | `bff56a532b254fff1e18e7d5896643a92f0be4308d3979969c76d81a38d47fc9` |
| `wood-edge.wav` | `d10538c2b77dd5a9a47a60bebe4833fc1e376c278780dc675aeff1d676317f62` |
| `wood-corner.wav` | `9bdb766f77359a376ee01e3cf9e78d3b27cab0decca1bf865be499345e1e0242` |
| `glass-center.wav` | `d3976180a5b21f68410d135d4e3974e959218d4feff0d21eb51e4419fc39866d` |
| `glass-edge.wav` | `116f226635f171708c837aea2785986c7273b91a7c644852497e0c93ce2205b7` |
| `glass-corner.wav` | `6dcd5d7f75625105582f408271adb9c10f67f12af1cb1906741a12b8b938b125` |
| `demo-sequence.wav` | `83356b06c86145a765464cd41df8240ccbdeea1da67b677875bcc9b27cf8bc73` |

The center raw interleaved PCM hashes are pinned in focused Rust tests. Steel's
existing exact PCM hash remains unchanged. Center/edge/corner share frequencies
and damping and differ only in modal participation, preserving the required
position relation.

## Decision and remaining uncertainty

Keep accepted wood-B bit-exact and replace rejected Glass-D with Glass-F for the
current laboratory/demo audition. Keep all reference audio, blind bundles,
model weights and evaluator artifacts external. Preserve the ordinary clip
mixer as the shipping/fault fallback.

The next discriminator is the product-owner audition of Glass-F in the demo;
failure rejects or revises it without touching wood-B. A controlled recording
of one known glass body at repeated positions/forces is still required before
held-out ranking, autonomous quality iteration, a generic material claim or
any P1 promotion.
