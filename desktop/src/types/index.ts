export type WorkflowStage =
  | "idle"
  | "importing"
  | "imported"
  | "speech-translating"
  | "speech-translated"
  | "exporting"
  | "exported"
  | "error";

export interface AudioMetadata {
  title?: string | null;
  duration_ms?: number | null;
  media_type?: string | null;
  size_bytes?: number | null;
  cover?: { media_type: string; bytes: number[] } | null;
}

export interface ImportResult {
  id: string;
  file_name: string;
  metadata: AudioMetadata;
}

export type TranslateResult = string;

export interface ExportResult {
  format: string;
  bytes: number[];
  suggested_name: string;
}
