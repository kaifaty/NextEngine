# Nonlocal corrected GPU term audit — independent revision-2 re-review

| Field | Value |
| --- | --- |
| Research ID | `NCGA0` revision 2 |
| Verdict | `GO / SUPPORTED_BOUNDED` |
| Reviewed candidate | `22bf11adfeedc602c4fd25c460efece33ac4b298` |
| Reviewed tree | `abe89171472a7491f6179ecee409c21f9478f648` |
| Revision-1 base | `349f12e66be970276d91c96bda1f33fad1a9919e` |
| Revision-1-to-repair diff SHA-256 | `b234b72cdc597ef4ac67f8bed632620b8e94f8a76b7934dba15c66f21034ffe5` |
| Revision-2 contract SHA-256 | `10303b06727c703a2a958b3c6e49663a9b1b6ee7ad462e8f15cba267474d3e6f` |
| Reviewer Release binary | `04ed0f101b808119f52817aae6cb3762867c04294513bfe4d59f0589bb57c11a` |
| Exact stdout SHA-256 | `5342fb400c07d429645f66ebb1f596d3e1a2fcd69e16dc5f8d6640c700cfbcd7` |

## Verdict

The single permitted revision-2 re-review found no load-bearing defect. The
frozen claim is supported: corrected scalar and full-undirected-pair forces
correspond between the independent host and strict CUDA implementations on the
frozen RTX 3080/CUDA 13.3 profile.

This verdict does not apply to neighborhood/indexing, matrix assembly, local
solve, SISSM, trajectory, performance, runtime authority, canonical authority
or product-ready water.

## Independent closure

- Candidate/tree, revision-2 contract, revision-1-to-repair diff and all five
  repaired source hashes matched exactly.
- CUDA uses full-force coefficients `lambda,2*mu`; the deliberate adversary
  uses `lambda/2,mu`.
- The separate energy translation unit implements
  `mu*|P_t delta|^2 + lambda/2*|P_n delta|^2`, the frozen direction,
  `epsilon=h*1e-8`, first-endpoint-only central differencing and the correct
  `-F_i dot d` sign comparison.
- A separate 70-digit calculation reproduced the central and analytic
  derivatives to approximately `1e-63`.
- Both half-force fixtures failed both the component and energy gates; the old
  factor-of-two identity cannot pass the repaired harness.

## Reproduced execution

- Exact target: PASS twice, byte-identical stdout.
- Ten cold CUDA repetitions per process: byte-identical payloads.
- Roots: fixture `046e111e760788e67d5a1ae04448a4a9d606c07974f386d9b046fe874c089a87`,
  corrected `a915ddbcddfd0cc607db83b734402e3744784e30014be20b0d16aae78f9b198b`,
  energy `199335b485e68f18dd76472372177b47f4364c108947a4b2b2d7357c4245bac9`,
  source-negative `f92b85b6fef687c2ef8e39578ba99736546a1585d657e4f365bb60e15a3df4a2`,
  half-force `defab98e3bbd1363487bba0aca483f9b129a4dc9ce6df6f244294365c8c3ad7d`.
- Corrected energy errors: bulk `7.802745e-8`, shear `1.300229e-7` versus
  `2e-5`.
- Half-force errors: bulk `0.04034567`, shear `0.01617667`; both rejected.
- Compute Sanitizer `memcheck`, `initcheck`, `synccheck`: zero errors.
- Retained GPU: `11/11` PASS with exact reuse; NPR1-B: expected
  `KERNEL_SOURCE_GRADIENT_MISMATCH`; FCR0: PASS with root `ea5c4b42...`.

## Next admissible step

NCGA0 is complete. The next work, if authorized, is a new frozen experiment for
corrected neighborhood construction and local energy/source/matrix assembly on
tiny fixed particle sets. NCGA0 remains its immutable local-term boundary and
the historical source-shaped path remains a negative/non-regression control.
