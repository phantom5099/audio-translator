import type { WorkflowStage } from "../types";
export interface WorkflowStep { number: string; label: string; activeStages: WorkflowStage[]; }
export const workflowSteps: WorkflowStep[] = [
  { number: "01", label: "导入视频", activeStages: ["idle", "parsing", "parsed"] },
  { number: "02", label: "翻译内容", activeStages: ["translating", "translated"] },
  { number: "03", label: "导出字幕", activeStages: ["exporting", "exported"] },
];
export function getStatusText(stage: WorkflowStage, error?: string) {
  return ({ idle: "拖入本地视频，开始解析", parsing: "正在解析视频内容", parsed: "视频解析完成，可以开始翻译", translating: "正在翻译字幕内容", translated: "翻译完成，可以导出字幕文件", exporting: "正在生成字幕文件", exported: "字幕文件已导出", error: error ?? "流程暂时无法继续" })[stage];
}
export const isWorkflowBusy = (stage: WorkflowStage) => ["parsing", "translating", "exporting"].includes(stage);
