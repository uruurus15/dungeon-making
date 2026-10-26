//! 会話の舞台: 背景の上に 立ち絵を並べ、しゃべっている人を明るく、ほかの人を暗くする。
//! 行の頭が 「名前「」 ならその人、頭が無い行は default(村長など)か ナレーション。
//! 会話シーン(talk.rs)と 店の人の話(shop.rs)が 同じ描きかたを使う。

use crate::ui::*;
use macroquad::prelude::*;

/// 立ち絵のある人。(名前, 立ち絵の絵, 横の中心, 高さ)
const CAST: [(&str, &str, f32, f32); 4] = [("アレン", "stand_a", 112.0, 262.0), ("メイミ", "stand_m", 468.0, 262.0), ("ジェリー", "stand_j", 236.0, 120.0), ("村長", "stand_mayor", 468.0, 262.0)];

/// 行の話し手と、「名前「」」を外した しゃべる内容。頭が「＊」の行は 地の文(だれのセリフでもない)
pub fn parse(line: &str, default: Option<&'static str>) -> (Option<&'static str>, String) {
    if let Some(rest) = line.strip_prefix('＊') {
        return (None, rest.to_string());
    }
    for (n, _, _, _) in CAST {
        if let Some(rest) = line.strip_prefix(&format!("{}「", n)) {
            return (Some(n), format!("「{}", rest));
        }
    }
    (default, if default.is_some() && line.starts_with('「') { line.to_string() } else if default.is_some() { format!("「{}」", line.trim_end_matches('」')) } else { line.to_string() })
}

/// その場面に出ている人(行の話し手)を 出てくる順に
pub fn cast_of<'a, I: Iterator<Item = &'a String>>(lines: I) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = vec![];
    for l in lines {
        let (w, _) = parse(l, None);
        if let Some(w) = w {
            if !v.contains(&w) {
                v.push(w);
            }
        }
    }
    v
}

/// 立ち絵を描く。bottom は足もとの y
pub fn draw_cast(ui: &Ui, cast: &[&str], speaker: Option<&str>, bottom: f32, walk: bool) {
    for (name, art, cx, h) in CAST {
        if !cast.contains(&name) {
            continue;
        }
        let lit = speaker.is_none() || speaker == Some(name);
        let hop = if walk {
            // 歩いているあいだは みんな ぴょこぴょこ はずむ
            ((ui.t * 9.0 + cx * 0.05).sin().abs() * 5.0).round()
        } else if speaker == Some(name) {
            ((ui.t * 6.0).sin().abs() * 3.0).round()
        } else {
            0.0
        };
        let c = if lit { WHITE } else { Color::new(0.5, 0.5, 0.55, 1.0) };
        if let Some(t) = ui.art.get(art) {
            let w = h * t.width() / t.height();
            ui.art.draw_tinted(art, cx - w / 2.0, bottom - h - hop, w, h, c);
        }
    }
}

/// メッセージ欄の 文字の 大きさ(どの場面でも 同じ。3行まで 枠に おさまる)
pub const MSG_SIZE: f32 = 18.0;
pub const MSG_LH: f32 = 22.0;

/// メッセージ欄に 文を 描く(大きさは いつも 同じ)
pub fn fit_text(ui: &Ui, text: &str, x: f32, y: f32, w: f32, _box_h: f32) {
    ui.text_wrap(text, x, y - 2.0, w, MSG_SIZE, CREAM, MSG_LH);
}

/// 話し手の名札(メッセージ欄の左上)
pub fn nameplate(ui: &Ui, who: &str, x: f32, y: f32) {
    let w = ui.measure(who, 20.0) + 24.0;
    draw_rectangle(x, y, w, 26.0, rgb(52, 30, 14));
    draw_rectangle_lines(x, y, w, 26.0, 2.0, rgb(232, 217, 168));
    ui.text(who, x + 12.0, y + 3.0, 20.0, rgb(255, 230, 140));
}

/// 立ち絵 + メッセージ欄(背景は呼ぶ側が先に描く)。box_y: メッセージ欄の上端、box_h: 高さ
pub fn draw_talk(ui: &Ui, line: &str, default: Option<&'static str>, cast: &[&str], box_y: f32, box_h: f32, walk: bool) {
    let (who, text) = parse(line, default);
    draw_cast(ui, cast, if walk { None } else { who }, box_y + 26.0, walk);
    msgbox(16.0, box_y, 544.0, box_h);
    if let Some(w) = who {
        nameplate(ui, w, 28.0, box_y - 14.0);
    }
    fit_text(ui, &text, 34.0, box_y + 16.0, 508.0, box_h);
}

/// 顔のアイコン(パーティの人の 顔だけ。立ち絵は出さない)
fn face_of(who: &str) -> Option<&'static str> {
    match who {
        "アレン" => Some("portrait_a"),
        "メイミ" => Some("portrait_m"),
        "ジェリー" => Some("portrait_j"),
        _ => None,
    }
}

/// 店の人・村長との会話: 背景に その人が描かれているので 立ち絵は出さず、
/// パーティの人が しゃべるときだけ メッセージ欄の左に 顔を出す
pub fn draw_talk_face(ui: &Ui, line: &str, default: Option<&'static str>, box_y: f32, box_h: f32) {
    let (who, text) = parse(line, default);
    msgbox(16.0, box_y, 544.0, box_h);
    if let Some(w) = who {
        nameplate(ui, w, 28.0, box_y - 14.0);
    }
    if let Some(face) = who.and_then(face_of) {
        let fy = box_y + (box_h - 72.0) / 2.0;
        draw_rectangle(26.0, fy, 72.0, 72.0, rgb(52, 36, 22));
        ui.art.draw(face, 27.0, fy + 1.0, 70.0, 70.0);
        draw_rectangle_lines(26.0, fy, 72.0, 72.0, 2.5, rgb(232, 217, 168));
        fit_text(ui, &text, 112.0, box_y + 14.0, 436.0, box_h);
    } else {
        fit_text(ui, &text, 34.0, box_y + 14.0, 508.0, box_h);
    }
}
