# NSR3-B4EP6 coefficient residual-attribution evidence -- 2026-08-22

Status: `PASS / WORKSPACE_EVALUATION_BASE_TAPE_SELECTED / DESIGN_ONLY`

## Result

The exact-output B4EP5 profile records 836 ten-millisecond samples. Complete
workspace construction owns 4.66 s while exact pressure-tape HVP application
owns 3.45 s. Workspace leads by `1.350724638x`, clearing the frozen `1.20x`
top-level rule.

Within workspace, evaluation plus base pressure-tape construction owns 2.73 s,
topology/filter/CSR owns 1.09 s and coefficient population owns 0.62 s.
Evaluation/base tape leads topology by `2.504587156x`. B4EP6 therefore selects
only B4EP7 evaluation/base-tape research/design. It authorizes no
implementation, solver-policy, parallel, GPU, runtime, B4E2 or production
change.

## Build and correspondence

| Field | Value |
|---|---|
| B4EP6 identity | `75b8a8b7ba137674091413e8f0eb7ace46f32a2c09059de260c06c36955db24d` |
| implementation | `7263d8929491b66ea94eb74713fab4c136efe5be` |
| compiler / profiler | GCC 15.2.0 / GNU gprof 2.46 |
| flags | `-O3 -DNDEBUG -g -pg -ffp-contract=off -fno-fast-math`; link `-pg` |
| compile commands | 15,261 bytes; `36df09834f855876778697b3cc89c95633721a6c4d685cecbd766f5cbb1bdf6b` |
| executable | 37,132,088 bytes; `8513ae29eaceff8d661b38935930c3332e53b7be2cc9180889bcb2eb909ca805` |
| GNU Build ID | `91e1403c98cb96ad365027e7cebe15668cf9c992` |

The process exits zero with empty stderr. Stdout is exactly 6,729 bytes with
SHA-256
`dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b`
and semantic result
`5cd61e3eb82f6a3cedfe4d7d8e39cbb5aca65c6e00c5ef9cd9e7555a71b9bf23`.
It is byte-identical to the selected B4EP5 Release report.

Instrumented wall/RSS are 11.61 s and 62,720 KiB at 99% CPU. These values
prove normal completion only; they are not compared with Release throughput.

## Profile artifacts

| Artifact | Size | SHA-256 |
|---|---:|---|
| `gmon.out` | 1,406,584 bytes | `2070d6775084c924e1cba7730f2e9f7020c607c2849eb840c4d218ef8dd48758` |
| flat profile | 50,638 bytes | `685eaf1f46c4107e9ab613efeb074e747b348d08c421c52346a113f3d90de4e1` |
| call graph | 349,906 bytes | `ab32c0f280460fdb9bfc84605216df25f2bcd7d5a1ddad771810458e5fa02020` |

B4EP6 evidence attestation root:
`e7690846477ab60a753510926f146dcb5d1eaa6661db31b6d164d6bf5cfa2945`.

External artifacts remain outside Git:

- build: `/home/kaifaty/.cache/nextengine/external/build-nonlocal-b4ep6.a2CueL`;
- run: `/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep6.cu7aFV`.

## Attribution

The command's main call tree owns 8.25 s. A separate spontaneous `_init`
bucket owns 0.11 s and is not assigned to a solver category.

| Top-level category | Inclusive CPU | Share of 8.25 s |
|---|---:|---:|
| complete workspace | 4.66 s | 56.484848% |
| exact pressure-tape HVP application | 3.45 s | 41.818182% |
| residual control/publication | 0.14 s | 1.696970% |

Workspace splits as follows:

| Workspace subcategory | Inclusive CPU |
|---|---:|
| evaluation + base pressure tape | 2.73 s |
| topology/filter/CSR | 1.09 s |
| coefficient population | 0.62 s |
| workspace-local residual | 0.22 s |

`evaluate_joint` contributes 1.98 s direct plus 0.36 s of its kernel child;
base pressure-tape construction adds 0.39 s. Cached-superset filtering is
0.60 s direct plus 0.38 s adjacency finalization; the one superset build adds
0.07 s and the one canonical parent topology path adds about 0.04 s.

Coefficient population is inlined into `build_joint_query_workspace`.
The call graph exposes 85,716,150 `weight_gradient` calls at 0.34 s direct
plus a 0.14 s kernel child, and another 85,716,150 direct kernel calls at
0.14 s. Source and the exact B4EP5 work counters establish that the second
folded path is `weight_second`. This mapping is an attribution inference;
the reported 0.62 s does not include unseparable workspace self time.

The flat profile displays 544 `apply_joint_pressure_tape` calls while the
candidate transaction reports 459 HVPs. Caller structure includes optimized
or folded validation/control paths, including 84 calls displayed directly
under the step solver and one report-level validation. Therefore function
call labels are not used to redefine the frozen physical counter; inclusive
sample ownership of the common apply routine is used only for cost ranking.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep6-evidence|v1|identity=75b8a8b7ba137674091413e8f0eb7ace46f32a2c09059de260c06c36955db24d|implementation=7263d8929491b66ea94eb74713fab4c136efe5be|stdout=dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b|result=5cd61e3eb82f6a3cedfe4d7d8e39cbb5aca65c6e00c5ef9cd9e7555a71b9bf23|binary=8513ae29eaceff8d661b38935930c3332e53b7be2cc9180889bcb2eb909ca805|compile=36df09834f855876778697b3cc89c95633721a6c4d685cecbd766f5cbb1bdf6b|gmon=2070d6775084c924e1cba7730f2e9f7020c607c2849eb840c4d218ef8dd48758|flat=685eaf1f46c4107e9ab613efeb074e747b348d08c421c52346a113f3d90de4e1|callgraph=ab32c0f280460fdb9bfc84605216df25f2bcd7d5a1ddad771810458e5fa02020|samples=836x0.01|categories=workspace4.66;hvp3.45;control0.14;unassigned0.11|workspace=filter1.09;eval2.73;coeff0.62;residual0.22|ratios=1.350724638;2.504587156|decision=workspace-evaluation-base-design
```

SHA-256:
`e7690846477ab60a753510926f146dcb5d1eaa6661db31b6d164d6bf5cfa2945`.

## Decision

Select `WORKSPACE_EVALUATION_BASE_TAPE_DOMINANT`. B4EP7 must first audit
whether density/evaluation and pressure-tape radius/compression work can be
fused without changing pair order, floating-point grouping, tape ownership or
old command bytes. If no exact mechanical discriminator exists, route to
scoped phase timing or stop; do not stack another cache speculatively.
