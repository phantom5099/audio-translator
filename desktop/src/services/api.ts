import type { ExportResult, ImportResult, TranslateResult } from "../types";

export interface AudioInputApi {
  importVideo(file: File): Promise<ImportResult>;
}

export interface TranslatorApi {
  translate(taskId: string): Promise<TranslateResult>;
  exportSubtitle(subtitleId: string): Promise<ExportResult>;
}
