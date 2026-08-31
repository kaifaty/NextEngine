# Physical sound R3A V11 B1R — joint modal FRF protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / FRESH_SYNTHETIC_REVISION / IMPLEMENTATION_NOT_RUN` |
| Parent result | [B1 V1 exact reject](physical-sound-r3a-v11-b1-force-response-oracle-result-2026-08-31.md) |
| Roadmap | [V11 B1](../plans/physical-sound-synthesis-roadmap-v11.md#b1--synthetic-forceresponse-oracle) |
| Product effect | None; authored clips remain authoritative |

## New falsifiable hypothesis

B1 V1 combined force conditioning and contact-local response coherence into one
broad-band hard mask. That mask removed legitimate modal evidence at quiet
contact responses even though force-only coverage was approximately `99.98%`.

B1R tests a materially different confidence factorization:

```text
force observability q_force(ω)
    -> whether this source can identify any transfer at ω

pooled response support q_mode(mode, contacts)
    -> whether enough contacts support one shared pole

contact residue confidence q_residue(mode, contact)
    -> whether this contact-specific amplitude is reliable
```

The hypothesis passes only if a force-only regularized H1 field yields one
shared explicit modal transfer that predicts fresh held forces, while weak
force, incoherent response and an unmeasured second impact still fail closed.

`PASS_KNOWN_TRUTH_FRF` opens only V11-B2 zero-decode source feasibility.
`REJECT_JOINT_MODAL_ESTIMATOR` keeps all fresh real waveforms closed.

## Immutable parent and execution boundary

The B1 V1 runner, manifest and result remain immutable. B1R may import its
force generators, numeric helpers and already supported Gabor/common-pole core,
but its manifest binds exact hashes of all dependencies.

The committed B1R runner has `freeze`, `preflight` and `run` stages. `run`
generates fit, then development, and generates holdout only if every
development, identity, uncertainty and OOD gate passes. Candidate selection is
fixed in code and manifest before numeric execution. Two independent complete
runs must emit byte-identical JSON and NPY files.

No network request, real sample, ObjectFolder protected row, V1 holdout sample
or external learned artifact may be read.

## Fresh fixture and roles

B1R retains the hard B1 signal shape so it tests the failed boundary rather
than an easier problem:

- `48,000 Hz`, `96,000` transfer samples, `262,144`-sample FFT;
- six contacts;
- frequencies `523, 911, 1487, 2381, 3769, 6029, 9137 Hz`;
- amplitude decays `4, 6.5, 9, 13, 19, 28, 42 s^-1`;
- fresh magnitude
  `(0.34 + 0.04*((11*contact + 5*mode + 3) mod 13))/(1 + 0.035*mode)`;
- fresh phase
  `0.071 + 0.31*contact + 0.17*mode + 0.029*contact*mode` radians;
- one global peak scale `0.25`, no per-contact normalization.

Truth remains inaccessible to estimation and enters only final scoring.

Clean force profiles and observation equations stay identical to B1 V1:

| Role | Profiles | Fresh force/response/room seed roots |
| --- | --- | --- |
| fit | `half_sine(9)`, `hann(19)`, `beta(37,2,4)`, `half_sine(61)` | `2026111201/2026111202/2026111203` |
| development | `hann(13)`, `beta(47,3,2)` | `2026111301/2026111302/2026111303` |
| holdout | `half_sine(25)`, `beta(53,2,5)` | `2026111401/2026111402/2026111403` |
| corruptions | V1 weak/coherence/missing-impact families | `2026111501/2026111502/2026111503` |

Force noise is `2e-4 * force peak`; response noise is `2e-4 * clean response
RMS`; the independent delayed room tail is `0.002 * clean response RMS`. The
same `SeedSequence` canonical contact/profile/repeat spawning rule applies.

## Frozen joint estimator

For each contact, compute the same pooled four-profile `Sxx`, `Syx`, `Syy`, H1
and magnitude-squared coherence as B1 V1. The distinction is mandatory:

1. `force_valid = 200…12,000 Hz AND Sxx/max_band(Sxx) >= 1e-5`;
2. `H_force = Syx / (Sxx + 1e-8*max_band(Sxx))`, multiplied only by the
   four-bin tapered `force_valid`; response coherence cannot zero this field;
3. inverse-transform all six `H_force` records and perform the unchanged
   input-driven Gabor/common-pole discovery: Blackman-Harris `1024/32`, first
   `256` frames, `-35 dB` region floor, `±1` neighboring bin, `24` pencil lags,
   maximum order `6`, score margin `20`, duplicate radius `1 Hz`, post-fit
   energy floor `-30 dB`;
4. for each discovered mode, define its support neighborhood as
   `±max(20 Hz, 4*decay/(2*pi))`; a contact supports the pole when median
   force conditioning is at least `1e-5` and median coherence is at least
   `0.95`; at least three of six contacts must support every retained pole;
5. fit contact cosine/sine residues jointly in time on the seven shared modes;
   per-mode/contact uncertainty is `1 - median coherence` in the same
   neighborhood. A residue is confident at coherence `>= 0.90` and every mode
   requires at least three confident contacts;
6. the selected transfer is the explicit seven-mode reconstruction, tapered
   only by the force-valid band. H1 waveform transfer remains a diagnostic and
   cannot replace a failed modal record.

Controls remain equal-weight direct division, shortest-response impulse
assumption and input-ignorant raw-output modal fitting. They receive neither
truth nor joint confidence. The exact noiseless impulse/held-force identity is
retained.

## Frozen gates

Development and holdout independently require:

| Endpoint | Gate |
| --- | ---: |
| Minimum force-only coverage, `200…10,000 Hz` | `>= 0.99` per contact |
| Truth modes / false positives | `7 / 0` |
| Minimum supporting contacts per mode | `>= 3` |
| Minimum confident residues per mode | `>= 3` |
| Maximum modal-frequency error | `<= 1.0 Hz` |
| Maximum absolute / relative damping error | `<= 1.5 s^-1 / 0.20` |
| Mean / maximum held joint-modal NRMSE | `<= 0.08 / 0.15` |
| Mean gain-matched log-spectrum RMSE | `<= 2.0 dB` |
| Joint-modal / impulse NRMSE | `<= 0.70` |
| Joint-modal / raw-output-modal NRMSE | `<= 0.80` |
| Joint-modal / direct-division NRMSE | `<= 1.50` |

Noiseless identity remains `<= 1e-11`. All values, residues and uncertainties
must be finite; every residue magnitude must be nonzero. The selected model is
exactly seven explicit modes and cannot contain a waveform residual.

OOD decisions remain fail-closed and use fresh seeds:

- weak `hann(1025)`: force-only high-band coverage `<= 0.35` gives
  `OOD_WEAK_EXCITATION`, zero modes;
- independent interference `0.75 * clean RMS`: median pooled coherence
  `<= 0.80` or coherent contact coverage `<= 0.50` gives
  `OOD_LOW_COHERENCE`, zero modes;
- unrecorded second impact with gain `0.8` and delays
  `401/613/887/1151`: median reconstruction NRMSE `>= 0.15` gives
  `OOD_MODEL_MISMATCH`, zero modes.

Hard gates also require exact role/counter order, no holdout after development
failure, canonical JSON, deterministic arrays, exact dependency hashes, zero
real/network access and byte identity of every emitted artifact.

## Decision and stop rule

- `PASS_KNOWN_TRUTH_FRF`: all development and holdout gates pass twice. Only a
  separate B2 source-feasibility protocol may then inspect internet metadata.
- `REJECT_JOINT_MODAL_ESTIMATOR`: any valid numeric, support or OOD gate fails.
  Do not tune this fresh development result or generate its holdout afterward.
- `INVALID_B1R_RUN`: lineage, ordering, serialization, finiteness or repeat
  fails and grants no estimator evidence.

The B1 V1 coverage/coherence thresholds remain historical facts and are not
retroactively weakened by this new factorization.
