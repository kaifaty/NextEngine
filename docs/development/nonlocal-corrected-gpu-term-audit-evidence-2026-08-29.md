# Nonlocal corrected GPU term audit evidence — 2026-08-29

| Field | Value |
| --- | --- |
| Research ID | `NCGA0` |
| Contract | `docs/plans/nonlocal-corrected-gpu-audit/00-corrected-term-correspondence-contract.md` |
| Contract SHA-256 | `71538590e6a6dfa8f2a7c55ffe02ccc91785be13e280d5ab6b997851a2e29bc2` |
| Result | `CORRECTED_TERM_CORRESPONDENCE_SUPPORTED_BOUNDED` |
| Product status | `REPORT_ONLY`; SPEC-38 and ADR-076 remain `Proposed` |
| Host | Linux x86-64, NVIDIA GeForce RTX 3080, compute capability 8.6, driver `610.43.02`, CUDA compiler/runtime `13.3.73` / `13030` |
| Build directory | `/tmp/nextengine-corrected-gpu-audit-wfVEtI` (outside Git) |

## Claim and conclusion

The separately implemented strict-`f32` CUDA evaluator matched the independent
host `long double` evaluator on all nine frozen scalar/pair fixtures. The largest
observed normalized mixed error was `3.6787060029602878e-07`, below the frozen
`2e-5` limit. All pair forces closed within the stricter `2e-7` zero bound.
Ten cold allocation/execution repetitions were byte-identical.

The source-shaped gradient negative control rejected all three mandatory cases:
`kernel_inner`, `kernel_outer`, and `compression_above_rest`. The apparatus
therefore distinguishes the old missing-chain-factor identity from the corrected
one.

This supports only corrected local term correspondence on this frozen profile and
device. It does not establish neighborhood/indexing, local-matrix, `3x3` solve,
SISSM, multi-step trajectory, performance, runtime authority, or product-ready
water.

## Frozen inputs and outputs

| Artifact | SHA-256 |
| --- | --- |
| Executable | `84915e5c5a2c2f000fa37993341879886fcb4e1ad5f4170f3c6f4e01f539e6b6` |
| Header/DTO | `f4675217b9df8e3bcb913bd7d3332da8a09533b526b6b376d46e07c759602b99` |
| Host reference | `150121df18c537252eacac0012e7bf12eb7be12d0e0eeac9e43b12b28cdd6b3f` |
| CUDA evaluator | `433803b880058d2f0aa4c4fa6207cf784468338647f25745abec637fe221e8b9` |
| Harness/fixtures | `9af84791d1bef1d7e3991b564e9fd0a8d55161d734c8fb557edb3d9c8d6c3d57` |
| Process stdout | `5cbf70a7a26c428d1eee0ee0c3e85aa9fd935e0079a48bb78f6155f035c77d41` |
| Ordered fixture root | `046e111e760788e67d5a1ae04448a4a9d606c07974f386d9b046fe874c089a87` |
| Corrected CUDA payload root | `a915ddbcddfd0cc607db83b734402e3744784e30014be20b0d16aae78f9b198b` |
| Source-shaped control root | `f92b85b6fef687c2ef8e39578ba99736546a1585d657e4f365bb60e15a3df4a2` |

Two independent process invocations produced the exact stdout hash above. Each
invocation included ten cold CUDA allocations and executions.

## Per-fixture results

| Fixture | Maximum mixed error | Corrected | Source-shaped rejection required/observed |
| --- | ---: | --- | --- |
| `kernel_inner` | `1.0721831557097558e-07` | PASS | yes / yes |
| `kernel_outer` | `3.6787060029602878e-07` | PASS | yes / yes |
| `compression_above_rest` | `2.918080013748059e-07` | PASS | yes / yes |
| `compression_below_rest` | `0` | PASS | no / no |
| `bulk_viscosity` | `1.6754489784442939e-07` | PASS | no / no |
| `shear_viscosity` | `9.6970803442619768e-08` | PASS | no / no |
| `surface_repulsive` | `7.4623827533981668e-09` | PASS | no / no |
| `surface_attractive` | `4.0655643573916933e-09` | PASS | no / no |
| `surface_outside` | `0` | PASS | no / no |

## Commands and checks

Fresh configure and build:

```bash
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B /tmp/nextengine-corrected-gpu-audit-wfVEtI \
  -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build /tmp/nextengine-corrected-gpu-audit-wfVEtI \
  --target nonlocal-corrected-cuda-terms -j 12
```

The target compiled for `sm_86` with `--fmad=false --prec-div=true
--prec-sqrt=true --ftz=false`; host reference compilation used
`-ffp-contract=off -fno-fast-math`. `-Wall -Wextra -Wpedantic -Werror` passed.

Primary command:

```bash
/tmp/nextengine-corrected-gpu-audit-wfVEtI/nonlocal-corrected-cuda-terms \
  --self-test
```

Result: PASS twice; both stdout SHA-256 values were
`5cbf70a7a26c428d1eee0ee0c3e85aa9fd935e0079a48bb78f6155f035c77d41`.

CUDA diagnostics:

```bash
compute-sanitizer --tool memcheck --error-exitcode=86 \
  /tmp/nextengine-corrected-gpu-audit-wfVEtI/nonlocal-corrected-cuda-terms --self-test
compute-sanitizer --tool initcheck --error-exitcode=86 \
  /tmp/nextengine-corrected-gpu-audit-wfVEtI/nonlocal-corrected-cuda-terms --self-test
compute-sanitizer --tool synccheck --error-exitcode=86 \
  /tmp/nextengine-corrected-gpu-audit-wfVEtI/nonlocal-corrected-cuda-terms --self-test
```

Result: all three reported `ERROR SUMMARY: 0 errors`.

Non-regression controls from the same fresh build:

- retained source-shaped CUDA gather/pointer-swap/specialized path: `11/11`
  PASS, with exact reused-instance output in all 11 cases;
- NPR1-B term control: expected exit 1 and
  `KERNEL_SOURCE_GRADIENT_MISMATCH`, result root
  `069aff09f7919fae33f86b2c86cb4a39d654188a15bd8c14868a9dfd5237e5c9`;
- FCR0 corrected host formula control: PASS, result root
  `ea5c4b423ce47b89699150dd17086ac63ccc4e4a59ec234a829a066022b54412`.

## Hypothesis update

| Hypothesis | Result | Evidence update |
| --- | --- | --- |
| H1 corrected terms translate faithfully to strict CUDA `f32` | supported in scope | 9/9 pass, exact cold repeats, no sanitizer finding |
| H2 a local CUDA sign/coefficient/branch defect remains | not observed in scope | every frozen branch passed its independent host comparator |
| H3 the prior decisive failure is shared source mathematics | strengthened, not globally proved | old path remains internally correspondent; old formula control fails; corrected local CUDA passes |
| H4 apparatus cannot distinguish identities | refuted in scope | all three mandatory source-shaped cases reject |

## Smallest next discriminator

Freeze a new contract for corrected neighborhood construction plus local
energy/source/matrix assembly on tiny fixed particle sets. Keep the corrected
term target immutable as its positive boundary and retain one deliberately
permuted-neighbor or source-shaped negative control. Do not port the full solver
or run performance experiments until that boundary closes.
