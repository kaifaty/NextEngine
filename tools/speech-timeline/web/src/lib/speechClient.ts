import type {
  DiagnosticAudioRecord,
  DashboardBootstrap,
  JsonObject,
  ServiceBounds,
  SpeechEvent,
} from "../types";

interface SendResult {
  sentBytes: number;
  limitReached: boolean;
}

export interface VadCalibrationInput {
  noiseFloorDbfs: number;
  durationMs: number;
}

export type AsrAudioRoute = "raw" | "gain_only" | "enhanced" | "whisper";
export type AsrModelRoute = string;

export async function loadBootstrap(): Promise<DashboardBootstrap> {
  const response = await fetch("/api/bootstrap", {
    cache: "no-store",
    credentials: "same-origin",
  });
  if (!response.ok) {
    throw new Error(`dashboard bootstrap failed (${response.status})`);
  }
  const value: unknown = await response.json();
  if (!isObject(value) || value.schema_version !== 1 || value.status !== "ready") {
    throw new Error("dashboard bootstrap has an unsupported schema");
  }
  if (
    typeof value.uri !== "string" ||
    typeof value.token !== "string" ||
    typeof value.protocol !== "string" ||
    !isObject(value.service)
  ) {
    throw new Error("dashboard bootstrap is incomplete");
  }
  return value as unknown as DashboardBootstrap;
}

export async function loadDiagnosticAudio(): Promise<DiagnosticAudioRecord[]> {
  const response = await fetch("/api/diagnostic-audio", { cache: "no-store" });
  if (response.status === 404) return [];
  if (!response.ok) throw new Error("не удалось получить диагностические записи");
  const payload: unknown = await response.json();
  if (!isObject(payload) || !Array.isArray(payload.records)) {
    throw new Error("сервис вернул неверный список диагностических записей");
  }
  return payload.records.flatMap((item): DiagnosticAudioRecord[] => {
    if (!isObject(item) || typeof item.id !== "string") return [];
    const duration = integer(item.duration_ms);
    const size = integer(item.byte_length);
    const created = integer(item.created_at_unix_ms);
    const enhancedAvailable = item.enhanced_available === true;
    const asrAudioRoute =
      typeof item.asr_audio_route === "string" ? item.asr_audio_route : null;
    const asrModel = typeof item.asr_model === "string" ? item.asr_model : null;
    return duration >= 0 && size >= 44 && created >= 0
      ? [{
          id: item.id,
          duration_ms: duration,
          byte_length: size,
          created_at_unix_ms: created,
          enhanced_available: enhancedAvailable,
          asr_audio_route: asrAudioRoute,
          asr_model: asrModel,
        }]
      : [];
  });
}

export class SpeechTimelineClient {
  private socket: WebSocket | null = null;
  private sessionId = "";
  private sentBytes = 0;
  private finishing = false;
  private sessionStarted = false;
  private expectedClose = false;
  private maxBufferedAmount = 0;
  private lastBufferLogMs = 0;
  private startedResolve: (() => void) | null = null;
  private startedReject: ((error: Error) => void) | null = null;
  bounds: ServiceBounds | null = null;

  constructor(
    private readonly bootstrap: DashboardBootstrap,
    private readonly onEvent: (event: SpeechEvent) => void,
  ) {}

  connectAndStart(
    locale: string,
    asrModel: AsrModelRoute,
    asrAudioRoute: AsrAudioRoute,
    vadCalibration?: VadCalibrationInput,
  ): Promise<void> {
    if (this.socket !== null) {
      throw new Error("speech session is already connected");
    }
    this.sessionId = crypto.randomUUID();
    this.sentBytes = 0;
    this.finishing = false;
    this.sessionStarted = false;
    this.expectedClose = false;
    this.maxBufferedAmount = 0;
    this.lastBufferLogMs = 0;
    return new Promise<void>((resolve, reject) => {
      this.startedResolve = resolve;
      this.startedReject = reject;
      const socket = new WebSocket(this.bootstrap.uri);
      this.socket = socket;
      socket.binaryType = "arraybuffer";
      socket.onopen = () => {
        this.sendControl({
          schema_version: 1,
          type: "client.hello",
          token: this.bootstrap.token,
        });
      };
      socket.onmessage = (message) =>
        this.handleMessage(message, locale, asrModel, asrAudioRoute, vadCalibration);
      socket.onerror = () => {
        if (!this.sessionStarted) {
          this.rejectStart(new Error("WebSocket connection failed"));
        }
      };
      socket.onclose = () => {
        if (this.expectedClose) return;
        if (!this.sessionStarted) {
          this.rejectStart(new Error("service closed before session.started"));
        } else {
          this.onEvent({
            schema_version: 1,
            type: "error",
            code: "SERVICE_DISCONNECTED",
            terminal: true,
            detail: "service connection closed before utterance.final",
          });
        }
      };
    });
  }

  sendPcm(bytes: Uint8Array): SendResult {
    if (this.socket?.readyState !== WebSocket.OPEN || this.bounds === null || this.finishing) {
      return { sentBytes: this.sentBytes, limitReached: false };
    }
    if (bytes.byteLength === 0 || bytes.byteLength % 2 !== 0) {
      throw new Error("browser capture emitted invalid PCM");
    }
    if (bytes.byteLength > this.bounds.maxFrameBytes) {
      throw new Error("browser PCM chunk exceeds the service frame limit");
    }
    const remaining = this.bounds.maxTurnBytes - this.sentBytes;
    if (remaining <= 0) {
      return { sentBytes: this.sentBytes, limitReached: true };
    }
    const accepted = bytes.byteLength > remaining ? bytes.slice(0, remaining) : bytes;
    this.socket.send(accepted);
    this.maxBufferedAmount = Math.max(this.maxBufferedAmount, this.socket.bufferedAmount);
    const now = performance.now();
    if (this.socket.bufferedAmount >= 64 * 1024 && now - this.lastBufferLogMs >= 1_000) {
      this.lastBufferLogMs = now;
      console.info("[speech-timeline] ws_buffer_slow", {
        bufferedAmount: this.socket.bufferedAmount,
        maxBufferedAmount: this.maxBufferedAmount,
      });
    }
    this.sentBytes += accepted.byteLength;
    return {
      sentBytes: this.sentBytes,
      limitReached: this.sentBytes >= this.bounds.maxTurnBytes,
    };
  }

  finish(): void {
    if (this.socket?.readyState !== WebSocket.OPEN || this.finishing) {
      return;
    }
    this.finishing = true;
    this.sendControl({
      schema_version: 1,
      type: "session.finish",
      session_id: this.sessionId,
    });
  }

  cancel(): void {
    this.expectedClose = true;
    if (this.socket?.readyState === WebSocket.OPEN && !this.finishing) {
      this.sendControl({
        schema_version: 1,
        type: "session.cancel",
        session_id: this.sessionId,
      });
    }
    this.socket?.close();
    this.socket = null;
  }

  close(): void {
    this.expectedClose = true;
    this.socket?.close();
    this.socket = null;
  }

  private handleMessage(
    message: MessageEvent,
    locale: string,
    asrModel: AsrModelRoute,
    asrAudioRoute: AsrAudioRoute,
    vadCalibration?: VadCalibrationInput,
  ): void {
    if (typeof message.data !== "string") {
      this.rejectStart(new Error("service returned an unexpected binary frame"));
      return;
    }
    let value: unknown;
    try {
      value = JSON.parse(message.data);
    } catch {
      this.rejectStart(new Error("service returned invalid JSON"));
      return;
    }
    if (!isObject(value) || value.schema_version !== 1 || typeof value.type !== "string") {
      this.rejectStart(new Error("service event has an unsupported schema"));
      return;
    }
    const payload = value as SpeechEvent;
    if (payload.type === "error" && payload.terminal === true) {
      this.expectedClose = true;
    }
    this.onEvent(payload);
    if (payload.type === "service.ready") {
      try {
        this.bounds = parseBounds(payload.bounds);
        requireAsrModel(payload.asr_model_routing, asrModel);
        requireAsrAudioRoute(payload.asr_audio_routing, asrAudioRoute);
      } catch (error) {
        this.rejectStart(error instanceof Error ? error : new Error(String(error)));
        this.socket?.close();
        return;
      }
      this.sendControl({
        schema_version: 1,
        type: "session.start",
        session_id: this.sessionId,
        locale,
        sample_rate_hz: 16_000,
        encoding: "pcm_s16le",
        channels: 1,
        asr_model: asrModel,
        asr_audio_route: asrAudioRoute,
        ...(vadCalibration
          ? {
              vad_calibration: {
                noise_floor_dbfs: vadCalibration.noiseFloorDbfs,
                duration_ms: vadCalibration.durationMs,
              },
            }
          : {}),
      });
    } else if (payload.type === "session.started") {
      this.sessionStarted = true;
      this.startedResolve?.();
      this.startedResolve = null;
      this.startedReject = null;
    } else if (payload.type === "error" && payload.terminal === true) {
      const code = typeof payload.code === "string" ? payload.code : "UNKNOWN_ERROR";
      this.rejectStart(new Error(code));
    }
  }

  private sendControl(value: JsonObject): void {
    this.socket?.send(JSON.stringify(value));
  }

  private rejectStart(error: Error): void {
    this.startedReject?.(error);
    this.startedResolve = null;
    this.startedReject = null;
  }
}

function requireAsrModel(value: unknown, model: AsrModelRoute): void {
  if (!isObject(value) || !Array.isArray(value.available_models)) {
    throw new Error("service.ready does not contain ASR model routing");
  }
  if (!value.available_models.includes(model)) {
    throw new Error(`ASR model is unavailable: ${model}`);
  }
}

function requireAsrAudioRoute(value: unknown, route: AsrAudioRoute): void {
  if (!isObject(value) || !Array.isArray(value.available_routes)) {
    throw new Error("service.ready does not contain ASR audio routing");
  }
  if (!value.available_routes.includes(route)) {
    throw new Error(`ASR audio route is unavailable: ${route}`);
  }
}

function parseBounds(value: unknown): ServiceBounds {
  if (!isObject(value)) {
    throw new Error("service.ready does not contain bounds");
  }
  const maxFrameBytes = integer(value.max_frame_bytes);
  const maxTurnBytes = integer(value.max_turn_bytes);
  const sampleRateHz = integer(value.sample_rate_hz);
  const duration = integer(value.max_turn_duration_ms);
  if (
    maxFrameBytes <= 0 ||
    maxTurnBytes < maxFrameBytes ||
    sampleRateHz !== 16_000 ||
    duration <= 0
  ) {
    throw new Error("service.ready contains invalid bounds");
  }
  return {
    maxFrameBytes,
    maxTurnBytes,
    maxTurnDurationMs: duration,
    sampleRateHz,
  };
}

function integer(value: unknown): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value)) {
    return -1;
  }
  return value;
}

function isObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
