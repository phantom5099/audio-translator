export type WorkflowStage =
  | "idle"
  | "parsing"
  | "parsed"
  | "translating"
  | "translated"
  | "exporting"
  | "exported"
  | "error";

export interface VideoFile {
  name: string;
  sizeBytes: number;
  durationSeconds: number;
}

export interface ImportResult {
  taskId: string;
  file: VideoFile;
}

export interface TranslateResult {
  subtitleId: string;
  cueCount: number;
}

export interface ExportResult {
  fileName: string;
  content: Blob;
}
