# NSR3-B4C1 -- compact CSR pressure-radius tape

Status: `PASS / JOINT_PRESSURE_RADIUS_TAPE_CANDIDATE / B4C2_DESIGN_AUTHORIZED`

Parent B4C0R selects `JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE`; semantic
SHA-256 is `bc60400325bfa0e7a3109fe8df037ff8ff2bd8e2362f378347e61c7514ad8047`
and JSON-without-final-LF SHA-256 must equal
`5ecaa4d5356611863dbd80fd8e261f16aa3410907e4a15739875521fb578d493`.

## Identity

```text
joint-pressure-radius-tape-csr-r0
```

Use the exact B4C0R pair list and outer state. Build `u32` CSR offsets and
directed pair indices, one binary64 radius per unique pair and one binary64
compression per fluid centre. CSR rows must reproduce the B4C0R adjacency
order exactly. Capacity validates before tape allocation; failure returns no
partial tape.

## Correctness controls

Reuse P1 initial, P1 feasible forecast, P2 detached initial and signed-cutoff
states. Add P1 compressed by `0.99` as a broadly active holdout. For each:

- CSR-derived participants equal every nested adjacency row exactly;
- stored radii equal direct inherited `norm` exactly;
- stored compression equals the outer evaluation exactly;
- taped HVP equals untaped B4C0R HVP bit-for-bit for three deterministic joint
  directions and one fluid-only direction;
- full fluid and support reaction rows are included;
- identity/reverse/coprime-affine inputs retain the same tape digest and HVP;
- inactive cases emit zero pressure response without reading absent records.

Corrupt/out-of-range pair index, offset overflow and insufficient admitted
tape payload must return a typed failure and zero offsets, indices, radii and
compression output.

## Work and memory gates

Report `P`, `D`, active centres and:

```text
untaped radial work for K=3 HVPs = 3*(P + 3*D_active)
taped radial work                 = P build + 0 HVP
```

Require taped radial work strictly below half the untaped work for every
active control; inactive controls report rather than claim a ratio.

At `50,000` fluid samples require the exact payload ceilings:

- existing pair list `<=64,000,000` bytes;
- radii `<=64,000,000` bytes;
- directed indices `<=32,000,000` bytes;
- offsets `<=200,004` bytes;
- compression `<=400,000` bytes;
- combined `<=160,600,004` bytes.

This is capacity evidence, not a product memory budget.

## Repeatability and exit

Two reports must be byte-identical. B4C0R, B4C0 and all earlier raw reports
remain exact.

PASS selects `JOINT_PRESSURE_RADIUS_TAPE_CANDIDATE` and authorizes only B4C2
one-substep current/trial/forecast substitution contract design. FAIL blocks
that substitution.

No full trajectory, canonical continuation, nominal run, CUDA, performance,
runtime, schema or production authority is granted.

Execution passes all frozen gates; see the
[dated evidence](../../development/nonlocal-nsr3b4c1-pressure-tape-evidence-2026-08-21.md).
