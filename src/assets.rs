//! 素材(画像・音・フォント)と セーブの 読み書き。
//! デスクトップ: assets/ フォルダから 直接 読み、セーブは ファイル。
//! Web(wasm): 素材は assets.pack(1つに まとめたもの)を 最初に まとめて ダウンロードし、セーブは ブラウザの localStorage。

use std::collections::HashMap;
use std::sync::OnceLock;

static PACK: OnceLock<HashMap<String, Vec<u8>>> = OnceLock::new();

/// assets.pack を 読む(あれば)。形式: 件数(u32 LE) のあと、(名前の長さ u32, 名前, データの長さ u32, データ) の くりかえし
pub async fn init() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(bytes) = macroquad::file::load_file("assets.pack").await {
            let _ = PACK.set(parse_pack(&bytes));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // デスクトップでも assets.pack が 実行ファイルの となりに あれば 使う
        if let Ok(bytes) = std::fs::read(crate::paths::asset("assets.pack")) {
            let _ = PACK.set(parse_pack(&bytes));
        }
    }
}

fn parse_pack(b: &[u8]) -> HashMap<String, Vec<u8>> {
    let mut m = HashMap::new();
    let u32at = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize;
    if b.len() < 4 {
        return m;
    }
    let n = u32at(0);
    let mut i = 4;
    for _ in 0..n {
        if i + 4 > b.len() {
            break;
        }
        let nl = u32at(i);
        i += 4;
        let name = String::from_utf8_lossy(&b[i..i + nl]).to_string();
        i += nl;
        let dl = u32at(i);
        i += 4;
        m.insert(name, b[i..i + dl].to_vec());
        i += dl;
    }
    m
}

/// 素材を 読む。rel は「assets/img/mon_bat.png」のような 相対パス
pub async fn read(rel: &str) -> Option<Vec<u8>> {
    if let Some(p) = PACK.get() {
        return p.get(rel).cloned();
    }
    #[cfg(target_arch = "wasm32")]
    {
        macroquad::file::load_file(rel).await.ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read(crate::paths::asset(rel)).ok()
    }
}

/// 文字の フォント: デスクトップは Windows の 日本語フォントが あればそれ、なければ 同梱の NotoSansJP(OFL)。Web は 同梱のみ
pub async fn font_bytes() -> Option<Vec<u8>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        for c in ["C:/Windows/Fonts/YuGothB.ttc", "C:/Windows/Fonts/meiryob.ttc", "C:/Windows/Fonts/meiryo.ttc", "C:/Windows/Fonts/msgothic.ttc"] {
            if let Ok(b) = std::fs::read(c) {
                return Some(b);
            }
        }
    }
    read("assets/font/NotoSansJP-Bold-subset.ttf").await
}

// ---------------- セーブ・設定 ----------------

/// 保存した 文字列を 読む。name は "save" / "save_debug" / "settings"
pub fn load_text(name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        quad_storage::STORAGE.lock().ok()?.get(name)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(file_of(name)).ok()
    }
}

pub fn save_text(name: &str, text: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(mut s) = quad_storage::STORAGE.lock() {
            s.set(name, text);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::write(file_of(name), text);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn file_of(name: &str) -> std::path::PathBuf {
    match name {
        "save" => crate::paths::save_path(false),
        "save_debug" => crate::paths::save_path(true),
        _ => crate::paths::settings_path(),
    }
}

pub fn save_name(debug: bool) -> &'static str {
    if debug {
        "save_debug"
    } else {
        "save"
    }
}
