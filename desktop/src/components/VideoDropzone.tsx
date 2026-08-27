import { useRef, useState, type DragEvent } from "react";
import { formatDuration, formatSize } from "../utils/format";
import type { VideoFile } from "../types";
interface Props { file?: VideoFile; disabled: boolean; onFile: (file?: File) => void; }
export function VideoDropzone({ file, disabled, onFile }: Props) {
  const inputRef = useRef<HTMLInputElement>(null); const [dragActive, setDragActive] = useState(false);
  const onDrop = (event: DragEvent<HTMLDivElement>) => { event.preventDefault(); setDragActive(false); onFile(event.dataTransfer.files[0]); };
  return <div className={`dropzone ${dragActive ? "drag-active" : ""} ${file ? "has-file" : ""}`} onDragEnter={(event) => { event.preventDefault(); setDragActive(true); }} onDragOver={(event) => event.preventDefault()} onDragLeave={() => setDragActive(false)} onDrop={onDrop} onClick={() => !disabled && inputRef.current?.click()} role="button" tabIndex={0} onKeyDown={(event) => { if ((event.key === "Enter" || event.key === " ") && !disabled) inputRef.current?.click(); }}><input ref={inputRef} type="file" accept="video/*" hidden onChange={(event) => onFile(event.target.files?.[0])} /><div className="drop-icon">▣</div><p className="drop-title">{file ? file.name : "将本地视频拖到这里"}</p><p className="drop-hint">{file ? `${formatSize(file.sizeBytes)} · ${formatDuration(file.durationSeconds)}` : "支持 MP4、MOV、MKV 等视频格式"}</p></div>;
}
