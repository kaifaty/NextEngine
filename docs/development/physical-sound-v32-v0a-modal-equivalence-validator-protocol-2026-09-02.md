# Physical sound V32 V0a — modal-equivalence validator protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-02` |
| Status | `FROZEN_AFTER_V0_CLEAN_FALSE_POSITIVE / BEFORE_V0A_FEATURE_VALUES` |
| Roadmap package | V32 `V0a` |
| Profile | [`physical-sound-v32-v0a-modal-equivalence-validator.v1.json`](../../lab/profiles/physical-sound-v32-v0a-modal-equivalence-validator.v1.json) |
| Replaces | [V0-v1 rejected result](physical-sound-v32-v0-validator-mechanics-result-2026-09-02.md) |
| Product effect | None; synthetic mechanics only, authored clips authoritative |

## Falsifiable successor hypothesis

V0-v1 falsely treated every equal clean audio hash with a different target ID
as retrieval. T0 proves two different causal interventions can lawfully produce
the same PCM and exact modal signature.

V0a changes one scientific rule:

```text
RetrievalCopyDetected iff candidate audio equals a clean target hash
  outside the requested target's exact modal-equivalence class.
```

The requested target and copied clean target are equivalent only when their
complete ordered modal signatures are exactly equal in:

```text
ordinal, family_index_a, family_index_b,
frequency_hz, decay_per_second,
contact_participation, pickup_participation, signed_gain
```

No tolerance, target-ID exception or observed-record allowlist participates.
The successor passes only if both lawful clean alias pairs become `Pass` and
the plate-PCM-as-beam mutation remains `Reject(RetrievalCopyDetected)`.

## Frozen lineage

V0a is based on commit `4260b4d24148bb0ce2adc2158138a4fd8cd1bf72` and
binds:

| Parent | SHA-256 |
| --- | --- |
| T0 owner | `4a0c5a5ea7cece8dfca2c8d346695bc826b9b375d94c2ef5aa1d723760391d6f` |
| T0 profile | `44d83804398b2e08a54dbe28e4e3269098b1112785d8377dac34a11d4e5eebe8` |
| V0-v1 profile | `8865cf8ca52f76816ad538c1ebedd44ba8ee68a6a96e1f1980a50cd35ebe4709` |
| V0-v1 frozen core | `267ab74bd1ff56623aa19b25d060013adabdc1b8e695562b127aa1279e4f16b4` |
| V0-v1 reject result | `b01f85f3f255a6ce4b35f40c0e729b675649f9de0222dd5b53665949938be852` |

V0a is a thin policy adapter over the frozen V0 core. Its release must bind
both adapter and core hashes. It may not import P1/T0 generator code.

## Unchanged V0 mechanics

Except for the retrieval equivalence and the necessary V0a identity/cache
domain, every frozen V0-v1 value remains byte-for-byte semantic input:

- candidate view and ignored label fields;
- canonical PCM, clipping, silence/DC and provenance rules;
- modal coverage/parameter and physical-decay checks;
- lag-256 de-enveloped periodicity threshold `1e-5`;
- 12-block carrier-demodulated envelope rule and `1e-6` monotonic tolerance;
- reason precedence and `UnsupportedTargetCase` OOD;
- 16 candidates, 35 files, resource bounds and atomic publication;
- zero real/protected signal, network and model access.

V0a uses a new cache domain and profile hash, so failed V0-v1 cache entries
cannot be relabelled as successor evidence. Candidate labels remain excluded.

## Implementation and gates

The adapter may reuse deterministic parsing/feature construction from the
hash-pinned V0 core. After core features are produced it must:

1. recompute copied clean target IDs from audio hashes;
2. derive exact modal signatures from the trusted clean catalog;
3. remove only copied IDs whose signature equals the requested target;
4. publish the filtered IDs as the retrieval feature;
5. reapply the unchanged full reason precedence from the already computed
   features, not force `Pass` after removing retrieval.

V0a passes only when:

1. two complete runs publish byte-identical 35-file trees and stdout;
2. the exact T0 matrix is `9 Pass / 7 Reject` with all seven reasons once;
3. both causal alias classes pass while plate→beam copy rejects;
4. ignored labels, cold/warm cache and reverse order remain byte-identical;
5. every V0 hard/modal/temporal/envelope/OOD/corruption/atomic probe still
   passes under the successor profile;
6. core, adapter, protocol, profile and T0 identities match;
7. resource and zero-access gates pass.

A pass authorizes only V32 M0 bounded-correction protocol work. It is not a
real validator release and grants no real material, threshold, training,
admission, cooker, demo, runtime or ProductCheck credit.

## Stop rules

- Do not change another threshold, specialist or precedence in V0a.
- Do not enumerate the two observed alias pairs in implementation.
- Do not accept a copy based on waveform similarity; equivalence is exact
  trusted modal identity only.
- If plate→beam escapes or another clean case rejects, V0a closes. A later
  successor needs a new preregistered hypothesis.
- Generated features, decisions and caches remain outside Git.
