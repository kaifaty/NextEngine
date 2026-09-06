# Original Nonlocal GPU dynamic visual evidence — 2026-09-01

## Result

`SUPPORTED_BOUNDED` on the finite NGQ2 revision-2 game corpus.

The original five-iteration fused-owner/compact-u16 GPU route advances both a
4,000- and 16,000-particle falling dam for 96 steps (`0.4 s`) with analytic GPU
box contact. Both lanes pass the predeclared containment, motion, sphere-depth
surface and topology bands. This is evidence for plausible stylized game water
on these two fixtures, not laboratory fidelity or production readiness.

Revision 1 is separately preserved as
`APPARATUS_INCONCLUSIVE / CSR_CAPACITY_123`: impact step 42 required degree
`124`. Revision 2 predeclared `160` slots per sample and reached maximum degree
`138` without changing equations, five iterations, fixtures or quality gates.

## Exact implementation and host

```text
branch                         codex/water-research
base before this change        4d35ec75
binary                         40742b6a5e66c535976510d09db03dca63fb259370171c7645627e40b31122c5
profile                        nuv-basin-48k-analytic-contact-game.v5
profile root                   cd6dd8f9ea4304a603b9f58d2891787fc5f9173c50edc4d635efa6fbc7b5901b
CUDA/compiler                  13.3.73 / RTX 3080 sm_86
build directory                /tmp/nextengine-nonlocal-game-quality-build.zdvs81
```

Source identities before commit:

```text
cuda_baseline.cu  c083532804a625a4f02c1d7611f6104768a4e552a0cccfa32b6576c1a99b88ae
cuda_baseline.hpp 3f5c24adf91bf1036168caa44cac83be6442f13b8b8abf9375538025b3fc9592
main.cpp          e9a1332429b2cb6f8ef99dbf439c1a5088b5bbdbb46ffe9aed4d956583b1fdb4
```

## Frozen commands

```bash
cmake --build /tmp/nextengine-nonlocal-game-quality-build.zdvs81 \
  --target nonlocal-feasibility -j2

/tmp/nextengine-nonlocal-game-quality-build.zdvs81/nonlocal-feasibility \
  --game-visual-corpus \
  > /tmp/nonlocal-game-visual-final-a.json

/tmp/nextengine-nonlocal-game-quality-build.zdvs81/nonlocal-feasibility \
  --game-visual-corpus --frames /tmp/nonlocal-game-visual-final \
  > /tmp/nonlocal-game-visual-final-b.json
```

Raw JSON SHA-256:

```text
A 995d046d4cb0c1aca10e282d3175031c9f1a61f1dae180bfe66fefa91e9dd86d
B d2dfe60f7088535a27cedd513b73e0171ad3bb49069f4d76e2b29c1c2e859afe
```

The raw files differ in measured timings and `montage_written`. Simulation,
trace, every frame and result roots are exact across the two processes:

```text
corpus result c2f1f6e75f1238409298cb6db6f16c2aeb0baca3ad4ebbeff216ba3c39f98274
4k trace     369ac07d797d755abbee8b965e8d6fe635693534938c00cfbc684f75e5a71bbb
4k result    18f48386fb1c178e8bf22cf1fa74315abf0be4f8decb31ffa15cda97a35278aa
16k trace    61c45089cf57d41045d7ca1be59657b93845cc4a266173dd121ada2f84578a05
16k result   8e3e9f9584f91622b274f6635452abe65230cf24b2bf24d1636e735d3fd74f85
observer     ef81f8e4fcf34ecb9e128362395edd62fa758a87109d534023b20c1d3bc4ac3b
```

## Decisive visual and invariant metrics

| Metric | 4k | 16k | Gate |
| --- | ---: | ---: | ---: |
| accepted steps | 96 | 96 | 96 |
| maximum degree | 138 | 138 | <= 160 |
| maximum speed | 3.4563 m/s | 3.6228 m/s | <= 10 m/s |
| contact velocity error | 9.91e-6 m/s | 1.72e-5 m/s | <= 1e-4 m/s |
| penetration | 0 | 0 | <= 1e-12 m |
| vertical COM drop | 0.2036 m | 0.1692 m | >= 0.075 m |
| front advance | 0.6037 m | 0.6449 m | >= 0.10 m |
| final/initial wet area | 2.01875 | 1.66464 | [1.05, 3.00] |
| depth-p95 drop | 0.1856 m | 0.1250 m | >= 0.025 m |
| final silhouette largest component | 99.154% | 100% | >= 98% |
| final silhouette satellite area | 0.846% | 0% | <= 2% |
| final particle components/satellites | 1 / 0 | 1 / 0 | largest >= 98%, satellites <= 2% |

At steps 0/24/48/72, both lanes have one material silhouette component and
zero satellite area. At step 96 the 4k raster has a second small presentation
component (0.846% area), while the particle graph remains one connected body.
This is inside the deliberately stylized game band but is the first visual
quality limitation to watch in a longer or renderer-smoothed corpus.

The unsmoothed top-view montages contain frames 0/24/48/72/96 from left to
right. Generated artifacts remain outside Git:

```text
/tmp/nonlocal-game-visual-final-falling-dam-4k.ppm   e5055e47a5cf17e5d2ef41fcb622a563cf371fb3b4aff76a88ee295937b7c74c
/tmp/nonlocal-game-visual-final-falling-dam-16k.ppm  41f1f661a452f8b7f9e9040c30cb29ca034ee39164ffda9930421abb637f26eb
/tmp/nonlocal-game-visual-final-falling-dam-4k.png    6f474a10c2b4a04d71f5ad8746a9b21891ec9e53e73070d4e9c4cd51c5a5b794
/tmp/nonlocal-game-visual-final-falling-dam-16k.png   06ff07fd3b280d1ad57fd7b2430549f6ea7e43623639d7b8fb993d8ed8542965
```

## Diagnostic CUDA timings

Observer work, host reconstruction and file output are excluded. These are
dynamic 4k/16k diagnostics, not the required revised 48k performance gate.

| Lane | A p95 / p99 | B p95 / p99 | contact p95 |
| --- | --- | --- | --- |
| 4k | 1.6456 / 1.9528 ms | 1.6282 / 2.1996 ms | 0.004096 ms |
| 16k | 1.8412 / 1.9364 ms | 1.8545 / 1.9743 ms | 0.004224 / 0.004096 ms |

Because revision 2 raises allocation from `N*123` to `N*160`, the prior 48k
`3.80 ms` evidence is not inherited. The next performance discriminator is a
fresh two-process 48k measurement at capacity 160.

## Controls and checks

- observer empty/non-finite rejection: PASS;
- mask/depth mutation root sensitivity: PASS;
- frame emission on/off preserves all semantic roots: PASS;
- focused CMake Release build: PASS;
- `--game-quality-smoke`: PASS, SHA
  `c9a0c1df2075678af474ca20f3bc7b1965d09f3c55e3747b4163b66fbedd18d1`;
- CUDA `--self-test`: PASS, SHA
  `04fef6896598604315b15d8b4bfdac22b2911dcab57aa3c92e7ca6c19480bddc`;
- `--cpu-h3-profile-corpus`: PASS, SHA
  `0b02fc8c12d8870b1b9ae1d9c135fbabe684daa450d7ce73debc59ba4e581950`;
- `git diff --check`: PASS;
- independent read-only review: `NOT_RUN` (no independently authorized
  reviewer in this stage), so the result remains author-side bounded evidence.

## Decision and remaining risk

The fast GPU route is visually plausible enough on the frozen 4k/16k corpus
to continue. It is not yet a combined quality-and-budget success because the
necessary capacity headroom has not been timed at 48k. After that measurement,
the smallest renderer-facing successor is a visual-only smoothing/extraction
prototype over these same accepted frames; it must not feed back into physics.
