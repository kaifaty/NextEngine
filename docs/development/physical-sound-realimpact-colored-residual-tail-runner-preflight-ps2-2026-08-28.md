# PS-2 REALIMPACT colored-residual ZIP-tail runner control — 2026-08-28

## Decision

`RealImpactColoredResidualTailParserFixtureSupported`.

The follow-on to the frozen archive-identity boundary is implemented. After a
successful identity audit, it can derive exactly one 65,536-byte ZIP-tail range
for each Pan, PiePan and Cup archive, acquire those three tails once, parse the
central directories offline and emit one exact 30-byte observation local-header
range per object for a later separately frozen stage.

This result supports the parser and request architecture only. No real identity
audit exists in the current sandbox, so no real tail manifest or preflight was
created and no range request was issued. It grants no material, quality,
transfer, domain, runtime or production credit.

## Frozen implementation and control

| Artifact | SHA-256 / result |
| --- | --- |
| ZIP-tail runner | `5987b04807fe21177fd017165ef299a87ca8ddac6fa14f818209b67644a8fb74` |
| Bound identity runner | `898d1af13bf4dda4c3efed77b611ad757ccfe71f0548532e29c1a38c1a6c7c58` |
| Bound observation-discovery core | `3a574c6ecfa52603cfaa59b86be872aa4e42c99e0d7dc5e1cdcca0a399b92f74` |
| Synthetic fixture A/B | `8be574bdc16ee8dd7d540a7a8b7d50dc20a7c0736612fa7986c83d6182c8e605`, byte-identical |
| Synthetic tail | `3d287ac48ca329f926e00483166dc2e1b29e9b8035c4c9eb1a3d73b5e2a9d5d7` |
| Network/range/local-header/member/shadow bytes | `0 / 0 / 0 / 0 / 0` |

The external fixture reports are under
`/tmp/nextengine-physical-sound-colored-residual-tail-v3` and are not repository
artifacts.

## Boundary and controls

The accepted order is intentionally split:

1. the existing identity runner performs three body-free `HEAD` requests;
2. a successful immutable identity audit seeds a new tail manifest;
3. a repeated offline preflight freezes three exact 65,536-byte tail ranges;
4. acquisition reads each tail exactly once with zero retry or prefix growth;
5. an offline audit parses the central directories and derives, but does not
   read, the next three 30-byte local-header ranges;
6. any local-header or member access requires another manifest and preflight.

The synthetic control uses a prefixed 12-entry deflated ZIP whose complete
archive is larger than the allowed tail. Both runs recovered all 12 central-
directory records, selected the unique `deconvolved_0db.npy` observation and
derived range `[70000, 70029]` without reading that local header or any member
payload.

Spatula shadow objects are absent from the roster and remain sealed. Failed
identity or tail acquisition stops without retry, alternate object, wider
prefix, header access or payload access. The authored-clip fallback remains
mandatory.

## Checks

- pinned identity-runner and discovery-core hashes: exact;
- Ruff format/check: pass;
- Python byte compilation: pass;
- synthetic fixture A/B: byte-identical;
- all four parser assertions: pass;
- network, range, local-header, member and shadow bytes: zero;
- current ProductChecks: not run because no public, runtime, content or
  production contract changed.

## Next action

When network execution is available, perform only the three frozen body-free
identity `HEAD` requests and audit the immutable result offline. A successful
audit may then bind the implemented ZIP-tail runner; do not combine identity
and range access in one evidence boundary.
