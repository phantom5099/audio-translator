import type { AudioInputApi, TranslatorApi } from "./api";
import type { ExportResult, ImportResult, TranslateResult } from "../types";

const wait = (milliseconds: number) =>
  new Promise<void>((resolve) => window.setTimeout(resolve, milliseconds));

const makeId = (prefix: string) =>
  `${prefix}-${Math.random().toString(36).slice(2, 10)}`;

export const mockAudioInputApi: AudioInputApi = {
  async importVideo(file): Promise<ImportResult> {
    await wait(1600);
    return {
      taskId: makeId("task"),
      file: {
        name: file.name || "示例视频.mp4",
        sizeBytes: file.size || 32_400_000,
        durationSeconds: 312,
      },
    };
  },
};

export const mockTranslatorApi: TranslatorApi = {
  async translate(_taskId): Promise<TranslateResult> {
    await wait(2200);
    return { subtitleId: makeId("subtitle"), cueCount: 18 };
  },

  async exportSubtitle(_subtitleId): Promise<ExportResult> {
    await wait(800);
    const content = new Blob(
      [
        "1\n00:00:00,000 --> 00:00:03,000\n这是一个 Mock 字幕示例。\n\n",
        "2\n00:00:03,000 --> 00:00:06,000\n字幕文件将在接入真实翻译服务后生成。\n",
      ],
      { type: "text/plain;charset=utf-8" },
    );
    return { fileName: "translated-subtitles.srt", content };
  },
};
