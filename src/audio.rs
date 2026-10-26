//! 効果音と音楽。assets/sfx/<イベント名>.ogg と assets/bgm/<曲名>.ogg を起動時に読む。
//! 読み込みに失敗しても(音声デバイスが無い・ファイルが無い)ゲームは無音のまま動く。
//! 効果音は Openverse(CC0/PD中心)から tools/fetch_sfx.py で集めたもの。出典は assets/sfx/CREDITS.md。
//! 音楽は tools/gen_music.py で合成したもの。

use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};
use macroquad::time::get_time;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

/// 効果音のイベント(tools/sfx_table.py と同じ名前。ファイルは assets/sfx/<名前>.ogg)
pub const SFX: &[&str] = &[
    "move", "ok", "cancel", "blocked", "step", "dig", "fill", "place", "break", "undo", "encounter", "chest", "stairs", "portal", "attack", "hit", "crit", "down", "fire", "heal", "sleepspell",
    "item", "bomb", "flee", "boss", "win", "lose", "buy", "sell", "eat", "equip", "quest", "morning", "sleep",
];

/// 曲(assets/bgm/<名前>.ogg)と 音量
/// 音楽の曲と基準の音量(オプションの音量はこれにかける)。効果音より小さめ
pub const BGM: &[(&str, f32)] = &[("town", 0.8), ("battle", 0.85), ("boss", 0.8)];

/// 同じ音が この秒数のあいだに重ならないようにする
const MIN_GAP: f64 = 0.07;

pub struct Audio {
    sfx: HashMap<&'static str, Sound>,
    bgm: HashMap<&'static str, Sound>,
    cur: RefCell<Option<&'static str>>,
    last: RefCell<HashMap<&'static str, f64>>,
    /// 0=鳴らす 2=ぜんぶ消す(Mキー)
    pub mode: Cell<u8>,
    /// 音量 0.0〜1.0
    pub bgm_vol: Cell<f32>,
    pub sfx_vol: Cell<f32>,
    /// この時刻まで 音楽をはじめない(勝利のファンファーレなどのあいだ)
    quiet_until: Cell<f64>,
}

impl Audio {
    /// 何も鳴らさない
    pub fn silent() -> Audio {
        Audio { sfx: HashMap::new(), bgm: HashMap::new(), cur: RefCell::new(None), last: RefCell::new(HashMap::new()), mode: Cell::new(0), bgm_vol: Cell::new(0.6), sfx_vol: Cell::new(0.8), quiet_until: Cell::new(0.0) }
    }

    pub async fn load() -> Audio {
        let mut a = Audio::silent();
        for name in SFX {
            if let Some(bytes) = crate::assets::read(&format!("assets/sfx/{}.ogg", name)).await {
                if let Ok(s) = load_sound_from_bytes(&bytes).await {
                    a.sfx.insert(name, s);
                }
            }
        }
        for (name, _) in BGM {
            if let Some(bytes) = crate::assets::read(&format!("assets/bgm/{}.ogg", name)).await {
                if let Ok(s) = load_sound_from_bytes(&bytes).await {
                    a.bgm.insert(name, s);
                }
            }
        }
        a
    }

    pub fn sfx(&self, name: &'static str) {
        if self.mode.get() >= 2 || self.sfx_vol.get() <= 0.0 {
            return;
        }
        let Some(s) = self.sfx.get(name) else { return };
        let now = get_time();
        let mut last = self.last.borrow_mut();
        if let Some(t) = last.get(name) {
            if now - *t < MIN_GAP {
                return;
            }
        }
        last.insert(name, now);
        play_sound(s, PlaySoundParams { looped: false, volume: 0.9 * self.sfx_vol.get() });
    }

    /// 音楽を切りかえる(いま鳴っている曲と同じなら何もしない)。None で止める
    pub fn bgm(&self, want: Option<&'static str>) {
        let want = if self.mode.get() == 0 && self.bgm_vol.get() > 0.0 && get_time() >= self.quiet_until.get() { want } else { None };
        let mut cur = self.cur.borrow_mut();
        if *cur == want {
            // 音量だけ かえる
            if let Some(c) = *cur {
                if let Some(s) = self.bgm.get(c) {
                    let base = BGM.iter().find(|(n, _)| *n == c).map(|(_, v)| *v).unwrap_or(0.5);
                    set_sound_volume(s, base * self.bgm_vol.get());
                }
            }
            return;
        }
        if let Some(c) = *cur {
            if let Some(s) = self.bgm.get(c) {
                stop_sound(s);
            }
        }
        *cur = None;
        if let Some(w) = want {
            if let Some(s) = self.bgm.get(w) {
                let vol = BGM.iter().find(|(n, _)| *n == w).map(|(_, v)| *v).unwrap_or(0.5);
                play_sound(s, PlaySoundParams { looped: true, volume: vol * self.bgm_vol.get() });
                *cur = Some(w);
            }
        }
    }

    /// 勝ち・負けのファンファーレ: 音楽を止めて 鳴らし、終わるまで 音楽をはじめない
    pub fn jingle(&self, name: &'static str, secs: f64) {
        self.bgm(None);
        self.quiet_until.set(get_time() + secs);
        self.sfx(name);
    }
}

/// 戦闘のメッセージ1行に合う効果音。party は味方の名前
pub fn sfx_for_message(msg: &str, party: &[String]) -> Option<&'static str> {
    let hits_party = party.iter().any(|n| msg.contains(&format!("{} に ", n)) || msg.contains(&format!("{}に ", n)));
    if msg.contains("ひっさつ") {
        return Some("crit");
    }
    if msg.contains("たおした") || msg.contains("うごけなくなった") {
        return Some("down");
    }
    if msg.contains("バクダン") && msg.contains("なげた") {
        return Some("bomb");
    }
    if msg.contains("にげだした") || msg.contains("にげきれた") {
        return Some("flee");
    }
    if msg.contains("すいみん") || msg.contains("ねむってしまった") {
        return Some("sleepspell");
    }
    if msg.contains("かいふく") {
        return Some("heal");
    }
    if msg.contains("もえあがれ") || msg.contains("炎") || msg.contains("ほのお") || msg.contains("せいなる光") {
        return Some("fire");
    }
    if msg.contains("をつかった") {
        return Some("item");
    }
    if msg.contains("のダメージ") {
        return Some(if hits_party { "hit" } else { "attack" });
    }
    None
}
