import { invoke } from "@tauri-apps/api/core";
import type { ExportResult, ImportResult } from "../types";

export interface AudioInputApi {
  importVideo(path: string): Promise<ImportResult>;
}

export interface TranslatorApi {
  translate(assetId: string): Promise<string>;
  exportSubtitle(translationId: string): Promise<ExportResult>;
}

// 翻译目标语言默认简体中文
const DEFAULT_TARGET_LANGUAGE = "zh-CN";

export const tauriAudioInputApi: AudioInputApi = {
  async importVideo(path: string): Promise<ImportResult> {
    return invoke<ImportResult>("import_audio", {
      source: { LocalFile: path },
    });
  },
};

export const tauriTranslatorApi: TranslatorApi = {
  async translate(assetId: string): Promise<string> {
    return invoke<string>("start_translation", {
      assetId,
      request: {
        source_language: null,
        target_language: DEFAULT_TARGET_LANGUAGE,
        constraints: {
          preserve_numbers: false,
          preserve_placeholders: false,
          preserve_line_breaks: false,
          max_chars_per_line: null,
          allow_rewrite_source: false,
        },
        options: { values: {} },
      },
    });
  },

  async exportSubtitle(translationId: string): Promise<ExportResult> {
    return invoke<ExportResult>("export_subtitle", {
      translationId,
      request: {
        // 字幕导出默认 Srt + Utf8 + 保留换行
        format: "Srt",
        encoding: "Utf8",
        line_policy: "Preserve",
      },
    });
  },
};
