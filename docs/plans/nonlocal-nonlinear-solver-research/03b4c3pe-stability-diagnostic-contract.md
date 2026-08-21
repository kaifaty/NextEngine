# NSR3-B4C3PE -- macro publication stability diagnostic

Status: `FROZEN / DIAGNOSTIC_IMPLEMENTATION_AUTHORIZED`

Parent B4C3P is an exact FAIL with JSON-without-final-LF SHA-256
`2cbeaafe6b7cdf06a1d982bbdb4134823aecc71247b3fed496bd48e5e32b5b1c`
and semantic SHA-256
`e79566a815a2491510318acf1793b693c642b06c84b167f0a4a1e1bd1344965b`.

## Identity and scope

```text
identity  b9ef5dc35652e3a708ae3b36aa6fae12e1a1ede8b8817dc1ea7ba1179e33ca65
text      nextengine.nonlocal.macro-stability-diagnostic|v1|direct-propagated|fine-contamination|levels=48,96,192
```

Reuse the B4C3P profile, macro-ledger policy, fixtures, levels, private solver,
parallel lane scheduling and roots exactly. This stage changes no gate and
selects no candidate.

## Per-frame decomposition

For every case, level and macro frame retain private prepublication state in
addition to the already committed decoded state. Against the corresponding
binary fixed lane report position/velocity RMS values for:

- durable start error;
- private propagated end error;
- direct publication error;
- published end error.

Compute independent binary64 floors for each comparison. Require only finite
values, exact frame alignment and the forward triangle closure:

```text
published_error <= propagated_error + direct_error
                   + gamma(operations)*state_scale.
```

Report propagation gain `propagated/start` only when start exceeds its floor;
otherwise increment an unresolved counter. Report maximum resolved gain and
its frame.

## Fine-192 contamination

At every frame compute candidate fine-192 published error and binary 96/192
temporal difference. When the latter exceeds its computed binary64 floor,
report their ratio; otherwise mark unresolved. Always report absolute candidate
error and utilization of `0.05dx` for position and `0.001c` for velocity.

Aggregate maxima must identify case, field and frame. Preserve exact contact
time and terminal-contact diagnostics; do not turn them into new tolerances.

## Repeatability and exit

Run two complete diagnostics and require byte identity. PASS means only that
the decomposition is exact and may authorize stability-budget design. FAIL
means the measurement model itself is invalid. B4C3P remains FAIL in both
cases; adaptive redesign, B4C3TC, nominal, CUDA, runtime/schema and production
remain blocked.
