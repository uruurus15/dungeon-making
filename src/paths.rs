//! 実行ファイルの場所から assets/ を探す(どのフォルダから起動しても同じ場所を使う)。

use std::path::PathBuf;
use std::sync::OnceLock;

static ROOT: OnceLock<PathBuf> = OnceLock::new();

/// assets/ があるフォルダ(なければカレント)
pub fn root() -> &'static PathBuf {
    ROOT.get_or_init(|| {
        if let Ok(exe) = std::env::current_exe() {
            let mut d = exe.parent().map(|p| p.to_path_buf());
            while let Some(dir) = d {
                if dir.join("assets").is_dir() {
                    return dir;
                }
                d = dir.parent().map(|p| p.to_path_buf());
            }
        }
        std::env::current_dir().unwrap_or_default()
    })
}

pub fn save_path(debug: bool) -> PathBuf {
    root().join(if debug { "savedata_debug.json" } else { "savedata.json" })
}

pub fn asset(rel: &str) -> PathBuf {
    root().join(rel)
}

/// 音量・画面などの設定(セーブとは別に 保存して、起動のたびに読む)
pub fn settings_path() -> PathBuf {
    root().join("settings.json")
}
