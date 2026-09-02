# Physical sound V32 V0 — validator mechanics result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `REPEAT_EXACT_REJECT / CLEAN_FALSE_POSITIVE / NO_PUBLICATION` |
| Protocol | [V32 V0 validator mechanics protocol](physical-sound-v32-v0-validator-mechanics-protocol-2026-09-02.md) |
| Profile | [`physical-sound-v32-v0-validator-mechanics.v1.json`](../../lab/profiles/physical-sound-v32-v0-validator-mechanics.v1.json) |
| Owner | [`physical_sound_v32_v0_validator_mechanics_v1.py`](../../lab/scripts/physical_sound_v32_v0_validator_mechanics_v1.py) |
| Product effect | None; authored clips remain authoritative and SPEC-45 remains `Proposed` |

## Result

The frozen V0 owner rejects on the first affected clean candidate:

```text
intervention-density -> Reject(RetrievalCopyDetected)
```

The expected result is `Pass`. Both complete CLI attempts exit `2`, emit the
same stderr bytes and publish no output directory. The exact stderr SHA-256 is
`3b06d3131c6309ccb8896cea6f6f8904ee763123b70357a12cd213c712a38224`.

Decision: `V0_VALIDATOR_MECHANICS_REJECT_CLEAN_CAUSAL_ALIAS`.

This is a validator-protocol defect, not evidence against T0 or against
automatic validation. No real threshold, model, role, signal or protected
value opened.

## Exact cause

V0-v1 froze retrieval as “candidate audio hash equals a clean hash for any
different target ID”. T0 legitimately contains two clean causal equivalence
classes:

| Clean IDs | Shared audio SHA-256 | Modal relation |
| --- | --- | --- |
| `intervention-density`, `intervention-uniform-scale` | `bbea973933971afc1e02f872da6e507cd812b2aa6dc1ee397ce6ddb9ea2c9b65` | exact modal signatures equal |
| `intervention-thickness`, `intervention-youngs-modulus` | `6ada05a4d7b80174631ef51e333978e22f7bfd39dc127e4c01baa4cd6f69451e` | exact modal signatures equal |

These aliases follow the frozen analytic interventions: density×4 and uniform
scale×2 both produce the same `0.5` frequency ratio without changing gains;
Young's modulus×4 and thickness×2 both produce ratio `2`. Equal target PCM is
therefore correct inside each equivalence class.

The actual `mutation-spectral-copy` remains different: plate PCM is presented
for the beam target, whose modal signature is not equivalent. Retrieval must
be defined over `(audio hash, target modal signature)`, not target ID alone.

## Frozen identities and verification

| Artifact | SHA-256 |
| --- | --- |
| V0 profile | `8865cf8ca52f76816ad538c1ebedd44ba8ee68a6a96e1f1980a50cd35ebe4709` |
| V0 owner | `267ab74bd1ff56623aa19b25d060013adabdc1b8e695562b127aa1279e4f16b4` |
| Repeated stderr | `3b06d3131c6309ccb8896cea6f6f8904ee763123b70357a12cd213c712a38224` |

The focused frozen-reject suite passes `3/3`: it reproduces the complete-entry
atomic reject, proves the two hash aliases have exact modal signatures, and
proves ignored labels do not enter the candidate view.

## Consequence

V0-v1 is closed. Do not weaken the retrieval threshold or special-case the
observed record IDs. V0a must freeze a general equivalence rule before any new
validator feature values:

```text
RetrievalCopyDetected iff
  candidate audio equals a clean target other than its own equivalence class
  AND that clean target's exact modal signature differs from the requested target.
```

All other V0 specialists, thresholds, candidate-view fields, label isolation,
cache composition, precedence and resource limits remain unchanged. V0a must
retain a negative control proving that the plate-to-beam spectral copy still
rejects.
