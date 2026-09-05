import { workflowSteps } from "../config/workflow";
import type { WorkflowStage } from "../types";
export function StageTrack({ stage }: { stage: WorkflowStage }) {
  return <div className="stage-track" aria-label="处理进度">{workflowSteps.map((step, index) => { const active = step.activeStages.includes(stage); const done = index === 0 ? ["imported", "speech-translated", "exported"].includes(stage) : index === 1 ? ["speech-translated", "exported"].includes(stage) : stage === "exported"; return <div className={`stage ${active ? "active" : ""} ${done ? "done" : ""}`} key={step.label}><span>{step.number}</span><strong>{step.label}</strong></div>; })}</div>;
}
