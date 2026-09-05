import { useReducer } from "react";
import { isWorkflowBusy } from "../config/workflow";
import { tauriAudioInputApi, tauriTranslatorApi } from "../services/api";
import { initialWorkflowState, workflowReducer } from "./workflowState";

export function useWorkflow() {
  const [state, dispatch] = useReducer(workflowReducer, initialWorkflowState);

  const importVideo = async (path?: string) => {
    if (!path || isWorkflowBusy(state.stage)) return;
    dispatch({ type: "parse-started" });
    try {
      dispatch({ type: "parse-succeeded", result: await tauriAudioInputApi.importVideo(path) });
    } catch {
      dispatch({ type: "failed", message: "视频解析失败，请重新导入" });
    }
  };

  const translate = async () => {
    if (state.stage !== "parsed" || !state.file?.id) return;
    dispatch({ type: "translate-started" });
    try {
      dispatch({ type: "translate-succeeded", result: await tauriTranslatorApi.translate(state.file.id) });
    } catch {
      dispatch({ type: "failed", message: "翻译失败，请稍后重试" });
    }
  };

  const exportSubtitle = async () => {
    if (state.stage !== "translated" || !state.translationId) return;
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

  return { state, importVideo, translate, exportSubtitle };
}
