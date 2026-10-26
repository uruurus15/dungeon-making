//! ダンジョン画面: 上=マップ(シャベルで掘る・部屋を置く)、下=パーティ・階数と評価・ミニマップ。

use crate::battle::BattleSt;
use crate::data::*;
use crate::overlays::Overlay;
use crate::quest::main_quest;
use crate::sim::*;
use crate::town::{comma, TownSt};
use crate::ui::*;
use crate::{App, Scr};
use macroquad::prelude::*;

pub enum DSub {
    Play,
    Menu(ListMenu),
    Rooms(ListMenu, Vec<String>, bool),
    Magic(ListMenu),
    Warp(ListMenu),
    HealWho(ListMenu),
    CureWho(ListMenu),
    ConfirmExit(ListMenu),
}

pub struct DunSt {
    pub sub: DSub,
    /// 「うめる」の確認待ち(押しまちがい防止): (対象マス, 残り秒)
    pub arm: Option<((usize, usize), f32)>,
    /// 3D: 長押しで進んでいる向き(キーごと。押しはじめの向きで 進みつづける)
    pub hold_world: [Option<Dir>; 4],
}

impl DunSt {
    pub fn new() -> DunSt {
        DunSt { sub: DSub::Play, arm: None, hold_world: [None; 4] }
    }
}

const TS: f32 = 36.0;
/// 上のバー(所持金・日数・メインクエスト)の高さ。マップはこの下に描く
const OFFY: f32 = 62.0;

fn mon_sprite(id: &str) -> &'static str {
    crate::data::mon(id).sprite
}

fn room_art(kind: &str, open: bool) -> String {
    if open {
        format!("room_{}_open", kind)
    } else {
        format!("room_{}", kind)
    }
}

fn tri(px: f32, py: f32, d: Dir, c: Color) {
    // (px,py)=タイルの左上。辺の中央に小さな三角を描く
    let (a, b, cc) = match d {
        Dir::Up => (vec2(px + 12.0, py + 1.0), vec2(px + 24.0, py + 1.0), vec2(px + 18.0, py + 8.0)),
        Dir::Down => (vec2(px + 12.0, py + TS - 1.0), vec2(px + 24.0, py + TS - 1.0), vec2(px + 18.0, py + TS - 8.0)),
        Dir::Left => (vec2(px + 1.0, py + 12.0), vec2(px + 1.0, py + 24.0), vec2(px + 8.0, py + 18.0)),
        Dir::Right => (vec2(px + TS - 1.0, py + 12.0), vec2(px + TS - 1.0, py + 24.0), vec2(px + TS - 8.0, py + 18.0)),
    };
    draw_triangle(a, b, cc, c);
}

impl App {
    /// 瘴気発生装置を 置いた階は、うっすら 紫の 蜃気楼が かかる(置いた数が 多いほど こい)
    pub fn draw_haze(&self, y0: f32, y1: f32) {
        let n = self.g.miasma_count();
        if n == 0 {
            return;
        }
        let t = self.ui.t;
        let base = 0.03 + 0.011 * n as f32;
        draw_rectangle(0.0, y0, LW, y1 - y0, Color::new(0.55, 0.2, 0.8, base));
        // ゆらめく 帯(蜃気楼)
        let step = 6.0;
        let mut y = y0;
        while y < y1 {
            let w = (y * 0.045 + t * 1.6).sin() * 0.5 + 0.5;
            let w2 = (y * 0.11 - t * 0.9).sin() * 0.5 + 0.5;
            let a = base * (0.3 + 0.9 * w * w2);
            let dx = (y * 0.07 + t * 2.0).sin() * 5.0;
            draw_rectangle(dx - 6.0, y, LW + 12.0, step, Color::new(0.75, 0.35, 1.0, a));
            y += step;
        }
        // うっすら 立ちのぼる けむりの 玉
        for k in 0..(4 + 2 * n) {
            let ph = t * 0.25 + k as f32 * 1.7;
            let (x, yy) = ((k as f32 * 97.0 + (ph * 0.8).sin() * 40.0).rem_euclid(LW), y1 - ((ph * 40.0 + k as f32 * 53.0).rem_euclid(y1 - y0)));
            draw_circle(x, yy, 22.0 + 6.0 * (ph).sin(), Color::new(0.6, 0.25, 0.85, base * 0.45));
        }
    }

    /// カメラ位置(主人公が上のバーに隠れないよう、バーの下の領域の中央に置く)
    fn cam(&self) -> (f32, f32) {
        let g = &self.g;
        let mw = g.dungeon.w() as f32 * TS;
        let mh = g.dungeon.h() as f32 * TS;
        let vh = SH - OFFY;
        let cx = ((g.pos.0 as f32 + 0.5) * TS - LW / 2.0).clamp(0.0, mw - LW);
        let cy = ((g.pos.1 as f32 + 0.5) * TS - vh / 2.0).clamp(0.0, mh - vh);
        (cx, cy)
    }

    fn draw_map(&self) {
        let g = &self.g;
        let ui = &self.ui;
        let (cx, cy) = self.cam();
        draw_rectangle(0.0, 0.0, LW, SH, rgb(20, 16, 24));
        let sx = |x: usize| x as f32 * TS - cx;
        let sy = |y: usize| y as f32 * TS - cy + OFFY;
        let d = &g.dungeon;
        let ent = d.entrance();
        for y in 0..d.h() {
            for x in 0..d.w() {
                let (px, py) = (sx(x), sy(y));
                if px < -TS || py < OFFY - TS || px > LW || py > SH {
                    continue;
                }
                match d.tile(x, y) {
                    Tile::Rock => {
                        let locked = !g.diggable(x, y);
                        ui.art.draw_tinted("tile_rock", px, py, TS, TS, if locked { rgb(150, 150, 160) } else { WHITE });
                    }
                    Tile::Bedrock => ui.art.draw("tile_bedrock", px, py, TS, TS),
                    Tile::Barrier => ui.art.draw("tile_barrier", px, py, TS, TS),
                    Tile::Floor => {
                        if (x, y) == ent {
                            ui.art.draw("tile_stairs_up", px, py, TS, TS);
                        } else {
                            ui.art.draw("tile_floor", px, py, TS, TS);
                        }
                    }
                    Tile::Portal => {
                        ui.art.draw("tile_floor", px, py, TS, TS);
                        let s = 1.0 + 0.08 * pulse(ui.t);
                        ui.art.draw("tile_portal", px - TS * (s - 1.0) / 2.0, py - TS * (s - 1.0) / 2.0, TS * s, TS * s);
                    }
                    Tile::StairsDown => ui.art.draw("tile_stairs_down", px, py, TS, TS),
                }
            }
        }
        // 部屋
        for r in &d.rooms {
            let (px, py) = (sx(r.x), sy(r.y));
            ui.art.draw(&room_art(&r.kind, r.chest_open), px, py, r.w as f32 * TS, r.h as f32 * TS);
            if r.open4 {
                // 4方向から入れる印: 中央の辺に小さな三角
                let (w, h) = (r.w as f32 * TS, r.h as f32 * TS);
                let c = rgb(255, 214, 80);
                draw_triangle(vec2(px + w / 2.0 - 6.0, py + 1.0), vec2(px + w / 2.0 + 6.0, py + 1.0), vec2(px + w / 2.0, py + 8.0), c);
                draw_triangle(vec2(px + w / 2.0 - 6.0, py + h - 1.0), vec2(px + w / 2.0 + 6.0, py + h - 1.0), vec2(px + w / 2.0, py + h - 8.0), c);
                draw_triangle(vec2(px + 1.0, py + h / 2.0 - 6.0), vec2(px + 1.0, py + h / 2.0 + 6.0), vec2(px + 8.0, py + h / 2.0), c);
                draw_triangle(vec2(px + w - 1.0, py + h / 2.0 - 6.0), vec2(px + w - 1.0, py + h / 2.0 + 6.0), vec2(px + w - 8.0, py + h / 2.0), c);
            } else {
                tri(sx(r.front.0), sy(r.front.1), r.door, rgb(255, 214, 80));
            }
            for (k, m) in r.mons.iter().enumerate() {
                if let Some(m) = m {
                    let (tx, ty) = (r.x + k % r.w, r.y + k / r.w);
                    ui.art.draw(mon_sprite(m), sx(tx) + 2.0, sy(ty) - 4.0, TS - 4.0, TS - 4.0);
                    // 魔物がすみついている印(赤い!)
                    draw_bang(sx(tx) + TS - 7.0, sy(ty) + 7.0, 7.0, ui.t + k as f32, ui);
                }
            }
            if g.no_spawn_room(r) {
                // 階段が近くて魔物がすみつかない部屋
                let (w, h) = (r.w as f32 * TS, r.h as f32 * TS);
                draw_rectangle(px, py, w, h, rgba(60, 60, 70, 140));
                draw_line(px + 6.0, py + 6.0, px + w - 6.0, py + h - 6.0, 3.0, rgba(230, 230, 240, 200));
                draw_line(px + w - 6.0, py + 6.0, px + 6.0, py + h - 6.0, 3.0, rgba(230, 230, 240, 200));
            }
        }
        for r in &d.roamers {
            let bob = (ui.t * 4.0 + r.x as f32).sin() * 1.5;
            ui.art.draw(mon_sprite(&r.mon), sx(r.x) + 1.0, sy(r.y) - 3.0 + bob, TS - 2.0, TS - 2.0);
        }
        if let Some((fx, fy)) = g.front() {
            let p = pulse(ui.t);
            draw_rectangle_lines(sx(fx) + 1.0, sy(fy) + 1.0, TS - 2.0, TS - 2.0, 3.0, rgba(120, 220, 255, (130.0 + 120.0 * p) as u8));
        }
        ui.art.draw("hero_map", sx(g.pos.0) - 1.0, sy(g.pos.1) - 6.0, TS + 2.0, TS + 2.0);
        self.draw_haze(OFFY, SH);
    }

    fn draw_hud(&mut self, st: &DunSt) -> Option<&'static str> {
        let g = &self.g;
        let ui = &self.ui;
        // 上のバー: 所持金・日数・階・いまの目標
        draw_rectangle(0.0, 0.0, LW, OFFY, rgb(26, 22, 34));
        draw_line(0.0, OFFY, LW, OFFY, 3.0, rgb(150, 190, 210));
        chip(ui, 8.0, 6.0, 170.0, 30.0, &format!("{} G", comma(g.gold)), 19.0);
        chip(ui, 186.0, 6.0, 92.0, 30.0, &format!("{}日目", g.day), 17.0);
        chip(ui, 286.0, 6.0, 72.0, 30.0, &format!("B{}F", g.floor_no()), 17.0);
        if self.debug {
            let (x, y) = g.pos;
            ui.text(&format!("DEBUG ({},{}) 評価{} F1", x, y, g.rating()), 370.0, 12.0, 14.0, rgb(255, 150, 150));
        }
        let tog = Rect::new(LW - 134.0, 6.0, 126.0, 30.0);
        round_button(ui, tog, "Tab", if g.map3d { "2Dにする" } else { "3Dにする" }, true, self.inp.in_rect(tog));
        let mq = main_quest(g);
        let mut line = format!("▶ {}", mq.title);
        if !mq.progress.is_empty() {
            // まだ満たしていない条件だけ(帯に収まるように)
            let p: Vec<String> = mq.progress.iter().filter(|(_, a, b)| a < b).map(|(n, a, b)| format!("{} {}/{}", n, a, b)).collect();
            if !p.is_empty() {
                line += &format!("   {}", p.join("・"));
            }
        }
        let mut size = 16.0;
        while size > 12.0 && ui.measure(&line, size) > LW - 20.0 {
            size -= 1.0;
        }
        ui.text(&line, 10.0, 40.0, size, rgb(255, 230, 140));
        draw_rectangle(8.0, SH - 42.0, 250.0, 34.0, rgba(20, 30, 40, 225));
        draw_rectangle_lines(8.0, SH - 42.0, 250.0, 34.0, 2.0, rgb(150, 190, 210));
        ui.text(&format!("シャベルMP = {} / {}", g.mp, g.mp_max), 18.0, SH - 36.0, 20.0, if g.mp == 0 { rgb(255, 150, 130) } else { CREAM });
        let front_is_floor = g.front().map(|(x, y)| g.dungeon.tile(x, y) == Tile::Floor && g.dungeon.room_at(x, y).is_none() && (x, y) != g.dungeon.entrance()).unwrap_or(false);
        let front_room = g.front_room().is_some();
        let labels = [
            ("C", "メニュー"),
            ("V", "まほう"),
            ("Z", if front_is_floor { "うめる" } else { "あなを ほる" }),
            ("X", if front_room { "へやを こわす" } else { "へやを おく" }),
            ("U", "もどす"),
        ];
        let mut clicked: Option<&'static str> = None;
        let playing = matches!(st.sub, DSub::Play);
        let n = labels.len() as f32;
        for (i, (letter, label)) in labels.iter().enumerate() {
            let r = Rect::new(LW - 168.0, SH - n * 38.0 - 8.0 + i as f32 * 38.0, 160.0, 34.0);
            round_button(ui, r, letter, label, true, self.inp.in_rect(r));
            if playing && self.inp.clicked(r) {
                clicked = Some(["x", "y", "a", "b", "u"][i]);
            }
        }
        clicked
    }

    fn draw_bottom(&self) {
        let g = &self.g;
        let ui = &self.ui;
        draw_rectangle(0.0, SH, LW, SH, rgb(34, 30, 42));
        draw_rectangle_lines(0.0, SH, LW, SH, 4.0, rgb(90, 80, 110));
        for (k, &i) in g.members().iter().enumerate() {
            let m = &g.party[i];
            let y = SH + 10.0 + k as f32 * 138.0;
            gold_box(10.0, y, 200.0, 128.0);
            ui.text(&m.name, 24.0, y + 8.0, 22.0, CREAM);
            ui.text(m.status_text(), 24.0, y + 36.0, 17.0, if m.poison { rgb(214, 150, 255) } else { rgb(190, 235, 190) });
            ui.text(&format!("HP {:>3}/{:<3}", m.hp, m.hp_max), 24.0, y + 62.0, 19.0, rgb(150, 235, 120));
            bar(24.0, y + 86.0, 170.0, m.hp as f32 / m.hp_max as f32, rgb(110, 210, 90));
            ui.text(&format!("MP {:>3}/{:<3}", m.mp, m.mp_max), 24.0, y + 94.0, 19.0, rgb(214, 160, 255));
        }
        gold_box(222.0, SH + 10.0, 344.0, 48.0);
        ui.text(&format!("B{}F", g.floor_no()), 240.0, SH + 20.0, 26.0, CREAM);
        ui.text_right(&format!("ひょうか  {}", g.rating()), 552.0, SH + 22.0, 24.0, CREAM);
        let d = &g.dungeon;
        // 広い階でも 枠(幅344・高さ約350)に収まるように 1マスの大きさを縮める
        let cell = (19.0f32).min(336.0 / d.w() as f32);
        let ox = 222.0 + (344.0 - cell * d.w() as f32) / 2.0;
        let oy = SH + 68.0;
        draw_rectangle(ox - 6.0, oy - 6.0, cell * d.w() as f32 + 12.0, cell * d.h() as f32 + 12.0, rgb(40, 90, 160));
        for y in 0..d.h() {
            for x in 0..d.w() {
                let t = d.tile(x, y);
                let c = match t {
                    Tile::Rock => {
                        if g.diggable(x, y) {
                            rgb(110, 78, 46)
                        } else {
                            rgb(84, 62, 44)
                        }
                    }
                    Tile::Bedrock => rgb(40, 90, 160),
                    Tile::Barrier => rgb(120, 70, 50),
                    Tile::Floor => rgb(200, 190, 170),
                    Tile::Portal => rgb(170, 70, 230),
                    Tile::StairsDown => rgb(255, 214, 80),
                };
                draw_rectangle(ox + x as f32 * cell, oy + y as f32 * cell, cell - 1.0, cell - 1.0, c);
                if t == Tile::Barrier {
                    draw_circle(ox + x as f32 * cell + cell / 2.0, oy + y as f32 * cell + cell / 2.0, 2.2, rgb(235, 70, 60));
                }
            }
        }
        let ent = d.entrance();
        draw_rectangle(ox + ent.0 as f32 * cell, oy + ent.1 as f32 * cell, cell - 1.0, cell - 1.0, rgb(90, 200, 80));
        for r in &d.rooms {
            let c = match r.kind.as_str() {
                "エサおけ" => rgb(220, 150, 60),
                "ゴミすてば" => rgb(100, 150, 80),
                "わらのベッド" => rgb(220, 90, 90),
                "さかば" => rgb(230, 190, 70),
                "さいだん" => rgb(150, 150, 220),
                "はたけ" => rgb(130, 190, 90),
                "いろり" => rgb(230, 110, 50),
                "水のみば" => rgb(80, 150, 230),
                "りっぱなベッド" => rgb(190, 60, 110),
                "はなぞの" => rgb(240, 150, 200),
                "どうじょう" => rgb(170, 130, 80),
                _ => rgb(250, 220, 90),
            };
            for (k, &(tx, ty)) in r.tiles().iter().enumerate() {
                draw_rectangle(ox + tx as f32 * cell + 1.0, oy + ty as f32 * cell + 1.0, cell - 3.0, cell - 3.0, c);
                if r.mons[k].is_some() {
                    draw_circle(ox + tx as f32 * cell + cell / 2.0, oy + ty as f32 * cell + cell / 2.0, 4.0, rgb(230, 40, 40));
                }
            }
        }
        for r in &d.roamers {
            draw_circle(ox + r.x as f32 * cell + cell / 2.0, oy + r.y as f32 * cell + cell / 2.0, 3.5, rgb(255, 70, 70));
        }
        let hp = 0.5 + 0.5 * (ui.t * 8.0).sin();
        draw_circle(ox + g.pos.0 as f32 * cell + cell / 2.0, oy + g.pos.1 as f32 * cell + cell / 2.0, 5.0, Color::new(1.0, 1.0, 0.4 + 0.5 * hp, 1.0));
    }

    pub fn dungeon(&mut self, st: &mut DunSt) -> Option<Scr> {
        if self.g.map3d {
            self.draw_map3d();
        } else {
            self.draw_map();
        }
        self.draw_bottom();
        let btn = self.draw_hud(st);
        if matches!(st.sub, DSub::Play) && self.inp.clicked(Rect::new(LW - 134.0, 6.0, 126.0, 30.0)) {
            self.g.map3d = !self.g.map3d;
        }
        let mut next: Option<Scr> = None;
        let center = Rect::new(236.0, SH + 100.0, 328.0, 300.0);
        match &mut st.sub {
            DSub::Play => {
                let prev = self.g.pos;
                let mut dir = None;
                let m3 = self.g.map3d;
                // 3D: W(↑)=すすむ(長押しで 進みつづける)。A/S/D(←↓→)は ちょん押し=その向きを向く、長押し=その向きへ すすむ
                let keep_facing: Option<Dir> = None;
                if m3 {
                    let f = self.g.facing;
                    if self.inp.up {
                        dir = Some(f);
                    }
                    for i in 1..4 {
                        let target = match i {
                            1 => f.opposite(),
                            2 => crate::view3d::turn(f, false),
                            _ => crate::view3d::turn(f, true),
                        };
                        match self.inp.dk[i] {
                            1 => {
                                self.g.facing = target;
                                st.arm = None;
                            }
                            2 => {
                                // 長押しのはじまり: その向きを向いて 進む
                                st.hold_world[i] = Some(target);
                                self.g.facing = target;
                                dir = Some(target);
                            }
                            3 => {
                                let wd = st.hold_world[i].unwrap_or(target);
                                self.g.facing = wd;
                                dir = Some(wd);
                            }
                            _ => {}
                        }
                    }
                } else if self.inp.up {
                    dir = Some(Dir::Up);
                } else if self.inp.down {
                    dir = Some(Dir::Down);
                } else if self.inp.left {
                    dir = Some(Dir::Left);
                } else if self.inp.right {
                    dir = Some(Dir::Right);
                }
                // マップのとなりのマスをクリック: 向きを変える / もう一度で進む(2Dのみ)
                if !m3 && dir.is_none() && self.inp.click && self.inp.mouse.y > OFFY && self.inp.mouse.x < LW - 176.0 {
                    let (cx, cy) = self.cam();
                    let tx = ((self.inp.mouse.x + cx) / TS).floor() as i32;
                    let ty = ((self.inp.mouse.y - OFFY + cy) / TS).floor() as i32;
                    let (dx, dy) = (tx - self.g.pos.0 as i32, ty - self.g.pos.1 as i32);
                    dir = match (dx, dy) {
                        (0, -1) => Some(Dir::Up),
                        (0, 1) => Some(Dir::Down),
                        (-1, 0) => Some(Dir::Left),
                        (1, 0) => Some(Dir::Right),
                        _ => None,
                    };
                    if let Some(d) = dir {
                        if self.g.facing != d {
                            self.g.facing = d;
                            dir = None;
                        }
                    }
                }
                if let Some(d) = dir {
                    if !m3 && (self.inp.shift || self.g.facing != d) {
                        // 向いていない方向のキー: 1回押すと向きだけ変わる(Shift+矢印でも同じ)
                        // 向いている方向と同じキーをもう一度押すと進む
                        self.g.facing = d;
                        st.arm = None;
                    } else {
                        st.arm = None;
                        match self.g.step(d) {
                            StepResult::Blocked => {
                                if self.g.pos == self.g.dungeon.entrance() && d == Dir::Down {
                                    st.sub = DSub::ConfirmExit(ListMenu::new());
                                }
                            }
                            StepResult::Moved => self.ui.audio.sfx("step"),
                            StepResult::Battle(srcs) => {
                                self.ui.audio.sfx("encounter");
                                next = Some(Scr::Battle(Box::new(BattleSt::new(&self.g, &srcs, prev))));
                            }
                            StepResult::Chest(m) => {
                                self.ui.audio.sfx("chest");
                                self.say(m)
                            }
                            StepResult::Descend => {
                                self.ui.audio.sfx("stairs");
                                self.g.descend();
                                let f = self.g.floor_no();
                                self.say(format!("地下{}階へ おりた!", f));
                            }
                            StepResult::Stairs => {
                                let n = self.g.floor_no() + 1;
                                self.say(format!("地下{}階への階段だ。(この先はまだ工事中…つづく)", n));
                            }
                        }
                    }
                }
                if let Some(f) = keep_facing {
                    if matches!(next, None) {
                        self.g.facing = f;
                    }
                }
                if let Some((_, t)) = &mut st.arm {
                    *t -= get_frame_time();
                    if *t <= 0.0 {
                        st.arm = None;
                    }
                }
                let a = self.inp.a || btn == Some("a");
                let b = self.inp.b || btn == Some("b");
                if a {
                    // 何もない床に Z: まちがって埋めないよう、もう一度押したときだけ埋める
                    let front = self.g.front();
                    let front_empty = front.map(|(x, y)| self.g.dungeon.tile(x, y) == Tile::Floor && self.g.dungeon.room_at(x, y).is_none() && (x, y) != self.g.dungeon.entrance() && !self.g.dungeon.roamer_at(x, y)).unwrap_or(false);
                    if front_empty && st.arm.map(|(p, _)| Some(p) != front).unwrap_or(true) {
                        st.arm = front.map(|p| (p, 1.5));
                        self.say("もう一度 Zで うめる(まちがい防止)");
                    } else {
                        st.arm = None;
                        match self.g.dig() {
                            Ok(m) => {
                                self.ui.audio.sfx(if m.contains("埋め") { "fill" } else { "dig" });
                                self.say(m)
                            }
                            Err(e) => {
                                self.ui.audio.sfx("blocked");
                                self.say(e)
                            }
                        }
                    }
                } else if b {
                    if self.g.front_room().is_some() {
                        st.sub = DSub::Rooms(ListMenu::new(), vec![], true);
                    } else if self.g.can_place_here() {
                        let names: Vec<String> = ITEMS.iter().filter(|d| d.kind == Kind::Room && self.g.count(d.name) > 0).map(|d| d.name.to_string()).collect();
                        if names.is_empty() {
                            self.say("置ける部屋を持っていない。家具屋で買おう");
                        } else {
                            st.sub = DSub::Rooms(ListMenu::new(), names, false);
                        }
                    } else {
                        self.say("ここには置けない(壁か、床の上にだけ置ける)");
                    }
                } else if self.inp.x || btn == Some("x") {
                    st.sub = DSub::Menu(ListMenu::new());
                } else if self.inp.y || btn == Some("y") {
                    st.sub = DSub::Magic(ListMenu::new());
                } else if self.inp.u || btn == Some("u") {
                    match self.g.undo() {
                        Ok(m) => {
                            self.ui.audio.sfx("undo");
                            self.say(m)
                        }
                        Err(e) => {
                            self.ui.audio.sfx("blocked");
                            self.say(e)
                        }
                    }
                }
            }
            DSub::Menu(m) => {
                parchment(center.x - 8.0, center.y - 8.0, center.w + 16.0, center.h + 16.0);
                let rows = vec![
                    Row::new("ステータス・そうび・どうぐ"),
                    Row::new("クエストを みる"),
                    Row::new("ひとつ もどす").right(format!("{}", self.g.undo.len())).enabled(!self.g.undo.is_empty()),
                    Row::new("町へ かえる"),
                    Row::new("とじる"),
                ];
                match m.run(&self.ui, &self.inp, &rows, center, 52.0, 20.0, Style::Parchment, true) {
                    MenuEv::Confirm(0) => {
                        self.ov = Some(Overlay::menu());
                        st.sub = DSub::Play;
                    }
                    MenuEv::Confirm(1) => {
                        self.ov = Some(Overlay::Board(crate::board::BoardSt::new(), false));
                        st.sub = DSub::Play;
                    }
                    MenuEv::Confirm(2) => {
                        match self.g.undo() {
                            Ok(m) => {
                                self.ui.audio.sfx("undo");
                                self.say(m)
                            }
                            Err(e) => self.say(e),
                        }
                        st.sub = DSub::Play;
                    }
                    MenuEv::Confirm(3) => next = Some(self.leave_to_town()),
                    MenuEv::Confirm(_) | MenuEv::Cancel => st.sub = DSub::Play,
                    MenuEv::None => {}
                }
            }
            DSub::Rooms(m, names, removing) => {
                parchment(center.x - 8.0, center.y - 8.0, center.w + 16.0, center.h + 16.0);
                if *removing {
                    let kind = self.g.front_room().map(|i| self.g.dungeon.rooms[i].kind.clone()).unwrap_or_default();
                    let cost = if kind.is_empty() { ROOM_COST } else { room_def(&kind).cost };
                    let rows = vec![Row::new(format!("{} をこわす", kind)).right(format!("MP{}", cost)), Row::new("やめる")];
                    match m.run(&self.ui, &self.inp, &rows, center, 56.0, 22.0, Style::Parchment, true) {
                        MenuEv::Confirm(0) => {
                            match self.g.remove_room() {
                                Ok(m) => {
                                    self.ui.audio.sfx("break");
                                    self.say(m)
                                }
                                Err(e) => self.say(e),
                            }
                            st.sub = DSub::Play;
                        }
                        MenuEv::Confirm(_) | MenuEv::Cancel => st.sub = DSub::Play,
                        MenuEv::None => {}
                    }
                } else {
                    let rows: Vec<Row> = names
                        .iter()
                        .map(|n| {
                            let def = room_def(n);
                            let size = if def.w > 1 { "3x3 " } else { "" };
                            Row::new(format!("{}{}  x{}", size, n, self.g.count(n))).right(format!("MP{}", def.cost)).enabled(self.g.can_place(n))
                        })
                        .collect();
                    self.ui.text("部屋を えらぶ(灰色=置けない)", center.x, center.y - 34.0, 18.0, CREAM);
                    if let Some(n) = names.get(m.cursor.min(names.len().saturating_sub(1))) {
                        if let Some(why) = self.g.place_reason(n) {
                            self.ui.text(&format!("× {}", why), center.x, center.y + center.h + 6.0, 16.0, rgb(255, 190, 170));
                        }
                    }
                    match m.run(&self.ui, &self.inp, &rows, center, 50.0, 21.0, Style::Parchment, true) {
                        MenuEv::Confirm(i) => {
                            let n = names[i].clone();
                            match self.g.place_room(&n) {
                                Ok(m) => {
                                    self.ui.audio.sfx("place");
                                    self.say(m)
                                }
                                Err(e) => {
                                    self.ui.audio.sfx("blocked");
                                    self.say(e)
                                }
                            }
                            st.sub = DSub::Play;
                        }
                        MenuEv::Cancel => st.sub = DSub::Play,
                        _ => {}
                    }
                }
            }
            DSub::Magic(m) => {
                parchment(center.x - 8.0, center.y - 8.0, center.w + 16.0, center.h + 16.0);
                let knows = |id: &str| self.g.party[0].spells.iter().any(|s| s == id) && self.g.party[0].mp >= spell(id).mp;
                let rows = vec![
                    Row::new("リターン").right("MP0"),
                    Row::new("ワープ").right("MP0").enabled(self.g.reached >= 2),
                    Row::new("ひとりかいふく").right(format!("MP{}", spell("heal").mp)).enabled(knows("heal")),
                    Row::new("ぜんいんかいふく").right(format!("MP{}", spell("healall").mp)).enabled(knows("healall")),
                    Row::new("ひとりなおす").right(format!("MP{}", spell("cure").mp)).enabled(knows("cure")),
                ];
                match m.run(&self.ui, &self.inp, &rows, center, 50.0, 21.0, Style::Parchment, true) {
                    MenuEv::Confirm(0) => {
                        self.g.warp(1);
                        self.say("1階の入口へ もどった");
                        st.sub = DSub::Play;
                    }
                    MenuEv::Confirm(1) => st.sub = DSub::Warp(ListMenu::new()),
                    MenuEv::Confirm(2) => st.sub = DSub::HealWho(ListMenu::new()),
                    MenuEv::Confirm(3) => {
                        let s = spell("healall");
                        self.g.party[0].mp -= s.mp;
                        let amount = 20 + self.g.party[0].wis_total();
                        for p in self.g.party.iter_mut().filter(|p| p.alive()) {
                            p.hp = (p.hp + amount).min(p.hp_max);
                        }
                        self.ui.audio.sfx("heal");
                        self.say("みんなのHPが かいふくした!");
                        st.sub = DSub::Play;
                    }
                    MenuEv::Confirm(4) => st.sub = DSub::CureWho(ListMenu::new()),
                    MenuEv::Confirm(_) | MenuEv::Cancel => st.sub = DSub::Play,
                    MenuEv::None => {}
                }
            }
            DSub::Warp(m) => {
                parchment(center.x - 8.0, center.y - 8.0, center.w + 16.0, center.h + 16.0);
                self.ui.text("どの階の 入口へ とぶ?", center.x, center.y - 34.0, 20.0, INK);
                let rows: Vec<Row> = (1..=self.g.reached).map(|f| Row::new(format!("地下{}階", f)).right(if f == self.g.floor_no() { "いま居る階" } else { "" })).collect();
                match m.run(&self.ui, &self.inp, &rows, center, 52.0, 22.0, Style::Parchment, true) {
                    MenuEv::Confirm(i) => {
                        self.g.warp(i + 1);
                        self.say(format!("地下{}階の入口へ とんだ", i + 1));
                        st.sub = DSub::Play;
                    }
                    MenuEv::Cancel => st.sub = DSub::Play,
                    _ => {}
                }
            }
            DSub::CureWho(m) => {
                parchment(center.x - 8.0, center.y - 8.0, center.w + 16.0, center.h + 16.0);
                self.ui.text("だれを なおす?", center.x, center.y - 34.0, 20.0, INK);
                let mem = self.g.members();
                let rows: Vec<Row> = mem.iter().map(|&k| Row::new(self.g.party[k].name.clone()).right(self.g.party[k].status_text())).collect();
                match m.run(&self.ui, &self.inp, &rows, center, 56.0, 21.0, Style::Parchment, true) {
                    MenuEv::Confirm(i) => {
                        let i = mem[i];
                        let s = spell("cure");
                        if self.g.party[0].mp >= s.mp && self.g.party[i].alive() {
                            self.g.party[0].mp -= s.mp;
                            let p = &mut self.g.party[i];
                            p.poison = false;
                            p.stun = false;
                            self.ui.audio.sfx("heal");
                            self.say("げんきになった!");
                        }
                        st.sub = DSub::Play;
                    }
                    MenuEv::Cancel => st.sub = DSub::Play,
                    MenuEv::None => {}
                }
            }
            DSub::HealWho(m) => {
                parchment(center.x - 8.0, center.y - 8.0, center.w + 16.0, center.h + 16.0);
                self.ui.text("だれを なおす?", center.x, center.y - 34.0, 20.0, INK);
                let mem = self.g.members();
                let rows: Vec<Row> = mem.iter().map(|&k| Row::new(self.g.party[k].name.clone()).right(format!("HP{}/{}", self.g.party[k].hp, self.g.party[k].hp_max))).collect();
                match m.run(&self.ui, &self.inp, &rows, center, 56.0, 21.0, Style::Parchment, true) {
                    MenuEv::Confirm(i) => {
                        let i = mem[i];
                        let s = spell("heal");
                        if self.g.party[0].mp >= s.mp && self.g.party[i].alive() {
                            self.g.party[0].mp -= s.mp;
                            let amount = 15 + self.g.party[0].wis_total();
                            let p = &mut self.g.party[i];
                            p.hp = (p.hp + amount).min(p.hp_max);
                            self.say("HPが かいふくした!");
                        }
                        st.sub = DSub::Play;
                    }
                    MenuEv::Cancel => st.sub = DSub::Play,
                    MenuEv::None => {}
                }
            }
            DSub::ConfirmExit(m) => {
                parchment(center.x - 8.0, center.y - 8.0, center.w + 16.0, center.h + 16.0);
                let up = self.g.floor_no() > 1;
                self.ui.text(if up { "うえの階へ もどる?" } else { "ダンジョンを でて 町へ もどる?" }, center.x, center.y - 34.0, 20.0, INK);
                let rows = vec![Row::new("もどる"), Row::new("やめる")];
                match m.run(&self.ui, &self.inp, &rows, Rect::new(center.x, center.y, center.w, 130.0), 56.0, 24.0, Style::Parchment, true) {
                    MenuEv::Confirm(0) => next = self.exit_up(st),
                    MenuEv::Confirm(_) | MenuEv::Cancel => st.sub = DSub::Play,
                    MenuEv::None => {}
                }
            }
        }
        next
    }

    /// どこからでも町へ帰る(次に「ダンジョンへ」で階を選べる)
    pub fn leave_to_town(&mut self) -> Scr {
        self.g.return_to_entrance();
        Scr::Town(TownSt::new(Some("アレン「町へ もどってきた。いらないものを 売って、うちで ごはんに しよう。」".to_string())))
    }

    /// 入口から: 2階以降なら上の階へ、1階なら町へ
    fn exit_up(&mut self, st: &mut DunSt) -> Option<Scr> {
        if self.g.floor_no() > 1 {
            self.g.ascend();
            st.sub = DSub::Play;
            let f = self.g.floor_no();
            self.say(format!("地下{}階へ もどった", f));
            None
        } else {
            self.g.return_to_entrance();
            Some(Scr::Town(TownSt::new(Some("アレン「町へ もどってきた。いらないものを 売って、うちで ごはんに しよう。」".to_string()))))
        }
    }
}
