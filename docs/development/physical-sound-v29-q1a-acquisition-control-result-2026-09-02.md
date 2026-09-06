# Physical sound V29 Q1a-M acquisition control result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `Q1A_OWNER_COMPLETE / OFFICIAL_CAPTURE_PENDING_PROVIDER_COOLDOWN / NO_ARTIFACT / NO_SOURCE_POWER_CREDIT` |
| Owner commit | `0c07a943eb13f95d47cad1a2e3bc458670180cfd` |
| Protocol | [Q1a-M source growth](physical-sound-v29-q1a-metal-source-growth-protocol-2026-09-02.md) |
| Profile | `lab/profiles/physical-sound-v29-q1a-source-growth.v1.json` |

## Outcome

The metadata-only Q1a owner and its focused guards are complete. The official
capture is not complete: repeated public-page acquisition attempts terminated
on Freesound connection/read timeouts, and atomic output publication correctly
left no `capture.json`, raw page set, audit or source-power result.

This is an acquisition availability result only. It does not establish that a
candidate project is eligible or ineligible, does not change Q1-M, and grants
no role, freshness or protected-payload access.

## Implemented boundary

The owner has two explicit phases:

1. `capture` resolves the frozen Git baseline, requests only the exact
   preregistered public HTML URLs, rejects redirect/non-HTML/oversize responses
   and atomically publishes raw pages plus a canonical hash manifest outside
   the repository;
2. `audit` is network-free, checks author/pack/sound/license/material/action
   identity, exact-Steel policy, conflicts, freshness and access ledgers, then
   exhaustively searches a whole-project protected-role partition.

The transport uses one sequential keep-alive `curl` batch with one GET per
exact URL, HTTPS-only routing, no redirect, no retry, a `1 MiB` per-page ceiling
and fail-early atomic publication. `curl` is capture transport, not evidence
authority; raw response hashes and normalized publisher identities are.

## Focused verification

At owner commit `0c07a943…0cfd`:

- Q1a tests: `12/12 PASS` on the repository-pinned Python `3.12.13`;
- combined Q0-M/Q1-M/Q1a tests: `35/35 PASS`;
- Python bytecode compilation: `PASS`;
- staged diff whitespace check: `PASS`;
- Black/Ruff: `NOT_RUN`, because neither module is installed in the pinned lab
  environment; this is a tool-availability fact, not a passing lint claim.

The tests cover strict profile identities, duplicate projects, exact uploader
and pack membership, compatible license, unqualified Steel versus
Stainless/Carbon/Zinc-plated Steel, whole-token impact action, zero-signal
capture counters, symlink/repository-output rejection, one-process keep-alive
transport and the distinction between aggregate floors and whole-project
partition feasibility.

## Bounded acquisition diagnosis

The first acquisition variants timed out on different public HTML pages and
published no partial output. Small controls then showed:

- both previously failing sound-page HEAD requests returned HTTP `200` in
  approximately `1.17 s` and `1.11 s`;
- one full public sound-page GET returned HTTP `200`, `text/html`, `39,145`
  bytes in approximately `1.40 s`;
- one two-URL `curl` transfer returned HTTP `200` for both pages and reported
  one new connection for the first URL and zero for the second, proving
  keep-alive reuse;
- the later official batch still failed its first connection after `15 s`.

These controls discriminate a transient/provider connection condition from a
known missing or redirected source. They do not form a frozen capture and are
not counted as source evidence.

## Access and artifact accounting

- No audio, preview, waveform, API payload, mesh, force sample, PCM sample,
  feature or role signal was requested or decoded.
- The failed external attempt roots contain no files.
- No source HTML, capture, dataset, checkpoint, generated WAV or model artifact
  entered Git.
- Q1-M remains `Q1M_SOURCE_POWER_INSUFFICIENT_SOURCE_GROWTH_REQUIRED`; every
  role remains empty and every protected payload stays sealed.

## Decision and next action

Do not retry acquisition until a provider cooldown interval has elapsed, and
do not alter the profile, exact-Steel policy or power thresholds based on these
network failures. The next action is one fresh atomic capture into a new
external root, followed by offline audit A/B and recursive comparison.

If that capture succeeds but the whole-project partition remains OOD, publish
the exact best-frontier deficits and start the smallest gap-directed internet
increment. If provider availability remains persistently inadequate, add a
provider-independent metadata adapter under a fresh protocol; do not ask the
user to record or validate sounds and do not manufacture an incomplete corpus.
