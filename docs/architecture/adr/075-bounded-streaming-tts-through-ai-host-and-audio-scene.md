# ADR-075: Bounded streaming TTS through `ai-host` and `AudioScene`

| Поле | Значение |
|---|---|
| ID | ADR-075 |
| Статус | Proposed |
| Версия | 0.1 |
| Дата предложения | 2026-08-17 |
| Последняя проверка | 2026-08-17 |
| Нормативные зависимости | [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-16](../16-text-canonical-multimodal-dialogue-and-model-packs.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [SPEC-36](../36-streaming-tts-and-spatial-speech-presentation.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-017](017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md), [ADR-028](028-platform-session-and-presentation-authority.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Контекст

Локальные TTS experiments показали, что usable Russian speech возможна на
целевом class hardware, а VoxCPM2 PyTorch даёт лучшее из проверенных качество
интонации. Однако standalone demo не отвечает на product questions:

- как PCM попадает в общий spatial mixer, а не прямо в device;
- как голос следует за ртом NPC, учитывает направление, комнату, portal и
  occlusion;
- как process framing, cancellation, buffering и callback остаются bounded;
- как model/voice hashes, license, consent, cache and replay закрываются без
  превращения waveform в gameplay authority;
- как TTS делит GPU с renderer;
- какой fallback сохраняет полностью рабочую offline игру.

Текущий `AudioEmitterRecordV1` принимает только cooked clip revision,
`AudioMixerV1` резолвит только `NeutralAudioV1`, а desktop callback использует
mutex и heap allocation. Прямая передача model PCM в SDL обошла бы
`AudioScene`, spatial effects, voice admission, displayless capture и device
recovery. Встраивание model runtime в Rust process нарушило бы Accepted
ADR-005 и не дало бы заметного ускорения само по себе: основной cost находится
в model kernels, а не в Python orchestration.

## Предлагаемое решение

1. TTS MUST оставаться в optional supervised external `ai-host`. Engine и host
   общаются через bounded versioned length-prefixed engine-owned protocol;
   model/vendor/runtime objects не пересекают boundary.
2. Один request MUST представлять одну уже validated canonical sentence.
   `ai-host` возвращает ordered, hash-bound dry mono PCM segments. Generated
   duration, PCM and timing tracks are PresentationOnly.
3. Speech MUST enter an engine-owned `SpeechStream` source inside
   `AudioScene`, then pass the ordinary emitter transform, mouth directivity,
   attenuation, propagation/room response, dialogue bus, voice limit, mixer
   and device path. Direct `ai-host → device` playback is forbidden.
4. `AudioScene` MUST expose only engine-owned source/acoustic/result values.
   Engine-native zone/portal/room response is the mandatory fallback; Steam
   Audio or another SDK MAY be a private optional adapter.
5. IPC reader, speech ring, mixer and final device ring MUST be bounded.
   The audio callback MUST NOT allocate, lock, decode, resample, call IPC/model
   code or run propagation. Overflow/underflow/cancellation has stable
   diagnostics and can lose presentation quality, never gameplay correctness.
6. Model packs and model-specific voice bindings MUST be immutable,
   content-addressed, separately installed and license/provenance/resource
   closed. Voice cloning is default-deny without exact scoped consent.
7. Resource admission MUST include renderer plus TTS model/process/buffers and
   safety margin. Process isolation does not imply VRAM isolation; an
   unprovable or oversized profile fails before model load and selects the
   next authored/subtitle route.
8. Replay MUST NOT synthesize again. It uses canonical text and optional exact
   recorded PCM for an explicit audiovisual capture; gameplay roots always
   exclude TTS/audio presentation state.
9. VoxCPM2 PyTorch MAY be the first quality-first experimental consumer, but
   protocol admission does not select it as shipping default. Model quality,
   spatial integration and co-resident resources remain separate checks.

This ADR specializes only the TTS vertical. SPEC-16/ADR-017 stay Deferred
Proposed for ASR, LLM dialogue, remote providers and the broad model-pack
stack. Accepted ADR-005 process/offline invariants remain unchanged.

## Product impact

When implemented, an NPC can speak generated text from the correct world
position and room context without creating a second dialogue or audio path.
Projects without a model, suitable GPU, consent or audio device keep the same
authored dialogue/subtitle outcome. Generated audio remains optional quality,
not a simulation dependency.

The proposal intentionally adds no generic network AI service, no runtime
training, no authoritative lip-sync clock and no requirement to bundle model
weights. Current clip playback remains the safe incremental baseline.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `TTS-PROTOCOL-P1` | handshake, malformed frames, exact retry/collision, cancel, crash/restart | bounded decode, one terminal outcome, fresh host session and no process fault crossing into gameplay | disable route; authored voice/subtitle |
| `TTS-STREAM-P1` | ordered PCM plus reorder/gap/overflow/underflow/barge-in faults | no callback block/allocation/lock, no silent sample reorder/drop and bounded cancellation | cancel speech stream; subtitle |
| `TTS-SPATIAL-P1` | moving NPC, listener rotation, mouth direction, two rooms and portal state | one common mixer path yields the declared direction, attenuation, occlusion and room response | engine-native distance/pan/directivity/room fallback |
| `TTS-OFFLINE-P1` | play without host/model/network/device | identical mandatory dialogue/gameplay hashes and no tick wait | text/subtitle and optional authored clip |
| `TTS-REPLAY-P1` | replay with absent/different TTS installation | no TTS launch or regeneration; exact gameplay roots | subtitle; optional capture reports missing PCM |
| `TTS-RESOURCE-P1` | renderer and resident TTS on declared target profile | joint admission and measured tails prevent OOM/unbounded queues | reject/downgrade route before load |
| `TTS-VOICE-P1` / `VOICE-L1` | licensed preset and authorized fixed-reference emotion/pronunciation corpus | identity/quality target is explicit; invalid consent never clones | licensed voice/subtitle |

The exact contracts, bounds and mapped checks are defined in SPEC-36.

## Рассмотренные варианты

- **Direct model-to-device playback** — `Rejected`: bypasses spatial scene,
  mixer priority/effects, displayless capture and device recovery.
- **In-process Rust/Python/C++ TTS runtime** — `Rejected` as baseline by
  ADR-005: it weakens crash/resource isolation, and a language rewrite alone
  does not accelerate dominant model kernels. A private host may itself use
  Rust/C++ after parity evidence without changing the engine protocol.
- **Generate a cooked audio asset per segment** — `Rejected`: creates
  unbounded asset churn and conflates ephemeral stream/cache with immutable
  project content.
- **Use generated duration as dialogue/gameplay timing** — `Rejected`: model,
  hardware and backend changes would alter authoritative outcome.
- **Steam Audio public contracts** — `Rejected`: vendor handles and scene data
  would escape the replaceable adapter boundary. A normalized engine result
  plus zone/room fallback is sufficient.
- **Select Nano-vLLM because it is fastest** — `Rejected`: retained listening
  evidence found worse intonation and its observed VRAM is higher. It remains
  a separately named throughput candidate.
- **Accept the whole SPEC-16/ADR-017 stack now** — `Rejected`: the current
  product need is narrower and ADR-046 requires a real consumer per contract.

## Последствия

- `crates/contracts` needs current-only TTS protocol/pack/voice/acoustic values
  and a V2 audio source/emitter/snapshot successor when implementation starts.
- `crates/application` needs the bounded optional host supervisor and joint
  presentation resource admission.
- `crates/presentation` needs streaming source state, preallocated render path,
  directivity/room response and optional propagation adapter normalization.
- the desktop sink needs a preallocated non-locking SPSC callback path;
  current mutex/callback allocation is not sufficient for the proposed gate.
- one reference-game NPC sentence and authored fallback must become the first
  production consumer before promotion.
- all exact model/runtime/generated artifacts remain outside Git; only bounded
  human-readable evidence and immutable identities are documented.

Until promotion every executable check above is `NOT_RUN`; the proposal is an
implementation-ready decision, not a shipped capability claim.

## Supersession

Accepting this decision requires its production consumer and synchronized
status updates to SPEC-36, SPEC-08/SPEC-16, index, routing and traceability.
Mandatory cloud, in-process generative TTS, audio-derived gameplay authority or
direct device bypass requires a new superseding ADR. Replacing the private
model or propagation adapter behind the same contracts does not.
