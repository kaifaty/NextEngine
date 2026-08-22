# NSR3-B4E2D7R18R3 normalized divided precancellation evidence

Date: `2026-08-22`

Status: `PASS / NORMALIZED_DIVIDED_PRECANCELLATION_CANDIDATE / D7R19_BLOCKED`

Implementation commit: `9f43ebb437ca5208205133b06f5520c8d7aae478`.

## Result

The exact outer-1/trial-2 replay selects:

```text
NORMALIZED_DIVIDED_PRECANCELLATION_CANDIDATE
```

The pairwise-precancelled normalized formula restores the independently
certified positive reduction. This validates one exact replay pair only. The
candidate did not accept the trial, continue the solve, update dual state or
publish state. D7R19 remains blocked.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r18r3.zb1FCq`.

Two independent clean GCC 15.2 Release builds produce byte-identical
5,559,304-byte executables:

```text
SHA-256  b8abea9d3972c93558776566ae5eef4d80d6eba959ec3b7a235b1c263e55fa4b
Build ID 5227f5208442054dd0b02552f3041e4aa85bd1ac
```

Build directories:

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r3-a.tNfV5v
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r3-b.oZ9S0J
```

One fresh R3 process from each build exits zero with empty stderr and emits
the same 3,161-byte stdout:

```text
stdout SHA-256 e0b36e34a047de1d8bb1435fa6f68cf5a53ab8a4260a95719775c03e4608d3e5
semantic result 884dcda82ec013903db389a6bd873326d2501f452babc8a7d2cedcda1faebf75
```

Each clean binary directly preserves the frozen parents:

```text
D7R18R2 10,996 bytes 3659eac888c22eae5bcf7ae8c5f8a426bcbc12bd24bd3320c23be0c176815d77
D7R13   15,854 bytes 514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de
```

## Exact replay

The replay reproduces:

```text
active root c76bea9f1bff57c16e27a08c4fc51dad7029af1730c11ac80e568ebe112601a5
inner root  b7f44b27a51e6249b0d88b29cd08af42a7490e69ea5fed4e081cf64f48b44515
outer/trial 1/2
```

All frozen binary64 anchors are exact. The inherited formula remains
negative:

| Quantity | Value |
|---|---:|
| predicted reduction | `3.163331278531801e-22` |
| raw reduction | `-3.0715532249809066e-19` |
| inherited normalized divided | `-3.0714726551614456e-19` |

No inherited acceptance or radius behavior was changed to reach the pair.

## Candidate and independent oracles

The pairwise formula returns:

| Quantity | Value |
|---|---:|
| candidate reduction | `3.163242142020881e-22` |
| PHR reduction | `1.2234374161630194e-14` |
| inertia reduction | `-1.223437384530598e-14` |
| candidate / predicted ratio | `0.9999718219487397` |

The large PHR and inertia components nearly cancel, but each is computed as a
delta before the center reduction. The remaining signal is no longer obtained
by subtracting two rounded active states.

Direct normalized long-double naive and compensated reductions are both
positive and resolve under the frozen 1024-ULP rule. Direct normalized
binary128 reports:

```text
compensated reduction 3.163279198222889070258071943532397115e-22
resolved ULP ratio    3.444506024969770152e18
candidate rel. error  1.171448983345765152e-5
predicted rel. error  1.646402535101436061e-5
```

Both errors are far below the frozen 5% limit. Binary64/binary128 pair
membership is exact. Reference repeat and reference/aligned normalized roots
are byte-exact:

```text
candidate root e97885f63e5536f148e2138af81c95f5e758bc375695d521736e737ee6b72c84
long root      1508a213e24d11a4e4b133d51e4cd236ed944a76f34982d88c8c4dd800233d06
binary128 root f64485bb135d3b3ce1564d2ac8af4bc9272229edfc7266ce96d44b7122bfaf85
```

## Work and rollback

The unchanged reference replay uses two outer updates, 14 inner trials and 28
HVPs. Candidate work is bounded to four workspaces, 2,484 pair-union visits,
two long-double audits and two binary128 audits. Maximum live candidate
workspaces is two; all-pair candidate calls are zero. Four invalid cases reject
before pair visits.

Candidate acceptances and candidate-owned outer updates are both zero. Caller
position and dual roots remain exact, and the report publishes no state.

## Decision

Preserve R3 as the exact formula certificate and do not turn its replay hook
into an acceptance path. Research/freeze a separately bounded complete
normalized private transaction whose only solver change is replacement of the
rounded-state divided numerator by the certified pairwise formula. It must
repeat all R2 cross-profile, precision, confirmation/holdout, work and rollback
controls.

D7R19, nominal execution, timing, runtime binary128 and production claims
remain blocked until that complete transaction passes.
