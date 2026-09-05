import { useReducer } from "react";
import { isWorkflowBusy } from "../config/workflow";
import { tauriAudioInputApi, tauriTranslatorApi } from "../services/api";
import { initialWorkflowState, workflowReducer } from "./workflowState";

export function useWorkflow() {
  const [state, dispatch] = useReducer(workflowReducer, initialWorkflowState);

  const importMedia = async (path?: string) => {
    if (!path || isWorkflowBusy(state.stage)) return;
    dispatch({ type: "import-started" });
    try {
      dispatch({ type: "import-succeeded", result: await tauriAudioInputApi.importMedia(path) });
    } catch {
      dispatch({ type: "failed", message: "音频导入失败，请重新选择" });
    }
  };

  const startSpeechTranslation = async () => {
    if (state.stage !== "imported" || !state.file?.id) return;
    dispatch({ type: "speech-translate-started" });
    try {
      dispatch({ type: "speech-translate-succeeded", result: await tauriTranslatorApi.startSpeechTranslation(state.file.id) });
    } catch {
      dispatch({ type: "failed", message: "语音翻译失败，请稍后重试" });
    }
  };

  const exportSubtitle = async () => {
    if (state.stage !== "speech-translated" || !state.translationId) return;
    dispatch({ type: "export-started" });
    try {
      const result = await tauriTranslatorApi.exportSubtitle(state.translationId);
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

  return { state, importMedia, startSpeechTranslation, exportSubtitle };
}
