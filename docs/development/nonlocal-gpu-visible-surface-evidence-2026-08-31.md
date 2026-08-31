# NCGP8 corrected-axis visible-surface evidence — 2026-08-31

## Current verdict

`H8A_VISIBLE_SURFACE_SUPPORTED_BOUNDED / REPAIRED_CANDIDATE /
INDEPENDENT_RE_REVIEW_PENDING / PERFORMANCE_NOT_RUN`.

The exact NCGP6 hydrostatic step-112 CPU/GPU witness reproduces, and its
corrected-axis top-view sphere presentation is close under every pre-frozen
NCGP8 observable. Two clean Release builds and two complete fresh processes
are byte-identical. Retained controls and all three Compute Sanitizer tools
pass. The initial independent review reproduced the numerical H8A result but
found incomplete retained-bulk, validation-work, top-sheet-control and JSON
root closure. Commit `8f7d9850` is the single allowed repair batch; its one
permitted re-review is in progress. This remains an author candidate rather
than authority to resume the complete corpus.

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
- initial source commit: `7db62f7459834c7f71bfba4fa4bf9ece6033548c`;
- repaired source commit:
  `8f7d98501f3dc595ce9666bb0d27a1c4564eaccb`;
- source tree:
  `bbaf24af906b788505dac4f610eba908aca026e9`;
- source root:
  `ad0a835ec276eed665ff7a89ef4276cae3be1467375c7c006ee9838a06cf1e8e`;
- clean Release binary A/B SHA-256:
  `c5213dd1f354f86e93497ef87b6856801c38b6636809179c6fdb8545afb98e86`;
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
`6ff062f5c432c3fe6fab02736ebbb75d37f3c8b0db726d77cc04947956db9db3`.
Both complete witness processes pass H8A with byte-identical stdout SHA-256
`f22c68b1e1eeaeff53913b098bb08a14b7b540551cb9fe2bd69bf38fd998f14b`
and result root
`ad982ab8cf5363b5222b4a5014076d55df393cba0c1facf01e2361d1e9cbce84`.
Raw JSON remains outside Git at `/tmp/ncgp8-repair-a-witness.json` and
`/tmp/ncgp8-repair-b-witness.json` for this host session.

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
`ff44ccc5eba113280170a022fcf35cbf6c44543982ec051dcc3cfa61814a79ad`.
The CPU image root is
`0ddd574daea83b46a818a31bd1ffd5d66e11300f1fd3b12e2bd051dd534564d8`;
the comparison metrics root is
`654bffc9c5b1d531a9067fd22fd89847627030ca782effcea20419b750402223`.

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

The repaired witness recomputes all six NCGP7 fields (GPU, CPU and permuted at
50 mm and 25 mm), both field comparisons and their work roots. Every frozen
field/work/metrics root is exact; the retained-bulk closure root is
`0ed939290e3efd0e2dcd960af424b3846ee1f1a2feaa1f2f38ce814f6c445595`.
Both coarse and fine bulk mass/density/velocity/centre-of-mass bands pass.
The result no longer relies on the literal NCGP7 final root alone.

Observer validation now replays and accounts for the depth reductions and
component flood fill. For the CPU/GPU comparison it seals 100,426 depth-record
reads/reductions, 41,476 flood pixels, 331,808 flood-neighbour tests and 12
validation-root derivations. Each image publishes ordered contribution, mask,
depth, component, work and image roots with its exact work counters. The
negative sheet fixture shifts exactly the 16 samples in the top layer by
50 mm; the depth-record mutation control also exercises the real receipt.

All build-A/build-B control outputs are byte-identical:

| Control | stdout SHA-256 | Result |
| --- | --- | --- |
| NCGP8 visible observer | `6ff062f5c432c3fe6fab02736ebbb75d37f3c8b0db726d77cc04947956db9db3` | PASS |
| NCGP7 Eulerian apparatus | `0a2ae8b713ad6f9805728968b5ce621927b3eeeeef981fe27577a3b419ad1d9d` | PASS |
| corrected profile | `84768a122b91c68b5d130b1a4d3d75a531abf5e3c82360d323b17ef89efaee18` | PASS |
| compensated graph | `01f633aa12e055242ebad8783c270a033e197c483e593f9b3782c6308c01c2e8` | PASS |
| analytical boundary | `a166f42660084ed6e54bdcf2911dfab652337498ead7ececa98ec0848f7cf6bc` | PASS |
| transaction rollback | `44111838dd62c2d83e2336f7b4a080c1898602153a5357f64233beb8469fb7a9` | PASS |
| physics/HVP/predictor | `4fb9a58459e4de7754886f819634a4cc61cd2c14c532be81a7545c5b231c1727` | PASS |
| NCGP6 gate apparatus | `9ef6954c9c8a6c824ccacd974469045becc23fa3773214f2fa12c9fcaae2dbad` | PASS |

The NCGP8 self-test executes exact-zero/relabel, vertical and horizontal
translation, wrong-axis, deletion, sparse-tail, top-sheet-only, strict-radius,
pixel-pitch, omitted-topology, depth-record/depth/component/work and
result-root controls.

Compute Sanitizer memcheck, initcheck and synccheck each execute the retained
physics suite, exit zero and report `ERROR SUMMARY: 0 errors`. Their captured
stdout/stderr SHA-256 is identical:
`e42bbcfbf51e8990b8403cbba3fa0e12ed6c86127e8bfcd7744b68d16c5bc2bc`
for stdout and the empty-file root
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
for stderr.

## Decision boundary

Pending the one permitted independent re-review, the repaired author result
supports H8A only for the exact
hydrostatic step-112 presentation. Performance remains `NOT_RUN`; the prior
`~1.0--1.18 ms p95` result is still neighbor-only.

If review returns GO, the smallest successor is a separately frozen complete
4k product-correctness corpus: hydrostatic hold, dam-break and orifice for 240
steps, retaining strict same-state/operator/invariant gates while applying the
reviewed visible-surface observer to independently evolved trajectories. Only
after that corpus, 16k/50k capacity and 240-step 50k correctness pass may the
full GPU-step timing run.
