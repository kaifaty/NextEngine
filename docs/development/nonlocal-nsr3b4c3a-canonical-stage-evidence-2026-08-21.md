# NSR3-B4C3A canonical staging evidence -- 2026-08-21

Status: `PASS / JOINT_PRESSURE_CANONICAL_STAGE_CANDIDATE / B4C3T_DESIGN_AUTHORIZED`

## Reproduction

```text
nonlocal-formula-reclosure --canonical-stage-self-test
```

Two reports are byte-identical:

```text
raw JSON plus LF  80920420f90893e0fad756ef169469fb41770e77b68b0c6ce31ede21b2338ee3
JSON without LF   45b4f8965be67f90878a6f4d703c978ed74914457cd7f64aaccbc84491dd8a48
semantic result   fdaa0befd5538cdf76ff8601a655bf1f9726b5ced19db36943a665f8b06bb5fe
```

Whole-chain harness wall times were `62.34 s` and `61.97 s`, with maximum
resident sets of `8,180/8,352 KiB`. Each execution includes all parent gates,
repeated and reordered publication runs, and independent binary64 comparison.
These are validation-harness observations, not solver performance timings.

## Selected transaction

Both one-frame controls pass the unchanged embedded gate and atomically commit
only the fine level.

| Control | Coarse/fine | Committed | Max position publication error | Max velocity publication error | Binary position RMS | Binary velocity RMS |
|---|---:|---:|---:|---:|---:|---:|
| P1 supported | `21/42` | 42 | `4.9999639080e-7 m` | `4.9885779494e-7 m/s` | `4.0590279205e-6 m` | `3.5759876162e-4 m/s` |
| P2 detached | `1/2` | 2 | `4.2187500001e-7 m` | `4.99999999999e-7 m/s` | `2.6562500001e-7 m` | `1.000000000001e-6 m/s` |

Every next substep consumes the exact decoded previous frame. Sample IDs,
sample count and step sequence are exact. No coarse root appears in the
committed batch; final state equals the last decoded fine frame. Repeat,
reverse and coprime-affine input orders produce exact frames, states and
trajectory roots:

```text
P1  fca52b74f342c574bd5a7e31ae47e3df9090ccd44b35b2fd48cdb085bc81fb56
P2  6f3e0d2e9d907dbe974f00488c3d8a1aac3139e23add85773eddc53a572380ce
```

P1/P2 contact feature sets equal their binary64 fine references and contact
time error is zero. All four typed failure controls pass: forced solver failure
after two private stages, nonfinite input, position overflow and duplicate ID
commit zero frames and preserve the pre-transaction root/state.

## Preserved negative measurement evidence

The first implementation report rejected P2 only because it measured a true
half-microunit velocity tie by subtracting a binary64 re-decode:
`5.0000000000050004e-7 > 0.5e-6`. Its hashes are:

```text
raw JSON plus LF  e963ba1ea22f39871726af9fe17a91c5328bb5b676c4a3eec374ced4d89a1ff3
JSON without LF   69f09cfe3dac291ceabd33dd95255fc066a1a1a4fe617e2b0f47051749eae2e6
semantic result   41c8b390291403dde2ec1edbc092575ea44ff074a1249ad5a2703907ddf4e7f9
```

The selected repair compares the original binary64 input promoted to
`long double` against the canonical integer divided by the decimal scale in
`long double`. It preserves the exact integer quantizer and frozen half-unit
bound; no solver formula, root identity or physical threshold changed.

## Historical regression

```text
NPR1-A raw  464a55bc33741f18ddc4b3c1cda6b46b5248bb653789a5e31585482f871ee207
B4C1 raw    54173d3b15ace2d86827abf8e372d6e098c9964b5fa63a9de5c2b8909e75f030
B4C2Q raw   c2f0d8a4d19ab7682692022099f75ac0fe393c3fada6ea92deb046c25830479f
B4C2T raw   adfcce42be3edb43be54b6e5c68aa7d57171979167f52c10e88143623361e625
```

## Decision

Select `JOINT_PRESSURE_CANONICAL_STAGE_CANDIDATE`. This authorizes only B4C3T
full canonical-controller physical-bound design. Full canonical trajectories,
B4C4 packaging, B4D nominal execution, CUDA, runtime and production authority
remain blocked.
