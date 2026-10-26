//! 描画の部品: フォント・パネル・リストメニュー・入力。論理解像度 576x864(DS二画面を縦に並べた形)。

use crate::art::Art;
use macroquad::prelude::*;

pub const LW: f32 = 576.0;
pub const LH: f32 = 864.0;
pub const SH: f32 = 432.0; // 1画面の高さ

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}
pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

pub const CREAM: Color = Color::new(0.97, 0.93, 0.80, 1.0);
pub const INK: Color = Color::new(0.20, 0.13, 0.08, 1.0);
pub const GOLDEN: Color = Color::new(0.93, 0.76, 0.25, 1.0);

// ---------------- 入力 ----------------

#[derive(Default)]
pub struct Input {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub ok: bool,
    pub cancel: bool,
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub u: bool,
    /// Shift を押している(矢印でその場で向きだけ変える)
    pub shift: bool,
    /// 決定キーを押しっぱなし(メッセージ早送り)
    pub fast: bool,
    hold_frames: u32,
    pub l: bool,
    pub r: bool,
    pub mouse: Vec2,
    pub click: bool,
    pub rclick: bool,
    pub moved: bool,
    last_mouse: Vec2,
    hold: [f32; 4],
    was_down: [bool; 4],
    long_press: [bool; 4],
    /// 方向キーごとの 動き: 0=なし 1=ちょん押し(はなした)2=長押しのはじまり 3=長押しのくりかえし(3Dのダンジョンで使う)
    pub dk: [u8; 4],
    /// 検証用の入力スクリプト(--keys): 1フレームに1トークン、間隔をあけて消費する
    pub script: std::collections::VecDeque<String>,
    pub script_wait: u32,
    pub shot: Option<String>,
    /// 消費したスクリプトのキー数(wait/mark/shot は数えない)
    pub used: u32,
    pub scripted: bool,
}

const REPEAT_DELAY: f32 = 0.22;
const REPEAT_RATE: f32 = 0.085;

impl Input {
    pub fn poll(&mut self, view: &View) {
        let dt = get_frame_time();
        let mut out = [false; 4];
        self.dk = [0; 4];
        let arrow = [[KeyCode::Up, KeyCode::W], [KeyCode::Down, KeyCode::S], [KeyCode::Left, KeyCode::A], [KeyCode::Right, KeyCode::D]];
        for i in 0..4 {
            let pressed = is_key_pressed(arrow[i][0]) || is_key_pressed(arrow[i][1]);
            let down = is_key_down(arrow[i][0]) || is_key_down(arrow[i][1]);
            if pressed {
                self.hold[i] = 0.0;
                out[i] = true;
                self.long_press[i] = false;
            } else if down {
                self.hold[i] += dt;
                if self.hold[i] > REPEAT_DELAY {
                    self.hold[i] -= REPEAT_RATE;
                    out[i] = true;
                    self.dk[i] = if self.long_press[i] { 3 } else { 2 };
                    self.long_press[i] = true;
                }
            } else {
                self.hold[i] = 0.0;
            }
            // はなした: 長押しにならなかったら ちょん押し
            if self.was_down[i] && !down && !self.long_press[i] {
                self.dk[i] = 1;
            }
            if !down {
                self.long_press[i] = false;
            }
            self.was_down[i] = down;
        }
        self.up = out[0];
        self.down = out[1];
        self.left = out[2];
        self.right = out[3];
        let p = |k: KeyCode| is_key_pressed(k);
        // 左手: Z/Space/Enter=決定・掘る  X=キャンセル・部屋  C/Esc=メニュー  V=まほう  U=もどす(矢印かWASDで移動)
        self.a = p(KeyCode::Z) || p(KeyCode::Space) || p(KeyCode::Enter);
        self.ok = self.a;
        self.b = p(KeyCode::X);
        self.x = p(KeyCode::C) || p(KeyCode::Escape);
        self.y = p(KeyCode::V);
        self.u = p(KeyCode::U);
        self.l = p(KeyCode::Q) || p(KeyCode::PageUp);
        self.r = p(KeyCode::E) || p(KeyCode::PageDown);
        self.cancel = p(KeyCode::Escape) || p(KeyCode::Backspace) || p(KeyCode::X);
        self.shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        self.fast = is_key_down(KeyCode::Z) || is_key_down(KeyCode::Space) || is_key_down(KeyCode::Enter);
        if self.hold_frames > 0 {
            self.hold_frames -= 1;
            self.fast = true;
        }
        let m = view.to_logical(mouse_position());
        self.moved = (m - self.last_mouse).length() > 0.5;
        self.last_mouse = m;
        self.mouse = m;
        self.click = is_mouse_button_pressed(MouseButton::Left);
        self.rclick = is_mouse_button_pressed(MouseButton::Right);
        if self.rclick {
            self.cancel = true;
        }
        self.run_script();
    }

    fn run_script(&mut self) {
        if self.script_wait > 0 {
            self.script_wait -= 1;
            return;
        }
        let Some(tok) = self.script.pop_front() else { return };
        self.script_wait = 3;
        if !(tok == "wait" || tok == "mark" || tok.starts_with("shot:")) {
            self.used += 1;
        }
        match tok.as_str() {
            "wait" => self.script_wait = 40,
            "mark" => self.script_wait = 0,
            "up" => self.up = true,
            "down" => {
                self.down = true;
                self.dk[1] = 1;
            }
            "left" => {
                self.left = true;
                self.dk[2] = 1;
            }
            "right" => {
                self.right = true;
                self.dk[3] = 1;
            }
            "ok" => self.ok = true,
            "cancel" => self.cancel = true,
            "a" | "dig" => {
                self.a = true;
                self.ok = true;
            }
            "b" | "room" => self.b = true,
            "x" | "menu" => self.x = true,
            "y" | "magic" => self.y = true,
            "u" | "undo" => self.u = true,
            "hold" => self.hold_frames = 400,
            "tup" => {
                self.up = true;
                self.shift = true;
            }
            "tdown" => {
                self.down = true;
                self.shift = true;
            }
            "tleft" => {
                self.left = true;
                self.shift = true;
            }
            "tright" => {
                self.right = true;
                self.shift = true;
            }
            t if t.starts_with("shot:") => {
                self.shot = Some(t[5..].to_string());
                self.script_wait = 6;
            }
            _ => {}
        }
    }

    pub fn in_rect(&self, r: Rect) -> bool {
        r.contains(self.mouse)
    }

    pub fn clicked(&self, r: Rect) -> bool {
        self.click && r.contains(self.mouse)
    }
}

// ---------------- 画面への写し ----------------

/// 描画先の解像度の倍率(文字や絵を 大きい画面でもくっきり見せる)
pub const RT_K: f32 = 2.0;

pub const TIP_DEFAULT: &str = "Z/Enter:けってい  X/Esc:もどる  C:メニュー  V:まほう  U:もどす  F11:ぜんがめん  F9:がめん  M:おと";
/// ダンジョン(3D): W=すすむ、A/S/D=ちょん押しで その向き・長押しで その向きへ すすむ
pub const TIP_3D: &str = "Z:ほる X:へや C:メニュー V:まほう U:もどす  WASD:ちょん押し=むきを かえる 長押し=すすむ  Tab:2D  F11  M:おと";
pub const TIP_2D: &str = "Z:ほる X:へや C:メニュー V:まほう U:もどす  矢印:むき→もう一度で すすむ  Shift+矢印:むきだけ  Tab:3D  F11  M:おと";

/// 画面の並べ方。0=オート(横長の窓ならPC) 1=スマホ縦 2=PC横
pub struct View {
    pub scale: f32,
    pub ox: f32,
    pub oy: f32,
    pub mode: u8,
    /// PC横(上画面を左、下画面を右に並べる)か
    pub pc: bool,
    /// 窓の中の、上画面・下画面の四角(pc のとき)
    pub rects: [Rect; 2],
    /// 下の帯(操作のヒントを出す)
    pub footer: Rect,
    /// 下の帯に出す 操作のヒント(場面で変える)
    pub tip: &'static str,
}

impl View {
    pub fn new() -> View {
        let mut v = View { scale: 1.0, ox: 0.0, oy: 0.0, mode: 0, pc: false, rects: [Rect::new(0.0, 0.0, 1.0, 1.0); 2], footer: Rect::new(0.0, 0.0, 0.0, 0.0), tip: TIP_DEFAULT };
        v.update();
        v
    }

    pub fn update(&mut self) {
        let (sw, sh) = (screen_width(), screen_height());
        self.pc = match self.mode {
            1 => false,
            2 => true,
            _ => sw >= sh * 1.15,
        };
        if !self.pc {
            self.scale = (sw / LW).min(sh / LH);
            self.ox = (sw - LW * self.scale) / 2.0;
            self.oy = (sh - LH * self.scale) / 2.0;
            self.rects = [Rect::new(self.ox, self.oy, LW * self.scale, SH * self.scale), Rect::new(self.ox, self.oy + SH * self.scale, LW * self.scale, SH * self.scale)];
            self.footer = Rect::new(0.0, 0.0, 0.0, 0.0);
            return;
        }
        // PC横: 左に上画面、右に下画面。下に 操作のヒントの帯
        let foot = (sh * 0.075).clamp(30.0, 90.0);
        let gap = (sw * 0.02).clamp(12.0, 60.0);
        let margin = gap;
        let avail_h = sh - foot;
        self.scale = ((sw - 2.0 * margin - gap) / (2.0 * LW)).min((avail_h - 2.0 * margin) / SH);
        let tw = LW * self.scale;
        let th = SH * self.scale;
        let x0 = (sw - (2.0 * tw + gap)) / 2.0;
        let y0 = (avail_h - th) / 2.0;
        self.ox = x0;
        self.oy = y0;
        self.rects = [Rect::new(x0, y0, tw, th), Rect::new(x0 + tw + gap, y0, tw, th)];
        self.footer = Rect::new(0.0, sh - foot, sw, foot);
    }

    /// 窓の座標 → 論理座標(576x864 の中)。画面の外なら 遠くの点
    pub fn to_logical(&self, p: (f32, f32)) -> Vec2 {
        if !self.pc {
            return vec2((p.0 - self.ox) / self.scale, (p.1 - self.oy) / self.scale);
        }
        for (i, r) in self.rects.iter().enumerate() {
            if r.contains(vec2(p.0, p.1)) {
                return vec2((p.0 - r.x) / self.scale, (p.1 - r.y) / self.scale + i as f32 * SH);
            }
        }
        vec2(-9999.0, -9999.0)
    }

    /// 描いた論理画面(rt)を 窓に写す
    pub fn draw(&self, rt: &Texture2D, ui: &Ui) {
        clear_background(rgb(16, 12, 10));
        let k = RT_K;
        if !self.pc {
            draw_texture_ex(rt, self.ox, self.oy, WHITE, DrawTextureParams { dest_size: Some(vec2(LW * self.scale, LH * self.scale)), flip_y: true, ..Default::default() });
            return;
        }
        // 背景: 暗い木目
        let (sw, sh) = (screen_width(), screen_height());
        draw_rectangle(0.0, 0.0, sw, sh, rgb(34, 24, 18));
        let step = (sh / 54.0).max(8.0);
        let mut y = 0.0;
        while y < sh {
            draw_rectangle(0.0, y, sw, 1.0, rgba(0, 0, 0, 40));
            y += step;
        }
        for (i, r) in self.rects.iter().enumerate() {
            let b = (self.scale * 4.0).max(4.0);
            draw_rectangle(r.x - b, r.y - b, r.w + 2.0 * b, r.h + 2.0 * b, rgb(92, 60, 30));
            draw_rectangle_lines(r.x - b, r.y - b, r.w + 2.0 * b, r.h + 2.0 * b, 3.0, rgb(52, 30, 14));
            // render target は 上下が逆(flip_y)。上画面の絵は テクスチャの後ろ半分にある
            let src_y = if i == 0 { SH * k } else { 0.0 };
            draw_texture_ex(
                rt,
                r.x,
                r.y,
                WHITE,
                DrawTextureParams { dest_size: Some(vec2(r.w, r.h)), source: Some(Rect::new(0.0, src_y, LW * k, SH * k)), flip_y: true, ..Default::default() },
            );
        }
        // 下の帯: 操作のヒント
        let f = self.footer;
        draw_rectangle(f.x, f.y, f.w, f.h, rgb(24, 16, 12));
        let size = (f.h * 0.4).max(14.0);
        let tip = self.tip;
        let w = ui.measure(tip, size);
        ui.text(tip, f.x + (f.w - w) / 2.0, f.y + (f.h - size) / 2.0, size, rgb(220, 200, 160));
    }
}

// ---------------- フォント・テキスト ----------------

pub struct Ui {
    fonts: [Option<Font>; 3],
    pub art: Art,
    pub t: f32,
    pub audio: crate::audio::Audio,
}

const CLASSES: [f32; 3] = [18.0, 24.0, 34.0];

/// ゲームに出てくる文字の元(台詞・データ・メニューの文字列)
const SRC_TEXT: &str = concat!(
    include_str!("script.rs"),
    include_str!("data.rs"),
    include_str!("sim.rs"),
    include_str!("shop.rs"),
    include_str!("home.rs"),
    include_str!("overlays.rs"),
    include_str!("town.rs"),
    include_str!("dungeon.rs"),
    include_str!("battle.rs"),
    include_str!("quest.rs"),
    include_str!("board.rs"),
    include_str!("main.rs"),
    include_str!("mimic.rs"),
    include_str!("talk.rs"),
    include_str!("stage.rs"),
    include_str!("audio.rs"),
    include_str!("art.rs"),
    include_str!("layout.rs"),
    include_str!("debug.rs"),
    include_str!("ui.rs"),
);

impl Ui {
    pub fn new(art: Art, bytes: Option<Vec<u8>>) -> Ui {
        let mk = || bytes.as_ref().and_then(|b| load_ttf_font_from_bytes(b).ok());
        let fonts = [mk(), mk(), mk()];
        // 文字の絵(アトラス)は 描いている途中で大きくなると 画面じゅうの文字が 黒くつぶれる。
        // ソースに出てくる文字を 起動時に先に 用意して、ゲーム中は 大きくならないようにする
        let mut chars: Vec<char> = SRC_TEXT.chars().filter(|c| !c.is_control() && !c.is_whitespace()).collect();
        chars.sort_unstable();
        chars.dedup();
        for (f, class) in fonts.iter().zip(CLASSES) {
            if let Some(f) = f {
                f.populate_font_cache(&chars, class as u16);
            }
        }
        Ui { fonts, art, t: 0.0, audio: crate::audio::Audio::silent() }
    }

    fn params(&self, size: f32, color: Color) -> (usize, TextParams<'_>) {
        let ci = if size <= 18.5 {
            0
        } else if size <= 24.5 {
            1
        } else {
            2
        };
        let class = CLASSES[ci] * RT_K;
        (
            ci,
            TextParams { font: self.fonts[ci].as_ref(), font_size: class as u16, font_scale: size / class, color, ..Default::default() },
        )
    }

    pub fn measure(&self, s: &str, size: f32) -> f32 {
        let (ci, p) = self.params(size, WHITE);
        measure_text(s, self.fonts[ci].as_ref(), p.font_size, p.font_scale).width
    }

    /// y は文字の上端
    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        let (_, p) = self.params(size, color);
        draw_text_ex(s, x, y + size * 0.82, p);
    }

    pub fn text_shadow(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        let (_, mut p) = self.params(size, Color::new(0.0, 0.0, 0.0, 0.7));
        draw_text_ex(s, x + 1.5, y + size * 0.82 + 1.5, p.clone());
        p.color = color;
        draw_text_ex(s, x, y + size * 0.82, p);
    }

    pub fn text_center(&self, s: &str, cx: f32, y: f32, size: f32, color: Color) {
        let w = self.measure(s, size);
        self.text(s, cx - w / 2.0, y, size, color);
    }

    pub fn text_right(&self, s: &str, rx: f32, y: f32, size: f32, color: Color) {
        let w = self.measure(s, size);
        self.text(s, rx - w, y, size, color);
    }

    /// 幅で折り返して描く。描いた行数を返す
    pub fn text_wrap(&self, s: &str, x: f32, y: f32, w: f32, size: f32, color: Color, line_h: f32) -> usize {
        let mut lines = 0;
        for para in s.split('\n') {
            let mut cur = String::new();
            for ch in para.chars() {
                let mut t = cur.clone();
                t.push(ch);
                // 行のはじめに 閉じかっこや句読点が来ないように、その1文字は はみ出しても 前の行に残す
                let hang = matches!(ch, '」' | '』' | '。' | '、' | ')' | '!' | '?' | '…');
                if self.measure(&t, size) > w && !cur.is_empty() && !hang {
                    self.text(&cur, x, y + lines as f32 * line_h, size, color);
                    lines += 1;
                    cur = ch.to_string();
                } else {
                    cur = t;
                }
            }
            self.text(&cur, x, y + lines as f32 * line_h, size, color);
            lines += 1;
        }
        lines
    }
}

// ---------------- パネル ----------------

pub fn panel_wood(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, rgb(122, 78, 44));
    let mut yy = y;
    let mut alt = false;
    while yy < y + h {
        let ph = 30.0_f32.min(y + h - yy);
        draw_rectangle(x, yy, w, ph, if alt { rgb(118, 74, 41) } else { rgb(130, 84, 48) });
        draw_rectangle(x, yy + ph - 2.0, w, 2.0, rgb(78, 48, 26));
        // 木目
        draw_line(x + 20.0 + (yy * 7.0) % 90.0, yy + 8.0, x + 90.0 + (yy * 7.0) % 90.0, yy + 8.0, 1.0, rgb(106, 66, 36));
        draw_line(x + 260.0 + (yy * 3.0) % 120.0, yy + 18.0, x + 340.0 + (yy * 3.0) % 120.0, yy + 18.0, 1.0, rgb(106, 66, 36));
        yy += 30.0;
        alt = !alt;
    }
    draw_rectangle_lines(x, y, w, h, 3.0, rgb(52, 30, 14));
}

pub fn panel_brick(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, rgb(58, 46, 30));
    let bh: f32 = 34.0;
    let bw: f32 = 78.0;
    let mut row = 0;
    let mut yy = y;
    while yy < y + h {
        let off = if row % 2 == 0 { 0.0 } else { bw / 2.0 };
        let mut xx = x - off;
        while xx < x + w {
            let tone = ((xx * 13.0 + yy * 7.0) as i32 % 5) as f32 * 2.0;
            let rx = xx.max(x);
            let rw = (xx + bw - 3.0).min(x + w) - rx;
            let rh = bh.min(y + h - yy) - 3.0;
            if rw > 0.0 && rh > 0.0 {
                draw_rectangle(rx, yy, rw, rh, rgb((86.0 + tone) as u8, (70.0 + tone) as u8, (46.0 + tone) as u8));
            }
            xx += bw;
        }
        yy += bh;
        row += 1;
    }
}

pub fn parchment(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, rgb(240, 226, 190));
    draw_rectangle_lines(x + 3.0, y + 3.0, w - 6.0, h - 6.0, 2.0, rgb(150, 100, 50));
    draw_rectangle_lines(x, y, w, h, 3.0, rgb(106, 66, 30));
    for (cx, cy) in [(x + 8.0, y + 8.0), (x + w - 8.0, y + 8.0), (x + 8.0, y + h - 8.0), (x + w - 8.0, y + h - 8.0)] {
        draw_circle(cx, cy, 4.0, rgb(186, 130, 50));
    }
}

pub fn msgbox(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, rgb(86, 56, 32));
    draw_rectangle(x + 4.0, y + 4.0, w - 8.0, h - 8.0, rgb(104, 68, 38));
    draw_rectangle_lines(x, y, w, h, 3.0, rgb(232, 217, 168));
    draw_rectangle_lines(x + 4.0, y + 4.0, w - 8.0, h - 8.0, 1.5, rgb(52, 30, 14));
}

pub fn gold_box(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, rgba(24, 18, 10, 235));
    draw_rectangle_lines(x, y, w, h, 3.0, rgb(232, 192, 64));
}

pub fn chip(ui: &Ui, x: f32, y: f32, w: f32, h: f32, label: &str, size: f32) {
    draw_rectangle(x, y, w, h, rgba(20, 30, 40, 220));
    draw_rectangle_lines(x, y, w, h, 2.0, rgb(150, 190, 210));
    ui.text_right(label, x + w - 8.0, y + (h - size) / 2.0, size, CREAM);
}

pub fn round_button(ui: &Ui, r: Rect, letter: &str, label: &str, enabled: bool, hover: bool) {
    let base = if enabled { rgb(54, 74, 112) } else { rgb(70, 70, 76) };
    let c = if hover && enabled { rgb(86, 112, 160) } else { base };
    draw_rectangle(r.x, r.y, r.w, r.h, c);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.5, rgb(190, 205, 235));
    draw_circle(r.x + 18.0, r.y + r.h / 2.0, 12.0, rgb(230, 224, 200));
    draw_circle_lines(r.x + 18.0, r.y + r.h / 2.0, 12.0, 2.0, rgb(60, 50, 40));
    ui.text_center(letter, r.x + 18.0, r.y + r.h / 2.0 - 9.0, 18.0, INK);
    ui.text(label, r.x + 38.0, r.y + (r.h - 20.0) / 2.0, 20.0, CREAM);
}

pub fn bar(x: f32, y: f32, w: f32, frac: f32, col: Color) {
    draw_rectangle(x, y, w, 5.0, rgb(40, 34, 28));
    draw_rectangle(x, y, w * frac.clamp(0.0, 1.0), 5.0, col);
}

// ---------------- リストメニュー ----------------

pub struct Row {
    pub label: String,
    pub right: String,
    pub enabled: bool,
    /// 赤い「!」(報告できる・新入荷 など)
    pub badge: bool,
}

impl Row {
    pub fn new(label: impl Into<String>) -> Row {
        Row { label: label.into(), right: String::new(), enabled: true, badge: false }
    }
    pub fn right(mut self, r: impl Into<String>) -> Row {
        self.right = r.into();
        self
    }
    pub fn badge(mut self, b: bool) -> Row {
        self.badge = b;
        self
    }
    pub fn enabled(mut self, e: bool) -> Row {
        self.enabled = e;
        self
    }
}

#[derive(PartialEq, Debug)]
pub enum MenuEv {
    None,
    Confirm(usize),
    Cancel,
}

#[derive(Default)]
pub struct ListMenu {
    pub cursor: usize,
    pub scroll: usize,
    init: bool,
    /// 左右キーを別の用途に使うとき true(一覧のページ送りをしない)
    pub no_page: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Style {
    Wood,
    Parchment,
    Dark,
}

impl ListMenu {
    pub fn new() -> ListMenu {
        ListMenu::default()
    }

    /// 入力を処理して描画する。disabled の行は選べるが確定しても Confirm にならない(None を返す)
    pub fn run(&mut self, ui: &Ui, inp: &Input, rows: &[Row], r: Rect, row_h: f32, size: f32, style: Style, active: bool) -> MenuEv {
        let n = rows.len();
        if !self.init {
            self.init = true;
            if let Some(i) = rows.iter().position(|r| r.enabled) {
                if self.cursor == 0 {
                    self.cursor = i;
                }
            }
        }
        let vis = ((r.h - 8.0) / row_h).floor().max(1.0) as usize;
        if self.cursor >= n {
            self.cursor = n.saturating_sub(1);
        }
        let mut ev = MenuEv::None;
        let before = self.cursor;
        if active && n > 0 {
            if inp.down {
                self.cursor = (self.cursor + 1) % n;
            }
            if inp.up {
                self.cursor = (self.cursor + n - 1) % n;
            }
            if inp.right && vis < n && !self.no_page {
                self.cursor = (self.cursor + vis).min(n - 1);
            }
            if inp.left && vis < n && !self.no_page {
                self.cursor = self.cursor.saturating_sub(vis);
            }
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        }
        if self.cursor >= self.scroll + vis {
            self.scroll = self.cursor + 1 - vis;
        }
        let (fg, fg_off, hl, sep) = match style {
            Style::Wood => (CREAM, rgb(170, 140, 110), rgb(255, 224, 120), rgb(96, 60, 32)),
            Style::Parchment => (INK, rgb(150, 130, 100), rgb(190, 40, 30), rgb(200, 170, 120)),
            Style::Dark => (CREAM, rgb(130, 120, 110), rgb(255, 224, 120), rgb(90, 76, 56)),
        };
        for k in 0..vis {
            let i = self.scroll + k;
            if i >= n {
                break;
            }
            let ry = r.y + 4.0 + k as f32 * row_h;
            let rr = Rect::new(r.x + 4.0, ry, r.w - 8.0, row_h);
            if active && inp.moved && rr.contains(inp.mouse) {
                self.cursor = i;
            }
            let selected = i == self.cursor;
            if selected {
                draw_rectangle_lines(rr.x + 2.0, rr.y + 2.0, rr.w - 4.0, rr.h - 4.0, 2.5, hl);
                // 葉っぱのカーソル
                draw_triangle(vec2(rr.x + 8.0, rr.y + row_h / 2.0 - 8.0), vec2(rr.x + 8.0, rr.y + row_h / 2.0 + 8.0), vec2(rr.x + 22.0, rr.y + row_h / 2.0), rgb(110, 200, 80));
            } else {
                draw_line(rr.x + 8.0, rr.y + row_h - 1.0, rr.x + rr.w - 8.0, rr.y + row_h - 1.0, 1.0, sep);
            }
            let col = if rows[i].enabled { fg } else { fg_off };
            ui.text(&rows[i].label, rr.x + 30.0, rr.y + (row_h - size) / 2.0, size, col);
            if rows[i].badge {
                let bx = rr.x + 30.0 + ui.measure(&rows[i].label, size) + 18.0;
                draw_bang(bx, rr.y + row_h / 2.0, 9.0, ui.t, ui);
            }
            if !rows[i].right.is_empty() {
                ui.text_right(&rows[i].right, rr.x + rr.w - 12.0, rr.y + (row_h - size) / 2.0, size, col);
            }
            if active && inp.clicked(rr) {
                self.cursor = i;
                if rows[i].enabled {
                    ev = MenuEv::Confirm(i);
                }
            }
        }
        if n > vis {
            let frac = self.scroll as f32 / (n - vis) as f32;
            let th = ((vis as f32 / n as f32) * (r.h - 8.0)).max(18.0);
            draw_rectangle(r.x + r.w - 6.0, r.y + 4.0 + frac * (r.h - 8.0 - th), 4.0, th, rgba(255, 255, 255, 90));
        }
        if active {
            if inp.ok && n > 0 && rows[self.cursor].enabled && ev == MenuEv::None {
                ev = MenuEv::Confirm(self.cursor);
            }
            if inp.cancel {
                ev = MenuEv::Cancel;
            }
            // 効果音(カーソルが動いたときだけ。マウスが乗っただけでも動けば鳴る)
            match ev {
                MenuEv::Confirm(_) => ui.audio.sfx("ok"),
                MenuEv::Cancel => ui.audio.sfx("cancel"),
                MenuEv::None => {
                    if self.cursor != before {
                        ui.audio.sfx("move");
                    }
                }
            }
        }
        ev
    }
}

/// 赤い丸に白い「!」(誘導マーク)
pub fn draw_bang(cx: f32, cy: f32, r: f32, t: f32, ui: &Ui) {
    let k = 1.0 + 0.12 * (t * 6.0).sin();
    draw_circle(cx, cy, r * k + 1.5, rgb(255, 255, 255));
    draw_circle(cx, cy, r * k, rgb(214, 36, 36));
    ui.text_center("!", cx, cy - r * 0.95, r * 1.9, rgb(255, 255, 255));
}

pub fn pulse(t: f32) -> f32 {
    0.5 + 0.5 * (t * 5.0).sin()
}
