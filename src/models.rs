//! 3Dモデル一覧(F4 で 開く・とじる)。施設と魔物の 立体モデルを ならべて ゆっくり回す。
//! ←→=1つずつ ↑↓=1行(WASDも同じ) Q/E=ページ V=えらんだモデルを 大きく Z=回転を止める/動かす X/F4=もどる

use crate::data::*;
use crate::model3d::draw_model;
use crate::ui::*;
use crate::view3d::{Cam, VP};
use crate::{App, Scr};
use macroquad::prelude::*;

pub struct ModelsSt {
    pub page: usize,
    pub sel: usize,
    pub spin: bool,
    pub angle: f32,
    /// えらんだモデルを 大きく 1体だけ 見る
    pub big: bool,
}

impl ModelsSt {
    pub fn new() -> ModelsSt {
        ModelsSt { page: 0, sel: 0, spin: true, angle: 0.6, big: false }
    }
}

const COLS: usize = 4;
const ROWS: usize = 3;
const PER: usize = COLS * ROWS;

/// (分類, 名前, 絵の名前, 大きさ)
pub fn all_models() -> Vec<(&'static str, String, String, f32)> {
    let mut v = vec![];
    for r in ROOM_DEFS {
        let art = format!("room_{}", r.name);
        let size = if crate::modeling::hand_cells(&art) == 3 && crate::modeling::has_hand(&art) { 0.32 } else { 0.95 };
        v.push(("施設", r.name.to_string(), art.clone(), size));
        if chest_tier(r.name).is_some() {
            v.push(("施設", format!("{}(あけた)", r.name), format!("{}_open", art), 0.95));
        }
    }
    v.push(("地形", "階段(上り・入口)".to_string(), "tile_stairs_up".to_string(), 0.95));
    v.push(("地形", "階段(下り)".to_string(), "tile_stairs_down".to_string(), 0.95));
    v.push(("地形", "魔穴".to_string(), "tile_portal".to_string(), 0.95));
    let mut seen = std::collections::BTreeSet::new();
    for m in MONS {
        if seen.insert(m.sprite) {
            v.push((if m.boss { "ボス" } else { "魔物" }, m.name.to_string(), m.sprite.to_string(), if crate::bosses::is_boss_art(m.sprite) { 0.78 } else { 0.95 }));
        }
    }
    v
}

impl App {
    pub fn models_screen(&mut self, st: &mut ModelsSt) -> Option<Scr> {
        let list = all_models();
        let pages = list.len().div_ceil(PER);
        // 選択は 1つずつ(←→=となり、↑↓=1行、Q/E=ページ)。ページは 選んだものに 合わせて 自動で かわる
        let total = list.len();
        let mut idx = st.page * PER + st.sel;
        if self.inp.right {
            idx = (idx + 1) % total;
        }
        if self.inp.left {
            idx = (idx + total - 1) % total;
        }
        if self.inp.down {
            if idx + COLS < total {
                idx += COLS;
            }
        }
        if self.inp.up {
            if idx >= COLS {
                idx -= COLS;
            }
        }
        if self.inp.r {
            idx = ((idx / PER + 1) % pages) * PER;
        }
        if self.inp.l {
            idx = ((idx / PER + pages - 1) % pages) * PER;
        }
        idx = idx.min(total - 1);
        st.page = idx / PER;
        st.sel = idx % PER;
        let on_page = (list.len() - st.page * PER).min(PER);
        if self.inp.ok {
            st.spin = !st.spin;
        }
        if self.inp.y {
            st.big = !st.big;
        }
        if st.spin {
            st.angle += get_frame_time() * 0.9;
        }
        let ui = &self.ui;
        draw_rectangle(0.0, 0.0, LW, SH, rgb(34, 28, 40));
        // 上画面: 4x3 の並び(えらんだ物は 枠がつく)
        let (cw, ch) = (LW / COLS as f32, SH / ROWS as f32);
        let cam = Cam::new(0.0, 0.0, 0.0);
        let range: Vec<usize> = if st.big { vec![st.sel] } else { (0..on_page).collect() };
        for k in range {
            let (name_cat, name, art, size) = &list[st.page * PER + k];
            if st.big {
                draw_rectangle(6.0, 6.0, LW - 12.0, SH - 12.0, rgb(52, 44, 64));
                draw_ellipse(LW / 2.0, 205.0 + 0.5 * 700.0 / 2.4, 190.0, 22.0, 0.0, rgba(0, 0, 0, 90));
                VP.with(|v| v.set((LW / 2.0, 205.0, 700.0)));
                if let Some(m) = self.model(art, 0) {
                    let a = st.angle;
                    draw_model(&m, &cam, 0.0, 0.85 - 2.4, (a.sin(), a.cos()), *size, 0.0, 1.0);
                }
                VP.with(|v| v.set((LW / 2.0, 62.0 + 185.0, 230.0)));
                ui.text_center(name, LW / 2.0, SH - 34.0, 22.0, CREAM);
                continue;
            }
            let (gx, gy) = ((k % COLS) as f32, (k / COLS) as f32);
            let (x0, y0) = (gx * cw, gy * ch);
            let cell_col = if k == st.sel { rgb(70, 58, 84) } else { rgb(46, 38, 56) };
            draw_rectangle(x0 + 3.0, y0 + 3.0, cw - 6.0, ch - 6.0, cell_col);
            if k == st.sel {
                draw_rectangle_lines(x0 + 3.0, y0 + 3.0, cw - 6.0, ch - 6.0, 3.0, rgb(255, 214, 80));
            }
            // 床
            draw_ellipse(x0 + cw / 2.0, y0 + ch * 0.44 + 0.5 * 215.0 / 2.4, cw * 0.3, 8.0, 0.0, rgba(0, 0, 0, 90));
            // この枠に 投影を 合わせる
            VP.with(|v| v.set((x0 + cw / 2.0, y0 + ch * 0.44, 215.0)));
            if let Some(m) = self.model(art, 0) {
                // 床の線が ちょうど 枠の下の方に 来るように 手前に置く
                let a = st.angle + k as f32 * 0.5;
                draw_model(&m, &cam, 0.0, 0.85 - 2.4, (a.sin(), a.cos()), *size, 0.0, 1.0);
            }
            VP.with(|v| v.set((LW / 2.0, 62.0 + 185.0, 230.0)));
            let _ = name_cat;
            ui.text_center(name, x0 + cw / 2.0, y0 + ch - 24.0, 15.0, CREAM);
        }
        // 下画面: えらんだモデルの くわしい 情報と 操作
        panel_wood(0.0, SH, LW, SH);
        ui.text("3Dモデル いちらん", 18.0, SH + 10.0, 24.0, CREAM);
        ui.text_right(&format!("{}/{}ページ  全{}体", st.page + 1, pages, list.len()), LW - 18.0, SH + 14.0, 18.0, CREAM);
        let (cat, name, art, _) = &list[st.page * PER + st.sel];
        let info = self.model(art, 0).map(|m| (m.voxels, m.quads.len(), m.hand));
        gold_box(14.0, SH + 52.0, LW - 28.0, 120.0);
        ui.text(&format!("{}  {}", cat, name), 30.0, SH + 62.0, 24.0, GOLDEN);
        if let Some((vx, q, hand)) = info {
            if hand {
                ui.text(&format!("手で組んだ モデル  面 {}  (なめらかな面で 組んだ)", q), 30.0, SH + 100.0, 17.0, rgb(150, 235, 120));
            } else {
                ui.text(&format!("立方体 {} こ  面 {}  (絵から ふくらませて 作った 立体)", vx, q), 30.0, SH + 100.0, 17.0, CREAM);
            }
        } else {
            ui.text("(このモデルの 絵が ありません)", 30.0, SH + 100.0, 17.0, rgb(255, 150, 130));
        }
        ui.text(&format!("もとの絵: {}.png", art), 30.0, SH + 128.0, 16.0, rgb(200, 190, 160));
        let _ = &list;
        // 同じページの 名前の一覧
        for k in 0..on_page {
            let (c, n, _, _) = &list[st.page * PER + k];
            let (col, row) = (k % 2, k / 2);
            let col_c = if k == st.sel { rgb(255, 230, 140) } else { CREAM };
            ui.text(&format!("{} {}", if k == st.sel { "▶" } else { " " }, n), 22.0 + col as f32 * 270.0, SH + 192.0 + row as f32 * 28.0, 17.0, col_c);
            let _ = c;
        }
        ui.text("←→:1つずつ  ↑↓:1行  Q・E:ページ  Z:回転  V:大きく  X・F4:もどる", 18.0, SH + 392.0, 15.0, rgb(220, 205, 170));
        if self.inp.cancel {
            return self.prev_scr.take();
        }
        None
    }
}
