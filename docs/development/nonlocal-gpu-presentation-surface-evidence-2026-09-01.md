# Nonlocal GPU presentation surface evidence — 2026-09-01

## Result and ceiling

`PRESENTATION_SURFACE_SUPPORTED_BOUNDED` for the NGQ5 edge-aware extractor on
the exact accepted 4k/16k falling-dam keyframes.

The prototype converts every sphere-depth mask to one connected height-field
mesh, preserves the exact parent simulation roots and passes the frozen mask,
depth and mesh gates. It is presentation-only: no extracted value is fed back
to particles, contacts, density, velocity or neighbor work.

This is not final renderer quality. The debug normal-lighting montage still
shows a particle-lattice texture, and the current CPU implementation is too
slow to place naively on every rendered frame. The bounded result admits the
surface representation and edge-aware rule; GPU/renderer implementation,
smooth normals and final art direction remain open.

## Frozen lineage

```text
branch                         codex/water-research
base before this change        7eba5872
profile                         nuv-basin-48k-analytic-contact-game-cap160.v6
profile SHA-256                3f482db40880099b9088c6b2ad8d7d427e455d78750267cb04410ff5ac040661
NGQ4 rev1 contract             b450bd0532c0b5d5b669e146dd232fa7579f0f9cc704b701bba4a1707b8d65c9
NGQ4 rev2 contract             a0feec23110650975011ce9d050faaf1148bec750db9a735c48bd24a8ac17eb0
NGQ5 rev1 contract             52677be33f613e5742a753c268d4b9c2f83173fdf5de6ae41fb3455852383d88
binary SHA-256                 798a7c0ad39b83140617c17f0b90353e80a5603cdee3a7ce7bc19de09ecea147
CUDA/compiler                  13.3.73 / RTX 3080 sm_86
```

Measured source SHA-256 values before commit:

```text
cuda_baseline.cu   88cd95d113b7e9ad540575c1addb53056fd2ac371df2ba4521e6b44f35f857cd
cuda_baseline.hpp  1fa41b350dd464990190f0271574f9714530aaac8abfc8fe64b96a2347da8eb1
main.cpp           2e8c565f9f322451f8ef4ccfa28d8622600982ed03c8ac6b71d1df45b703742c
```

## Negative controls and causal update

NGQ4 revision 1 rejected the desired circle-to-cell fill at frame 0: raw
`4,800` pixels became a connected `6,400`-pixel footprint, area ratio
`1.333333`. Common coverage was `1.0` and depth RMSE only `2.491 mm`, so the
near-unit area gate was the wrong continuous-surface observable. Raw JSON
SHA-256: `2995bb1b7064fb285107c4174040830774d8447205515e75a40de64ea6026815`.

NGQ4 revision 2 corrected only that gate, then exposed the real algorithmic
failure at 4k step 48: ordinary binomial height smoothing crossed the moving
front, producing RMSE `57.321 mm`, p95 `148.925 mm` and maximum change
`275.410 mm`. Raw JSON SHA-256:
`5c30f87566b4b59b116ae7756698395bd5f7e616e61fcad83a5f1999765b4a82`.

NGQ5 changed only the height weights to a one-pass bilateral rule with range
sigma equal to particle radius (`0.025 m`). It also recomputed the isotropic
path on the same base depth as a root-bound causal control. At 4k step 48 the
bilateral result is RMSE `3.249 mm`, p95 `5.801 mm`, maximum `13.589 mm`, while
the isotropic control exactly repeats the NGQ4 failure. The same separation
occurs in 16k. This supports HG5A: cross-edge averaging, not closed-pixel
assignment or height-field topology, was the first smoothing failure.

The domain-plus-range weighting principle is consistent with Tomasi and
Manduchi’s edge-preserving bilateral filter (ICCV 1998,
DOI `10.1109/ICCV.1998.710815`):
<https://projects.iq.harvard.edu/sites/projects.iq.harvard.edu/files/imagenesmedicas/files/tomasi1998kg.pdf>.
The paper does not supply the water-specific sigma or gates; those were frozen
locally before the NGQ5 run.

## Accepted metrics

| Metric across five frames | 4k | 16k | Gate |
| --- | ---: | ---: | ---: |
| components | `1` every frame | `1` every frame | `1` |
| area-ratio range | `1.0393 .. 1.3333` | `1.0359 .. 1.3333` | `[0.95,1.40]` |
| minimum common coverage | `0.99154` | `1.0` | `>=0.95` |
| maximum bbox expansion | `0 px` | `0 px` | `<=1 px` |
| maximum bilateral RMSE | `4.035 mm` | `3.855 mm` | `<=25 mm` |
| maximum bilateral p95 change | `8.257 mm` | `7.681 mm` | `<=50 mm` |
| diagnostic maximum change | `17.537 mm` | `17.948 mm` | report only |
| maximum isotropic-control p95 | `148.925 mm` | `141.897 mm` | causal control |
| final mesh vertices | `10,071` | `33,634` | `>0` |
| final mesh triangles | `19,708` | `66,510` | `>0` |

All newly filled pixels have a retained raw 8-neighbor. The final 4k cleanup
culls `82` raw satellite pixels while keeping `99.154%` common coverage; 16k
culls none.

## Exact roots and two-process reproduction

```text
parent visual corpus    c2f1f6e75f1238409298cb6db6f16c2aeb0baca3ad4ebbeff216ba3c39f98274
surface controls        8fefb524acbc7cde18a292105c83a44ba105cc0fc9754bde311a855c09a851da
4k surface result       c58e7c5193078ddb6194df99cccd6bfa30b4f669db18981bb09d601371932656
16k surface result      7b089aad02765ddce52345ba8dfe7687c44d169f42c2f217a6f3f803dc11dd8d
surface corpus result   a98f189b5fc4b35099376745605bc9e2851a1b4b6605cfa3c9e581d6e2adf391
semantic subset         44712cc513446300a9a192a97021acb3e92bb725940bc87dcf7ef49f4afe78cc
```

Raw JSON SHA-256:

```text
without file output  2dad84b1c582b31cdffd7f9c5050397ef33330eb1d125f8a928f83d368b5fd16
with file output     7f694608bc4ad9561752b455827f48a82f40cf2999287764d2f2aaae373b9b87
```

File output changes only timings and file flags. All parent, frame, lane and
corpus roots are exact across processes.

## Diagnostic extraction cost

The host-only prototype processes five sparse keyframes, not every simulation
or render frame:

| Lane | Process A total / average | Process B total / average |
| --- | ---: | ---: |
| 4k | `17.838 / 3.568 ms` | `18.383 / 3.677 ms` |
| 16k | `69.088 / 13.818 ms` | `69.519 / 13.904 ms` |

These numbers are not part of the GPU physics distribution and are not a
renderer performance gate. They show that a direct CPU implementation is only
a reference/prototype; the next implementation belongs in GPU compute or the
renderer surface pipeline.

## Generated artifacts

Artifacts remain outside Git:

```text
4k montage PPM    5b450fa446a3e145c97a97e65cfff9e93bbcd5b2d5ce62d465390232ca6d0572
16k montage PPM   8cc668332162cfaef82c233f6919226bedab34d5b3338ebdfbff59f2b9c1a3ee
4k final OBJ      23ba876522dae24ba6e7fe0051d33c12032e1ffb279a8f84fc36736756e358e0
16k final OBJ     0a20f71161acfc8108d1953a7a8ef4dd21ab83693ed7b457e6ce773d24c3632d
4k montage PNG    6ed365dc4d3373b655a808085e760932d818045fb5c4776016bc363d892f05b7
16k montage PNG   afde47ab9066438e5971e86b487790f9e024522bf69f7ef95521b26e8adf231b
```

OBJ line counts independently match the JSON vertex/triangle counts. Blender
rendering was `NOT_RUN` because Blender is not installed on this host. The
native debug montage is sufficient for algorithm inspection but visibly
overemphasizes residual particle-scale height texture.

## Checks and decision

- focused strict Release build: PASS;
- two surface processes, file output off/on: PASS and semantic-exact;
- empty/non-finite/mask/depth controls: PASS;
- parent 4k/16k trace/result roots: exact;
- OBJ vertex/face recount: exact;
- CUDA self-test: PASS, SHA-256
  `a532a0851fc6ed5385d8b8edbc33a09173a52acb2e7138b4e9e25c546fb9a05e`;
- P2 capacity-160 correspondence: PASS, SHA-256
  `87dda39c7f767d0e829c9f0b246afd398a1533b2844650c69aaec30a0385a605`;
- independent review: `NOT_RUN`;
- `git diff --check`: PASS.

Decision: retain the connected edge-aware height-field/mesh as the
renderer-facing prototype. Do not add more CPU filter passes. The smallest
next step is a GPU/renderer port with smooth normal reconstruction and an
incremental performance measurement; it must remain presentation-only.
