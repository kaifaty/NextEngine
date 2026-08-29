# NSR3-B4EP10SIRDIREQ4 executor-adjusted CPU evidence -- 2026-08-22

Status: `FAIL / ADJUSTED_CPU_STABILITY / DEDICATED_HOST_REQUIRED`

## Result

All three fresh serialized Q1 processes pass exact semantics and accounting.
Each retains identity `6518ed98...2964`, result `e5ddff76...14f`, SIRDI and
SIRDIR results, five physics roots, 4,089 region intervals, 32,712 active
worker intervals and zero clock failures. All stderr files are empty.

The derived integer accounting is:

| Run | Transaction T | Region R | Active A | Outside O | Residual X | Adjusted E |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 45,276,157,959 | 35,227,864,711 | 16,914,056,558 | 10,048,293,248 | 18,313,808,153 | 26,962,349,806 |
| 2 | 44,219,279,944 | 33,939,040,243 | 17,100,604,659 | 10,280,239,701 | 16,838,435,584 | 27,380,844,360 |
| 3 | 43,884,993,697 | 33,849,611,334 | 16,516,089,349 | 10,035,382,363 | 17,333,521,985 | 26,551,471,712 |

Every run satisfies `A <= R <= T` and `E + X == T` exactly. Range ratios are:

| Metric | Observed | Gate | Result |
|---|---:|---:|---|
| executor-adjusted E | `1.0312364097` | `<= 1.03` | FAIL |
| outside-region O | `1.0243994029` | `<= 1.05` | PASS |
| active-worker A | `1.0353906604` | `<= 1.05` | PASS |
| transaction T | `1.0317002270` | diagnostic | -- |
| region R | `1.0407169631` | diagnostic | -- |
| excluded residual X | `1.0876193374` | diagnostic | -- |

The adjusted metric misses by about 0.124 percentage points. The threshold was
frozen before execution, so this is FAIL; no rounding, outlier deletion,
repeat or gate relaxation is admissible.

GNU external/internal CPU ratios are
`1.0062691283 / 1.0061222176 / 1.0064943909`, median `1.0062691283`, inside
the required `[1.00, 1.05]`. Diagnostic wall is `6.34 / 6.13 / 6.12 s` and RSS
is `91,120 / 90,948 / 91,816 KiB`; neither receives speed credit.

## Reproducibility

No source change was made. The executed SIRDI/Q1 binary has SHA-256
`0b1899192fe22726ef23c192bd68d35b15797db9ea48c9910184e90affe2ff30`,
size 4,425,408 bytes and ELF Build ID
`07c0788d0c2f34695dc40c2a61ffae7ea5f6abcc`. The Release
`compile_commands.json` remains
`79e135ca1ecbb1d76a862c944b574472ecce6fe4141684590f2da33488f03594`.

Source SHA-256 values remain
`997894ddbb802357ccfe3945b828a5368d72ecc3f239d4c2f2418272777f2aab`,
`7148e864634909e03dfcc929d0fb5f71f828973c5dacecac295054ab858d4dd5`
and `a5282a1944a1da4a63c7391cf6c3b5538b3beb38cbcfb8d57678ac01af42808e`
for `boundary_reference.cpp`, `boundary_reference.hpp` and
`formula_reclosure_main.cpp`. Raw reports and GNU-time records are under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sirdireq4.kCTNH6`.

## Decision

Close Q4 FAIL. The shared desktop host does not qualify whole-process or
executor-adjusted CPU for candidate selection. SIRDI remains selected; Q3
remains closed and must not be rerun.

A dedicated/quiescent performance lane is now a prerequisite for another
candidate A/B. Fundamental numerical, structural and GPU research may continue
with exact/cost evidence, but cannot claim CPU/wall improvement from this host.
No B4E2, broad-corpus, runtime/GPU/schema or production authority is granted.
