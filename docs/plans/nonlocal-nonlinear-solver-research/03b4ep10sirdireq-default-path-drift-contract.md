# NSR3-B4EP10SIRDIREQ -- default-path drift qualification contract

Status: `CLOSED / HOST_UNQUALIFIED / NO_SOURCE_DRIFT_CLAIM`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdireq-default-path-drift|v1|parent=35a1d41b78d132429334a34d8c99e6d2870b2b8a68ee949beb5a3c69375dff10:b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9:539f1ec507e439adc50b14cdf5da616024e041a3131aa56a2002c40a83e4e7e7|accepted=f33bf3aa68283a4391d91c74329a8ae9349927d2:52b329275643dfbfdc7887dc6846bcc4fb99735ce3b8e08507157ea4aa7ff5c6:479b9a05c2765b749ad0bad2c47303d9b06e1024c5b05caf2b320c98a8cdd8a8:d49add40666265e50efb5b856015b2ee9685aac783f7ddd83a3b3a63413dbeb6|current=b8a1eddffcf37a6281e2f67bc80e9a9ef07b2f04:17afc61c1e293416425f6242f8577e34c3481482e8b933cba82236ce0ee234ce:fc66fe7e48a20f44e0e8fb6041942f36e246c1547c17a98d5d674539188ce0f1:c23403bbe0d76880f385a8ab5e691770a0a5a55bde28450e7a9e075dbf479042|build=independent-release;gcc15.2.0;cmake4.2.3;ninja1.13.2|command=nominal-hydro-directed-scratch-reuse-8|timing=one-warmup-each;three-pairs=AB,BA,AB;serialized;affinity0-7;monotonic-wall+gnu-time|gates=exact-both-3of3;accepted-median-ns<=4720000000;ranges<=1.10|route=drift-if-median-paired-slowdown>=1.05+median-total-cpu-ratio>=1.05;no-drift-if-both<1.05;host-unqualified-if-health-or-range-fails;mixed-otherwise|credit=qualification-only;no-speedup|reference=closed|authority=research-only
```

Identity SHA-256:
`d22dfdad22a9a3d02dde9ec3301c1e36124c82d3a075410a0828d2d84e8ea9d6`.

## Build boundary

Create two temporary detached source worktrees outside the active research
worktree. Configure independent Ninja Release builds with GCC 15.2.0, CMake
4.2.3 and Ninja 1.13.2. Do not patch either source. Record source, executable,
ELF Build ID and `compile_commands.json` hashes. Remove neither source nor raw
evidence before the decision is closed.

The accepted source is exactly `f33bf3aa68283a4391d91c74329a8ae9349927d2`;
the current source is exactly `b8a1eddffcf37a6281e2f67bc80e9a9ef07b2f04`.
The three named source-file hashes in the identity projection must match
before configure.

## Correspondence and measurement

Both binaries execute only:

```text
--nominal-hydro-directed-scratch-reuse-8
```

Require accepted SIRDI stdout SHA-256 `539f1ec5...e7e7`, result
`b4f847cb...777e9`, B4EP10SII result, correspondence and all five physics
roots from every warmup and measured process. Stderr must be empty.

Use fixed eight-worker OpenMP placement on physical CPUs `0..7`. Run one
warmup per source and three serialized pairs in order `AB`, `BA`, `AB`, where
A is accepted and B is current. Record monotonic wall nanoseconds and GNU time
user/system/RSS. Record but do not mutate governor, driver, load and competing
process observations.

## Gates and routing

First require exactness `3/3` for both, accepted median wall no more than
4,720,000,000 ns, accepted range ratio no more than 1.10 and current range
ratio no more than 1.10.

- If a health/range gate fails, close `HOST_UNQUALIFIED`; do not attribute
  source drift or run another short-margin A/B.
- Otherwise, median paired current slowdown at least `1.05` together with
  current/accepted median total-CPU ratio at least `1.05` closes
  `DEFAULT_PATH_DRIFT_CONFIRMED` and routes to source-delta isolation.
- If both ratios are below `1.05`, close `NO_MATERIAL_DRIFT` and route to a
  topology-residual discriminator.
- If the two ratios disagree across `1.05`, close `MIXED_WALL_CPU` and freeze
  one narrower measurement before implementation.

No branch grants speed credit or changes selected SIRDI. B4E2, broad corpus,
runtime/GPU/schema and production remain blocked.

Execution preserves both source outputs exactly and both range gates pass, but
the accepted-source median is `4.801272953 s`, above the frozen `4.72 s`
health bound. The contract therefore closes `HOST_UNQUALIFIED`; the observed
current/accepted wall and total-CPU ratios `0.977436/0.982361` grant no source
attribution. See the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdireq-default-path-drift-evidence-2026-08-22.md).
