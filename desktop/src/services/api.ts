import type { ExportResult, ImportResult, TranslateResult } from "../types";

export interface TranslatorApi {
  importVideo(file: File): Promise<ImportResult>;
  translate(taskId: string): Promise<TranslateResult>;
  exportSubtitle(subtitleId: string): Promise<ExportResult>;
}
