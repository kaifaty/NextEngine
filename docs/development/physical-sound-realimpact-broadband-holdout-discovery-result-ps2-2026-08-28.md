# PS-2 broad-band independent holdout discovery result — 2026-08-28

## Decision

`BroadbandIndependentHoldoutDiscoveryCacheVerified`.

Official REALIMPACT `43_IronMortar` is frozen as the object- and name-family-
disjoint method holdout. One bounded acquisition used exactly two HTTP range
requests to read the ZIP tail and observation local header. Two offline audits
are byte-identical. No archive member payload was opened.

## Frozen lineage

| Artifact | SHA-256 / decision |
| --- | --- |
| Runner | `c59aea1d0fd8ae2dd565929120b207f33326c697d2daacaf82469e0830c679cd` |
| External manifest | `931f6a72f16f9a52bc7386d40aad1b5e0d5f674bd0be410ed5d35a5c090bf5e7` |
| Preflight A/B | `3b4c5b6a6e79d5c9a69ff1bc017415e522874ca365a53741d831ffe10341c8bd` / `BroadbandIndependentHoldoutDiscoveryPreflightSupported` |
| Acquisition | `84308e8e28623d114f1d0002c9a7f5cc94bb151dc78d7aed1be1dbf35d39d080` / `BroadbandIndependentHoldoutArchiveEntryDiscovered` |
| Audit A/B | `67c92621d1a58507f92303b709427ecfb6ad0eb972e497621ba9ce8bf4c48a42` / `BroadbandIndependentHoldoutDiscoveryCacheVerified` |
| Iron V2 parent | `f2fb359faaccb36544203780f07cc1d359c5d529be7f7f0d533ae39bd50b6a73` |

External artifacts remain under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-iron-mortar-broadband-holdout-discovery-v1`.

## Verified discovery

| Field | Value |
| --- | ---: |
| Archive bytes | `2305935628` |
| ZIP tail SHA-256 | `f8bd43a7…a13af` |
| Central directory | offset `2305934323`, `1283` bytes, 12 entries |
| Central directory SHA-256 | `f7d069c9…99a9c` |
| Observation entry | `43_IronMortar/preprocessed/deconvolved_0db.npy` |
| Compressed / uncompressed | `2302282721 / 2500500128` bytes |
| CRC32 | `9da9ace4` |
| Local header / data offset | `3045909 / 3046013` |
| Local-header SHA-256 | `61ce30bf…7d16` |
| Acquisition network requests | `2` |
| Audio / Planter payload bytes | `0 / 0` |

The observation size closes the payload-free shape arithmetic exactly:

`128 + 3000 * 208375 * 4 = 2500500128`.

Therefore impact ordinal zero contains 600 listener rows of 208,375 samples,
longer than the 60,029-sample V2 input slice and both predictive windows. This
proves capacity only; it reveals no waveform values.

## Next action

Freeze a separate one-impact holdout protocol before payload access. It must:

- bind this immutable discovery and the complete Iron V2 algorithm/gates;
- read only the minimum condition arrays needed to prove rows 0…599 share one
  impact point and contain synchronized microphone IDs 0…14;
- authorize one bounded observation deflate prefix sufficient to decode only
  impact ordinal zero;
- use input-derived onset and complete region discovery without threshold
  tuning, opened-region ranking or object substitution;
- publish support or rejection twice and retain authored-clip fallback.

No result from this discovery grants material, quality, domain, physics or
runtime credit.
