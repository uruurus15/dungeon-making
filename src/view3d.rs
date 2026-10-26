//! ダンジョンの 3D 表示(世界樹の迷宮のような、マス目の一人称視点)。
//! カメラは なめらかに動く(進む・向きを変えるときも 動いている間を描く)。
//! 世界の座標は マスの中心が整数。1マスは 幅1・高さ1(床 +0.5、天井 -0.5)。
//! 部屋は「小部屋」として描く: とびらの奥に 床・壁・天井があり、中に 施設の絵が立ち、魔物が立っている。
//! 掘る・部屋を置く・埋める は 2D と同じ(向いている先のマスが対象)。

use crate::art::Art;
use crate::sim::*;
use crate::ui::*;
use macroquad::prelude::*;

pub(crate) const NEAR: f32 = 0.12;
const FOCAL: f32 = 230.0;
/// 画面の枠(上のバーの下から 下画面の手前まで)
const VIEW_TOP: f32 = 62.0;
const VIEW_H: f32 = 370.0;
/// カメラは 自分のマスの中心から これだけ うしろ
const EYE_BACK: f32 = 0.85;
/// 描くマスの範囲(自分から 何マスまで)
const RADIUS: i32 = 9;

#[derive(Clone, Copy)]
pub(crate) struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Cam {
    pub ex: f32,
    pub ey: f32,
    /// 前・右の向き(世界の平面)
    pub fx: f32,
    pub fy: f32,
    pub rx: f32,
    pub ry: f32,
}

impl Cam {
    pub fn new(px: f32, py: f32, yaw: f32) -> Cam {
        let (fx, fy) = (yaw.sin(), -yaw.cos());
        let (rx, ry) = (yaw.cos(), yaw.sin());
        Cam { ex: px - fx * EYE_BACK, ey: py - fy * EYE_BACK, fx, fy, rx, ry }
    }
    pub fn to_cam(&self, wx: f32, wy: f32, h: f32) -> V3 {
        let (dx, dy) = (wx - self.ex, wy - self.ey);
        V3 { x: dx * self.rx + dy * self.ry, y: h, z: dx * self.fx + dy * self.fy }
    }
}

thread_local! {
    /// 画面への投影: (中心x, 中心y, 焦点距離)。モデル一覧が 小さい枠に描くときに 変える
    pub static VP: std::cell::Cell<(f32, f32, f32)> = const { std::cell::Cell::new((LW / 2.0, VIEW_TOP + VIEW_H / 2.0, FOCAL)) };
}

pub(crate) fn project(v: V3) -> Vec2 {
    let (cx, cy, f) = VP.with(|c| c.get());
    vec2(cx + f * v.x / v.z, cy + f * v.y / v.z)
}

/// 奥ほど 暗く
pub(crate) fn shade(z: f32, tint: f32) -> Color {
    let b = ((1.12 - 0.12 * z) * tint).clamp(0.16, 1.0);
    Color::new(b, b, b, 1.0)
}

#[derive(Clone, Copy)]
pub(crate) struct CV {
    pub p: V3,
    pub u: f32,
    pub v: f32,
    /// 頂点ごとの色(あれば。モデルの なめらかな陰影・色ムラに使う)
    pub col: Option<Color>,
}

pub(crate) fn lerp3(a: V3, b: V3, t: f32) -> V3 {
    V3 { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t, z: a.z + (b.z - a.z) * t }
}

/// 近い面で 切りとって(手前に出たぶん)、三角形にして 描く
pub(crate) fn emit(tex: Option<&Texture2D>, poly: &[CV], tint: f32, flat: Option<Color>) {
    let mut out: Vec<CV> = Vec::with_capacity(6);
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let (ain, bin) = (a.p.z >= NEAR, b.p.z >= NEAR);
        if ain {
            out.push(a);
        }
        if ain != bin {
            let t = (NEAR - a.p.z) / (b.p.z - a.p.z);
            let col = match (a.col, b.col) {
                (Some(x), Some(y)) => Some(Color::new(x.r + (y.r - x.r) * t, x.g + (y.g - x.g) * t, x.b + (y.b - x.b) * t, 1.0)),
                _ => None,
            };
            out.push(CV { p: lerp3(a.p, b.p, t), u: a.u + (b.u - a.u) * t, v: a.v + (b.v - a.v) * t, col });
        }
    }
    if out.len() < 3 {
        return;
    }
    let verts: Vec<Vertex> = out
        .iter()
        .map(|c| {
            let s = project(V3 { z: c.p.z.max(NEAR), ..c.p });
            let b = shade(c.p.z, tint);
            let col = match (c.col, flat) {
                (Some(f), _) | (None, Some(f)) => Color::new(f.r * b.r, f.g * b.g, f.b * b.b, 1.0),
                (None, None) => b,
            };
            Vertex::new(s.x, s.y, 0.0, c.u, c.v, col)
        })
        .collect();
    let mut idx: Vec<u16> = vec![];
    for i in 1..(verts.len() as u16 - 1) {
        idx.extend([0, i, i + 1]);
    }
    draw_mesh(&Mesh { vertices: verts, indices: idx, texture: tex.cloned() });
}

/// 平らな四角(角 c0..c3、絵は uv=(u0,v0,u1,v1))を n×n に分けて描く(近いほど 細かく)
fn plane(tex: Option<&Texture2D>, c: [V3; 4], uv: (f32, f32, f32, f32), tint: f32, flat: Option<Color>) {
    if c.iter().all(|p| p.z < NEAR) {
        return;
    }
    let mid = lerp3(lerp3(c[0], c[2], 0.5), lerp3(c[1], c[3], 0.5), 0.5);
    let n = if mid.z < 1.8 {
        6
    } else if mid.z < 4.2 {
        3
    } else {
        1
    };
    let at = |s: f32, t: f32| -> V3 { lerp3(lerp3(c[0], c[1], s), lerp3(c[3], c[2], s), t) };
    for j in 0..n {
        for i in 0..n {
            let (s0, s1, t0, t1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32, j as f32 / n as f32, (j + 1) as f32 / n as f32);
            let mk = |s: f32, t: f32| CV { p: at(s, t), u: uv.0 + (uv.2 - uv.0) * s, v: uv.1 + (uv.3 - uv.1) * t, col: None };
            emit(tex, &[mk(s0, t0), mk(s1, t0), mk(s1, t1), mk(s0, t1)], tint, flat);
        }
    }
}

/// 立っている絵(魔物・施設)。p=カメラ座標での足もと、size=大きさ(マス)、lift=床からの高さ
fn billboard(art: &Art, name: &str, p: V3, size: f32, lift: f32, tint: f32) {
    let Some(tex) = art.get(name) else { return };
    if p.z < 0.25 {
        return;
    }
    let c = project(V3 { x: p.x, y: 0.5, z: p.z });
    let px = FOCAL * size / p.z;
    let b = shade(p.z, tint);
    draw_texture_ex(tex, c.x - px / 2.0, c.y - px - FOCAL * lift / p.z, b, DrawTextureParams { dest_size: Some(vec2(px, px)), ..Default::default() });
}

pub fn dir_name(f: Dir) -> &'static str {
    match f {
        Dir::Up => "北",
        Dir::Right => "東",
        Dir::Down => "南",
        Dir::Left => "西",
    }
}

/// 左に曲がる/右に曲がる(上から見て)
pub fn turn(f: Dir, right: bool) -> Dir {
    match (f, right) {
        (Dir::Up, true) | (Dir::Down, false) => Dir::Right,
        (Dir::Right, true) | (Dir::Left, false) => Dir::Down,
        (Dir::Down, true) | (Dir::Up, false) => Dir::Left,
        (Dir::Left, true) | (Dir::Right, false) => Dir::Up,
    }
}

fn yaw_of(f: Dir) -> f32 {
    match f {
        Dir::Up => 0.0,
        Dir::Right => std::f32::consts::FRAC_PI_2,
        Dir::Down => std::f32::consts::PI,
        Dir::Left => -std::f32::consts::FRAC_PI_2,
    }
}

/// マスの種類
#[derive(Clone, Copy, PartialEq)]
enum C {
    Wall(Tile),
    /// 通路・4方向の部屋の中(歩ける)
    Open,
    /// 1方向の部屋の中(とびらから 見える小部屋。歩けない): (部屋の番号, マスの番号)
    Chamber(usize, usize),
}

fn cell(d: &Dungeon, x: i32, y: i32) -> C {
    if x < 0 || y < 0 || x as usize >= d.w() || y as usize >= d.h() {
        return C::Wall(Tile::Bedrock);
    }
    let (ux, uy) = (x as usize, y as usize);
    if let Some(i) = d.room_at(ux, uy) {
        let r = &d.rooms[i];
        if !r.open4 {
            return C::Chamber(i, r.slot_at(ux, uy).unwrap_or(0));
        }
    }
    if d.walkable(ux, uy) {
        C::Open
    } else {
        C::Wall(d.tile(ux, uy))
    }
}

/// となりあう2マスが 行き来できる(間に壁が無い)か
fn passable(d: &Dungeon, a: (i32, i32), b: (i32, i32)) -> bool {
    match (cell(d, a.0, a.1), cell(d, b.0, b.1)) {
        (C::Wall(_), _) | (_, C::Wall(_)) => false,
        (C::Open, C::Open) => true,
        (C::Chamber(i, _), C::Chamber(j, _)) => i == j,
        (C::Chamber(i, _), C::Open) => door_pair(d, i, a, b),
        (C::Open, C::Chamber(i, _)) => door_pair(d, i, b, a),
    }
}

/// 部屋 i の とびら(入口側の中央のマス ↔ 入口の前の通路)か。room_cell=部屋のマス、out_cell=通路のマス
fn door_pair(d: &Dungeon, i: usize, room_cell: (i32, i32), out_cell: (i32, i32)) -> bool {
    let r = &d.rooms[i];
    (r.front.0 as i32, r.front.1 as i32) == room_cell && r.stand().map(|s| (s.0 as i32, s.1 as i32)) == Some(out_cell)
}

fn room_art(r: &Room) -> String {
    format!("room_{}{}", r.kind, if r.chest_open { "_open" } else { "" })
}

impl crate::App {
    /// 絵から作った 3Dモデル(作ったら ためておく)。lod=細かさ(0=細かい 1=あらい)
    pub fn model(&self, art_name: &str, lod: u8) -> Option<std::rc::Rc<crate::model3d::Model>> {
        let key = (art_name.to_string(), lod);
        if let Some(m) = self.models.borrow().get(&key) {
            return Some(m.clone());
        }
        if let Some(m) = crate::modeling::hand_model(art_name) {
            let m = std::rc::Rc::new(m);
            self.models.borrow_mut().insert(key, m.clone());
            return Some(m);
        }
        let tex = self.ui.art.get(art_name)?;
        let img = tex.get_texture_data();
        let (n, thick) = if art_name.starts_with("room_") { (if lod == 0 { 30 } else { 16 }, 0.11) } else { (if lod == 0 { 28 } else { 16 }, 0.13) };
        let m = std::rc::Rc::new(crate::model3d::build(&img, n, thick));
        self.models.borrow_mut().insert(key, m.clone());
        Some(m)
    }

    /// カメラの位置・向きを 目標へ なめらかに近づける(戻り値: x, y, 向き)
    fn cam3_step(&self) -> (f32, f32, f32) {
        let g = &self.g;
        let (tx, ty, tyaw) = (g.pos.0 as f32, g.pos.1 as f32, yaw_of(g.facing));
        let [mut x, mut y, mut yaw, floor, init] = self.cam3.get();
        let dt = get_frame_time().min(0.1);
        if init == 0.0 || floor != g.floor_no() as f32 || (x - tx).abs() + (y - ty).abs() > 2.2 {
            (x, y, yaw) = (tx, ty, tyaw);
        } else {
            let k = 1.0 - (-dt * 17.0).exp();
            x += (tx - x) * k;
            y += (ty - y) * k;
            let mut dy = tyaw - yaw;
            while dy > std::f32::consts::PI {
                dy -= 2.0 * std::f32::consts::PI;
            }
            while dy < -std::f32::consts::PI {
                dy += 2.0 * std::f32::consts::PI;
            }
            yaw += dy * (1.0 - (-dt * 15.0).exp());
            // 目標に ほぼ着いたら ぴたっと止める
            if (tx - x).abs() + (ty - y).abs() < 0.004 {
                x = tx;
                y = ty;
            }
            if dy.abs() < 0.004 {
                yaw = tyaw;
            }
        }
        self.cam3.set([x, y, yaw, g.floor_no() as f32, 1.0]);
        (x, y, yaw)
    }

    pub fn draw_map3d(&self) {
        let g = &self.g;
        let ui = &self.ui;
        let d = &g.dungeon;
        let art = &ui.art;
        draw_rectangle(0.0, 0.0, LW, crate::ui::SH, rgb(14, 11, 18));
        let (px, py, yaw) = self.cam3_step();
        let cam = Cam::new(px, py, yaw);
        let ent = d.entrance();
        let rock = art.get("tile_rock");
        let ceil = art.get("tile_bedrock");

        // 描くマスを 遠い順に(近いものを あとから重ねる)
        let (cx0, cy0) = (g.pos.0 as i32, g.pos.1 as i32);
        let mut cells: Vec<((i32, i32), f32)> = vec![];
        for y in (cy0 - RADIUS)..=(cy0 + RADIUS) {
            for x in (cx0 - RADIUS)..=(cx0 + RADIUS) {
                if matches!(cell(d, x, y), C::Wall(_)) {
                    continue;
                }
                let v = cam.to_cam(x as f32, y as f32, 0.0);
                // 後ろ・視野の外は 描かない
                if v.z < -1.6 || v.x.abs() > 1.35 * v.z.max(0.0) + 1.9 {
                    continue;
                }
                let dist = (x as f32 - cam.ex).powi(2) + (y as f32 - cam.ey).powi(2);
                cells.push(((x, y), dist));
            }
        }
        cells.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 3x3 の 手組みモデルは、その部屋の いちばん 手前の マスを 描くときに 1回だけ(ゆか・天井に 塗りつぶされないように)
        let mut last_cell: std::collections::BTreeMap<usize, (i32, i32)> = Default::default();
        for &((x, y), _) in &cells {
            if let Some(i) = d.room_at(x as usize, y as usize) {
                last_cell.insert(i, (x, y));
            }
        }
        for &((x, y), _) in &cells {
            let (fx, fy) = (x as f32, y as f32);
            let me = cell(d, x, y);
            let (x0, x1, y0, y1) = (fx - 0.5, fx + 0.5, fy - 0.5, fy + 0.5);
            let c3 = |wx: f32, wy: f32, h: f32| cam.to_cam(wx, wy, h);
            // ---- 床 ----
            let mut floor_tex: Option<&Texture2D> = art.get("tile_floor");
            let mut uv = (0.0, 0.0, 1.0, 1.0);
            let mut room_idx: Option<(usize, usize)> = None;
            let mut skip_floor = false;
            match me {
                C::Open => {
                    if let Some(i) = d.room_at(x as usize, y as usize) {
                        room_idx = d.rooms[i].slot_at(x as usize, y as usize).map(|s| (i, s));
                    }
                    // 階段・魔穴は 3Dモデル(ゆかの 絵は 使わない)
                    if matches!(d.tile(x as usize, y as usize), Tile::StairsDown | Tile::Portal) {
                        skip_floor = true;
                    }
                }
                C::Chamber(i, s) => room_idx = Some((i, s)),
                C::Wall(_) => {}
            }
            if let Some((i, s)) = room_idx {
                let r = &d.rooms[i];
                // 3x3 などの 大きい部屋: 絵を 床いちめんに 貼る(マスごとの部分)
                if r.w * r.h > 1 && !crate::modeling::has_hand(&room_art(r)) {
                    let (sx, sy) = ((s % r.w) as f32, (s / r.w) as f32);
                    let (rw, rh) = (r.w as f32, r.h as f32);
                    uv = (sx / rw, sy / rh, (sx + 1.0) / rw, (sy + 1.0) / rh);
                    if let Some(t) = art.get(&room_art(r)) {
                        // 絵の下地に 床
                        plane(art.get("tile_floor"), [c3(x0, y0, 0.5), c3(x1, y0, 0.5), c3(x1, y1, 0.5), c3(x0, y1, 0.5)], (0.0, 0.0, 1.0, 1.0), 0.85, None);
                        floor_tex = Some(t);
                    }
                }
            }
            if !skip_floor {
                plane(floor_tex, [c3(x0, y0, 0.5), c3(x1, y0, 0.5), c3(x1, y1, 0.5), c3(x0, y1, 0.5)], uv, 1.0, None);
            }
            // ---- 天井 ----
            plane(ceil, [c3(x0, y0, -0.5), c3(x1, y0, -0.5), c3(x1, y1, -0.5), c3(x0, y1, -0.5)], (0.0, 0.0, 1.0, 1.0), 0.55, None);
            // ---- 壁(となりと行き来できない面。カメラの側から見える面だけ)----
            // (となりの向き, 面の両はし, 面の内向きの法線)
            let faces: [((i32, i32), (f32, f32), (f32, f32), (f32, f32)); 4] = [
                ((0, -1), (x0, y0), (x1, y0), (0.0, 1.0)),
                ((0, 1), (x1, y1), (x0, y1), (0.0, -1.0)),
                ((-1, 0), (x0, y1), (x0, y0), (1.0, 0.0)),
                ((1, 0), (x1, y0), (x1, y1), (-1.0, 0.0)),
            ];
            for (n, a, b, normal) in faces {
                let nb = (x + n.0, y + n.1);
                let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
                let facing_cam = (cam.ex - mid.0) * normal.0 + (cam.ey - mid.1) * normal.1 > 0.0;
                if passable(d, (x, y), nb) {
                    // とびら(部屋の側から 1回だけ 木の枠を描く)
                    if let C::Chamber(i, _) = me {
                        if door_pair(d, i, (x, y), nb) {
                            let dir = (b.0 - a.0, b.1 - a.1);
                            let at = |s: f32, h: f32| c3(a.0 + dir.0 * s, a.1 + dir.1 * s, h);
                            let wood = rgb(92, 58, 30);
                            plane(None, [at(0.0, -0.5), at(0.1, -0.5), at(0.1, 0.5), at(0.0, 0.5)], (0.0, 0.0, 1.0, 1.0), 1.0, Some(wood));
                            plane(None, [at(0.9, -0.5), at(1.0, -0.5), at(1.0, 0.5), at(0.9, 0.5)], (0.0, 0.0, 1.0, 1.0), 1.0, Some(wood));
                            plane(None, [at(0.0, -0.5), at(1.0, -0.5), at(1.0, -0.34), at(0.0, -0.34)], (0.0, 0.0, 1.0, 1.0), 1.0, Some(wood));
                        }
                    }
                    continue;
                }
                if !facing_cam {
                    continue;
                }
                if let C::Chamber(..) = me {
                    // 小部屋の壁: ふつうの岩でなく、しっくいの壁と 木の巾木(部屋らしく見せる)
                    let at = |s: f32, h: f32| c3(a.0 + (b.0 - a.0) * s, a.1 + (b.1 - a.1) * s, h);
                    plane(None, [at(0.0, -0.5), at(1.0, -0.5), at(1.0, 0.5), at(0.0, 0.5)], (0.0, 0.0, 1.0, 1.0), 1.0, Some(rgb(186, 160, 120)));
                    plane(None, [at(0.0, 0.34), at(1.0, 0.34), at(1.0, 0.5), at(0.0, 0.5)], (0.0, 0.0, 1.0, 1.0), 1.0, Some(rgb(96, 62, 34)));
                    plane(None, [at(0.0, -0.5), at(1.0, -0.5), at(1.0, -0.4), at(0.0, -0.4)], (0.0, 0.0, 1.0, 1.0), 1.0, Some(rgb(96, 62, 34)));
                    continue;
                }
                let (name, tint) = match cell(d, nb.0, nb.1) {
                    C::Wall(Tile::Bedrock) => ("tile_bedrock", 1.0),
                    C::Wall(Tile::Barrier) => ("tile_barrier", 1.0),
                    C::Wall(_) => ("tile_rock", if g.diggable(nb.0.max(0) as usize, nb.1.max(0) as usize) { 1.0 } else { 0.72 }),
                    _ => ("tile_rock", 1.0),
                };
                let tex = art.get(name).or(rock);
                plane(tex, [c3(a.0, a.1, -0.5), c3(b.0, b.1, -0.5), c3(b.0, b.1, 0.5), c3(a.0, a.1, 0.5)], (0.0, 0.0, 1.0, 1.0), tint, None);
            }
            // ---- 置いてある物・魔物 ----
            let center = c3(fx, fy, 0.0);
            // 階段・魔穴の 3Dモデル(出口は 北へ むかって 光が さす)
            let terrain = match d.tile(x as usize, y as usize) {
                Tile::Portal => Some("tile_portal"),
                Tile::StairsDown => Some("tile_stairs_down"),
                Tile::Floor if (x as usize, y as usize) == ent => Some("tile_stairs_up"),
                _ => None,
            };
            // 自分が 乗っている 階段・魔穴の 上では 描かない(目の前を ふさぐため)
            if let (Some(name), false) = (terrain, (x, y) == (g.pos.0 as i32, g.pos.1 as i32)) {
                if let Some(m) = self.model(name, 0) {
                    crate::model3d::draw_model(&m, &cam, fx, fy, (0.0, -1.0), 1.0, 0.0, 1.0);
                }
            }
            let lod = if center.z > 3.6 { 1u8 } else { 0u8 };
            let to_cam_front = (-cam.fx, -cam.fy);
            if let Some((i, s)) = room_idx {
                let r = &d.rooms[i];
                let small = r.w * r.h == 1;
                if !small && crate::modeling::has_hand(&room_art(r)) && last_cell.get(&i) == Some(&(x, y)) {
                    // 3x3 の 部屋: 手組みの 全体を 部屋の まんなかに 1回だけ 置く(いちばん 奥の マスを 描くとき)
                    if let Some(m) = self.model(&room_art(r), 0) {
                        let (cxr, cyr) = (r.x as f32 + (r.w as f32 - 1.0) / 2.0, r.y as f32 + (r.h as f32 - 1.0) / 2.0);
                        let dfr = r.door.d();
                        crate::model3d::draw_model(&m, &cam, cxr, cyr, (dfr.0 as f32, dfr.1 as f32), 1.0, 0.0, 1.0);
                    }
                }
                // 小さい部屋: とびらの向きを 前にして、施設の模型を 奥に、魔物を 手前に置く
                let df = r.door.d();
                let front = (df.0 as f32, df.1 as f32);
                let right = (front.1, -front.0);
                if small {
                    if let Some(m) = self.model(&room_art(r), lod) {
                        crate::model3d::draw_model(&m, &cam, fx - front.0 * 0.12, fy - front.1 * 0.12, front, 0.78, 0.0, 1.0);
                    }
                }
                if let Some(Some(mn)) = r.mons.get(s) {
                    if let Some(m) = self.model(crate::data::mon(mn).sprite, lod) {
                        if small {
                            crate::model3d::draw_model(&m, &cam, fx + front.0 * 0.22 + right.0 * 0.26, fy + front.1 * 0.22 + right.1 * 0.26, front, 0.5, 0.0, 1.0);
                        } else {
                            crate::model3d::draw_model(&m, &cam, fx, fy, to_cam_front, 0.7, 0.0, 1.0);
                        }
                    }
                }
            }
            if let Some(r) = d.roamers.iter().find(|r| (r.x as i32, r.y as i32) == (x, y)) {
                let bob = 0.02 * (ui.t * 4.0 + r.x as f32).sin();
                if let Some(m) = self.model(crate::data::mon(&r.mon).sprite, lod) {
                    crate::model3d::draw_model(&m, &cam, fx, fy, to_cam_front, 0.8, bob, 1.0);
                }
            }
        }

        self.draw_haze(VIEW_TOP, VIEW_TOP + VIEW_H + 6.0);
        // 向いている先の印(掘る・置く・埋めるの対象)
        if let Some((fx, fy)) = g.front() {
            let a = (120.0 + 130.0 * pulse(ui.t)) as u8;
            let col = rgba(120, 220, 255, a);
            let (fxf, fyf) = (fx as f32, fy as f32);
            let c3 = |wx: f32, wy: f32, h: f32| cam.to_cam(wx, wy, h);
            let pts: [V3; 4] = if matches!(cell(d, fx as i32, fy as i32), C::Wall(_)) {
                // 壁: 自分のマスと 向こうのマスの さかいの面
                let (dx, dy) = g.facing.d();
                let m = (g.pos.0 as f32 + dx as f32 * 0.5, g.pos.1 as f32 + dy as f32 * 0.5);
                let (sx, sy) = (-dy as f32 * 0.5, dx as f32 * 0.5);
                [c3(m.0 - sx, m.1 - sy, -0.5), c3(m.0 + sx, m.1 + sy, -0.5), c3(m.0 + sx, m.1 + sy, 0.5), c3(m.0 - sx, m.1 - sy, 0.5)]
            } else {
                [c3(fxf - 0.5, fyf - 0.5, 0.5), c3(fxf + 0.5, fyf - 0.5, 0.5), c3(fxf + 0.5, fyf + 0.5, 0.5), c3(fxf - 0.5, fyf + 0.5, 0.5)]
            };
            for i in 0..4 {
                let (mut a, mut b) = (pts[i], pts[(i + 1) % 4]);
                // 手前に出る線は 切る
                if a.z < NEAR && b.z < NEAR {
                    continue;
                }
                if a.z < NEAR {
                    a = lerp3(a, b, (NEAR - a.z) / (b.z - a.z));
                }
                if b.z < NEAR {
                    b = lerp3(b, a, (NEAR - b.z) / (a.z - b.z));
                }
                let (pa, pb) = (project(a), project(b));
                draw_line(pa.x, pa.y, pb.x, pb.y, 3.0, col);
            }
        }
        // 向き・操作のヒント
        let tag = format!("{}を向いている", dir_name(g.facing));
        let w = ui.measure(&tag, 16.0) + 18.0;
        draw_rectangle(LW - w - 8.0, VIEW_TOP + 8.0, w, 26.0, rgba(20, 30, 40, 215));
        draw_rectangle_lines(LW - w - 8.0, VIEW_TOP + 8.0, w, 26.0, 2.0, rgb(150, 190, 210));
        ui.text(&tag, LW - w + 1.0, VIEW_TOP + 12.0, 16.0, CREAM);
        ui.text("W:すすむ  A・S・D:ちょん押し=むき 長押し=すすむ", 10.0, VIEW_TOP + 12.0, 15.0, rgb(200, 215, 225));
        // 目の前の部屋の名前
        if let Some((fx, fy)) = g.front() {
            if let Some(i) = d.room_at(fx, fy) {
                let r = &d.rooms[i];
                let label = if r.alive() > 0 { format!("{}(魔物がいる)", r.kind) } else { r.kind.clone() };
                let w = ui.measure(&label, 18.0) + 24.0;
                draw_rectangle(LW / 2.0 - w / 2.0, VIEW_TOP + 40.0, w, 28.0, rgba(20, 30, 40, 220));
                draw_rectangle_lines(LW / 2.0 - w / 2.0, VIEW_TOP + 40.0, w, 28.0, 2.0, rgb(255, 214, 80));
                ui.text_center(&label, LW / 2.0, VIEW_TOP + 44.0, 18.0, CREAM);
            }
        }
    }
}
