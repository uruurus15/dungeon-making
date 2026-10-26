//! 会話シーン(背景 + メッセージ欄 + 話す人の顔)。仲間が加わる場面などに使う。
//! 行の頭が 「アレン「」「メイミ「」「ジェリー「」なら その人の立ち絵が動く(描きかたは stage.rs)。台詞は src/script.rs にある。

use crate::ui::*;
use crate::{App, Scr};
use macroquad::prelude::*;
use std::collections::VecDeque;

pub struct TalkSt {
    /// 背景(art の名前)
    pub bg: &'static str,
    pub lines: VecDeque<String>,
    /// 話が終わったあとの画面
    pub then: Option<Scr>,
    /// 頭に名前の無い行を しゃべる人(村長など)。None ならナレーション
    pub default: Option<&'static str>,
    /// 立ち絵を出している人(アレンは最初から。ほかは しゃべり始めたときに 出てくる)
    pub cast: Vec<&'static str>,
    /// 立ち絵でなく 顔のアイコンで見せる(村長・店の人との会話)
    pub faces: bool,
    /// 歩いている場面(背景が 横に ながれ、みんな はずむ)
    pub walk: bool,
}

impl TalkSt {
    pub fn new(bg: &'static str, lines: Vec<String>, then: Scr) -> TalkSt {
        // アレンは はじめから場にいる。ほかの人は 最初にしゃべる行で 登場する
        let cast = if crate::stage::cast_of(lines.iter()).contains(&"アレン") { vec!["アレン"] } else { vec![] };
        TalkSt { bg, lines: lines.into(), then: Some(then), default: None, cast, faces: false, walk: false }
    }

    /// 頭に名前の無い行を しゃべる人を決める
    pub fn speaker(mut self, who: &'static str) -> TalkSt {
        self.default = Some(who);
        self
    }

    /// 歩いている場面
    pub fn walking(mut self) -> TalkSt {
        self.walk = true;
        self
    }

    /// 村長・店の人との会話: 立ち絵は出さず、パーティの人の顔を メッセージ欄の左に出す
    pub fn faces(mut self) -> TalkSt {
        self.faces = true;
        self
    }

    /// 台詞が無くても 立ち絵に出す人を足す
    pub fn with_cast(mut self, who: &'static str) -> TalkSt {
        if !self.cast.contains(&who) {
            self.cast.push(who);
        }
        self
    }
}

impl App {
    pub fn talk(&mut self, st: &mut TalkSt) -> Option<Scr> {
        if st.walk {
            // 背景を 左へ ゆっくり ながす(おなじ絵を 2まい ならべて つなぐ)
            let off = (self.ui.t * 26.0) % LW;
            self.ui.art.draw(st.bg, -off, 0.0, LW, SH);
            self.ui.art.draw(st.bg, LW - off, 0.0, LW, SH);
        } else {
            self.ui.art.draw(st.bg, 0.0, 0.0, LW, SH);
        }
        let line = st.lines.front().cloned().unwrap_or_default();
        if let (Some(w), _) = crate::stage::parse(&line, st.default) {
            if !st.cast.contains(&w) {
                st.cast.push(w);
            }
        }
        if st.faces {
            crate::stage::draw_talk_face(&self.ui, &line, st.default, 322.0, 98.0);
        } else {
            crate::stage::draw_talk(&self.ui, &line, st.default, &st.cast, 322.0, 98.0, st.walk);
        }
        panel_wood(0.0, SH, LW, SH);
        let blink = if (self.ui.t * 2.0) as i32 % 2 == 0 { "▼ Zで つぎへ" } else { "" };
        self.ui.text_center(blink, LW / 2.0, SH + 380.0, 20.0, CREAM);
        if self.inp.ok || self.inp.click {
            self.ui.audio.sfx("ok");
            st.lines.pop_front();
            if st.lines.is_empty() {
                return st.then.take();
            }
        }
        None
    }
}
