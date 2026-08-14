# TRAIN-4 support authorization and quantized-clearance research

| Field | Value |
| --- | --- |
| Date | 2026-08-14 |
| Scope | Optimizer-free bounded research for `REQ-HUM-DATA-005/007` |
| Status | `BOUNDED_PASS / CLIP_GLOBAL_ONLY` |
| Accepted bounded evidence | V7/R49 fresh scene `17/17`, required safety `0` |
| Claim ceiling | Permits one clip-global prototype; does not authorize V19, TRAIN-4 Advance or optimization |

## Frozen constraints

The cycle retained every V5 acceptance bound: active tangential motion
`2000 µm/frame`, active normal motion `1000 µm/frame`, active residual
`5000 µm`, collider floor `-2 µm`, joint-velocity reserve `2500` basis
points and root vertical velocity `200060 µm/s`. Fresh scenes remained the
only acceptance authority under ADR-070. Indexed partial reset stayed
`NOT_RUN / report-only`; optimizer steps and training runs remained zero.

## Evidence sequence

| Evidence | Result | Interpretation |
| --- | --- | --- |
| R36 V5 four-case offline, canonical `a6eba9ada17f50ccdda4215de9f5753940b886ba50ee818195f6ff6c7d7c65a5` | `PASS 4/4` | V5 was safe to test in fresh scenes |
| R37 V5 fresh discriminator, file SHA-256 `72dedf7ca29180f55b5f747aa03a7865d91171a3d5c7a2d259a5b10b81018bc4` | `PASS 4/4` | Permitted the ordered all-17 bounded build |
| R38 V5 all-17 offline, canonical `d2c5ee19cf8bb70508c5105dfe2e9ee3c68ecd5bd1e75f4520efe7fdc6dd223b` | `PASS 17/17` | Analytic closure alone did not prove PhysX safety |
| R39 V5 all-17 fresh, file SHA-256 `70555aa69b736c825e9085fab726f6533e29184f67c44144cab77cbe8c286299` | `FAIL 1/17` at `cmu16@249`: left-ankle impact `6319967 µN·s` | Five-millimetre clearance without active support delayed landing and increased impact |
| R40/R41 bilateral A/B, worker SHA-256 `011fa70859ef025c1d16dc4d1afc7dbc9bec530abcb8f8c7d6ccf5a60fa4dd28` / `ab514250d3054e9bcf338ece4af1b3b845df586d20d1cab8d91e3acdc5174b4b` | V1 touched on tick 2 and peaked at `4599445 µN·s`; V5 touched on tick 3 and failed on tick 5 | Confirmed delayed-contact causality rather than a root-velocity-closure defect |
| R43 V6 five-case fresh, file SHA-256 `380ed7fd6815b2bd5ed896ba96a3fefa0eae3fa70aa417052a0063545f365a14` | `PASS 5/5` with unsupported target `0 µm` | Support-authorized clearance fixed `@249` and retained the earlier discriminator |
| R45 V6 all-17 fresh, file SHA-256 `ba60b124b8b9e4c8df024edff92b949d9dd68f213d02c0e675277d285f09aa40` | `FAIL 1/17`: `cmu16@415` right-ankle hard-ROM excess `297 µrad` | A smaller inventory did not expose the slot-16 boundary; all-17 remains mandatory |
| R46 exact V1 `@415`, worker SHA-256 `f92ad083ac60d6e4ab3e7adfeeb7258a2ec53a8d375ae9ec6369012d1024c128` | `PASS`; maximum right-ankle-pitch error `221121 µrad` | The V6 one-microradian reference change, not zero-clearance policy itself, selected the failing trajectory |
| R47 V7 all-17 offline, canonical `1e56a2d3d14c8d3a8291639da49d4fda46263aa37c1d6682205343d4043202bb` | `PASS 17/17`; V1 bytes exact at `@249` and `@415` | A one-micrometre quantization deadband preserves admissible source samples above the unchanged `-2 µm` floor |
| R48 exact slot-16 V7 worker, SHA-256 `2965ce0f4ace3de1155885ef539cde1a2040e3325e622a9a3a7f167655eb5d18` | `PASS` | Authorized the full fresh matrix |
| R49 V7 all-17 fresh, canonical `fc8ca9ba733c1baf30c930f71c840fe81f95f2494510423002817914856d6097`, file SHA-256 `6b977a870c50b25545c2b73bc371575b38b40f6a914d1638ab19a3c7a56b5f0c` | `PASS 17/17`; every required-safety count `0`; controls regressed `0`; all three targeted categories `0` | Permits only the clip-global successor |

All generated evidence is under the external
`humanoid-motor-rebuild-v1/evaluations/TRAIN-4` root and was produced from a
clean repository commit.

## Mechanism and decision

The V5 unsupported-flight target lifted `cmu16@249` from an early, continuous
landing into a later discrete impact. Clearance is therefore authorized only
when another foot is an active same-frame support. Frames with no credible
support use a zero-clearance nonpenetration target. V6 demonstrated that
policy, but a sub-micrometre FK deficit caused a one-microradian ankle edit at
`@415`; the all-17 PhysX trajectory crossed hard ROM even though the five-case
probe passed. V7 adds a tightly validated `0..1 µm` quantization deadband. It
does not change the collider floor or any safety limit and preserves the exact
V1 projected bytes for both all-flight controls.

The bounded result does not resolve projection-domain inconsistency.
Independent windows still assign different values to the same source frame,
and the earlier 801-frame `cmu16` counterfactual failed complete-clip bounds.
The generic report field `accepted_for_full_v19_build` describes only the
selected-case predicate; the explicit R49 field
`full_v19_corpus_authorized=false` and gate decision
`PERMIT_CLIP_GLOBAL_PROTOTYPE_ONLY` are authoritative.

## Next increment

Build one immutable clip-global research prototype over the three selected
clips. Each clip must be solved exactly once, and every selected episode must
be a byte-exact slice of that one trajectory. The prototype must prove:

1. one value for every `(clip_id, reference_frame)` and exact overlap identity;
2. complete-clip point, collider, joint and root invariants under unchanged
   limits;
3. selected-window offline and fresh-scene `17/17` safety from clip slices;
4. no new corpus, native, visual or exhaustive claim until the complete clips
   pass.

The next research step is to localize the non-convergent full-clip constraints
before choosing another solver identity. Repeating window-local overlays,
relaxing bounds or starting an optimizer remains forbidden.
