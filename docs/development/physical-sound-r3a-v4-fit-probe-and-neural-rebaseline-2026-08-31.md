# Physical sound R3A V4 fit probe and neural rebaseline — 2026-08-31

Status: `REPRODUCIBLE_REJECT_FIT_REPRESENTATION / DEVELOPMENT_UNOPENED /
NEURAL_RATE_DISTORTION_NEXT / CLIP_FALLBACK_REQUIRED / NO_RUNTIME_CREDIT`

## Result

The bounded task-specific modal/residual representation is rejected before
development evaluation. All three frozen capacity points satisfy the shared
decoder and per-contact byte budgets, but every one of the twelve fit contacts
fails the unchanged multiresolution log-spectrum endpoint. Opening another
object or decoding any row-`2407` contact is therefore not justified.

This is a narrower result than “physical sound needs a neural waveform at
runtime.” It says that the tested stable-pole, contact-gain, time-domain basis
and object-spectral-bin factorization cannot reconstruct its own fit evidence
inside the frozen quality/budget boundary. A neural decoder remains external;
the first deployable neural route may bake validated output into ordinary
authored clip assets without adding runtime model inference.

## Frozen boundary

- implementation commit:
  `e05d55934f74e66a369edf0ea9ee5e2c46ed41a4`;
- four already-opened objects: Blue Bowl, Large Swan, Plastic Bin and Purple
  Scoop;
- per object: contacts `0..2` are fit, contact `3` is already-opened
  representation development and row `2407` remains sealed;
- no more than three capacities: `compact`, `balanced`, `extended`;
- fit-only object-global poles, contact modal gains, learned long/transient
  residual bases and object-global residual STFT bins;
- deterministic float64 reference inverse with a float32 cooked-size budget;
- unchanged five quality endpoints;
- maximum shared decoder `4 MiB`, maximum contact record `64 KiB`;
- external outputs only; no weights, arrays, WAVs or datasets enter Git.

The preflight reads and hashes the exact parents without decoding waveform
values. The fit runner memory-maps each contact array and slices only the first
three rows. Development, row `2407`, method holdout and admission shadow reads
remain zero.

## Exact repeated evidence

Two preflight and fit runs reproduce every JSON and NumPy artifact byte for
byte.

| Artifact | SHA-256 |
| --- | --- |
| Manifest | `6f9fe00ea14c99da2b2fc8b71b22af6772725ac5f28430a2417b8717819386b1` |
| Preflight report | `2d1eb841a264de4178d63adf3c907e52a77f4a08dd59b5f3658605576de308b0` |
| Fit report | `f212711714b60970d781d0d3c84297e9ab99884cf60d660df786164798b7d9e0` |
| Model descriptor | `147a345e8fe34e254af0ba13db0bb42c7557b586aacc5bdfddfc3fc77d1aa428` |
| Sorted JSON/NPY artifact tree | `fedfc0056bad476ffb0451b7c11fc16dda5e7c89e36e5709dc4c751ea99d849d` |

The exact decision is `REJECT_FIT_REPRESENTATION`.

| Capacity | Shared bytes | Maximum contact bytes | Fit spectrum range | Contacts failing spectrum |
| --- | ---: | ---: | ---: | ---: |
| `compact` | `419,968` | `19,764` | `6.5078–11.8433 dB` | `12/12` |
| `balanced` | `829,696` | `38,476` | `9.1867–11.9636 dB` | `12/12` |
| `extended` | `1,110,528` | `64,612` | `8.3188–12.0782 dB` | `12/12` |

The non-monotonic quality frontier is itself evidence: allocating more of the
same factorization does not consistently reduce the frozen spectral error.
Level and envelope are usually preserved, so the failure is not a trivial
normalization or optimizer defect. Narrow low- and high-frequency structure is
lost by the factorized basis.

## Bounded successful-control search

An exploratory, non-candidate control encoded the twelve fit contacts with
the host FFmpeg `libopus` encoder at `168 kbps` CBR, then added six hundred
5 ms RMS-envelope scalars and one output gain. It used at most `65,170` bytes.
Eleven contacts pass all five absolute endpoints. Large Swan fit contact `2`
still fails only spectrum at `6.1148 dB`; its modal error is `28.69` cents and
all other endpoints pass.

This control has no representation, deterministic-cooker, licensing, runtime
or quality credit. It is not a request to retry a general codec. It establishes
two useful bounds:

1. the frozen `64 KiB` record is close to, but not universally above, a
   conventional near-ceiling solution;
2. the missing capability is rate allocation against this task's spectral and
   modal loss, not another larger time-domain PCA basis.

## Research discriminator

Primary neural-codec work supports changing the optimization target rather
than continuing adjacent analytical bases:

- [SoundStream](https://arxiv.org/abs/2107.03312) combines a convolutional
  encoder/decoder, residual vector quantization, adversarial, feature-matching
  and multiscale spectral reconstruction losses.
- [EnCodec](https://arxiv.org/abs/2210.13438) uses a quantized latent space,
  multiscale spectrogram adversary and explicit loss balancing.
- [Improved RVQGAN / DAC](https://arxiv.org/abs/2306.06546) adds periodic
  activations, improved codebook utilization, multiband complex-STFT
  discrimination and multiscale log-mel reconstruction. The paper explicitly
  identifies tonal, pitch and high-frequency failures as codec concerns.

These sources do not prove that an off-the-shelf model passes Next Engine's
gate; V3B already showed that NDAC-75 does not. They support a falsifiable next
hypothesis: train a small task-specific rate-distortion model on published
impact audio with the frozen Next Engine spectral/modal/envelope objectives,
rather than asking a universal perceptual checkpoint to preserve them by
accident.

## Decision

Roadmap V7 replaces the analytical-representation-first sequence with:

1. a hash-closed, internet-only impact training corpus disjoint from all
   representation development and holdout objects;
2. a bounded task-specific neural codec experiment with no more than three
   frozen latent capacities and direct multiresolution spectral/modal losses;
3. evaluation on the four already-opened development contacts;
4. one new source-disjoint holdout only after a development pass;
5. an exact-object contact-to-latent field only after neural representation
   sufficiency is shown;
6. offline baking into a bounded contact-conditioned clip atlas for the first
   product experiment, with authored fallback and no runtime neural inference;
7. deterministic modal/residual distillation only as a later optimization,
   not a prerequisite for learning whether the sound task is viable.

## Rejected alternatives

- decode the four development contacts with the failed V4 fit;
- add a fourth capacity or enlarge the `64 KiB` record after seeing results;
- weaken the spectrum or modal thresholds;
- open a fifth object or any row `2407` contact;
- repeat NDAC, Opus or another universal codec as the candidate;
- put PyTorch, a neural decoder or network access into the runtime;
- ask the user to record impacts or manually approve every generated sound.

## Smallest next action

Freeze the V7 neural-representation corpus and training manifest. The first
implementation commit must contain only the external corpus projection,
model/loss/capacity identity, synthetic overfit and deterministic checkpoint
controls. It must not read development contacts or start a long training run.
