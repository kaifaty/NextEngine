# NCGP9 complete 4k first-result evidence — 2026-08-31

## Identity

- source commit: `42ded232ae60d2b9a03477f56e7094ae1116bc49`
- source tree: `904507c3443546b17fcafd6cfc9e283a4110961c`
- source root: `915561ac7230855f211d4472409eca7cb62495cf3cc2f2aff42ff25a51964160`
- contract root: `4e60276e2c866b2858d1c5e5b22afd709694a7ba975697a32c17fd6d22ddefe3`
- clean Release binary: `dcc11c81f8e20a94ce991915cf8d31e82244b4fc348d61537b8ed371a6ac1bcf`
- host: RTX 3080, CUDA runtime/driver 13.3

## Frozen primary result

Exact command:

```text
nonlocal-corrected-cuda-complete-4k --complete-4k-corpus
```

The process exits 37 with `PHYSICS_REFUTED_BOUNDED` before step 1 of
`hydrostatic-hold` at `same_state_operator`.

- stdout SHA-256: `da52d96524bce6a618ef914a37e4eacfbd182d69b1928ea657a95dfc47b074dd`
- corpus result root: `0c32d56896e0362f76049c04cfe97b5bc6e491774b5586dc7fde201d7f37632b`
- scenario result root: `3f49af29727c1f925c4357a3ee2c7276bd681ee0b620dff6a4dc5d947f10a8fe`
- HVP relative L2: `0.32755949546463403`
- HVP cosine loss: `0.04731592731351629`
- active centres: GPU `1362`, CPU `978`
- first differing ID: `1714`
- density at that ID: GPU `1000.0001220703125`, CPU
  `999.99969942743689 kg/m^3`
- density relative RMSE / maximum: `2.4013923097354017e-7` /
  `9.174761784151997e-7`

The density field is close, but the exact pressure kink converts the small
binary32 summation error into a different active set and a large HVP error.
Dam break, orifice, 16k/50k and performance remain `NOT_RUN`.

## Frozen pressure-only discriminator

Exact command:

```text
nonlocal-corrected-cuda-complete-4k --diagnose-4k-operator-pressure
```

- stdout SHA-256: `f353b0e811470b711dfa5ddb78df4ef33e38ce7edeea7d1263c4ef6b5b066bf7`
- result root: `bd2e555270f5cbf2ef16ddae4e8c0dc9274a2a50aad10f818e8c936389157482`
- pressure-f64 HVP relative L2: `5.450372290066293e-7`
- pressure-f64 cosine loss: `1.4587528233966918e-13`
- active centres: GPU `978`, CPU `978`, exact ID signature

This selects the already predeclared narrow pressure-f64 discriminator for a
separately frozen corpus. It does not authorize full binary64 state, tolerance
changes or timing.

## Retained controls

On the same source checkout, the current graph/operator/solver, corrected
profile, compensated graph, boundary, transaction, physics, product gate,
retained bulk and visible-surface self-tests all exit 0/PASS. The historical
standalone NCGP3 intermediate target has an unrelated `-Werror=unused-function`
build regression after later profile code was added; the current product-gate
target compiles and executes those retained paths.
