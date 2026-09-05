import type { ImportResult, TranslateResult, WorkflowStage } from "../types";

export interface WorkflowState {
  stage: WorkflowStage;
  file?: ImportResult;
  translationId?: string;
  error?: string;
}

export type WorkflowAction =
  | { type: "import-started" }
  | { type: "import-succeeded"; result: ImportResult }
  | { type: "speech-translate-started" }
  | { type: "speech-translate-succeeded"; result: TranslateResult }
  | { type: "export-started" }
  | { type: "export-succeeded" }
  | { type: "failed"; message: string };

export const initialWorkflowState: WorkflowState = { stage: "idle" };

export function workflowReducer(
  state: WorkflowState,
  action: WorkflowAction,
): WorkflowState {
  switch (action.type) {
    case "import-started":
      return { stage: "importing" };
    case "import-succeeded":
      return { stage: "imported", file: action.result };
    case "speech-translate-started":
      return { ...state, stage: "speech-translating", error: undefined };
    case "speech-translate-succeeded":
      return { ...state, stage: "speech-translated", translationId: action.result };
    case "export-started":
      return { ...state, stage: "exporting", error: undefined };
    case "export-succeeded":
      return { ...state, stage: "exported" };
    case "failed":
      return { ...state, stage: "error", error: action.message };
  }
}
