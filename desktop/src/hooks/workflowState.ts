import type { ImportResult, TranslateResult, WorkflowStage } from "../types";

export interface WorkflowState {
  stage: WorkflowStage;
  file?: ImportResult["file"];
  taskId?: string;
  subtitleId?: string;
  cueCount?: number;
  error?: string;
}

export type WorkflowAction =
  | { type: "parse-started" }
  | { type: "parse-succeeded"; result: ImportResult }
  | { type: "translate-started" }
  | { type: "translate-succeeded"; result: TranslateResult }
  | { type: "export-started" }
  | { type: "export-succeeded" }
  | { type: "failed"; message: string };

export const initialWorkflowState: WorkflowState = { stage: "idle" };

export function workflowReducer(
  state: WorkflowState,
  action: WorkflowAction,
): WorkflowState {
  switch (action.type) {
    case "parse-started":
      return { stage: "parsing" };
    case "parse-succeeded":
      return { stage: "parsed", ...action.result };
    case "translate-started":
      return { ...state, stage: "translating", error: undefined };
    case "translate-succeeded":
      return { ...state, stage: "translated", ...action.result };
    case "export-started":
      return { ...state, stage: "exporting", error: undefined };
    case "export-succeeded":
      return { ...state, stage: "exported" };
    case "failed":
      return { ...state, stage: "error", error: action.message };
  }
}
