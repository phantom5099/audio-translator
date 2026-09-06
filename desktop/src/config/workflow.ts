import type { WorkflowStage } from "../types";
export interface WorkflowStep { number: string; label: string; activeStages: WorkflowStage[]; }
export const workflowSteps: WorkflowStep[] = [
  { number: "01", label: "导入音频", activeStages: ["idle", "importing", "imported"] },
  { number: "02", label: "语音翻译", activeStages: ["speech-translating", "speech-translated"] },
  { number: "03", label: "导出字幕", activeStages: ["exporting", "exported"] },
];
export function getStatusText(stage: WorkflowStage, error?: string) {
  return ({ idle: "拖入本地音频或视频，开始导入", importing: "正在导入音频内容", imported: "音频导入完成，可以开始语音翻译", "speech-translating": "正在进行语音翻译", "speech-translated": "语音翻译完成，可以导出字幕文件", exporting: "正在生成字幕文件", exported: "字幕文件已导出", error: error ?? "流程暂时无法继续" })[stage];
}
export const isWorkflowBusy = (stage: WorkflowStage) => ["importing", "speech-translating", "exporting"].includes(stage);
