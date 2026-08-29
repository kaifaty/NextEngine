# NSR3-B4EP9 fused phase-timing evidence -- 2026-08-22

Status: `PASS / CPU_PARALLEL_ARCHITECTURE_RESEARCH_SELECTED`

## Result

Three fresh Release processes preserve one semantic result and all frozen
B4EP7I physics/work counters while reporting a tightly clustered conservative
parallelizable fraction:

| Run | Wall | RSS | Fraction | Result |
|---:|---:|---:|---:|---|
| 1 | 7.70 s | 63,316 KiB | `0.9221543365971554` | `PASS` |
| 2 | 7.60 s | 63,128 KiB | `0.92187898636494814` | `PASS` |
| 3 | 7.80 s | 61,724 KiB | `0.92138378827357814` | `PASS` |

The minimum, median, maximum and range are respectively
`0.92138378827357814`, `0.92187898636494814`,
`0.9221543365971554` and `0.0007705483235772581`. This clears the frozen
minimum `0.75`, median `0.80` and maximum-range `0.05` gates with substantial
margin.

B4EP9 therefore selects `CPU_PARALLEL_ARCHITECTURE_RESEARCH` for B4EP10.
This is an Amdahl discriminator, not a measured parallel speedup, throughput
claim or permission to change reduction order.

## Exact correspondence

All runs exit zero with empty stderr and share semantic result:
`44e93e6e4ee24dcc623fe36d4c99ad4456f482fe1b47d18410ffb08204f9cd72`.
Complete report hashes differ only because durations are intentionally present
in JSON and excluded from the semantic result:

- run 1: `da8279a59a531efa4d6e8323c5e3dac9c9bfe64bc4d91ab44281b5afaffc9db0`;
- run 2: `2059e63004dd4c72a781e82596b9c7d4d0a0cfebf3564c5e5abffe0c875f4abb`;
- run 3: `df7e76b890390fd5a2831f05515142570a9f9c4d34ab09dcac30a048ba21739c`.

Every run reports exactly one transaction, 226 topology calls, 226 calls of
each fused setup/pair/centre/finalize phase, 459 HVP applications and zero
timer failures. The transaction retains the B4EP7I query chain and exact
B4EP5/fused work receipts.

The final binary also reproduces exact stdout hashes for B4EP1, B4EP3,
B4EP3I, B4EP5, B4EP7D and B4EP7I:

```text
4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112
4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095
b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055
dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b
9b5453d91a99fc3c21c5024e578d1d5a5593d1bcc14063a421442d25d98487fe
8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095
```

These regressions ran concurrently because they are correspondence checks,
not timing evidence. The three B4EP9 measurements ran serially.

## Phase attribution

The median-fraction run measures a 7.322766262 s transaction:

| Phase | Time | Share |
|---|---:|---:|
| HVP apply | 3.409569416 s | 46.56% |
| fused pair pass | 1.611916917 s | 22.01% |
| topology/filter/CSR | 1.067567838 s | 14.58% |
| fused centre pass | 0.661650168 s | 9.04% |
| fused setup/allocation | 0.420512239 s | 5.74% |
| residual control | 0.151504639 s | 2.07% |
| fused finalize/ownership | 0.000045045 s | less than 0.01% |

The conservative numerator excludes setup, finalize and residual. Even so,
HVP plus the pair/centre/topology phases own 92.19% of the transaction. The
largest single target is HVP, but no single serial rewrite is selected:
B4EP10 must design a deterministic parallel decomposition across all four
admitted phases and preserve canonical reductions.

## Build identity

- implementation commit:
  `706f071d4f7331e64b722143707ae491e59f6030`;
- compiler: GCC 15.2.0, Release `-O3 -DNDEBUG -ffp-contract=off
  -fno-fast-math`;
- `compile_commands.json`: 15,121 bytes,
  `2e2eb6efae788fae20bcfe18dba0bcda0b5b1179375bbf99e2c4df7154ef2512`;
- executable: 3,841,488 bytes,
  `bdcfb256045675818c6329cb2e84e53dd82fb8974aec2250fcb7c104a6c096ad`;
- Build ID: `ee211127d0969160f6597541a5c281448c1e7fe2`.

Source hashes:

- `boundary_reference.cpp`:
  `236bfb2fb261892c05568d5346f28b5d67a67a9429c7e94d0ddf95ab192c1ae0`;
- `boundary_reference.hpp`:
  `a9f25bd291cdfc8e9a91943a5ddb4c4b8316fff274beea980678ed9747163042`;
- `formula_reclosure_main.cpp`:
  `a2e2ae3493298eb4282ccd78be9698085122a1ce3d806ae9e06c8a9f3a820290`.

External artifacts remain outside Git:

- build: `/home/kaifaty/.cache/nextengine/external/build-nonlocal-b4ep9.kKYspi`;
- timed runs: `/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep9.Qwv9Xd`;
- regressions: `/home/kaifaty/.cache/nextengine/external/regress-nonlocal-b4ep9.4ELtbb`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep9-evidence|v1|identity=a75c1db690e26805b7f5d900a053be789316688c9ac033fd7db902f26e37cad6|implementation=706f071d4f7331e64b722143707ae491e59f6030|result=44e93e6e4ee24dcc623fe36d4c99ad4456f482fe1b47d18410ffb08204f9cd72|binary=bdcfb256045675818c6329cb2e84e53dd82fb8974aec2250fcb7c104a6c096ad|build-id=ee211127d0969160f6597541a5c281448c1e7fe2|compile=2e2eb6efae788fae20bcfe18dba0bcda0b5b1179375bbf99e2c4df7154ef2512|reports=da8279a59a531efa4d6e8323c5e3dac9c9bfe64bc4d91ab44281b5afaffc9db0,2059e63004dd4c72a781e82596b9c7d4d0a0cfebf3564c5e5abffe0c875f4abb,df7e76b890390fd5a2831f05515142570a9f9c4d34ab09dcac30a048ba21739c|fractions=0.9221543365971554,0.92187898636494814,0.92138378827357814|min=0.92138378827357814|median=0.92187898636494814|max=0.9221543365971554|range=0.0007705483235772581|calls=1,226,226,226,226,226,459,0|regressions=4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112,4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095,b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055,dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b,9b5453d91a99fc3c21c5024e578d1d5a5593d1bcc14063a421442d25d98487fe,8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095|decision=cpu-parallel-architecture-research
```

SHA-256:
`b557d20b0e67755228c94b67be11a4371e2a9bd355a70a4bd176e2ecaa62872b`.

## Decision

Close B4EP9 as PASS. B4EP10 may research and freeze a deterministic CPU
parallel architecture. It may not implement threads until ownership,
partitioning, reduction order, scheduling, failure and A/B gates are frozen.
