# PS-2 REALIMPACT Shell Plate bounded-range E2 pilot — 2026-08-28

| Field | Result |
| --- | --- |
| Status | `FOUR_REALIMPACT_E2_OBJECTS / DISTINCT_GLASS_SHELL_GEOMETRY / FALLBACK_ONLY` |
| Scope | Published current-only `E2TransferResponse`; no E3, split, admission, quality or production credit |
| Frozen profile | `shell-plate-row-0-v1` |
| Acquisition report | `1a03ccbacb1a53a1b0d0e4ae21b7ca6869c60d147cae7079f5097d8f1d18a11b` |
| Inventory manifest/report | `6b19612a315671e23b604b900071362352720fe9fdf260cc2c29414419d5bb8d` / `68a9cee2812143b42d3ac72b6497537f824c31b16a5a3eeb3eb851d287613268` |
| External roots | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-shell-plate-e2-range-v1-final/` and `-repeat-final/` |

## Question and decision

`51_ShellPlate` is a useful fourth REALIMPACT discriminator. It adds a broad,
shallow shell geometry whose measured transfer differs structurally from the
two goblets and Blue Bowl. The row is admitted only to the typed development E2
inventory and remains `FallbackOutOfDomain`.

The material label is not inferred from the word `ShellPlate`. The
[REALIMPACT paper](https://arxiv.org/abs/2306.09944) states that the 50 physical
objects were purchased from ObjectFolder. The official
[ObjectFolder-Real table](https://objectfolder.stanford.edu/objectfolder-real-download)
identifies numeric object 51 as `Fruit_Bowl / Glass`; REALIMPACT publishes the
same numeric object under its alternate archive label `51_ShellPlate`.
Composition and material revision are still unavailable and are not claimed.

## Bounded source and exact result

The profile validates the official archive HTTP identity, EOCD, central
directory, every required ZIP member, six decoded NPY arrays, the decoded mesh
and one fixed compressed prefix of the transfer array.

| Measurement | Frozen value |
| --- | --- |
| Archive | `2,342,734,004` bytes; ETag `6433e570-8ba348b4`; modified `2023-04-10T10:31:12Z` |
| Central directory | offset `2,342,732,699`; `1,283` bytes; SHA-256 `b1633df3edddefac7a4a35dc4f8da5d00d14fc41c56d60d7d44a94dc91ee9645` |
| Range payload | `1,700,993` bytes, `0.072607%` of the archive |
| Transfer array | `3000 x 210424`, float32, 48 kHz; ZIP CRC32 `7cdd814d` |
| Selected row | row `0`, mesh vertex `15341`, position `[0.06131311, -0.07580242, 0.02394472]` m |
| Listener | `[0.23, -0.04345, -0.91]` m; angle `0`; distance offset `0`; microphone `0` |
| Row | `210,424` samples, `4.3838333333333335` s; SHA-256 `e795d04f6bfe12414dd6499d2e29f2772f63ce1784f4d5f6ecdda681b0c31219` |
| Level | peak `198.3330535888672`; RMS `4.061807743903193` |
| Audition WAV | SHA-256 `6f520b97420648ac8fe71592aed65d7272db41b490de1f5c406d337f77d61aae` |
| Acquisition metadata | SHA-256 `0847a18433bf3a3bb9586888cfd9ad9b90d4716c0294e501268ca00f23f69a15` |
| Provenance review | SHA-256 `66e572a1e69069907233172177662082bad7ebdb2bdc179be0efc32ef4c6cf25` |

The `48,070`-vertex mesh has SHA-256
`6c34b4350c2c3ce22722435d3baf7cc486a8a3a02b7da7d105910c811f79afd0`
and bounds `[-0.13006133, -0.1527149, -0.00840476]` to
`[0.16750934, 0.14633709, 0.03230974]` metres. Its approximately
`298 x 299 x 41` mm extent is distinct from the roughly `92 x 92 x 156–165` mm
goblets and `160 x 161 x 85` mm Blue Bowl. The row-zero coordinate equals mesh
vertex 15341 exactly.

## Reproducibility and validator controls

Two independent online acquisitions and two V2 inventory audits are
byte-identical. The typed adapter now freezes four exact REALIMPACT pilots. A
focused negative control replaces the Shell Plate transfer hash with the Blue
Bowl hash and is rejected as not matching the frozen `51_ShellPlate` pilot.

The generator now reads `material_family` from each frozen profile instead of
hard-coding `glass` globally. This does not change existing profiles, but it
prevents a later non-Glass object from silently receiving Glass identity.

The adapter grants only geometry, impact/listener position, object identity,
real-recording identity and force-deconvolved transfer. Raw force-profile
bytes, material-composition revision, repeat identity and support-fixture
revision remain explicitly unavailable.

## Consequence and next action

REALIMPACT now contributes four typed E2 objects, but E3 Glass coverage remains
`9/16`, reject parents remain `38/35`, and every entry remains in `dev`. This
pilot does not create a seventh E3 project, a Glass target group, a partitioned
holdout or a `Pass` decision.

The next package should continue exact-download discovery for the seven missing
Glass E3 target groups. When no stable E3 route is available, audit
`60_SkullCup` as the next numbered ObjectFolder Glass E2 candidate, preserving
the same bounded-range, material-binding and fallback-only rules.
