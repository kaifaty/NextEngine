# NSR3-B4EP8 fused residual-attribution evidence -- 2026-08-22

Status: `PASS / SERIAL_RESIDUAL_BALANCED / PHASE_TIMING_SELECTED`

## Result

The exact-output fused profile records 729 ten-millisecond samples. Complete
fused workspace owns 3.61 s and exact HVP application owns 3.59 s. Their ratio
is only `1.005571031x`, so neither clears the frozen `1.20x` leader rule.
Residual control/publication is about 0.07 s.

B4EP8 selects `SCOPED_INTERNAL_PHASE_TIMING` and no optimization. The prior
serial profile/fix cycle has reached a balanced boundary on this corpus.
B4EP9 may add exact, externally disabled phase timers to distinguish inlined
fused pair/center work and HVP subphases; it may not yet change algorithms,
parallelism, GPU/runtime or solver policy.

## Build and correspondence

| Field | Value |
|---|---|
| identity | `d8c6584ee93f8b3232441c061bacfb6ddb8f50d5a80805a57c6b53b0968a65e1` |
| implementation | `a3aa054217cfd9d21193f8943effe10b7ebce7bf` |
| compiler/profiler | GCC 15.2.0 / GNU gprof 2.46 |
| compile commands | 15,261 bytes; `b870a608c76beae35105d637d08196a4a8579c2c453b8f15fa475997320a0bc1` |
| executable | 37,430,784 bytes; `19b26e6fa4fc1585cea85665c1a712ebadfe2e88e843d488f9e6f70bb12dfb37` |
| Build ID | `3f6737e04fabb320d5c4c5f0ffcc965ef6e917ff` |

The process exits zero with empty stderr. Stdout is exactly 6,714 bytes with
SHA-256
`8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095`
and semantic result
`e3453dc158a8eace69b468ac84641cccdcad221dcc05695d06b6f400039e775c`.
It is byte-identical to B4EP7I.

Instrumented wall/RSS are 10.41 s and 63,328 KiB at 99% CPU; these are
completion diagnostics, not Release throughput.

## Artifacts

| Artifact | Size | SHA-256 |
|---|---:|---|
| `gmon.out` | 1,422,501 bytes | `5b8a5fe02fdadf645c826a365c7107be68fa88646429763146803f4ed0e10f17` |
| flat profile | 49,750 bytes | `853614bf481dc48d76441c578e6e8e595bb483836ddafbfe3b6c6e6324ddfbb9` |
| call graph | 347,006 bytes | `a4d352af06078b58a8728bf644da4e17c6861fc2ef24a79cc73a092fc55358d8` |

Evidence attestation root:
`8a8d15c316b7e03d1eccfad527543e6a428264f8c339be71efd875b0e7079c4b`.

External artifacts remain outside Git:

- build: `/home/kaifaty/.cache/nextengine/external/build-nonlocal-b4ep8.G48Abq`;
- run: `/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep8.keYqNk`.

## Attribution

The main call tree owns 7.27 s; a separate `_init` bucket owns 0.02 s.

| Top-level category | Inclusive CPU | Share of 7.27 s |
|---|---:|---:|
| complete fused workspace | 3.61 s | 49.656121% |
| exact pressure-tape HVP | 3.59 s | 49.381018% |
| residual control/publication | 0.07 s | 0.962861% |

Within workspace, cached topology/filter/CSR owns about 0.94 s: 0.84 s
filter/finalization, 0.07 s one-time superset build and about 0.03 s the
canonical parent topology path. The remaining 2.67 s contains inlined fused
pair/center work, allocation/ownership and evidence management.

The compiler inlines `build_joint_evaluation_tape_from_flat` into
`build_joint_query_workspace`. Kernel children expose 0.28 s gradient and
0.24 s second-derivative work plus 0.14 s direct kernel scale, but the pair
and center loops share 1.95 s of parent self samples. Gprof therefore cannot
separate those two frozen subcategories responsibly. This is a second,
independent reason for phase timing rather than another implementation.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep8-evidence|v1|identity=d8c6584ee93f8b3232441c061bacfb6ddb8f50d5a80805a57c6b53b0968a65e1|implementation=a3aa054217cfd9d21193f8943effe10b7ebce7bf|stdout=8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095|result=e3453dc158a8eace69b468ac84641cccdcad221dcc05695d06b6f400039e775c|binary=19b26e6fa4fc1585cea85665c1a712ebadfe2e88e843d488f9e6f70bb12dfb37|compile=b870a608c76beae35105d637d08196a4a8579c2c453b8f15fa475997320a0bc1|gmon=5b8a5fe02fdadf645c826a365c7107be68fa88646429763146803f4ed0e10f17|flat=853614bf481dc48d76441c578e6e8e595bb483836ddafbfe3b6c6e6324ddfbb9|callgraph=a4d352af06078b58a8728bf644da4e17c6861fc2ef24a79cc73a092fc55358d8|samples=729x0.01|categories=workspace3.61;hvp3.59;control0.07;unassigned0.02|ratio=1.005571031|workspace=topology0.94;inlined-fused+management2.67|decision=scoped-internal-phase-timing
```

SHA-256:
`8a8d15c316b7e03d1eccfad527543e6a428264f8c339be71efd875b0e7079c4b`.

## Decision

Select scoped, opt-in internal phase timing for B4EP9. Timers must be absent
from all default/old/fused correspondence commands and must not share a run
with throughput timing. No next optimization is selected by B4EP8.
