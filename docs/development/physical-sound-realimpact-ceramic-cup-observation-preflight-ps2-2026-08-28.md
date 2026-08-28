# REALIMPACT Ceramic Cup observation protocol — PS-2 — 2026-08-28

## Outcome

The payload-free discovery for official REALIMPACT `78_CeramicCup` executed
within its frozen two-request boundary and now repeats from immutable cache.
Acquisition report `2b183dae20e38350098ce7b986a356fd34033a1ebdd83d207cc311a2de80782b`
and offline audit `cd68ba79b73aceef579a8243674b1448ccc3248fb074d82486e796551f641b0b`
establish the exact ZIP structure without reading audio, metadata, geometry or
Planter payload bytes.

The next observation-only protocol is also frozen before payload access.
Manifest `71123b211ade913a390a88deb34b8e059dd05dd98ae294ba867fcf1c1a4bcae5`
binds runner `bad26592744ce90d7d7bf54382924be100b2a1e438c95844adaa5681759360e2`,
the immutable discovery, two metadata ranges, one `512 MiB` audio prefix, the
decoder, impact-zero rows `0..599`, reference row `7`, the existing V2
extractor and all five unchanged V2 holdout gates. Two clean local runs emit
byte-identical preflight report
`5f34993f14b46c30d8f916dc09ed27f583d8c33f36939f350459013ae2b18f21`
with decision `CeramicCupObservationProtocolFrozen`.

This checkpoint authorizes one exact three-request acquisition after commit.
It provides no valid observation, mechanics, material, perceptual quality,
domain admission, `Pass`, runtime role or ProductCheck credit.

## Executed discovery

| Artifact or fact | Frozen result |
| --- | --- |
| Discovery manifest / runner | `cf2b72ee…a6a9` / `3a574c6e…2f74` |
| Acquisition / offline audit | `2b183dae…782b` / `cd68ba79…1b0b` |
| ZIP tail | `eef6c538…351c`, `65536` bytes |
| Audio fixed local header | `a1b86037…47c`, `30` bytes |
| Central directory | offset `2320962957`, `1283` bytes, 12 entries, `ce6d4084…72f7` |
| Observation member | `deconvolved_0db.npy`, CRC32 `460b3ca0`, deflate, `2318456222 / 2506740128` compressed/uncompressed bytes |
| Observation data offset | `2506735` |
| Network requests / object payload / audio payload / Planter payload | `2 / 0 / 0 / 0` |

Both offline audit reports are byte-identical. The discovery runner re-parses
the central directory and local header and checks every recorded hash and
offset without network.

## Frozen observation access

The execution manifest allows exactly these ranges and no retry or growth:

| Role | Inclusive range | Bytes |
| --- | ---: | ---: |
| `vertexXYZ` through `listenerXYZ` local records | `614950..618705` | `3756` |
| `angle`, `distance`, `micID`, `vertexID` local records | `2505276..2506630` | `1355` |
| `deconvolved_0db.npy` raw-DEFLATE prefix | `2506735..539377646` | `536870912` |

The decoder is frozen to NPY `<f4`, C order, shape `[3000, 208895]`, a
128-byte header and exactly the first 600 rows / `501348000` raw bytes. The
metadata path requires two `[3000, 3]` `<f8` coordinate arrays and four
`[3000]` `<i8` identity arrays. It must prove one constant impact vertex, the
`10 × 4 × 15` angle/distance/microphone product, microphone order `0..14` and
that row `7` is microphone `7` before analysis.

The large prefix is streamed directly to an external staging file while its
length and SHA-256 are computed; it is not retained in memory. Any failed,
redirected, non-`206`, identity-changed, truncated or oversized request
publishes an immutable rejection. No retry or larger prefix is allowed.

## Frozen observation gate

The analysis path binds:

- Python parity source `cf93b1fc…6772`;
- Rust transfer gate source `2624656e…f80` and DSP `131bbf42…9ca4`;
- Rust fixture report/sample `2e3db2d3…5572` / `c8316f81…0477`;
- profile `injective-modal-16-fft65536-v2`; and
- minimum modes `6`, persistent recall `0.50`, maximum median frequency error
  `40 cents`, minimum decaying fraction `0.50` and maximum median tail RMSE
  `24 dB`.

The preflight fixture parity remains within the already frozen tolerance, with
maximum absolute error `3.0233593406592263e-12`. The runner contains no scalar,
shell, volumetric, boundary-element or cooker execution path. Only a fully
passing observation may authorize a separately frozen mechanics discriminator.

## Reproduction

```text
<lab-venv>/bin/python \
  lab/scripts/physical_sound_realimpact_observation_execute.py \
  --manifest <experiment>/observation-execution-manifest.json \
  --stage preflight \
  --output <fresh-external-output>
cmp <preflight-a>/report.json <preflight-b>/report.json
```

## Next action

Commit and transfer this hash-closed checkpoint. Then run the single authorized
three-request acquisition, decode the immutable cache once and execute the
observation gate twice without network. On rejection stop with authored clips;
on admission freeze a vector shell/hollow-volume discriminator. In either case
keep Planter sealed.
