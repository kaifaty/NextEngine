# Physical Sound R3A V12-C4a — object-41 real FRF fit result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `DATA_INSUFFICIENT_FORCE_COVERAGE / REPEAT_EXACT` |
| Target | ObjectFolder-Real `41 / Wrench_Large / Steel` |
| Opened role | Sixteen `estimator_fit` force records only |
| Microphone decode | `0` samples |
| Protected-role decode | `0` samples |
| Product effect | None; authored clips remain authoritative |

## Exact result

Run A and Run B are byte-identical:

| Artifact | SHA-256 |
| --- | --- |
| Force certificate | `e234b7ee30ce46a5890972ec3983e3322f4666424ee5dd21f55cbb380f516524` |
| Report | `4144a8fe88848076f68831da254ea929ed27b78f0f66b65c9b9bbe778837807f` |

External roots:

- `r3a-v12-c4a-object41-fit-a`;
- `r3a-v12-c4a-object41-fit-b`.

Both runs validate and decode exactly `4,608,000` force PCM16 values. They
scan/hash the same exact C3 prefix and report `0` microphone samples, `0`
protected samples and `0` network requests. Because the force certificate
fails, no response fitting, pole discovery, model, array or audition artifact
is produced.

## Why the certificate fails

All sixteen force records contain a valid single impact. Onsets lie at source
samples `48,000…48,004`; corrected-force maxima are finite. This is not a
missing-impact or baseline-SNR failure:

- per-contact fraction with `input_snr >= 25` is `0.913900…1.0` over the
  target band;
- median per-contact input SNR ranges from `86.431` to `6,700.028`;
- nevertheless, twelve contacts have zero bins at
  `relative_power >= 0.005` inside `200…9,500 Hz`;
- the four nonzero fractions are only `0.040717`, `0.029336`, `0.004076` and
  `0.017110` for contacts `19`, `9`, `15` and `4`;
- `4,631` target bins have one supporter, only `3` bins have two, and no bin
  has three or four supporters.

The full and every leave-quarter-out certificate therefore have supported
fraction, cumulative width and longest contiguous width all exactly zero.
The impact-force profiles are high-SNR but too spectrally narrow and mutually
non-overlapping for the preregistered shared FRF claim.

## Decision

Do not lower the relative-power threshold, reduce the four-contact support
rule, select only favorable contacts or decode any microphone/development/
holdout/validator/shadow role. A different estimator cannot recover response
information in a band that this acquisition set did not jointly excite.

V12 closes as a valid data-insufficient result. Roadmap V13 separates:

1. a canonical-impact modal-field lane, which may learn object-global
   frequency/damping and contact-dependent gains from force-normalized or
   publisher-deconvolved internet recordings but cannot claim arbitrary-force
   convolution; and
2. an upgrade-only physical-transfer lane, which remains blocked until a
   broadband paired force/response source passes a fresh acquisition
   certificate.

ObjectFolder object `41` is now a permanent acquisition-OOD fixture for the
automatic validator, not a source for threshold tuning.
