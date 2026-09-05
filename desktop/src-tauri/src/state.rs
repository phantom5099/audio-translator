use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use audio_translator::{
    asr::FasterWhisperAsrEngine,
    audio_input::{AudioAssetId, MediaAudioInputService},
    speech_translation::{AsrThenTranslationEngine, SpeechTranslationId, SpeechTranslationOutput},
    subtitle::SrtSubtitleExporter,
    translation::ArgosTranslator,
};
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub audio_input: MediaAudioInputService,
    pub speech_engine: AsrThenTranslationEngine,
    pub subtitle_exporter: SrtSubtitleExporter,
    pub audio_assets: Mutex<HashMap<AudioAssetId, PathBuf>>,
    pub translations: Mutex<HashMap<SpeechTranslationId, SpeechTranslationOutput>>,
}

impl AppState {
    pub fn new(app: &AppHandle) -> Self {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        let venv_python = project_root
            .join(".venv")
            .join("Scripts")
            .join("python.exe")
            .to_string_lossy()
            .into_owned();
        let whisper_model = std::env::var("AUDIO_TRANSLATOR_WHISPER_MODEL").unwrap_or_else(|_| {
            project_root
                .join("models")
                .join("faster-whisper-small")
                .to_string_lossy()
                .into_owned()
        });
        let argos_packages = std::env::var("ARGOS_PACKAGES_DIR").unwrap_or_else(|_| {
            project_root
                .join("models")
                .join("argos-packages")
                .to_string_lossy()
                .into_owned()
        });
        std::env::set_var("ARGOS_PACKAGES_DIR", &argos_packages);
        let ffprobe_path = resolve_sidecar(app, "ffprobe");
        let ffmpeg_path = resolve_sidecar(app, "ffmpeg");
        Self {
            audio_input: MediaAudioInputService::new(
                std::env::temp_dir().join("audio-translator-assets"),
                ffprobe_path,
                ffmpeg_path,
            ),
            speech_engine: AsrThenTranslationEngine::new(
                Box::new(FasterWhisperAsrEngine::new(whisper_model).with_python(venv_python.clone())),
                Box::new(ArgosTranslator::new().with_python(venv_python)),
            ),
            subtitle_exporter: SrtSubtitleExporter,
            audio_assets: Mutex::new(HashMap::new()),
            translations: Mutex::new(HashMap::new()),
        }
    }
}

fn resolve_sidecar(app: &AppHandle, name: &str) -> PathBuf {
    // 优先用 Tauri 构建系统注入的 target triple（cargo tauri dev/build）；
    // 未注入时（如直接 cargo build）回退到当前宿主推导的等价三元组。
    let triple = option_env!("TAURI_TARGET_TRIPLE")
        .map(str::to_owned)
        .or_else(host_target_triple)
        .unwrap_or_else(|| "x86_64-pc-windows-msvc".to_owned());
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let filename = format!("{name}-{triple}{suffix}");
    if let Ok(dir) = app.path().resource_dir() {
        let candidate = dir.join("binaries").join(&filename);
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(filename)
}

/// `TAURI_TARGET_TRIPLE` 未在编译期注入时，依据 `std::env::consts` 推导宿主目标三元组。
fn host_target_triple() -> Option<String> {
    use std::env::consts::{ARCH, OS};
    let arch = match ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        "x86" => "i686",
        _ => return None,
    };
    let triple = match OS {
        "windows" => format!("{arch}-pc-windows-msvc"),
        "macos" => format!("{arch}-apple-darwin"),
        "linux" => format!("{arch}-unknown-linux-gnu"),
        _ => return None,
    };
    Some(triple)
}
