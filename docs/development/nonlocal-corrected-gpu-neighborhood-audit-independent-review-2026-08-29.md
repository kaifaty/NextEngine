# Nonlocal corrected GPU neighborhood audit — independent review

| Field | Value |
| --- | --- |
| Research ID | `NCGA1` revision 1 |
| Verdict | `GO / SUPPORTED_BOUNDED` |
| Load-bearing findings | None |
| Repair used | None |
| Reviewed candidate | `083e11644d1a4639f1fc9a884d9b2dd24808f7b4` |
| Candidate tree | `68a99b8437bdf1e41fb86712afe949bbfc5ad188` |
| Review mode | Fresh detached read-only worktree; author evidence/task-state/roadmap withheld until verdict |
| Product status | `REPORT_ONLY`; SPEC-38 and ADR-076 remain `Proposed` |

## Verdict

NCGA1 revision 1 satisfies the frozen exact finite GPU neighborhood/index
correspondence contract. The reviewer found no load-bearing defect and used no
repair/re-review budget.

This verdict permits only a new separately frozen local energy/source/matrix
assembly audit. It does not establish private-float assembly, factorization,
the nonlinear solver, trajectories, stability, visual quality, performance,
runtime integration, canonical GPU authority or product-ready water.

## Identity closure

- architecture snapshot: `7d368b69e8a443cdace20473451a570e3c79ef4d`;
- frozen contract commit: `8855fbc7b713a883a7fdc281c87a7bd829388ce1`;
- implementation commit: `2781c2a314a23ce2000a901ca3ed0445b1caa03c`;
- candidate parent: `2781c2a314a23ce2000a901ca3ed0445b1caa03c`;
- parent-to-candidate `--binary --full-index` diff SHA-256:
  `0f4e8db3f9e897bfa64701b768e17bc5554942d4d9b936375a5927c99f91183a`;
- contract raw SHA-256:
  `fb8e9235cf0a1bc4c78b2bab245a46d467d906f4fa101285cd14c6f37c5e5b65`.

The reviewer independently reproduced all candidate source hashes recorded by
the author. The detached worktree remained clean.

## Independent code audit

- The target links only the NCGA1 CUDA candidate, harness, independent host
  reference and SHA-256 source. Historical `cuda_baseline.cu`, `oracle.cpp` and
  NCGA0 force oracles are absent.
- The host reference separately implements admission, mathematical floor
  division, packed signed cells and direct checked `O(N²)` integer membership.
- The CUDA path computes keys on device, sorts owner IDs, performs a stable
  cell-key radix sort, probes 27 cells through device range lookup, scans row
  counts and fills sorted `SampleId` CSR on device. Host code only copies and
  compares the result.
- Installed CUB 3.3.4 explicitly guarantees `DeviceRadixSort` stability. The
  preceding unique-`SampleId` sort therefore makes equal-cell storage exact in
  `(cell key, SampleId)` order.
- The comparator covers complete cell storage and CSR. Its invariant pass also
  checks strict owner/row order, self count, symmetry and maximum degree.
- Arithmetic is safe across the admitted domain: every signed axis square is
  below `INT64_MAX`, the three-axis unsigned sum is below `UINT64_MAX`, packed
  adjacent cells remain in 21 bits and pair capacity is at most `256²`.

## Independent reproduction

Environment: Linux x86-64, RTX 3080 `sm_86`, NVIDIA driver `610.43.02`, CUDA
compiler `13.3.73`, runtime/driver API `13030`, GCC `15.2.0`, CMake `4.2.3`.

- two clean external Release builds: PASS;
- reviewer binaries byte-identical:
  `8aa1f2e5d29a623814971f8a5f3e29ae577e2a16596e874ab6238f94f7988af1`;
- two executions return zero with byte-identical stdout:
  `0a89d92ceff561f6e8f128299ead356abaa7820cd05025fd2c1f6b3d85906b51`;
- fixture root: `9b7f2278b0af6f90f00867578a5f3f0229ae982e77aa5fc31827366256f18269`;
- oracle/candidate graph root:
  `c8b905f78a161fbb1fa40a4bd05c7458e9d47be98343e934f3c1e12c11ca01e8`;
- work root: `b01eb736e6b0f6afcbb127826e9a87dd68a42cff11bbddbf0bb30d19d3783f63`;
- negative-control root:
  `282e69554503ad3f724927bb58f40f6e600382bb221ad801ec0f23b32f71d190`.

All eight positives, four common-comparator negatives, input permutation,
admission controls and ten cold repeats passed. The reviewer wrote a separate
exact Python reconstruction that matched every per-fixture/aggregate graph
root and every work count/hash.

`memcheck`, `initcheck` and `synccheck` each exited zero with no errors. NCGA0
remained PASS with stdout
`5342fb400c07d429645f66ebb1f596d3e1a2fcd69e16dc5f8d6640c700cfbcd7`.
The historical eleven-case CUDA tiny control remained PASS with all cases and
reused-instance comparisons exact.

## Claim ceiling

Broader ProductChecks remain `NOT_RUN` because this is read-only Proposed
research and changes no production surface. The next admissible action is a new
contract for local energy/source/matrix assembly over a small frozen graph; no
full solver or performance inference follows from NCGA1.
