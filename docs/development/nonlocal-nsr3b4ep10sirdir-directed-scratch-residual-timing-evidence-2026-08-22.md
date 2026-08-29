# NSR3-B4EP10SIRDIR directed scratch residual timing evidence -- 2026-08-22

Status: `PASS / SOURCE_LOCAL_EVALUATION_SETUP_DISCRIMINATOR_SELECTED`

## Result

All three fresh serialized processes preserve B4EP10SIRDI semantic result
`b4f847cb...777e9`, B4EP10SII result `f7b1542f...25fb2`, correspondence
`1e4bedbb...08a35d` and all five frozen physics roots. Every run reports one
transaction, 226 topology calls, 226 evaluations, 459 HVPs, 685 scratch reuse
calls, 4,089 executor regions and 261,696 logical partitions. Reuse retains
the exact 454,936,226 requested slots, 374,945,086 active writes, 670,229
growth slots, one release and zero live buffers/failures.

The duration-free semantic result is byte-stable at
`1f66ab3c...5992e`. The four disjoint category shares are:

| Category | Run 1 | Run 2 | Run 3 | Median | Range |
|---|---:|---:|---:|---:|---:|
| topology | 0.291072 | 0.289194 | 0.282815 | 0.289194 | 0.008257 |
| source-local | 0.444580 | 0.447174 | 0.454877 | 0.447174 | 0.010297 |
| target fold | 0.190034 | 0.189497 | 0.191685 | 0.190034 | 0.002188 |
| control | 0.074315 | 0.074135 | 0.070623 | 0.074135 | 0.003692 |

Every range passes the frozen `0.05` limit. Source-local owns 44.72% and
leads topology by `1.546276x`, so it passes the `>=0.20` share and `>=1.20x`
leader route. Median top-level evaluation/HVP/residual shares are
`0.340054/0.298010/0.072742`; their ranges are at most `0.009447`.

Median executor orchestration is `0.014180` and imbalance `0.105622`, both
below `0.15`. Persistent-region and partition-balance work remain rejected.
The measured transaction medians at 4.452394944 s; the three containing
processes take 4.61/4.74/4.78 s. These instrumented durations grant no speed
credit and are not the uninstrumented 4.290 s throughput result.

## Interpretation

Scratch reuse changes the residual materially: source-local falls from the old
58.77% share to 44.72%, and the old directed group no longer has a clear
internal lead. In the selected candidate the median transaction shares are
15.88% directed, 14.62% setup, 7.45% HVP compression and 6.81% other local
scalar work. The largest individual source-local subphase is
`evaluation_setup` at 14.58%.

Code inspection shows that `evaluation_setup` combines a full directed-index
validation scan with capacity arithmetic and value-initializing seven output/
temporary vectors. Those are distinct hypotheses. The next discriminator
must split validation from buffer preparation before either a certificate or
another high-water buffer is implemented.

## Build and artifacts

- implementation commit: `225af12de50282734bfaee1c267031d07e96bd28`;
- executable SHA-256: `cbc726f1...796b`, size 4,326,768 bytes;
- Build ID: `9f5a4c5bff721718dafb2f23c5cacc60e20efeff`;
- `compile_commands.json` SHA-256: `78defa02...a66`;
- raw metrics SHA-256: `e9503fee...6955`.

Source SHA-256 values are `cecc0c3e...b4c2` for `boundary_reference.cpp`,
`f5175de2...13d2` for `boundary_reference.hpp` and `c46b3c5c...24a2` for
`formula_reclosure_main.cpp`. Raw outputs remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sirdir.9lJmA2`.

The Release build passes the repository `-Werror` policy. The refactored old
B4EP10SIR command retains semantic result `62a2205d...10aa3`; the old
B4EP10SIRDI stdout remains byte-exact at `539f1ec5...e7e7`.

## Decision

Close B4EP10SIRDIR as PASS. Research one evaluation-setup discriminator that
separates structural validation, capacity/control and buffer preparation;
change no allocation, certificate or arithmetic yet. B4E2, broad corpus,
runtime, CUDA/GPU, schema and production remain blocked.
