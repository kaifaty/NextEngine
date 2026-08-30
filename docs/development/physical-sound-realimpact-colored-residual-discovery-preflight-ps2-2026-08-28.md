# PS-2 REALIMPACT colored-residual discovery preflight — 2026-08-28

## Decision

`RealImpactColoredResidualArchiveIdentityDiscoveryFrozen`.

The successor's next network boundary is implemented and hash-closed. It can
make exactly one `HEAD` request for each fresh Pan, PiePan and Cup archive. It
cannot make a range request, read a response body, inspect ZIP structure,
access any member or touch Spatula shadow payload.

The acquisition was not run in the current managed sandbox because shell
network access is unavailable. No failed acquisition report was manufactured;
the accepted next action remains the same three one-shot requests when the
execution environment permits them.

## Frozen lineage

| Artifact | SHA-256 / result |
| --- | --- |
| Discovery runner | `898d1af13bf4dda4c3efed77b611ad757ccfe71f0548532e29c1a38c1a6c7c58` |
| Discovery manifest | `55be5ed2e71657a344251ccf6068720b0eb0a7f329866e680ca8fffc61b01abe` |
| Preflight A/B | `0d38bfb1e40838880d4f7955a74d8a9c14e0d480cff270fe722bb5cb56bf5d7b`, byte-identical |
| Successor manifest / preflight | `f03e428e…e564a` / `4c754f06…a360` |
| Network/range/body/member/shadow bytes | `0 / 0 / 0 / 0 / 0` |

Artifacts are external under
`/tmp/nextengine-physical-sound-colored-residual-discovery-v1`.

## Request contract

| Object | Role | Request |
| --- | --- | --- |
| `19_Pan` | calibration, family `pan` | one exact HTTPS `HEAD` |
| `37_PiePan` | calibration, family `pan` | one exact HTTPS `HEAD` |
| `22_Cup` | holdout, family `cup` | one exact HTTPS `HEAD` |

Every response must finish on the requested URL with status `200`,
`Accept-Ranges: bytes`, content length of at least 64 MiB, and non-empty `ETag`
and `Last-Modified`. The response body is never read. DNS must resolve only to
public addresses. Redirect identity drift, a missing header or the first
network failure rejects the acquisition and allows zero retries or object
substitutions.

The acquisition report—successful or rejected—is immutable. Only a successful
three-object report may be audited twice. Even then, a new protocol must freeze
exact ZIP-tail/local-header ranges before the first range request.

## Checks

- Ruff format/check and Python byte compilation: pass;
- manifest and preflight A/B: byte-identical;
- synthetic header parser validates length, identity headers and zero body;
- successor roster/manifest/preflight/fallback lineage: exact;
- network, range, response body, member and shadow payload: zero;
- current ProductChecks: not run because no runtime/content/public contract
  changed.

## Next action

When network execution is available, run the acquisition exactly once. If all
three identities pass, run the offline audit A/B and record its immutable hash.
Do not issue a range request in the same evidence boundary.
