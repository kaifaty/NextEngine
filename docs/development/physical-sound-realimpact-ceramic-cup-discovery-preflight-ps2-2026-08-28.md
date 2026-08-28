# REALIMPACT Ceramic Cup observation discovery preflight — PS-2 — 2026-08-28

## Outcome

The observation-first development discriminator now has a hash-closed,
payload-free discovery preflight for official REALIMPACT `78_CeramicCup`.
Manifest `cf2b72ee987a5bdbee7e67f250abc747aae7cd8ddc9a609ccfd3660cce92a6a9`
binds the object, archive HTTP identity, runner, exact byte ranges, request
ceiling and stop rule. Two local runs produce byte-identical report
`5fa54efefd589bf7ba16ca70aeb27810be5166309e31f0da43c26a60391a17ee`
with decision `CeramicCupObservationDiscoveryPreflightSupported`.

No archive request or object payload was made by the preflight. This package
authorizes only one future discovery execution with at most two HTTPS range
requests:

1. archive bytes `2320898726..2320964261`, exactly `65536` bytes containing the
   ZIP tail and expected central directory;
2. exactly the `30` fixed bytes of the audio entry's ZIP local header, at the
   offset learned from the first response.

The second range ends before the filename, extra field and compressed member
data. Audio, metadata, geometry and Planter payload bytes remain prohibited.

## Frozen identity

| Field | Value |
| --- | --- |
| Object | `78_CeramicCup`, development only |
| Official roster | REALIMPACT commit `fca2bd6c…f01987`, roster `3ee26ac9…ad5a` |
| Archive | `https://downloads.cs.stanford.edu/viscam/RealImpact/78_CeramicCup.zip` |
| Archive identity | `2320964262` bytes, ETag `"6433ded5-8a571aa6"`, `Mon, 10 Apr 2023 10:03:01 GMT` |
| Runner | `lab/scripts/physical_sound_realimpact_observation_discovery.py`, `3a574c6e…2f74` |
| Manifest | `cf2b72ee…a6a9` |
| Preflight A/B | `5fa54efe…17ee` |
| Preflight network/object/audio/Planter bytes | `0 / 0 / 0 / 0` |

The object appears in the frozen official
[REALIMPACT roster](https://github.com/samuel-clarke/RealImpact/blob/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/dataset/object_names.txt).
Repository and external experiment inventories had no prior `78_CeramicCup`
reference at selection time. It does not replace or weaken the sealed
`63_SmallPlanterCeramic` holdout.

## Safety and claim boundary

The runner rejects redirects, non-HTTPS or credential-bearing URLs, private or
non-global DNS results, non-`206` responses, changed ETag/Last-Modified/length,
range overrun, ZIP multi-disk/ZIP-entry changes and non-empty repository
outputs. Acquisition will store only the exact tail and fixed local header in
the external experiment cache. Offline audit must reproduce the parsed central
directory and observation offset before any new manifest is created.

This is archive discovery only. It establishes no valid observation, material,
support, force, geometry, mechanics, acoustic transfer, quality, domain
admission, `Pass`, runtime role or ProductCheck credit.

## Next action

Run the single authorized discovery execution, then audit its immutable cache
twice without network. If and only if those reports agree, freeze a separate
manifest for exact non-audio metadata/geometry and one impact's 600-row
observation block. That later protocol must run all five unchanged V2
observation-admission checks before any scalar, shell, volumetric, Bempp or
cooker comparison.
