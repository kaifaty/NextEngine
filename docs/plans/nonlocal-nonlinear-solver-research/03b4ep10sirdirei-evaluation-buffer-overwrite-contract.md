# NSR3-B4EP10SIRDIREI -- evaluation buffer overwrite contract

Status: `CLOSED / FAIL / DEFAULT_PATH_REGRESSION / IMPLEMENTATION_REVERTED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdirei-evaluation-buffer-overwrite|v1|parent=dfc1bb3d154f9e406c89d0fde2304983c837d9f27d43888be542f6e0e5697e1d:a60d07709cd12449feb6fc58c519387841edb68be29dd7493e8dc242d9f4c556:b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9|implementation=ecc9ab903b9919099d855a531b304506a4161184|commands=baseline:nominal-hydro-directed-scratch-reuse-8,candidate:nominal-hydro-directed-scratch-evaluation-buffer-overwrite-8|scope=six-f64-roles;gradient-unchanged;no-pool;no-lifetime-change|allocator=cxx17-stateful;default:value-init;candidate:zero-arg-double-default-init;other-construct-forwarded|work=calls226;slots345576600;bytes2764612800;gradient-bytes64133376-unchanged|semantics=sirdirea-coverage;workspace-lifetime-exact;roots-exact;old-command-exact|capacity=67108864;fail-closed|timing=external-monotonic+gnu-time;one-warmup-each;three-pairs=AB,BA,AB;serialized;affinity=0-7|gates=candidate-exact-3of3;wins3of3;median-paired-speedup>=1.05;candidate-range-ratio<=1.10;rss-delta-kib<=16384;median-total-cpu-ratio<=1.02|failure=retain-sirdi|reference=closed|credit=candidate-residual-attribution-research-only
```

Identity SHA-256:
`96a22c8e733cfd334db50bd4cace64cbdf6e49ad53e544aa50c6b461f68b8f99`.

## Implementation boundary

Add only:

```text
--nominal-hydro-directed-scratch-evaluation-buffer-overwrite-8
```

Use an internal C++17 allocator mode whose default behavior remains ordinary
value initialization. Candidate mode may change only zero-argument construction
of `double` to scalar default-initialization; it must forward explicit-value
construction and every non-`double` type unchanged. Allocator mode must
propagate through move assignment and compare by mode.

Change the internal density/radius/compression/HVP-gradient/HVP-second vector
type as needed to carry the allocator, but default construction must preserve
all old commands. Enable candidate mode only when building the exact SIRDI
owner-parallel evaluation/tape and its density-contribution local. Gradient,
center energy, topology/plan storage, directed scratch and physical arithmetic
remain byte-for-byte unchanged. No raw storage, reads outside logical size,
pooling, reserve-only indexing or public schema/type is allowed.

## Exact counters and safety

Report exactly 226 candidate evaluation calls and the following default-
initialization slots:

| Role | Slots |
|---|---:|
| density | 1,356,000 |
| radius | 85,716,150 |
| compression | 1,356,000 |
| HVP gradient | 85,716,150 |
| HVP second | 85,716,150 |
| density contribution | 85,716,150 |

Their sum must be 345,576,600 slots / 2,764,612,800 bytes. Report unchanged
gradient initialization at 2,672,224 slots / 64,133,376 bytes. Require zero
allocator-mode, size, coverage, lifetime or capacity failures and the exact
B4EP10SIRDIREA two-workspace/one-ephemeral ownership identities. Candidate
maximum added payload remains within 67,108,864 bytes.

## Correspondence

Each candidate process must reproduce B4EP10SIRDI result
`b4f847cb...777e9`, B4EP10SII result, correspondence, five physics roots,
query chain, all work/reuse/retention counts and duration-free semantics. The
old SIRDI command must remain byte-exact. Release builds must pass `-Werror`.

## External A/B

On the same binary and physical cores `0..7`, set fixed 8-worker OpenMP
placement, run one warmup per command, then serialized balanced rounds
`AB`, `BA`, `AB`, where A is SIRDI and B is the candidate. Record monotonic
wall nanoseconds plus GNU time user/system/RSS for every process.

Require candidate exactness `3/3`, candidate wall wins `3/3`, median paired
speedup at least `1.05`, candidate wall range ratio at most `1.10`, candidate
minus baseline median RSS no more than 16,384 KiB and candidate/baseline median
total CPU ratio at most `1.02`.

## Exit

PASS selects the candidate for residual attribution research only. Any semantic,
counter or performance gate failure retains SIRDI and stops this candidate; do
not lower gates or widen into a raw-storage/pool redesign. B4E2, broad corpus,
runtime/GPU/schema and production remain blocked.
