# PS-2 broad-band independent holdout discovery preregistration — 2026-08-28

## Outcome

Freeze official REALIMPACT `43_IronMortar` as the object-disjoint method
holdout before any archive member payload is opened. Discovery may read only a
65,536-byte ZIP tail and the 30-byte local header of
`43_IronMortar/preprocessed/deconvolved_0db.npy`.

The holdout tests whether the all-region partial-SVD common-pole method transfers
beyond the opened Iron Skillet. It grants no material identity, perceptual
quality, domain admission, physics or runtime credit.

## Deterministic selection

The official 50-object roster is frozen at REALIMPACT commit
`fca2bd6cbb7e9f96ac61328d2a0d51594bf01987` and SHA-256
`3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a`.
After the Iron Skillet parent, selection chooses the first roster name that:

- contains the exact token `Iron` or `Metal`;
- is not a `Pan` or `Skillet` family;
- has zero exact repository and external-store references at selection time.

`17_IronSkillet` is the parent. `43_IronMortar`, roster index 16, is the first
eligible entry and had `0/0` exact references at commit
`8397f7927851a07daed352393adcfbd4137b2c15`. Later `67_IronPlate` and
`90_MetalLadle` are already opened development objects and cannot be independent
holdouts.

The object name is only a deterministic selection token. It does not establish
material truth or authorize a metal-domain claim.

## Frozen archive identity

One HTTPS metadata request read zero response-body or member-payload bytes and
established:

| Field | Value |
| --- | --- |
| Archive | `https://downloads.cs.stanford.edu/viscam/RealImpact/43_IronMortar.zip` |
| Bytes | `2305935628` |
| ETag | `"6433e2e6-8971c90c"` |
| Last-Modified | `Mon, 10 Apr 2023 10:20:22 GMT` |
| Accept-Ranges | `bytes` |
| Future tail range | `2305870092..2305935627` |
| Expected observation | `43_IronMortar/preprocessed/deconvolved_0db.npy` |
| Future request ceiling | `2` |
| Member payload bytes allowed | `0` |

## Frozen lineage and stop rule

Discovery binds Iron V2 report `f2fb359f…6a73` and its result document. The
runner rejects parent drift, archive identity/range drift, redirects,
non-public hosts, non-206 responses, unsafe ZIP structure and any output inside
the repository.

The runner, external manifest and two byte-identical zero-network preflights
MUST be committed before the exact two-range acquisition. After acquisition,
two offline cache audits must match byte-for-byte. Only then may a separate
one-impact V2 holdout manifest authorize audio access. Failure stops without
substitution, larger ranges or threshold tuning.

## Frozen evidence identity

Runner, manifest and preflight hashes are filled after the implementation is
committed and the zero-access preflight repeats. External artifacts remain under
`~/.codex/experiments/nextengine/physical-sound/ps2-realimpact-iron-mortar-broadband-holdout-discovery-v1/`.
