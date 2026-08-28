# PS-2 existing-Iron broad-band counterfactual result — 2026-08-28

## Decision

`ExistingIronBroadbandCounterfactualRejected`.

Two byte-identical read-only executions reject the frozen real counterfactual at
its first bounded-capacity gate. The unchanged input-driven discovery finds 31
regions and 91 neighboring analysis bins, exceeding the preregistered maxima of
16 and 48. The runner therefore does not rank Iron peaks, execute common-pole
SVDs, fit amplitudes or inspect later gates.

This is not evidence that the recording lacks modal structure. Region/bin
selection is exactly invariant at scales `0.125/1/8`. The result establishes
only that the synthetic-supported candidate plus its first real execution
envelope does not transfer as frozen.

## Frozen lineage

| Artifact | SHA-256 / decision |
| --- | --- |
| Runner | `63d46bf476798e540cfc726a0e2bca58beaacc0f829ffff7ecc2b1083efb2e76` |
| External manifest | `8a445a4f0474ec560e659f6bf8ada9a90f4fb04fd5c04ef3e308843fcb43ff18` |
| Preflight A/B | `04e54415a3cf140364d27d541ccca4947039cde5608a668a3077d1452aefd357` / `ExistingIronBroadbandCounterfactualFrozen` |
| Analysis A/B | `7635f8aca44286e0a46709aa3268afd049d1038023768a8a265d6a0b24a5075d` / `ExistingIronBroadbandCounterfactualRejected` |
| Broad-band synthetic parent | `dcd832525fbd1955c9122fc52579e4a8ddd10ab76bc5d79393f8f9df9d270094` |
| Iron decode report / block | `47acdc35449390f639cda34990520c5f74cb8627d0932079345ca35735ffc099` / `e26d1df1d547d059bbcbc57453c522b8c23402461feec09ff6149bc37a9c7eeb` |
| Iron selector/tail parent | `02551f2995764197575d5dd30bb36bd44da5452e2af79533b43922f44fec48d1` |

Reports remain external under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-iron-skillet-broadband-counterfactual-v1`.
Each run records zero additional payload, network, physics and Planter access
and no quality, admission or runtime credit.

## Frozen observations

| Gate | Observed | Required | Result |
| --- | ---: | ---: | --- |
| Region count | 31 | 1..16 | **fail** |
| Analysis bins | 91 | at most 48 | **fail** |
| Scale-invariant region/bin signature | exact | exact | pass |

The frozen local-energy floor is `-30 dB` over 500–12,000 Hz. Of 31 regions,
17 lie within roughly 11 dB of the maximum, so the excess cannot be attributed
only to a few near-floor peaks. The opened report is descriptive evidence; it
does not authorize a new floor, top-K selection or a larger cap.

Only the existing rows `0..14`, onset sample `29` and the first 60,000
post-onset samples are addressed. The runner verifies the full existing block
and reads a bounded 3,601,740-byte slice; it downloads or decodes nothing new.
Because capacity rejects before pole estimation, no claim is available about
mode count, damping, spatial replication, reconstruction or predictive
advantage over the undamped control.

## Competing explanations and next action

- **H-capacity:** a real metal vessel legitimately exposes a denser set of
  energetic regions than the sparse seven-mode synthetic fixture, and the
  16/48 operational envelope is too small.
- **H-discovery:** the local `-30 dB` rule admits spectral leakage, transient
  structure or insignificant resonances that should be rejected by a
  source-independent criterion before expensive estimation.
- **H-both:** a denser real model is required, but the current region-by-region
  implementation also repeats unnecessary adjacent work.

The current result cannot distinguish these hypotheses because it deliberately
stops before estimation. Do not raise the Iron cap, choose the strongest 16,
change the floor or rerun a partial subset.

The next bounded package is primary-source research plus a fresh-seed dense
synthetic scaling/discriminator control. It must test whether all discovered
regions can be processed and pruned after estimation within a declared offline
budget, and include a leakage/nuisance control that can falsify indiscriminate
cap expansion. Only a successful frozen control may authorize an Iron V2
counterfactual. No new real object, payload, mechanics or Planter is allowed.

## Sources

- [Sirdey et al., Gabor/ESPRIT impact analysis](https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf)
- [Badeau, David and Richard, ESTER perturbation/order analysis](https://perso.telecom-paristech.fr/grichard/Publications/SP06_Badeau1.pdf)
- [SVD-based EDS order selection](https://ftp.esat.kuleuven.be/stadius/ida/reports/05-107.pdf)
