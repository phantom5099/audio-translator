export type WorkflowStage =
  | "idle"
  | "parsing"
  | "parsed"
  | "translating"
  | "translated"
  | "exporting"
  | "exported"
  | "error";

// 与后端 AudioMetadata (serde 默认 snake_case) 一致
export interface AudioMetadata {
  title?: string | null;
  duration_ms?: number | null;
  media_type?: string | null;
  size_bytes?: number | null;
  cover?: { media_type: string; bytes: number[] } | null;
}

// 与后端 AudioAssetResponse 一致：id 为 AudioAssetId (uuid 字符串)
export interface ImportResult {
  id: string;
  file_name: string;
  metadata: AudioMetadata;
}

// SpeechTranslationId 序列化为 uuid 字符串
export type TranslateResult = string;

// 与后端 SubtitleOutput 一致：bytes 为 Vec<u8> (数字数组)
export interface ExportResult {
  format: string;
  bytes: number[];
  suggested_name: string;
}
