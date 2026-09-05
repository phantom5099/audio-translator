import { AppHeader } from "../components/AppHeader";
import { StageTrack } from "../components/StageTrack";
import { MediaDropzone } from "../components/MediaDropzone";
import { WorkflowActions } from "../components/WorkflowActions";
import { WorkflowStatus } from "../components/WorkflowStatus";
import { isWorkflowBusy } from "../config/workflow";
import { useWorkflow } from "../hooks/useWorkflow";

export default function App() {
  const { state, importMedia, startSpeechTranslation, exportSubtitle } = useWorkflow();
  return (
    <main className="app-shell">
      <section className="workspace" aria-label="音频翻译工作区">
        <AppHeader />
        <StageTrack stage={state.stage} />
        <MediaDropzone file={state.file} disabled={isWorkflowBusy(state.stage)} onFile={importMedia} />
        <WorkflowStatus state={state} />
        <WorkflowActions stage={state.stage} onTranslate={() => void startSpeechTranslation()} onExport={() => void exportSubtitle()} />
      </section>
    </main>
  );
}
