# REALIMPACT independent observation preregistration — PS-2 — 2026-08-28

## Outcome

Official REALIMPACT `17_IronSkillet` is frozen as the next independent
development object before any archive member payload is opened. External
manifest
`a77f6d9750890af5bddbcc3d53bf79ae14d67f55f5c104a7fdaf22673599357f`
binds runner
`27b11de8c97f2ba7ee92d2b1c0dce02cee1ea732dc9a31e9426a5ffe181f70f1`,
the successful salience report, frozen roster, selection rule, archive HTTP
identity, two future byte ranges and the stop rule. Two local preflights emit
byte-identical report
`85c53d805c94f2c0975c86a96a4a3721dc140855d31a57776213b19704837ead`
with decision `IndependentObservationDiscoveryPreflightSupported`.

One metadata-only HTTPS `HEAD` request established the archive identity and
read zero response-body or member-payload bytes. The preflight itself performs
no network request. No geometry, audio, metadata member, physics or Planter
payload has been opened.

## Deterministic selection

The official 50-object roster is frozen at REALIMPACT commit
`fca2bd6cbb7e9f96ac61328d2a0d51594bf01987` and SHA-256
`3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a`.
Selection walks that published order and chooses the first object not already
opened or referenced by repository/external experiment evidence, excluding
reserved objects and `63_SmallPlanterCeramic`:

1. `100_Frisbee` is excluded because prior calibration planning references it;
2. `10_bowl` is excluded because prior development analysis opened it;
3. `17_IronSkillet` has zero exact repository and external-store references at
   selection time and becomes development object.

This metal object is deliberately object- and family-disjoint from the opened
Ceramic Cup method-development data. It grants no metal-domain or quality
credit.

## Frozen archive discovery

| Field | Value |
| --- | --- |
| Archive | `https://downloads.cs.stanford.edu/viscam/RealImpact/17_IronSkillet.zip` |
| Bytes | `2393994112` |
| ETag | `"6433da59-8eb17380"` |
| Last-Modified | `Mon, 10 Apr 2023 09:43:53 GMT` |
| Future tail range | `2393928576..2393994111` (`65536` bytes) |
| Future local header | exactly `30` bytes at the central-directory offset |
| Expected entry | `17_IronSkillet/preprocessed/deconvolved_0db.npy` |
| Future request ceiling | 2 |
| Member payload bytes allowed | 0 |

The runner rejects archive identity/range drift, redirects, non-public hosts,
non-`206` responses, unsafe ZIP structure and any output inside the repository.
The range pair can reveal only the ZIP central directory and fixed local-header
fields; it ends before observation member payload.

## Decision and next action

Status is
`INDEPENDENT_IRON_SKILLET_DISCOVERY_FROZEN / PRIOR_OBJECT_PAYLOAD_BYTES_ZERO /
PLANTER_SEALED / OBSERVATION_NOT_OPENED / MECHANICS_BLOCKED`.

Commit and transfer this checkpoint, then execute the exact two-range discovery
once and audit its immutable cache twice. Only byte-identical audits may
authorize a separate one-impact observation manifest using the unchanged
salience selector and adaptive estimator. Failure stops without object
substitution or threshold tuning.
