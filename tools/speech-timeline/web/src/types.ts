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
  | "recording"
  | "finalizing"
  | "complete"
  | "error";
