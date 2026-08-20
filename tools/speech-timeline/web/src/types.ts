export type JsonObject = Record<string, unknown>;

export interface ServiceBounds {
  maxFrameBytes: number;
  maxTurnBytes: number;
  maxTurnDurationMs: number;
  sampleRateHz: number;
}

export interface DashboardBootstrap {
  schema_version: 1;
  status: "ready";
  uri: string;
  token: string;
  protocol: string;
  service: JsonObject;
}

export interface DiagnosticAudioRecord {
  id: string;
  byte_length: number;
  duration_ms: number;
  created_at_unix_ms: number;
  enhanced_available: boolean;
  asr_audio_route: string | null;
}

export interface AffectSegment {
  start_sample: number;
  end_sample: number;
  label: string;
  score: number;
}

export interface AffectObservation {
  observation_id: number;
  start_sample: number;
  end_sample: number;
  scores: Record<string, number>;
  top_label: string;
  activity?: "speech" | "no_speech";
  voiced_ratio?: number;
  evidence_samples?: number;
}

export interface SpeechActivitySegment {
  start_sample: number;
  end_sample: number;
  state: "speech" | "no_speech";
  voiced_samples: number;
  voiced_ratio: number;
}

export interface TimelineUpdate {
  revision: number;
  transcript: {
    revision: number;
    text: string;
    stable_prefix: string;
    final: boolean;
    timing_precision: string;
  };
  vocal_affect: {
    revision: number;
    replace_from_sample: number;
    raw_observations_mode: "append" | "snapshot";
    raw_observations_total: number;
    raw_observations: AffectObservation[];
    segments: AffectSegment[];
    speech_activity: SpeechActivitySegment[];
  };
  fusion: {
    revision: number;
    alignment_grade: string;
    observed_vocal_expression: string;
  };
}

export interface FinalUtterance {
  text: string;
  observed_vocal_expression: string;
  observed_vocal_expression_source?: string;
  vocal_expression_summary?: {
    evidence_samples: number;
    evidence_duration_ms: number;
    confirmed_segment_count: number;
    label_support_samples: Record<string, number>;
  };
  alignment_grade: string;
  metrics?: JsonObject;
}

export interface SpeechEvent extends JsonObject {
  schema_version: 1;
  type: string;
}

export type ConnectionState =
  | "loading"
  | "ready"
  | "connecting"
  | "calibrating"
  | "recording"
  | "finalizing"
  | "complete"
  | "error";
