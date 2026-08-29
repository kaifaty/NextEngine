# NSR3-B4C4M0 workspace-reuse diagnostic evidence

Status: `PASS / RETAINED_ACCEPTED_WORKSPACE_DESIGN_AUTHORIZED`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --workspace-reuse-diagnostic-self-test
```

Two parent-gated reports executed concurrently and are byte-identical:

```text
status                 PASS / MEASUREMENT_ONLY
raw JSON + LF          4e2741dd41348304304603c5b80c803f133fef0ebdf9d03aed4e14af4ceccf50
raw JSON without LF    479dad60d89f3c6cec0555c9ee364f16644fafdbe4ae55966dee6b357d1bb38e
semantic result        4d7629015292ac8a3798b67175075bc37f32091153b72b179c725a90f74e91c7
wall time              114.10 s / 113.37 s
CPU utilization        308% / 307%
maximum RSS            16,884 KiB / 15,924 KiB
parent B4C3MC1 exact    true
```

The isolated diagnostic passes in `1.17 s` at raw-with-LF
`a51c413a69f365464b349f063f42f02796d3c2bcb2079b279d2a8a1b6c2a8501`,
raw-without-LF
`7bd7ab05cd2c591fa5ceec4462e37a061135ddc9b35bfb7372472b9ca849ff8e`
and the same semantic result.

## Query lifecycle

Recorded and record-disabled transactions have exact physical state, work,
trajectory/ledger roots and query-chain roots. Recording itself is therefore
non-perturbing.

| Measurement | P1 supported | P2 released |
|---|---:|---:|
| workspace builds | `327` | `12` |
| unique states | `261` | `5` |
| duplicate builds | `66` (`20.2%`) | `7` (`58.3%`) |
| consecutive equal states | `64` | `4` |
| maximum multiplicity | `3` | `3` |
| trial builds | `195` | `0` |
| current builds | `63` | `3` |
| substep-diagnostic builds | `63` | `3` |
| macro-ledger builds | `2` | `2` |

P1's 195 trial states are real distinct optimizer work and falsify broad
memoization as the first optimization. In contrast, every one of the 63 P1
and three P2 completed private substeps releases its accepted current workspace
and immediately reconstructs the same state for `RUN_SUBSTEP_DIAGNOSTIC`.

## Structural duplication

| Measurement | P1 supported | P2 released |
|---|---:|---:|
| support samples | `544` | `1,216` |
| support records rebuilt | `177,888` | `14,592` |
| immutable-index lower bound | `1` build | `1` build |
| nested adjacency rows | `15,696` | `324` |
| nested adjacency records | `1,516,235` | `12,830` |

Static-support indexing and flat-only CSR affect almost every unique trial as
well as repeated states, so they remain high-value later stages. They are more
invasive than retained accepted-state ownership and must not be combined with
it before correspondence is isolated.

## Decision

Authorize B4C4A retained accepted-workspace design first. The predeclared work
effect for this one-macro discriminator is exact: remove the 63 P1 and three P2
substep-diagnostic builds, reducing totals to `264` and `9`, without changing
any solver, physical, canonical or ledger value. The optimized query transcript
needs a new policy identity; historical query roots remain preserved as the
legacy control rather than falsely required to equal a transcript with omitted
queries.

Do not select generic state memoization. Keep immutable support indexing and
flat-only CSR as separately attributable B4C4B/B4C4C candidates after B4C4A.
B4D, nominal execution, CUDA, runtime and production remain blocked.
