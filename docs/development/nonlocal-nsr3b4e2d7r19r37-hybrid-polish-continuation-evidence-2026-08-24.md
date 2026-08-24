# NSR3-B4E2D7R19R37 hybrid polish continuation evidence -- 2026-08-24

Status: `PASS / HYBRID_POLISH_CONTINUATION_CANDIDATE`.

## Outcome

Starting from the exact R36 private endpoint, R37 accepts all 24 frozen
additional polish steps. It remains nonstationary, but every checkpoint and
fresh terminal metric improves:

| Additional step | Objective | Violation norm | Projected mapping | Active |
|---:|---:|---:|---:|---:|
| `0` | `3.5527997925597766e-17` | `8.4294718607511539e-9` | `7.4709090069224081e-10` | `876` |
| `6` | `2.0782237218556668e-17` | `6.4470516080696404e-9` | `5.6423406818929022e-10` | `868` |
| `12` | `1.2467811551212180e-17` | `4.9935581605128383e-9` | `4.3636709773615888e-10` | `858` |
| `24` | `4.5783797994002877e-18` | `3.0260138133856190e-9` | `2.6422690805514893e-10` | `846` |

Terminal ratios to R36 are objective `0.12886681115519841`, violation
`0.35898023783378163` and mapping `0.35367437591639933`.

The objective per-step geometric factors over the frozen `6/6/12` blocks are
`0.9145068`, `0.9183671` and `0.9199060`; mapping factors are `0.9542913`,
`0.9580734` and `0.9590555`. The recurrence therefore shows stable linear
convergence rather than an observed plateau or transition to a faster local
regime. The active count continues to change, so fixed-active-set local
behavior is not yet established.

## Controls and work

- R37 identity:
  `746d699d1f724f584dcd156838ea1500118e81000ee19a9d49d85de1aa3a574d`;
- exact R36 parent stdout:
  `13566a2dc1ed03336e906ba68a6a2c80af84cb98a38981e61e22595b50c07550`;
- checkpoint root:
  `e404d5997578df09bab907e16f37e2344f4c754cb7951fad2aa01a0c43f31f4e`;
- fresh terminal response defect: `6.2930931293254465e-16`;
- work: exactly `51` pair passes, zero HVP/model/trial/outer work;
- one workspace lifecycle, all 12 routes and exact rollback pass.

The first diagnostic's impossible zero-objective dense expectation failed
before nominal work and is retained in the
[diagnostic record](nonlocal-nsr3b4e2d7r19r37-first-diagnostic-2026-08-24.md).
Its narrow expected-value repair changed no nominal identity or algorithm.

## Clean reproducibility

Implementation commit: `65f78f77`.

Clean Release build directories:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r37-a.qfjAkW`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r37-b.z6CQPT`.

Both binaries have SHA-256
`1e6403d42ad7451b0f145aa26caff345d2ca79a01f2cd3ffa2c0965c4230a10a`,
size `7,380,736` bytes and ELF build-id
`4e1b26c40be886b8754240bf6dc4d28dda537b35`.

Fresh run directories:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r37-a.fVBgAs`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r37-b.inPMnT`.

The two `2,975`-byte stdout files are byte-exact. Their SHA-256 is
`f475c20302c444be1ca355a1c9f0ec39309bbe7cd33f4db73fad676bef1c3571`;
semantic result SHA-256 is
`c6a254243c8d549f7f20322ed61321eb94ff3d551e75c850d9b1a549a2d14118`.

No timing was taken or interpreted.

## Decision

Retain R36 plus unchanged polish as a correct linearized normal-step
reference, not a complete termination policy. Further unchanged continuation
would repeat the now-established linear regime. Research a bounded
active-set-aware direction-memory discriminator next. It must preserve exact
line globalization and restart to steepest descent on non-descent, invalid
curvature or unsafe active-set change before any candidate implementation.

R37 does not apply a correction, select a tolerance, evaluate a nonlinear
moved state or provide runtime/production authority.
