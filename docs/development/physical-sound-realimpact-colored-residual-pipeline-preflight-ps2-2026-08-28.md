# PS-2 REALIMPACT colored-residual boundary pipeline preflight — 2026-08-28

## Decision

`RealImpactColoredResidualBoundaryPipelineFrozen`.

The frozen colored-residual successor now has one fail-closed orchestration
entry point for its already authorized discovery boundaries. It does not
combine them: `identity`, `tail` and `header` remain separate explicit stages,
each acquisition executes at most once, and later stages require a successful
byte-identical offline audit from the preceding stage.

The pipeline preflight ran offline only. No identity, range, local-header,
member or shadow request was made. The payload decoder remains intentionally
unimplemented until real archive identities have been acquired and frozen, as
required by the successor preregistration.

## Frozen lineage

| Artifact | SHA-256 / result |
| --- | --- |
| Boundary pipeline runner | `34c16185790fac7dbccd00fcc00fd332a0917f6d071cbd70100aaeb4f7081e51` |
| Pipeline preflight A/B | `456ea8e9d51c62ab204611c9bb2d7f16b7aeb8b99e850a1357a0d38adc60cceb`, byte-identical |
| Successor manifest / preflight | `f03e428e…e564a` / `4c754f06…a360` |
| Identity manifest / preflight | `55be5ed2…01abe` / `0d38bfb1…f5d7b` |
| Tail / local-header runners | `5987b048…a8fb74` / `5666a78e…47538` |
| Network/range/real-header/member/shadow bytes | `0 / 0 / 0 / 0 / 0` |

External preflight reports are under
`/tmp/nextengine-physical-sound-colored-residual-pipeline-v1` and are not
repository artifacts.

## Executable boundary order

| Explicit stage | Required input | One allowed acquisition | Offline output |
| --- | --- | --- | --- |
| `identity` | frozen pipeline, successor and identity preflights | three body-free `HEAD` requests | two identical identity audits |
| `tail` | one successful identity audit | three exact 65,536-byte ranges | repeated preflight and two identical tail audits |
| `header` | one successful tail audit | three exact 30-byte ranges | repeated preflight and two identical header audits |

`cross_boundary_auto_continue` and `automatic_retry` are both `false`. A stage
rejection stops without retry, range growth, object substitution or later-stage
access. The pipeline never executes a member payload request and never admits a
quality, domain, runtime, physics or production claim. Spatula shadow remains
sealed and the authored-clip fallback remains mandatory.

## Failure controls

- an obsolete successor preflight is rejected before output creation;
- `tail` without an identity audit is rejected before output creation;
- every source and external frozen artifact is checked by exact size/hash;
- the pipeline preflight records zero network and range requests;
- real member and shadow payload bytes remain zero.

## Checks

- Ruff format/check and Python byte compilation: pass;
- pipeline preflight A/B: byte-identical;
- stale-lineage and missing-prerequisite controls: reject before output;
- `git diff --check`: pass;
- current ProductChecks: not run because no public, runtime, content or
  production contract changed.

## Next action

When shell network is available, run only the explicit `identity` stage once.
If it succeeds, inspect its immutable report and audits before separately
starting `tail`; do not let the orchestration cross that boundary automatically.
