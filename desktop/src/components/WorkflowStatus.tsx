import { getStatusText, isWorkflowBusy } from "../config/workflow";
import type { WorkflowState } from "../hooks/workflowState";

export function WorkflowStatus({ state }: { state: WorkflowState }) {
  const busy = isWorkflowBusy(state.stage);
  return (
    <div className="status-panel">
      <div className="status-row">
        <span className={`status-dot ${busy ? "processing" : state.stage === "error" ? "error" : ""}`} />
        <span>{getStatusText(state.stage, state.error)}</span>
        {busy && <span className="status-spinner" aria-hidden="true" />}
      </div>
      {busy && (
        <div className="progress-track">
          <div className="progress-fill" />
        </div>
      )}
    </div>
  );
}
