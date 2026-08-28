# REALIMPACT Iron Skillet discovery and observation protocol — PS-2 — 2026-08-28

## Outcome

The bounded archive discovery for official `17_IronSkillet` succeeded and its
immutable cache verifies twice. Acquisition report
`b9f659b42e42137d80d456c8a87e8ddd771b8182490e40b27c72c33b69b16f6c`
uses exactly two range requests and reads only a 65536-byte ZIP tail plus a
30-byte observation local header. Offline audits emit byte-identical report
`a3bd84f1ccb743ecbee3d68b88e9bfeb309fa8351204db20a1a787f2538a8dab`.
Observation member payload bytes remain zero.

A separate one-impact protocol is now frozen before payload access. Manifest
`573ff0d6f50619aebe222314b9161d2f6ab066dc2720f5b0aa1f1870ec168c93`
binds protocol runner
`7385241d75abe219430267ca21fe82ab15b6706b3368b67cb15127f6c7838273`,
the complete discovery lineage, exact member identities/ranges, inferred NPY
shape, 600-row decode, unchanged salience/adaptive sources and all gates. Two
preflights emit byte-identical report
`19c1f57aadbbc104cda8d0c665f9b0047ef947bb2b9a07006d7a454ae57b2680`
with decision `IndependentOneImpactObservationProtocolFrozen`.

Neither protocol preflight makes a network request or opens an archive member.
No physics, Planter, quality, admission or runtime credit exists.

## Verified discovery

| Field | Value |
| --- | --- |
| Archive bytes | `2393994112` |
| ZIP tail | `2393928576..2393994111`, SHA-256 `d5626b4c…752a` |
| Central directory | offset `2393992795`, `1295` bytes, 12 entries, SHA-256 `c8b0bb57…91b4` |
| Observation entry | `17_IronSkillet/preprocessed/deconvolved_0db.npy` |
| Compressed / uncompressed | `2391054312 / 2766588128` bytes |
| CRC32 | `3669823f` |
| Local header / data offset | `2937922 / 2938027` |
| Local-header SHA-256 | `cd26f6df…0f64` |
| Discovery network requests | 2 |
| Observation / Planter payload bytes | `0 / 0` |

The uncompressed observation size closes the payload-free shape arithmetic:

`128 + 3000 * 230549 * 4 = 2766588128`.

The four required condition arrays are each `128 + 3000 * 8 = 24128` bytes.
No array contents were used to choose the shape or thresholds.

## Frozen future access

The next execution runner may make exactly four requests once:

1. bytes `159..865` for `angle.npy` and `micID.npy` local records;
2. bytes `611263..611654` for `distance.npy`;
3. bytes `2393992534..2393992794` for `vertexID.npy`;
4. bytes `2938027..539808938`, exactly 512 MiB of the observation deflate stream.

The metadata total is 1360 bytes. The audio prefix may decode only rows
`0..599` of impact ordinal zero, exactly 553317600 `f32` bytes. Prefix growth,
retry, object substitution and additional payload are forbidden.

## Frozen row identity and candidate

The execution must prove from metadata that rows `0..599` share one vertex,
contain the expected 40 angle/distance condition pairs and order microphone IDs
`0..14` within each condition. Reference row `7` remains microphone 7. The
source-derived selector aggregates the 15 synchronized outputs at the fixed
reference condition and uses unchanged:

- 65536-point spatial-power FFT over `250..12000 Hz`;
- relative `-55 dB` local-peak floor and ±`10%` stronger-bin suppression;
- `900 ms` injective persistence within `40 cents`;
- scale probes `0.125/1/8`; and
- adaptive 20 Hz bandpass RMS-envelope decay fits.

The old top-16 path is diagnostic only and cannot admit or reject this object.

## Frozen gates

- selected persistent modes from 6 through 16 inclusive;
- onset-to-tail persistence recall at least `0.50`;
- median onset/tail frequency error at most `40 cents`;
- valid adaptive fit fraction at least `0.75`;
- decaying adaptive fraction at least `0.50`;
- median adaptive fit `R²` at least `0.95`; and
- exact persistent-bin identity at all three scalar gains.

Any acquisition, decode, identity, repeat or gate failure publishes rejection
and retains the authored clip. Gates cannot be weakened after payload access.

## Decision and next action

Status is
`IRON_SKILLET_DISCOVERY_VERIFIED / ONE_IMPACT_OBSERVATION_PROTOCOL_FROZEN /
EXECUTION_RUNNER_REQUIRED / OBSERVATION_NOT_OPENED / MECHANICS_BLOCKED /
PLANTER_SEALED`.

Implement and hash-close the four-stage execution runner against this manifest,
then repeat the zero-access protocol preflight. Only that committed checkpoint
may authorize the four frozen requests. Do not access payload with the protocol
validator itself.
