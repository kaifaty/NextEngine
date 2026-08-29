# NSR3-B4EP10SIRDIREQ default-path drift evidence -- 2026-08-22

Status: `HOST_UNQUALIFIED / NO_SOURCE_DRIFT_CLAIM`

## Result

Both untouched source checkpoints build reproducibly and execute the same
SIRDI command exactly. Every warmup and all six measured reports have stdout
SHA-256 `539f1ec5...e7e7`, semantic result `b4f847cb...777e9`, the retained
B4EP10SII result/correspondence/five physics roots and empty stderr.

The accepted checkpoint does not pass its frozen absolute health gate:

| Metric | Accepted `f33bf3a` | Current `b8a1edd` | Gate |
|---|---:|---:|---:|
| wall ns | `4,819,122,069 / 4,645,031,940 / 4,801,272,953` | `4,609,497,229 / 4,779,007,179 / 4,692,938,478` | -- |
| median wall ns | `4,801,272,953` | `4,692,938,478` | accepted `<= 4,720,000,000`: FAIL |
| range ratio | `1.0374787797476372` | `1.0367740648445458` | each `<= 1.10`: PASS |
| median total CPU s | `35.15` | `34.53` | diagnostic |
| effective cores | `7.320975` | `7.357863` | diagnostic |

Current/accepted paired wall ratios are
`0.956501 / 1.028843 / 0.977436`, median `0.977436`. Median total-CPU ratio is
`0.982361`. These values are evidence against a material default-path source
regression, but the contract forbids that attribution after the accepted
health failure. The only admissible decision is `HOST_UNQUALIFIED`.

The host used `amd-pstate-epp` with `performance` governor/EPP. Concurrent
desktop and developer processes were visible, and system load changed from
2.73 to 5.51 during the measurement window. No process, priority, governor,
sysctl or affinity outside the two benchmark commands was changed.

## Builds and artifacts

The accepted executable reproduces the original SIRDI artifact exactly:

- accepted executable SHA-256 `3f692644cb3fed6e227baf48ebb7cb7b578d085da75d90d31bb1e20ea07a4a02`,
  size 4,318,368 bytes, Build ID `218622804aa75cfd995b0b1dadd5a6d066a5126d`;
- current executable SHA-256 `8d30e0dc3fe4d496d72a98675d620a3fd6f7d535047b5d8f8a0bbfaf1e65ae71`,
  size 4,348,944 bytes, Build ID `defea5f156723f064008be187077ac3e07545335`;
- accepted/current `compile_commands.json` SHA-256
  `468ac613...c63 / 7cb1ee96...6503`.

The frozen source hashes all match. Temporary detached sources and builds are
under `/home/kaifaty/.cache/nextengine/external/sirdireq.qwKULg`. Raw timing
artifacts are under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sirdireq.VQEvC3`;
`metrics.json` SHA-256 is `7b116eb3...f59a3`.

## Decision

Close B4EP10SIRDIREQ as `HOST_UNQUALIFIED`. Do not bisect source, because the
healthy-source prerequisite failed and the paired data does not indicate a
regression. Do not use this result for a speedup or short-margin wall A/B.

Retain exact SIRDI. While the shared desktop host is noisy, continue only with
structural audits or an opt-in process/thread CPU-time attribution lane that
excludes external preemption and grants no wall-throughput credit. Final wall
selection still requires a separately qualified measurement window.
