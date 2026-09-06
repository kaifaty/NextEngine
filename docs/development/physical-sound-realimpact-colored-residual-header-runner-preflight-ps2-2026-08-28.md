# PS-2 REALIMPACT colored-residual local-header runner control — 2026-08-28

## Decision

`RealImpactColoredResidualHeaderParserFixtureSupported`.

The stage after ZIP-tail audit is implemented as a separate hash-closed
boundary. Given a successful immutable real tail audit, it can freeze exactly
three 30-byte observation local-header ranges, acquire them once, validate them
against their central-directory records offline and derive exact compressed
payload ranges for a later separately frozen stage.

No real tail audit exists in the current sandbox. Therefore no real header
manifest, preflight or request was created. The 30 header bytes below are from
the deterministic synthetic fixture only; real header, member and shadow bytes
all remain zero. This grants no material, quality, transfer, domain, runtime or
production credit.

## Frozen implementation and control

| Artifact | SHA-256 / result |
| --- | --- |
| Local-header runner | `5666a78eb8e9806fe0fff0a6215189efd8dbe3175b9d58c305ae7caad6447538` |
| Bound ZIP-tail runner | `5987b04807fe21177fd017165ef299a87ca8ddac6fa14f818209b67644a8fb74` |
| Bound observation-discovery core | `3a574c6ecfa52603cfaa59b86be872aa4e42c99e0d7dc5e1cdcca0a399b92f74` |
| Synthetic fixture A/B | `1de7356a926e3185e6949be8cb55ba327a69342f77cc8d3357af55539dd377aa`, byte-identical |
| Synthetic local header | `3088f05b2390147e35d343153d1464f263f224401f448ad7573bbe5e22cf6ea6` |
| Real network/range/header/member/shadow bytes | `0 / 0 / 0 / 0 / 0` |
| Synthetic header/member bytes | `30 / 0` |

The external fixture reports are under
`/tmp/nextengine-physical-sound-colored-residual-header-v1` and are not
repository artifacts.

## Boundary and controls

The runner carries the audited archive URL, size, ETag, Last-Modified and range
support forward from the tail result. That closes identity drift between ZIP
structure discovery and the next request. Its accepted order is:

1. consume only a successful three-object tail audit with exact bound runner;
2. generate and repeat an offline manifest/preflight for three exact 30-byte
   header ranges;
3. acquire each header once with exact `206`, `Content-Range`, length, URL,
   ETag and Last-Modified gates;
4. audit the immutable 90-byte cache offline;
5. validate signature, compression method, flags and filename length against
   the central-directory record;
6. derive, but do not read, each compressed member range;
7. require a new manifest/preflight before any member payload access.

The synthetic control read header range `[70000, 70029]`, derived data offset
`70074` and compressed payload range `[70074, 70105]`. All four assertions
passed: exact 30-byte bound, central-directory match, derived byte count and
in-archive range. Spatula shadows remain absent and sealed; the authored-clip
fallback remains mandatory.

## Checks

- pinned tail-runner and discovery-core hashes: exact;
- Ruff format/check: pass;
- Python byte compilation: pass;
- synthetic fixture A/B: byte-identical;
- all four local-header assertions: pass;
- real network, range, header, member and shadow bytes: zero;
- current ProductChecks: not run because no public, runtime, content or
  production contract changed.

## Next action

Run only the three already frozen body-free identity requests when network is
available. Identity audit, tail manifest/preflight/acquisition/audit and header
manifest/preflight/acquisition/audit must remain separate evidence boundaries.
