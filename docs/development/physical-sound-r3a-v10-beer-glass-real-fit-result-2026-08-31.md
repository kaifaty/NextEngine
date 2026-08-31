# Physical sound R3A V10 — Beer Glass real-fit result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `REPRODUCIBLE_REJECT / FROZEN_ONSET_GATE_FAILED / REPRESENTATION_NOT_FIT` |
| Decision | `REJECT_V9_REAL_REPRESENTATION` for this preregistered revision |
| Product effect | None; authored clips remain authoritative |

## Outcome

The V10 fit revision stops at its frozen preprocessing gate. Contacts `18`
and `29` never exceed the required threshold
`max(0.06 * peak, baseline_abs_median + 12 * baseline_abs_MAD)` because the
noise-derived threshold is greater than the maximum absolute sample in the
entire recording.

The runner therefore does not estimate modes, encode a residual, compute
candidate metrics, read development/query/holdout audio or authorize A2. This
is a valid negative result for the complete preregistered revision, but it is
not evidence that the V9 modal/time-varying-residual representation sounds bad:
that representation was never fit to these waveforms.

## Exact evidence

Two independent outputs are byte-identical:

- external runs: `r3a-v10-beer-glass-real-fit-run-a` and
  `r3a-v10-beer-glass-real-fit-run-b`;
- manifest SHA-256: `a8a82723f27481c9a92c07234f48faa64a24d7a26141448285a6bebc2629095d`;
- model SHA-256: `cc9f7ef5125581c050f0873b424ef122f95745b488f7e255f3b07b8381145959`;
- report SHA-256: `7d7bb630f69dfc52b46afe8f50e8cd130115c3daf3567f5b8b7fb75b5579db9b`;
- decoded fit samples: exactly `22 * 288,000 = 6,336,000`;
- samples admitted to representation fitting: `0`;
- development, exact-object query, representation holdout, method holdout and
  admission-shadow decoded counters: all `0`.

Failure values are exact PCM-derived amplitudes after baseline-median removal:

| Contact | Absolute peak | Peak threshold | Noise threshold | Relation |
| ---: | ---: | ---: | ---: | --- |
| `18` | `0.005462646484375` | `0.0003277587890625` | `0.00616455078125` | noise threshold `> peak` |
| `29` | `0.002593994140625` | `0.0001556396484375` | `0.0032958984375` | noise threshold `> peak` |

Both recordings nevertheless have their absolute maximum near the common
one-second event region: sample `48,135` for contact `18` and `48,039` for
contact `29`. The other twenty fit contacts cross the frozen detector between
samples `48,015` and `48,039`. That observation is diagnostic only: using it
to replace the frozen detector on these opened contacts would be post-hoc
tuning.

## Bounded research conclusion

Competing explanations were checked:

1. **No impact exists in the two recordings.** Evidence against: each absolute
   maximum is in the same approximately one-second event region as the twenty
   successful contacts.
2. **The modal/residual representation cannot fit weak glass impacts.** Not
   tested: no aligned target reached representation fitting.
3. **The generic baseline-MAD onset rule is incompatible with this source's
   acquisition/noise semantics.** Supported: its computed threshold is
   mathematically unreachable on two official fit recordings.

The official ObjectFolder paper describes synchronized five-second impact
recordings and measured force, while the compact contact-localization bundle
retains raw microphone PCM but omits force. The official benchmark loader binds
audio, contact coordinate and point cloud by one `(object, contact)` key but
does not provide onset timestamps. Consequently, the next revision must obtain
an onset axis from source semantics rather than infer it by tuning this failed
threshold.

Primary sources:

- [ObjectFolder Real download page](https://objectfolder.stanford.edu/objectfolder-real-download)
- [ObjectFolder 2.0 paper](https://ai.stanford.edu/~rhgao/publications/ObjectFolder_CVPR2023.pdf)
- [official contact-localization benchmark](https://github.com/objectfolder/contact-localization/tree/4bb002f519cab9d250bbbe045a6df0248bf1639f)

## Decision and next discriminator

- Keep all object-60 development/query and object-22 holdout WAVs closed.
- Do not lower the MAD multiplier, use a contact subset, exploit the observed
  one-second maximum or otherwise repair this opened revision.
- Run A1R source research in this order: published event timestamp; selected
  raw force channel; otherwise a new source-disjoint object/dataset with a
  published force or event-time axis.
- Before any new waveform decode, freeze the source revision, roles,
  synchronization transform, missing-data behavior and the same quality/cost
  gates.
- Only a new preregistered source-semantic fit may decide whether V9 itself
  advances to development.

No public contract, runtime model, content role, validator release, clip atlas
or real-quality claim is authorized.
