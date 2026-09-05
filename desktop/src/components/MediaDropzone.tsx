import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { formatDuration, formatSize } from "../utils/format";
import type { ImportResult } from "../types";

interface Props {
  file?: ImportResult;
  disabled: boolean;
  onFile: (path: string) => void;
}

const MEDIA_EXTENSIONS = [
  "mp4",
  "mov",
  "mkv",
  "avi",
  "webm",
  "m4a",
  "wav",
  "mp3",
  "flac",
  "aac",
];

export function MediaDropzone({ file, disabled, onFile }: Props) {
  const [dragActive, setDragActive] = useState(false);
  const [coverUrl, setCoverUrl] = useState<string | null>(null);
  const onFileRef = useRef(onFile);
  onFileRef.current = onFile;
  const disabledRef = useRef(disabled);
  disabledRef.current = disabled;

  useEffect(() => {
    const win = getCurrentWindow();
    let unlisten: UnlistenFn | undefined;
    let cancelled = false;
    win.onDragDropEvent((event) => {
      if (disabledRef.current) return;
      if (event.payload.type === "enter" || event.payload.type === "over") {
        setDragActive(true);
      } else if (event.payload.type === "leave") {
        setDragActive(false);
      } else if (event.payload.type === "drop") {
        setDragActive(false);
        const path = event.payload.paths?.[0];
        if (path) {
          onFileRef.current(path);
        }
      }
    }).then((un) => {
      if (cancelled) {
        un();
      } else {
        unlisten = un;
      }
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const cover = file?.metadata.cover;
    if (!cover || cover.bytes.length === 0) {
      setCoverUrl(null);
      return;
    }
    const blob = new Blob([new Uint8Array(cover.bytes)], { type: cover.media_type });
    const url = URL.createObjectURL(blob);
    setCoverUrl(url);
    return () => {
      URL.revokeObjectURL(url);
    };
  }, [file?.metadata.cover]);

  const onBrowse = async () => {
    if (disabledRef.current) return;
    const selected = await open({
      multiple: false,
      filters: [{ name: "音视频", extensions: MEDIA_EXTENSIONS }],
    });
    if (typeof selected === "string") {
      onFileRef.current(selected);
    }
  };

  const fileName = file?.file_name;
  const sizeBytes = file?.metadata.size_bytes ?? null;
  const durationMs = file?.metadata.duration_ms ?? null;
  const hasMeta = fileName != null && sizeBytes != null && durationMs != null;

  return (
    <div
      className={`dropzone ${dragActive ? "drag-active" : ""} ${file ? "has-file" : ""}`}
      onDragEnter={(event) => {
        event.preventDefault();
        setDragActive(true);
      }}
      onDragOver={(event) => event.preventDefault()}
      onDragLeave={() => setDragActive(false)}
      onDrop={(event) => event.preventDefault()}
      onClick={onBrowse}
      role="button"
      tabIndex={0}
      onKeyDown={(event) => {
        if ((event.key === "Enter" || event.key === " ") && !disabledRef.current) {
          event.preventDefault();
          void onBrowse();
        }
      }}
    >
      {coverUrl ? (
        <img className="drop-cover" src={coverUrl} alt={fileName ?? "封面"} />
      ) : (
        <div className="drop-icon">▣</div>
      )}
      <p className="drop-title">{fileName ?? "将本地音频或视频拖到这里"}</p>
      <p className="drop-hint">
        {hasMeta
          ? `${formatSize(sizeBytes as number)} · ${formatDuration(Math.floor((durationMs as number) / 1000))}`
          : "支持 MP4、MOV、MKV、MP3 等音视频格式"}
      </p>
    </div>
  );
}
