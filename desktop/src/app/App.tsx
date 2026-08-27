import { AppHeader } from "../components/AppHeader";
import { StageTrack } from "../components/StageTrack";
import { VideoDropzone } from "../components/VideoDropzone";
import { WorkflowActions } from "../components/WorkflowActions";
import { WorkflowStatus } from "../components/WorkflowStatus";
import { isWorkflowBusy } from "../config/workflow";
import { useWorkflow } from "../hooks/useWorkflow";

export default function App() {
  const { state, importVideo, translate, exportSubtitle } = useWorkflow();
  return (
    <main className="app-shell">
      <section className="workspace" aria-label="视频翻译工作区">
        <AppHeader />
        <StageTrack stage={state.stage} />
        <VideoDropzone file={state.file} disabled={isWorkflowBusy(state.stage)} onFile={importVideo} />
        <WorkflowStatus state={state} />
        <WorkflowActions stage={state.stage} onTranslate={() => void translate()} onExport={() => void exportSubtitle()} />
      </section>
    </main>
  );
}
