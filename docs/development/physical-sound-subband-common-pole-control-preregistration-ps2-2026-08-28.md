# PS-2 subband common-pole control preregistration — 2026-08-28

## Outcome

Freeze the first synthetic-only mathematical control after the Iron fixed-tail
diagnosis. It tests whether a multichannel Gabor coefficient sequence can
recover common exponentially damped poles without any fixed late survival
observation.

This package deliberately proves only a preselected high-energy subband core.
It does not discover active bands, merge duplicate estimates, prune by
perceptual energy, read real audio or authorize a real counterfactual.

## Primary-source boundary

Sirdey et al. show that a damped sinusoid remains a damped exponential in a
Gabor subband, allowing ESPRIT to use a longer observation at a lower effective
sample rate. Their real metal-plate example estimates model order per selected
channel and removes duplicate/insignificant modes after estimation. Badeau,
David and Richard establish the rotational-invariance/model-order problem;
the SVD order-selection work gives the direct shift-residual construction used
by this bounded implementation.

The candidate uses an inverse normalized least-squares rotational-invariance
residual as an ESTER-family order score. It is not claimed to reproduce the
paper's complete fast recurrence or full a-posteriori perturbation bound.

- [Gabor/ESPRIT impact analysis](https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf), retrieved SHA-256 `d52c9a3f…9bdd0`.
- [ESTER perturbation/order analysis](https://perso.telecom-paristech.fr/grichard/Publications/SP06_Badeau1.pdf), retrieved SHA-256 `bae56cfe…dc07`.
- [SVD-based EDS order selection](https://ftp.esat.kuleuven.be/stadius/ida/reports/05-107.pdf), retrieved SHA-256 `d3e38242…d25fd`.

## Frozen fixture

| Item | Value |
| --- | --- |
| Audio | 15 outputs, 48 kHz, 60,000 samples, onset 240 |
| Truth bands | `[1108,1142]`, `[2986,3017]`, `[5992]`, `[8990]` Hz |
| Amplitude decay | `[2,4]`, `[3,7]`, `[5]`, `[12]` per second |
| Gabor bins | `24,64,128,192`; centres `1125,3000,6000,9000` Hz |
| Gabor frame | 1024-sample Blackman-Harris, hop 32, first 256 frames |
| Pencil | 24 lag blocks × 15 outputs; candidate orders 1 through 6 |
| Node control | output 7 has exactly zero participation in the 1142 Hz mode |
| Nuisance | shared broadband transient decaying at 80/s plus independent `1e-7` noise |
| Seed | PCG64 `20260829` |
| Scales | `0.125`, `1`, `8` applied after the linear Gabor transform |

Both close pairs, 34 Hz and 31 Hz apart, lie below the ordinary
`48000/1024 = 46.875 Hz` FFT-bin spacing. The six common modes use different
output gains/phases. The node-channel comparator must see one mode in the
first band while the multi-output estimator recovers both.

## Frozen gates

- band orders equal `[2,2,1,1]` and all six truth modes are recovered;
- maximum frequency error is at most `0.25 Hz`;
- maximum amplitude-decay error is at most `0.50/s`;
- every selected order score exceeds its best competing order by at least
  `1000×`;
- node output 7 selects order one and misses exactly one of the two band-one
  truth modes;
- orders and numeric estimates repeat exactly at all three scales;
- both declared close pairs remain below one FFT bin.

The runner, manifest and two identical preflights MUST be committed before the
numeric run. Runs A/B then use the identical synthetic generator and must emit
byte-identical reports. Failure rejects this revision without threshold,
fixture or order-score changes. Success authorizes only a separate synthetic
broad-band discovery/duplicate-pruning control, never direct real reuse.

No network request, real payload, Iron parameter/early-tail reuse, physics
solver, Planter access, quality admission or runtime credit is allowed.

## Hash-closed preflight

Runner SHA-256 is `4acf8016…bf141`; external manifest SHA-256 is
`38025d25…f8703`. Preflights A/B repeat byte-identically at
`9fb1c488…64e4a` and decide `SubbandCommonPoleControlFrozen`. They bind the
fixed-tail diagnostic decision and result document, exact fixture, candidate,
runtime, gates and all three retrieved research hashes. Numeric control has not
run at this commit boundary; each preflight reports zero real payload, network,
physics and Planter access and no quality/admission/runtime credit.
