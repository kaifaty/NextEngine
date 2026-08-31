# Physical Sound V15-S0c — source sufficiency and role-freeze result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT / ZERO_SIGNAL / SOURCE_INSUFFICIENT` |
| Decision | `S0C_SOURCE_INSUFFICIENT_SOURCE_GROWTH_REQUIRED` |
| Protocol | [S0c protocol](physical-sound-v15-s0c-source-sufficiency-role-freeze-protocol-2026-09-01.md) |
| Roadmap | [V15 S0c](../plans/physical-sound-synthesis-roadmap-v15.md) |
| Product effect | None; authored clips remain authoritative and SPEC-45 remains `Proposed`. |

## Outcome

S0c deterministically joins the revision-aware S0a identity/exposure census,
the N1b source inventory and the S0b YCB capability inventory. It validates
the unchanged N1a `4/1/1/1/1` policy and emits no partial assignment.

The current frozen source evidence cannot supply one complete Metal split:

| Material | Fresh ObjectFolder candidate groups | Compact / preflight | YCB exact parents | Training usable | Evaluation complete | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Glass | `2` | `0 / 2` | `0` | `0` | `0` | `SOURCE_INSUFFICIENT` |
| Metal | `12` | `4 / 8` | `9` | `0` | `0` | `SOURCE_INSUFFICIENT` |
| Wood | `0` | `0 / 0` | `3` | `0` | `0` | `SOURCE_INSUFFICIENT` |

No dataset role is frozen and S1 remains blocked. This is a successful
negative source certificate, not a failed implementation and not permission to
lower a role minimum.

## Why the apparently large Metal pool receives no role credit

The twelve fresh S0a Metal groups are real structural candidates, but only
four have a compact N1b member root. N1b deliberately contains no complete
axis certificate or recording-parent certificate, so `metadata_candidate`
cannot be silently upgraded to `training_usable`. The other eight additionally
need member preflight.

All ten revision-resolved Metal groups with a RealImpact alias are already
exposed. None may become validator calibration, method holdout or admission
shadow. Their historical exposure would have to be proven generator-only
before any reuse even on the generator side.

YCB contributes nine exact Metal vertical parents with non-ambiguous geometry
routes. Three are repository-adapter referenced and six have freshness
unassessed. More importantly, the frozen capability rows do not bind contact
position to geometry, exact geometry bytes/metric scale, canonical excitation,
listener condition, recorded response or support as required by N1a. An
available URL and a plausible object name are therefore not role evidence.

Wood's three exact YCB parents are already adapter referenced and Glass has no
exact YCB vertical parent. Horizontal and manual material aggregates remain
ineligible object groups.

## Repeat-exact evidence

External root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v15-s0c-final.HWZBUA
```

`run-a` and `run-b` are byte-identical. Run-A identities:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `source-sufficiency.json` | `25,105` | `eecbe01910898aba0597188d05d226dc674dfc1bba59468f499c7625c83c923d` |
| `role-freeze-decision.json` | `1,178` | `f12e769f7bdd60ad451a01953ac20507ca8f15832b50ec66bec6cfd1908fe718` |
| `report.json` | `1,570` | `2c9fd6b84e3646af07df2509caa0a27e4ff40097cb0175423e962af1283c4cf3` |

The report binds `130` revision-aware groups, `14` fresh ObjectFolder-route
candidates, `18` target-material YCB rows and exactly zero role assignments.

## Access accounting

Both real builds report zero for:

- network requests and source payload bytes;
- archive member bodies and WAV/NPY headers;
- mesh values and video frames;
- PCM, force and protected signal values.

Only the four frozen canonical JSON inputs were read. No audio, array, mesh,
video, force, checkpoint or protected source body was opened.

## Executable guards

The implementation is:

- `lab/scripts/physical_sound_source_sufficiency_v1.py` — SHA-256
  `229c0cc483f5d2c4c721cb0206452bd7e1a918dfc4635d1e4c0a3543ad22f4c6`;
- `lab/tests/test_physical_sound_source_sufficiency_v1.py` — SHA-256
  `81ee94d1d93a33009856fc2aa003cfd11e94ba4ad4130caa0f4b6d6530416251`.

Nine focused methods cover repeat-exact output, all zero-signal counters,
input drift/non-canonical JSON, unknown schemas and states, dangling and
cross-material routes, false-fresh YCB promotion, missing-axis promotion,
protected exposed-object reuse, complete deterministic selection, partial
assignment, role-minimum reduction, recording-parent reuse, output replacement
and symlink/repository-output boundaries.

The focused S0c suite passes `9/9`; the combined N1a/N1b/S0a/S0b/S0c suite
passes `46/46`, and Python bytecode compilation passes. The mapped
`content-package` check passes with `123` records and `64` chunks. The broad
`boundary-scan` remains red only on the pre-existing tracked
`SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
S0c does not touch that file and no broad boundary-scan pass is claimed.

## Decision and next boundary

S0c closes the current source freeze as insufficient and opens only
`S0d — published-source growth`:

1. resolve whether historical exposed groups are generator-only or protected;
2. seek publisher-side per-object member/support/contact certificates before
   any batch body, avoiding the N1b approximately `393 GB` speculative
   ObjectFolder preflight ceiling;
3. search additional internet-published T2/T3 sources whose object, material,
   recording parent, contact/geometry, scale, excitation, listener, response
   and support axes can all be hash-bound;
4. rerun a versioned S0c role freeze only after at least five generator-ready
   and three fresh evaluation-complete Metal groups exist.

S0d may inspect publisher metadata and bounded member headers. It may not open
audio/force/protected signal, reduce `4/1/1/1/1`, reuse exposed protected
objects or authorize S1. Wood remains pending and Glass remains `FallbackOnly`
until each independently meets the same shape.
