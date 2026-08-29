# NSR3-B4E2D7R19R59 row-local enclosure evidence

Date: `2026-08-25`

Status: `PASS / ROW_LOCAL_ENCLOSURE_CERTIFICATE_CANDIDATE / ROLLBACK ONLY`.

Implementation commit: `219987a8`.

Frozen V2 identity SHA-256:
`bab527b0018f4265c8fedce046a07615dbd4fe95687eb6883487952bea14fb9e`.

## Result

R59 reproduces the exact R58 public result and every captured global-degree
upper bit-for-bit, then specializes the unchanged operation-count proof from
the global maximum adjacency degree to each row's exact flat-adjacency degree.

```text
rows                                      6000
degree minimum / maximum                 44 / 113
distinct degrees                              53
rows below maximum degree                   5992
current global-degree positives               234
row-local-degree positives                       0
current positives at maximum degree              0
strictly improved rows                        5992
closed current positives                       234
```

The current worst upper is `1.3678206846699582e-24` at row 2522, whose
actual degree is 102. The row-local worst upper is strictly negative,
`-1.9354174334860891e-23`, at row 4930 with degree 111. Frozen precedence
therefore selects `ROW_LOCAL_ENCLOSURE_CERTIFICATE_CANDIDATE`.

The candidate changes neither `gamma_factor` nor operator arithmetic. It only
replaces the densest-row work count with the exact conservative work count of
the row being certified; radius-skipped slots remain charged. The old global
upper is independently recomputed in the same binary64 order and must match
the R58 vector and aggregate before the local result is admitted.

## Roots

```text
source upper       9e8ad22488a6a49f66f7d2734c9573692a807cbfdc6c3d09874b24a590151ff8
current active     cda544504d01d5a92254b6c9cd657151a8e3cb5f4f7a3cc09ad9fcb38fb6f8fc
local active       82086595b4ed35abd2bf7f7cf6107359eaa9fe92be80df7aad1174dbf043bdb3
degree histogram   3d4a798f7d9ea0fc71816091e786778f2368d8c0ff944b45200b004fa09dabdd
row comparison     5253b1c5a41fbb798cbe25df4a9c16afe663e4167b332aa9e0114c99b6de82de
route               4affaebb6efc295574fae4b15c0677d54f65b92a6d073ec04c2d81d7fc949a19
semantic            e97d68233ce067e7fb948d6cd38bf79ecafb403e16961fadb9a3e9bcb0ce071d
```

The 6000-row scan performs zero new JVP, VJP, pair, quad, solver or binary128
passes. One moved workspace is built and released; rollback is exact.

## Pre-PASS source correction

Early nominal executions correctly stopped at `ROW_LOCAL_SOURCE_REJECTED` and
received no scientific credit. The implementation expected
`...492e6d798...` for the R51 master while the captured and historical root is
`...492e6c798...`. A 15-gate source mask isolated the last check; before/after
capture snapshots proved the 64-character root stable. Only that mistaken
expected hex digit was corrected. The frozen R59 identity did not encode the
mistyped digest, and no row scan ran before the source gate closed.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r59-final-a.6aModU
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r59-final-b.iWbXYq
binary SHA-256 c6c239018fada53f7a35679d103b6a10c410de376982fc60b69d3fc5ab9e5b29
size           8322752
ELF build-id   4c3d13e4aec35aab83e87711b631d0a82f581daa
stdout bytes   2499
stdout SHA-256 495bbd213784c4e4f7500c33e2be32c05d41b859145282770364a8fc3a143a4b
```

Both clean Release binaries and outputs are byte-exact; both processes exit
zero. Wall time is not performance evidence. Runtime and production authority
remain false.

R58 remains byte-exact at stdout SHA-256
`8078c06230d6436253df966eb617bee202237143a1cf5def0e1dac49028ec257`.

## Consequence

Preserve R59 as a private certificate candidate and end solver-depth,
fixed-point and generic EFT escalation for this witness. Research R60 as an
independent validation boundary: require row-local upper to dominate an
independently recomputed full binary128 forward enclosure on every row; add
dense and adversarial topology/degree controls, including a forced undercount
rejection. Only after that validation may a separate integration contract
replace the research-only certificate path or authorize restoration exit.
