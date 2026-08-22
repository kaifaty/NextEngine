# NSR3-B4EP10SIRDIREA evaluation buffer liveness research -- 2026-08-22

Status: `COMPLETE / TIMING_FREE_STRUCTURAL_AUDIT_SELECTED`

## Question

B4EP10SIRDIRE attributes a stable 89.50% of `evaluation_setup` to seven
value-initialized vectors. Which initializations are structurally redundant,
and how many independent storage lanes would an exact reuse path need?

## Ownership and dataflow inspection

The selected SIRDI builder creates the following arrays before any pair work:

| Buffer | Extent | Writer | First in-builder read | Lifetime |
|---|---|---|---|---|
| evaluation gradient | fluid + support | target-owner fold | none; published result | returned workspace |
| evaluation density | fluid | density fold | center compression | returned workspace |
| radius | pairs | pair kernel | directed evaluation | returned tape/HVP |
| compression | fluid | center kernel | plan/directed/target | returned tape/HVP |
| HVP gradient coefficient | pairs | pair kernel | directed evaluation | returned tape/HVP |
| HVP second coefficient | pairs | pair kernel | later HVP | returned tape/HVP |
| density contribution | pairs | pair kernel | density fold | builder-local |

On the selected split-incoming path every gradient target is assigned, every
fluid center receives density and compression, and every pair receives radius
and both HVP coefficients. The density-contribution array is also written for
every pair before the later density phase reads it. This makes reuse plausible,
but code inspection is not evidence of complete parallel coverage.

The returned six-buffer workspace cannot inherit the earlier one-buffer
directed-scratch proof. A trust-region current workspace and its trial coexist;
on acceptance the old current is released and the trial becomes current, while
on rejection only the trial is released. Accepted substep workspaces can then
move through the retention diagnostic. Existing ownership counters admit a
maximum of two live query workspaces, but a reuse design needs its own slot and
release proof. The builder-local density contribution remains sequential at
the call level and projects to one independent ephemeral lane.

## Smallest discriminator

Add one opt-in, timing-free shadow audit over the unchanged SIRDI transaction:

1. mark every role/index write and reject publication unless all requested
   indices were written;
2. check density-contribution and density reads only after their producer
   phases completed;
3. issue a workspace-lane receipt on successful publication, carry it through
   move/retention, and consume it on the existing release function;
4. trace the one builder-local ephemeral lane through every exit;
5. project per-role high-water growth for the exact observed two-workspace plus
   one-ephemeral schedule without changing vector ownership;
6. run missing-write and duplicate-release corrupt shadows.

Run two fresh processes. Require the SIRDI semantic result, all roots and work
counts exactly; 226 workspace acquire/releases, maximum two simultaneous
workspaces, one ephemeral lane, zero final live receipts, shadow payload at
most 8 MiB and projected reusable capacity at most 64 MiB. The high-water
growth/full-initialization byte ratio must be at most 2% before implementation
contract research is allowed.

## Decision

Freeze B4EP10SIRDIREA as a structural audit only. It may authorize research of
one buffer-reuse implementation contract, but cannot remove initialization,
introduce a pool or claim speedup itself. The optimized representation must be
designed only after this audit proves its exact lane count and role coverage.
