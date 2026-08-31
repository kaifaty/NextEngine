# NCGP8 corrected-axis visible-surface evidence — 2026-08-31

## Current verdict

`H8A_VISIBLE_SURFACE_SUPPORTED_BOUNDED / INDEPENDENT_REVIEW_PENDING /
PERFORMANCE_NOT_RUN`.

The exact NCGP6 hydrostatic step-112 CPU/GPU witness reproduces, and its
corrected-axis top-view sphere presentation is close under every pre-frozen
NCGP8 observable. Two clean Release builds and two complete fresh processes
are byte-identical. Retained controls and all three Compute Sanitizer tools
pass. The required independent review is still running, so this is an author
candidate result rather than final authority to resume the complete corpus.

NCGP7's three-dimensional bulk evidence remains useful. Its surface layer is
invalid as a product quantity of interest because it used `y` as height and
`x-z` as the image plane while the retained profile gravity is
`(0,0,-9.81)`. NCGP8 uses `z` height and the `x-y` plane.

## Frozen identity

- contract commits: `218ef850` (revision 1) and `1eec9162` (revision-2
  control corrigendum);
- revision-1 SHA-256:
  `9d64e5d7b62773353105310438de12da2b730131ef4549ecac0557deda16429b`;
- revision-2 SHA-256:
  `342f1acfa86ef0399d7c9ea1d7cd9b794f02ecd44544a70591b0fc91bbdc28f4`;
- aggregate NCGP8 contract root:
  `273d6b54d5cb6891204db730ab66566c1cb7b259c24ec7a4f29037eca590072f`;
- source commit:
  `7db62f7459834c7f71bfba4fa4bf9ece6033548c`;
- source tree:
  `9c37e4f72f1f9c5f7cb2a12715d9b57414affb0c`;
- source root:
  `a2da458552dedf6555f6233d8f317e531d235f17df9743f78e0f867ed3501f1d`;
- clean Release binary A/B SHA-256:
  `431bfac8931150e6cc4949d8f6307ec1cb215edffd6b07be8abb03552b7cf878`;
- environment: NVIDIA GeForce RTX 3080, SM 8.6, CUDA runtime/driver 13.3;
  allocated device memory `137251397` bytes;
- compiler flags: C++ `-O3 -Wall -Wextra -Wpedantic -Werror
  -ffp-contract=off -fno-fast-math`; CUDA `-O3 --fmad=false
  --prec-div=true --prec-sqrt=true --ftz=false`; `SM=86`.

## Exact commands and reproduction

```text
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B <fresh-a-or-b> -DCMAKE_BUILD_TYPE=Release
cmake --build <fresh-a-or-b> \
  --target nonlocal-corrected-cuda-visible-surface \
           nonlocal-corrected-cuda-eulerian-diagnostic -j2
<fresh>/nonlocal-corrected-cuda-visible-surface \
  --visible-surface-self-test
<fresh>/nonlocal-corrected-cuda-visible-surface \
  --diagnose-hydro-step112-visible-surface
```

Both self-tests pass with byte-identical stdout SHA-256
`f7f4a3330148deea405415251fdd7f8f8ee301e9dfc20dbe45a4c3ae07e9cd2a`.
Both complete witness processes pass H8A with byte-identical stdout SHA-256
`9681e4c17639bf3fc0fd50a4e46edc2fa02170b08b071b19c37e6a8557e98cae`
and result root
`1c953b54eefa402dbe9b66c554e3a21fcd60b2c28224398cd4fa6fcd9e998bd0`.
Raw JSON remains outside Git at `/tmp/ncgp8-a-step112.json` and
`/tmp/ncgp8-b-step112.json` for this host session.

## Parent witness closure

NCGP8 reconstructs the NCGP7 parent state and receipt byte-for-byte:

| Identity | Exact value |
| --- | --- |
| input root | `f96ad2a8bd7c7fcee2917ff6168c1aac59d7da56d2f5844a3cfcdab396eda80f` |
| 112-step receipt root | `5dcff7ae2f7f9f6f6758a42aea9806b4631d71fe2aeb1b03cb78dab5e2c19b9b` |
| corrected/permuted GPU state root | `cd2d44238d08968de263bfc6c4495e6e3535e49d05f0ab82f449a687dc9a4222` |
| CPU state root | `4bb62cc6c006d489b65fad6a6a87d8d81e0a8ade77a349fd1b03e2c8e1301af1` |
| NCGP7 result root | `d11a79d7360b9ea4b1fc3cc85c974d4e97592bc29cdc09483a6dde7f6fe70abc` |

The stable-ID witness also remains exact: position RMSE
`0.780996279 mm`, nearest-rank p99 `2.576882866 mm` and diagnostic maximum
`20.810782528 mm`. This preserves the NCGP6 failure rather than relabelling it.

## Corrected visible-surface result

The observer looks along `-z` through a `240 x 200` image with `12.5 mm`
pixel pitch. Each sample is one unsmoothed sphere of radius `25 mm`.

| Observable | Frozen H8A limit | Observed | Result |
| --- | ---: | ---: | --- |
| silhouette symmetric difference | `<= 1%` | `0.1926968%` | PASS |
| depth RMSE | `<= 6.25 mm` | `4.440802 mm` | PASS |
| depth p95 | `<= 12.5 mm` | `0.523432 mm` | PASS |
| depth p99 | `<= 25 mm` | `1.767857 mm` | PASS |
| material component count | exact | `1 / 1` | PASS |
| largest-component fraction difference | `<= 1%` | `0%` | PASS |
| CPU/GPU satellite-area fraction | `<= 1%` | `0% / 0%` | PASS |

There are `20,718` common wet pixels and `20,758` union wet pixels. Corrected
and stable-ID-permuted GPU image roots are exact:
`f09c316d7242dc481a19d89c442a9a37b374fc3a41b3af613093c2d383146b75`.
The CPU image root is
`9c4d217d7dc3f1003549c1f7a1a819663a7003911b020a67f1b3681c28a9f9a2`;
the comparison metrics root is
`82748e152be7aeebc69f54292d8ee388554d8ad8804dc8ef76dabbb3f50a0c06`.

The diagnostic maximum depth error is `0.507051559 m` at a rare common pixel.
It is not hidden or deleted. The frozen product observer deliberately treats
the maximum as a warning while gating the depth distribution, silhouette and
connected topology. This finite H8A result therefore says the outlier is not a
persistent visible-surface defect on this hydrostatic witness; it does not say
all future flows may ignore large local surface errors.

## Retained physics and apparatus

The witness preserves exactly 4,000 samples and 500 kg, zero basin penetration,
corrected/permuted identity and normalized momentum correspondence
`0.00174968%`. The GPU/CPU HVP maxima remain `125/110` under budget 128.

All build-A/build-B control outputs are byte-identical:

| Control | stdout SHA-256 | Result |
| --- | --- | --- |
| NCGP8 visible observer | `f7f4a3330148deea405415251fdd7f8f8ee301e9dfc20dbe45a4c3ae07e9cd2a` | PASS |
| NCGP7 Eulerian apparatus | `a4b98636f57516c09e1cbc89daed1601829799760668fc1e0dfab8fbb3c7852d` | PASS |
| corrected profile | `da062aa21ca22fc9331370e27ae975841e78ac1f12df94a8497481fe6fb3e375` | PASS |
| compensated graph | `59472fa2bf77af89d2974bad0b53e21290ace27e22f0c28161220da411b76022` | PASS |
| analytical boundary | `c3ca395cdd28c0f2486c7a735dd36d3b16391d1ac7f55d21d3e1ac9ba75c43a2` | PASS |
| transaction rollback | `b1d6a81359146e58c17832a854f4528a8d946f67464c8058e9c5404995da46fe` | PASS |
| physics/HVP/predictor | `ac5cb8ca6897b99f5faf51014adf6c687681f52ec7d5203ccd1c0188894b477b` | PASS |
| NCGP6 gate apparatus | `312c6255faf7874e1339c61135a00eb15be8b510c3e1dd62a18455f389b123d3` | PASS |

The NCGP8 self-test executes exact-zero/relabel, vertical and horizontal
translation, wrong-axis, deletion, sparse-tail, complete-sheet, strict-radius,
pixel-pitch, omitted-topology, depth/component/work and result-root controls.

Compute Sanitizer memcheck, initcheck and synccheck each execute the retained
physics suite, exit zero and report `ERROR SUMMARY: 0 errors`. Their captured
stdout/stderr SHA-256 is identical:
`6c7819e634b6ac4a4e733d922bf17836d4f5cf039c720c374f72bbf29358322c`.

## Decision boundary

Pending independent review, the author result supports H8A only for the exact
hydrostatic step-112 presentation. Performance remains `NOT_RUN`; the prior
`~1.0--1.18 ms p95` result is still neighbor-only.

If review returns GO, the smallest successor is a separately frozen complete
4k product-correctness corpus: hydrostatic hold, dam-break and orifice for 240
steps, retaining strict same-state/operator/invariant gates while applying the
reviewed visible-surface observer to independently evolved trajectories. Only
after that corpus, 16k/50k capacity and 240-step 50k correctness pass may the
full GPU-step timing run.
