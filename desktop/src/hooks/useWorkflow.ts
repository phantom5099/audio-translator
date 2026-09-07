import { useReducer } from "react";
import { isWorkflowBusy } from "../config/workflow";
import { tauriAudioInputApi, tauriSpeechTranslationApi, tauriSubtitleExportApi } from "../services/api";
import { initialWorkflowState, workflowReducer } from "./workflowState";

function describeError(error: unknown, fallback: string): string {
  if (typeof error === "string") return error.length > 200 ? `${error.slice(0, 200)}...` : error;
  if (error instanceof Error) {
    const message = error.message || fallback;
    return message.length > 200 ? `${message.slice(0, 200)}...` : message;
  }
  return fallback;
}

export function useWorkflow() {
  const [state, dispatch] = useReducer(workflowReducer, initialWorkflowState);

  const importMedia = async (path: string) => {
    if (!path || isWorkflowBusy(state.stage)) return;
    dispatch({ type: "import-started" });
    try {
      dispatch({ type: "import-succeeded", result: await tauriAudioInputApi.importMedia(path) });
    } catch (error) {
      dispatch({ type: "failed", message: describeError(error, "音频导入失败，请重新选择") });
    }
  };

  const importUrl = async (url: string) => {
    if (!url || isWorkflowBusy(state.stage)) return;
    dispatch({ type: "import-started" });
    try {
      dispatch({ type: "import-succeeded", result: await tauriAudioInputApi.importUrl(url) });
    } catch (error) {
      dispatch({ type: "failed", message: describeError(error, "音频导入失败，请检查 URL 是否可访问") });
    }
  };

  const startSpeechTranslation = async () => {
    if (state.stage !== "imported") return;
    dispatch({ type: "speech-translate-started" });
    try {
      dispatch({ type: "speech-translate-succeeded", result: await tauriSpeechTranslationApi.startSpeechTranslation(state.file.id) });
    } catch {
      dispatch({ type: "failed", message: "语音翻译失败，请稍后重试" });
    }
  };

  const exportSubtitle = async () => {
    if (state.stage !== "speech-translated") return;
    dispatch({ type: "export-started" });
    try {
      const result = await tauriSubtitleExportApi.exportSubtitle(state.translationId);
      const blob = new Blob([new Uint8Array(result.bytes)], {
        type: "text/plain;charset=utf-8",
      });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = result.suggested_name;
      anchor.click();
      URL.revokeObjectURL(url);
      dispatch({ type: "export-succeeded" });
    } catch {
      dispatch({ type: "failed", message: "字幕导出失败，请稍后重试" });
    }
  };

  return { state, importMedia, importUrl, startSpeechTranslation, exportSubtitle };
}
