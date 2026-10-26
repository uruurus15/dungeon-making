//! 3D モデル: 絵(assets/img の PNG)から ボクセル(小さな立方体)の立体を 作って 描く。
//! 絵の ふちからの距離で ふくらませて(まるみのある厚み)、前・うしろ・ふちの 面を 立てる。
//! 描くときは 向き(どちらを向いているか)・大きさ・高さを 決めて、世界の中で 回せる。
//! 魔物も 施設も 同じ作り方。一覧は models.rs(F4)。

use crate::view3d::*;
use macroquad::prelude::*;

/// 面: c=4つの角(モデルの座標: x=よこ -0.5..0.5, y=うえ 0..1, z=まえ/うしろ)、n=向き、col=色
pub struct Quad {
    pub c: [[f32; 3]; 4],
    /// 面の向き(モデルの座標。x=右 y=うえ z=まえ)
    pub nrm: [f32; 3],
    pub col: Color,
    /// 頂点ごとの向き(なめらかな 面。なければ 面ぜんたいが nrm)
    pub vn: Option<[[f32; 3]; 4]>,
    /// 頂点ごとの色(色ムラ。なければ col)
    pub vcol: Option<[Color; 4]>,
    /// 両面(うすい 草など)。うらがわも 描く
    pub dbl: bool,
}

pub struct Model {
    pub quads: Vec<Quad>,
    /// 手で組んだモデルか(絵から作ったものでなく)
    pub hand: bool,
    /// 立方体の数(一覧に出す)
    pub voxels: usize,
}

/// 絵から 作る。n=よこ・たての 分割数(多いほど 細かい)、thick=厚み(よこ幅に対する 割合)
pub fn build(img: &Image, n: usize, thick: f32) -> Model {
    let (w, h) = (img.width as usize, img.height as usize);
    let mut col = vec![[0.0f32; 3]; n * n];
    let mut solid = vec![false; n * n];
    for gy in 0..n {
        for gx in 0..n {
            let (x0, x1) = (gx * w / n, ((gx + 1) * w / n).max(gx * w / n + 1));
            let (y0, y1) = (gy * h / n, ((gy + 1) * h / n).max(gy * h / n + 1));
            let (mut a, mut r, mut g, mut b, mut cnt) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
            for y in y0..y1.min(h) {
                for x in x0..x1.min(w) {
                    let i = (y * w + x) * 4;
                    let al = img.bytes[i + 3] as f32 / 255.0;
                    a += al;
                    r += img.bytes[i] as f32 * al;
                    g += img.bytes[i + 1] as f32 * al;
                    b += img.bytes[i + 2] as f32 * al;
                    cnt += 1.0;
                }
            }
            if cnt > 0.0 && a / cnt > 0.45 && a > 0.0 {
                solid[gy * n + gx] = true;
                col[gy * n + gx] = [r / a / 255.0, g / a / 255.0, b / a / 255.0];
            }
        }
    }
    // ふちからの距離(city block)
    let big = 999i32;
    let mut dist: Vec<i32> = solid.iter().map(|&s| if s { big } else { 0 }).collect();
    for _ in 0..2 {
        for y in 0..n {
            for x in 0..n {
                if dist[y * n + x] == 0 {
                    continue;
                }
                let up = if y > 0 { dist[(y - 1) * n + x] } else { 0 };
                let lf = if x > 0 { dist[y * n + x - 1] } else { 0 };
                dist[y * n + x] = dist[y * n + x].min(up + 1).min(lf + 1);
            }
        }
        for y in (0..n).rev() {
            for x in (0..n).rev() {
                if dist[y * n + x] == 0 {
                    continue;
                }
                let dn = if y + 1 < n { dist[(y + 1) * n + x] } else { 0 };
                let rt = if x + 1 < n { dist[y * n + x + 1] } else { 0 };
                dist[y * n + x] = dist[y * n + x].min(dn + 1).min(rt + 1);
            }
        }
    }
    let maxh = (n as f32 * thick).max(1.0);
    let r = (n as f32 * 0.16).max(2.0);
    let hh: Vec<i32> = dist.iter().map(|&d| if d == 0 { 0 } else { ((maxh * ((d as f32).min(r) / r).sqrt()).round() as i32).max(1) }).collect();

    let nf = n as f32;
    let mut quads: Vec<Quad> = vec![];
    let mut voxels = 0usize;
    let at = |x: i32, y: i32| -> i32 {
        if x < 0 || y < 0 || x >= n as i32 || y >= n as i32 {
            0
        } else {
            hh[y as usize * n + x as usize]
        }
    };
    for gy in 0..n as i32 {
        for gx in 0..n as i32 {
            let hgt = hh[gy as usize * n + gx as usize];
            if hgt == 0 {
                continue;
            }
            let c = col[gy as usize * n + gx as usize];
            let color = Color::new(c[0], c[1], c[2], 1.0);
            voxels += (hgt * 2) as usize;
            let (xl, xr) = ((gx as f32 - nf / 2.0) / nf, (gx as f32 + 1.0 - nf / 2.0) / nf);
            let (yt, yb) = ((nf - gy as f32) / nf, (nf - gy as f32 - 1.0) / nf);
            let z = hgt as f32 / nf;
            // まえ・うしろ
            quads.push(Quad { c: [[xl, yb, z], [xr, yb, z], [xr, yt, z], [xl, yt, z]], nrm: [0.0, 0.0, 1.0], col: color, vn: None, vcol: None, dbl: false });
            quads.push(Quad { c: [[xl, yb, -z], [xr, yb, -z], [xr, yt, -z], [xl, yt, -z]], nrm: [0.0, 0.0, -1.0], col: color, vn: None, vcol: None, dbl: false });
            // ふち(となりより 厚い ぶんだけ)
            let mut side = |nh: i32, nrm: [f32; 3], mk: &dyn Fn(f32, f32) -> [[f32; 3]; 4]| {
                if nh < hgt {
                    let (za, zb) = (nh as f32 / nf, z);
                    quads.push(Quad { c: mk(za, zb), nrm, col: color, vn: None, vcol: None, dbl: false });
                    quads.push(Quad { c: mk(-zb, -za), nrm, col: color, vn: None, vcol: None, dbl: false });
                }
            };
            side(at(gx + 1, gy), [1.0, 0.0, 0.0], &|a, b| [[xr, yb, a], [xr, yb, b], [xr, yt, b], [xr, yt, a]]);
            side(at(gx - 1, gy), [-1.0, 0.0, 0.0], &|a, b| [[xl, yb, a], [xl, yb, b], [xl, yt, b], [xl, yt, a]]);
            side(at(gx, gy - 1), [0.0, 1.0, 0.0], &|a, b| [[xl, yt, a], [xr, yt, a], [xr, yt, b], [xl, yt, b]]);
            side(at(gx, gy + 1), [0.0, -1.0, 0.0], &|a, b| [[xl, yb, a], [xr, yb, a], [xr, yb, b], [xl, yb, b]]);
        }
    }
    Model { quads, hand: false, voxels }
}

/// 面の 明るさ(世界の光: 左うえ・手前から)。nw=世界の水平の法線、up=上向きの成分
fn lighting(nwx: f32, nwy: f32, up: f32) -> f32 {
    let (lx, ly, lu) = (0.35f32, 0.45f32, 0.82f32);
    let l = (lx * lx + ly * ly + lu * lu).sqrt();
    let nl = (nwx * nwx + nwy * nwy + up * up).sqrt().max(1e-4);
    let d = (nwx * lx + nwy * ly + up * lu) / l / nl;
    (0.78 + 0.40 * d).clamp(0.5, 1.15)
}

/// モデルを 世界に置いて描く。(wx,wy)=足もとの世界の位置、front=モデルの前が 向く方向(世界の平面)、
/// size=大きさ(マス)、lift=床からの高さ、tint=全体の明るさ
pub fn draw_model(m: &Model, cam: &Cam, wx: f32, wy: f32, front: (f32, f32), size: f32, lift: f32, tint: f32) {
    // 目の うしろ・すぐ そばに ある モデルは 描かない(設備を 背にしたとき 視界を ふさぐのを 防ぐ)
    let pivot = cam.to_cam(wx, wy, 0.0);
    if pivot.z < 0.45 * size.max(0.5) {
        return;
    }
    let fl = (front.0 * front.0 + front.1 * front.1).sqrt().max(1e-4);
    let (fx, fy) = (front.0 / fl, front.1 / fl);
    // モデルの「右」(正面から 見て 右)
    let (rx, ry) = (fy, -fx);
    let world_n = |n: [f32; 3]| (n[0] * rx + n[2] * fx, n[0] * ry + n[2] * fy, n[1]);
    type Item = ([V3; 4], [Color; 4], f32);
    let mut list: Vec<Item> = Vec::with_capacity(m.quads.len() / 2);
    for q in &m.quads {
        let (nwx, nwy, up) = world_n(q.nrm);
        let mut pts = [V3 { x: 0.0, y: 0.0, z: 0.0 }; 4];
        for (i, c) in q.c.iter().enumerate() {
            let (lx, ly, lz) = (c[0] * size, c[1] * size, c[2] * size);
            pts[i] = cam.to_cam(wx + lx * rx + lz * fx, wy + lx * ry + lz * fy, 0.5 - ly - lift);
        }
        // 見えない向き(うらがわ)は 描かない
        let nc = V3 { x: nwx * cam.rx + nwy * cam.ry, y: -up, z: nwx * cam.fx + nwy * cam.fy };
        let cz0 = (pts[0].z + pts[2].z) / 2.0;
        let cxm = (pts[0].x + pts[2].x) / 2.0;
        let cym = (pts[0].y + pts[2].y) / 2.0;
        let facing = nc.x * cxm + nc.y * cym + nc.z * cz0 < 0.0;
        // 目のすぐ手前・うしろに ある 面は 描かない(うしろに ある 大きな 設備が 画面いっぱいに なるのを 防ぐ)
        if cz0 < 0.3 {
            continue;
        }
        // 描く順番: 目からの きょり(まっすぐな 奥行きでなく)
        let cz = (cxm * cxm + cym * cym + cz0 * cz0).sqrt();
        if !facing && !q.dbl {
            continue;
        }
        let mut cols = [q.col; 4];
        for i in 0..4 {
            let base = q.vcol.map(|v| v[i]).unwrap_or(q.col);
            let n = q.vn.map(|v| v[i]).unwrap_or(q.nrm);
            let (a, b, c) = world_n(n);
            // 両面の 草は うらがわを むいたら 法線を ひっくり返す
            let sgn = if q.dbl && !facing { -1.0 } else { 1.0 };
            let l = lighting(a * sgn, b * sgn, c * sgn) * tint;
            cols[i] = Color::new((base.r * l).min(1.0), (base.g * l).min(1.0), (base.b * l).min(1.0), 1.0);
        }
        list.push((pts, cols, cz));
    }
    list.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    for (pts, cols, _) in list {
        let poly: Vec<CV> = (0..4).map(|i| CV { p: pts[i], u: 0.0, v: 0.0, col: Some(cols[i]) }).collect();
        emit(None, &poly, 1.0, None);
    }
}
