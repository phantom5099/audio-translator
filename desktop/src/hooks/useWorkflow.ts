import { useReducer } from "react";
import { isWorkflowBusy } from "../config/workflow";
import { mockApi } from "../services/mockApi";
import { initialWorkflowState, workflowReducer } from "./workflowState";
export function useWorkflow() {
  const [state, dispatch] = useReducer(workflowReducer, initialWorkflowState);
  const importVideo = async (file?: File) => { if (!file || isWorkflowBusy(state.stage)) return; dispatch({ type: "parse-started" }); try { dispatch({ type: "parse-succeeded", result: await mockApi.importVideo(file) }); } catch { dispatch({ type: "failed", message: "视频解析失败，请重新导入" }); } };
  const translate = async () => { if (state.stage !== "parsed" || !state.taskId) return; dispatch({ type: "translate-started" }); try { dispatch({ type: "translate-succeeded", result: await mockApi.translate(state.taskId) }); } catch { dispatch({ type: "failed", message: "翻译失败，请稍后重试" }); } };
  const exportSubtitle = async () => { if (state.stage !== "translated" || !state.subtitleId) return; dispatch({ type: "export-started" }); try { const result = await mockApi.exportSubtitle(state.subtitleId); const url = URL.createObjectURL(result.content); const anchor = document.createElement("a"); anchor.href = url; anchor.download = result.fileName; anchor.click(); URL.revokeObjectURL(url); dispatch({ type: "export-succeeded" }); } catch { dispatch({ type: "failed", message: "字幕导出失败，请稍后重试" }); } };
  return { state, importVideo, translate, exportSubtitle };
}
