# NSR3-B4EP10SIRDIREQ4 -- executor-adjusted CPU qualification contract

Status: `FROZEN / EXECUTION_PENDING`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdireq4-executor-adjusted-cpu-qualification|v1|parent=e2381ed56b0c6be4ff178d49f8e2f02706648c2be2359471c037c3acfb5b6d69:dc5089bb4e966b0f7d7f54467cadad43fae7b48648f418ba23bd4873bbc49b59:5a18e299a797f1c694e75fef36b48fb06d6c68c97a5eb7b061dc6d2995875ba0:35e0579bc942e1a2419699a049cff6aeb7393e578cd850443b5ef1f3701cecf4|q1=6518ed9875e227f25b298eeb6fd5b3eb3708d1e7014357125a700e9c0d112964:e5ddff76cff198c62120a7f26dcb6efe5ec0c9a7c908e325704af4047259f14f:e4c4dfd2bd472cec5eea51c444f594e22a57b12eb23b2b688a062239fcdeb4e0|implementation=64ef33a44400fcee79f4fcbdcf5bb129895a323e|command=nominal-hydro-directed-scratch-cpu-timing-8|metric=transaction-region-process+active-thread;outside=transaction-region-process;runtime-residual=region-process-active-thread;accounting=adjusted+runtime-residual==transaction|work=transaction1;regions4089;workers8;region-intervals4089;active-intervals32712|runs=3;fresh;serialized;affinity0-7;gnu-time|gates=exact3of3;adjusted-range-ratio<=1.03;outside-range-ratio<=1.05;active-range-ratio<=1.05;external/internal-median=1.00..1.05|route=pass:one-future-candidate-ab-contract-research;fail:dedicated-host-required|exclusions=runtime-residual-no-credit;wall-no-credit;q3-no-rerun|reference=closed|authority=measurement-research-only
```

Identity SHA-256:
`dbde8e1019a5e1c8598b87632264cf5eff40d3b42659485d661e18b8f1991a35`.

## Execution

Run the existing command only:

```text
--nominal-hydro-directed-scratch-cpu-timing-8
```

Use three fresh serialized processes under affinity `0-7`, capturing GNU
`user`, `system`, elapsed and RSS beside stdout/stderr. Do not modify source,
host policy, other processes, worker count or the command.

Each report must be Q1-exact: identity `6518ed98...2964`, result
`e5ddff76...14f`, SIRDI/SIRDIR results, five physics roots, 7,275 process
phase/total intervals, 4,089 region intervals, 32,712 active worker intervals,
1 ns-class clocks and zero clock/accounting failures. Stderr must be empty.

## Derivation and gates

For each run read unsigned integer values:

```text
T = cpu_timing.transaction_cpu_ns
R = cpu_timing.executor.region_process_cpu_ns
A = cpu_timing.executor.active_thread_cpu_ns
```

Require `A <= R <= T`, then derive with checked integer subtraction/addition:

```text
O = T - R
X = R - A
E = O + A
```

Require `E + X == T` exactly. Across three runs require maximum/minimum ratios
`E <= 1.03`, `O <= 1.05` and `A <= 1.05`. GNU `(user+system) / T` median must
remain in `[1.00, 1.05]`. Transaction, region, residual and wall ranges are
diagnostic and receive no failure or speed credit.

Do not silently drop an outlier, use floating subtraction for accounting,
repeat a failed run or relax gates.

## Route and authority

PASS qualifies `E` only as an algorithmic CPU surrogate for research/freeze of
one future candidate A/B contract. That future contract must still require
exact semantics, balanced pairs and its own predeclared improvement/stability
gates, while reporting excluded residual separately.

FAIL requires a dedicated/quiescent performance host before another candidate
A/B. Neither result reopens Q3 or grants wall speed, real-time/FPS, B4E2,
broad corpus, runtime/GPU/schema or production authority.
