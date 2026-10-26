//! 手で組んだ 3Dモデル(形から作る)。絵から作る model3d::build とちがい、部品(箱・面とり箱・まるい柱・たま・
//! なだらかな山・草のような うすい葉・多角形の板)を 組み合わせる。なめらかな 面は 頂点ごとに 向きと 色を持つ。
//! モデルの座標: x=よこ(-0.5..0.5)、y=うえ(0=ゆか)、z=まえ(+)/うしろ(-)。1マスの部屋に 入る大きさ。
//! いまあるのは わらのベッド・ゴミすてば。ほかの施設は まだ 絵から作った モデル。

use crate::model3d::{Model, Quad};
use macroquad::prelude::*;

pub(crate) type P = [f32; 3];

pub(crate) fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub(crate) fn add(a: P, b: P) -> P {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub(crate) fn mul(a: P, k: f32) -> P {
    [a[0] * k, a[1] * k, a[2] * k]
}
pub(crate) fn dot(a: P, b: P) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub(crate) fn cross(a: P, b: P) -> P {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
pub(crate) fn norm(a: P) -> P {
    let l = dot(a, a).sqrt().max(1e-6);
    mul(a, 1.0 / l)
}

/// ばらつき用の 決まった乱数(毎回 同じ形になる)
pub(crate) struct Rnd(pub u32);
impl Rnd {
    pub fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        ((self.0 >> 8) & 0xffff) as f32 / 65535.0
    }
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next()
    }
}

/// なめらかな ノイズ(-1..1)
pub(crate) fn noise(x: f32, z: f32, seed: f32) -> f32 {
    let h = |a: f32, b: f32| ((a * 127.1 + b * 311.7 + seed * 74.7).sin() * 43758.545).fract() * 2.0 - 1.0;
    let (xi, zi) = (x.floor(), z.floor());
    let (fx, fz) = (x - xi, z - zi);
    let (sx, sz) = (fx * fx * (3.0 - 2.0 * fx), fz * fz * (3.0 - 2.0 * fz));
    let (a, b, c, d) = (h(xi, zi), h(xi + 1.0, zi), h(xi, zi + 1.0), h(xi + 1.0, zi + 1.0));
    (a + (b - a) * sx) * (1.0 - sz) + (c + (d - c) * sx) * sz
}

pub(crate) fn col(r: u8, g: u8, b: u8) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}
pub(crate) fn shadec(c: Color, k: f32) -> Color {
    Color::new((c.r * k).clamp(0.0, 1.0), (c.g * k).clamp(0.0, 1.0), (c.b * k).clamp(0.0, 1.0), 1.0)
}
pub(crate) fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, 1.0)
}

/// 向き・位置の 変換(回転 + ずらし)
#[derive(Clone, Copy)]
pub(crate) struct Tf {
    m: [[f32; 3]; 3],
    t: P,
}

const ID: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

fn matmul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    r
}
pub(crate) fn rot_y(a: f32) -> [[f32; 3]; 3] {
    let (s, c) = a.sin_cos();
    [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]]
}
fn rot_x(a: f32) -> [[f32; 3]; 3] {
    let (s, c) = a.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
}
fn rot_z(a: f32) -> [[f32; 3]; 3] {
    let (s, c) = a.sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}
fn app(m: &[[f32; 3]; 3], p: P) -> P {
    [m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2], m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2], m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2]]
}

pub(crate) struct Mb {
    q: Vec<Quad>,
    tf: Tf,
    /// 大きい面を 分けた 小片ごとの 色ムラ(れんが・板・石の きめ)
    pub jitter: f32,
}

impl Mb {
    pub(crate) fn new() -> Mb {
        Mb { q: vec![], tf: Tf { m: ID, t: [0.0; 3] }, jitter: 0.0 }
    }

    /// これから作る 部品を (回転 yaw/pitch/roll のあと 位置 at に) 置く。f の中で 作る
    pub(crate) fn at(&mut self, at: P, yaw: f32, pitch: f32, roll: f32, f: impl FnOnce(&mut Mb)) {
        let saved = self.tf;
        self.tf = Tf { m: matmul(saved.m, matmul(rot_y(yaw), matmul(rot_x(pitch), rot_z(roll)))), t: add(saved.t, app(&saved.m, at)) };
        f(self);
        self.tf = saved;
    }

    /// 変換を 重ねる(pop で もどす)。頭だけ 前に かたむける ときなどに 使う
    pub(crate) fn from_model(model: Model) -> Mb {
        Mb { q: model.quads, tf: Tf { m: ID, t: [0.0; 3] }, jitter: 0.0 }
    }

    pub(crate) fn push(&mut self, at: P, yaw: f32, pitch: f32, roll: f32) -> Tf {
        let saved = self.tf;
        self.tf = Tf { m: matmul(saved.m, matmul(rot_y(yaw), matmul(rot_x(pitch), rot_z(roll)))), t: add(saved.t, app(&saved.m, at)) };
        saved
    }
    pub(crate) fn pop(&mut self, saved: Tf) {
        self.tf = saved;
    }

    /// 面を ひとつ 足す(角 c、頂点ごとの向き n・色 vc)。大きい面は 細かく 分ける
    /// (描く順番を 正しくするため。長いかまちが わらの上に かぶさって 見えるのを 防ぐ)
    pub(crate) fn quad(&mut self, c: [P; 4], n: [P; 4], vc: [Color; 4], dbl: bool) {
        let len = |a: P, b: P| dot(sub(a, b), sub(a, b)).sqrt();
        let (lu, lv) = (len(c[0], c[1]).max(len(c[3], c[2])), len(c[0], c[3]).max(len(c[1], c[2])));
        let (nu, nv) = (((lu / 0.07).ceil() as usize).clamp(1, 24), ((lv / 0.07).ceil() as usize).clamp(1, 24));
        if nu == 1 && nv == 1 {
            self.push_quad(c, n, vc, dbl);
            return;
        }
        let bil = |t: [P; 4], u: f32, v: f32| -> P { add(mul(add(mul(t[0], 1.0 - u), mul(t[1], u)), 1.0 - v), mul(add(mul(t[3], 1.0 - u), mul(t[2], u)), v)) };
        let bilc = |t: [Color; 4], u: f32, v: f32| -> Color {
            let l = |k: usize| mix(t[k], t[k + 1], u);
            let (a, b) = (mix(t[0], t[1], u), mix(t[3], t[2], u));
            let _ = l;
            mix(a, b, v)
        };
        for i in 0..nu {
            for j in 0..nv {
                let (u0, u1, v0, v1) = (i as f32 / nu as f32, (i + 1) as f32 / nu as f32, j as f32 / nv as f32, (j + 1) as f32 / nv as f32);
                let uv = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)];
                let cc = [0, 1, 2, 3].map(|k| bil(c, uv[k].0, uv[k].1));
                let nn = [0, 1, 2, 3].map(|k| norm(bil(n, uv[k].0, uv[k].1)));
                let mut vv = [0, 1, 2, 3].map(|k| bilc(vc, uv[k].0, uv[k].1));
                if self.jitter > 0.0 {
                    let m = bil(c, (u0 + u1) / 2.0, (v0 + v1) / 2.0);
                    let k = 1.0 + self.jitter * noise(m[0] * 43.0 + m[1] * 17.0, m[2] * 41.0 + m[1] * 29.0, 21.0);
                    vv = vv.map(|c| shadec(c, k));
                }
                self.push_quad(cc, nn, vv, dbl);
            }
        }
    }

    pub(crate) fn push_quad(&mut self, c: [P; 4], n: [P; 4], vc: [Color; 4], dbl: bool) {
        let m = &self.tf.m;
        let t = self.tf.t;
        let cc = [add(app(m, c[0]), t), add(app(m, c[1]), t), add(app(m, c[2]), t), add(app(m, c[3]), t)];
        let nn = [norm(app(m, n[0])), norm(app(m, n[1])), norm(app(m, n[2])), norm(app(m, n[3]))];
        let face = norm(add(add(nn[0], nn[1]), add(nn[2], nn[3])));
        self.q.push(Quad { c: cc, nrm: face, col: vc[0], vn: Some(nn), vcol: Some(vc), dbl });
    }

    pub(crate) fn flat(&mut self, c: [P; 4], n: P, color: Color) {
        self.quad(c, [n; 4], [color; 4], false);
    }

    /// 箱。(cx, cz)=よこ・おくゆきの 中心、y0=下の高さ、(sx, sy, sz)=大きさ
    pub(crate) fn bx(&mut self, cx: f32, y0: f32, cz: f32, sx: f32, sy: f32, sz: f32, c: Color, bottom: bool) {
        let (x0, x1, y1, z0, z1) = (cx - sx / 2.0, cx + sx / 2.0, y0 + sy, cz - sz / 2.0, cz + sz / 2.0);
        self.flat([[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]], [0.0, 0.0, 1.0], c);
        self.flat([[x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0]], [0.0, 0.0, -1.0], c);
        self.flat([[x1, y0, z0], [x1, y0, z1], [x1, y1, z1], [x1, y1, z0]], [1.0, 0.0, 0.0], c);
        self.flat([[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]], [-1.0, 0.0, 0.0], c);
        self.flat([[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]], [0.0, 1.0, 0.0], c);
        if bottom {
            self.flat([[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]], [0.0, -1.0, 0.0], c);
        }
    }

    /// 上のふちを 斜めに 面とりした 箱(ななめの面つき)
    pub(crate) fn rbox(&mut self, cx: f32, y0: f32, cz: f32, sx: f32, sy: f32, sz: f32, bevel: f32, c: Color) {
        let b = bevel.min(sx / 2.5).min(sz / 2.5).min(sy / 2.5);
        let (x0, x1, y1, z0, z1) = (cx - sx / 2.0, cx + sx / 2.0, y0 + sy, cz - sz / 2.0, cz + sz / 2.0);
        let yb = y1 - b;
        let (ix0, ix1, iz0, iz1) = (x0 + b, x1 - b, z0 + b, z1 - b);
        let s2 = std::f32::consts::FRAC_1_SQRT_2;
        // かべ
        self.flat([[x0, y0, z1], [x1, y0, z1], [x1, yb, z1], [x0, yb, z1]], [0.0, 0.0, 1.0], c);
        self.flat([[x0, y0, z0], [x1, y0, z0], [x1, yb, z0], [x0, yb, z0]], [0.0, 0.0, -1.0], c);
        self.flat([[x1, y0, z0], [x1, y0, z1], [x1, yb, z1], [x1, yb, z0]], [1.0, 0.0, 0.0], c);
        self.flat([[x0, y0, z0], [x0, y0, z1], [x0, yb, z1], [x0, yb, z0]], [-1.0, 0.0, 0.0], c);
        // ななめの 面とり
        let lc = shadec(c, 1.06);
        self.flat([[x0, yb, z1], [x1, yb, z1], [ix1, y1, iz1], [ix0, y1, iz1]], [0.0, s2, s2], lc);
        self.flat([[x0, yb, z0], [x1, yb, z0], [ix1, y1, iz0], [ix0, y1, iz0]], [0.0, s2, -s2], lc);
        self.flat([[x1, yb, z0], [x1, yb, z1], [ix1, y1, iz1], [ix1, y1, iz0]], [s2, s2, 0.0], lc);
        self.flat([[x0, yb, z0], [x0, yb, z1], [ix0, y1, iz1], [ix0, y1, iz0]], [-s2, s2, 0.0], lc);
        self.flat([[ix0, y1, iz0], [ix1, y1, iz0], [ix1, y1, iz1], [ix0, y1, iz1]], [0.0, 1.0, 0.0], shadec(c, 1.1));
    }

    /// 柱(円すい台)。p0〜p1 を 軸に、半径 r0〜r1。なめらか。caps=ふたをするか
    pub(crate) fn tube(&mut self, p0: P, p1: P, r0: f32, r1: f32, sides: usize, c: Color, caps: bool) {
        let axis = norm(sub(p1, p0));
        let helper = if axis[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
        let u = norm(cross(axis, helper));
        let v = cross(axis, u);
        let ring = |i: usize| -> P {
            let a = i as f32 / sides as f32 * std::f32::consts::TAU;
            add(mul(u, a.cos()), mul(v, a.sin()))
        };
        // 軸に そった 傾き(円すい台の ななめ)
        let slope = (r0 - r1) / sub(p1, p0).iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-4);
        let nrm_at = |d: P| norm(add(d, mul(axis, slope)));
        for i in 0..sides {
            let (d0, d1) = (ring(i), ring(i + 1));
            let (n0, n1) = (nrm_at(d0), nrm_at(d1));
            let k = |i: usize| 0.9 + 0.1 * ((i as f32 / sides as f32 * std::f32::consts::TAU).cos());
            let (c0, c1) = (shadec(c, k(i)), shadec(c, k(i + 1)));
            self.quad([add(p0, mul(d0, r0)), add(p0, mul(d1, r0)), add(p1, mul(d1, r1)), add(p1, mul(d0, r1))], [n0, n1, n1, n0], [c0, c1, c1, c0], false);
            if caps {
                let e0 = mul(axis, -1.0);
                self.quad([p0, add(p0, mul(d0, r0)), add(p0, mul(d1, r0)), add(p0, mul(d1, r0))], [e0; 4], [shadec(c, 0.7); 4], false);
                self.quad([p1, add(p1, mul(d0, r1)), add(p1, mul(d1, r1)), add(p1, mul(d1, r1))], [axis; 4], [shadec(c, 0.85); 4], false);
            }
        }
    }

    /// たま(だ円体)。cf=(緯度・経度の位置から 色を 返す関数)
    pub(crate) fn ellipsoid(&mut self, c: P, r: P, lat: usize, lon: usize, colf: &dyn Fn(f32, f32) -> Color) {
        let pt = |i: usize, j: usize| -> (P, P, Color) {
            let th = i as f32 / lat as f32 * std::f32::consts::PI; // 0=上
            let ph = j as f32 / lon as f32 * std::f32::consts::TAU;
            let d = [th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin()];
            let p = [c[0] + r[0] * d[0], c[1] + r[1] * d[1], c[2] + r[2] * d[2]];
            let n = norm([d[0] / r[0], d[1] / r[1], d[2] / r[2]]);
            (p, n, colf(th, ph))
        };
        for i in 0..lat {
            for j in 0..lon {
                let (a, b, cc, d) = (pt(i, j), pt(i, j + 1), pt(i + 1, j + 1), pt(i + 1, j));
                self.quad([a.0, b.0, cc.0, d.0], [a.1, b.1, cc.1, d.1], [a.2, b.2, cc.2, d.2], false);
            }
        }
    }

    /// なだらかな 山(高さの 関数)。x0..x1, z0..z1 を nx×nz に 区切る。h<=base の ところは 作らない
    pub(crate) fn heightfield(&mut self, x0: f32, x1: f32, z0: f32, z1: f32, nx: usize, nz: usize, base: f32, h: &dyn Fn(f32, f32) -> f32, colf: &dyn Fn(f32, f32, f32) -> Color) {
        let at = |i: usize, j: usize| -> (f32, f32) { (x0 + (x1 - x0) * i as f32 / nx as f32, z0 + (z1 - z0) * j as f32 / nz as f32) };
        let e = 0.004;
        let vert = |i: usize, j: usize| -> (P, P, Color) {
            let (x, z) = at(i, j);
            let y = h(x, z);
            let n = norm([-(h(x + e, z) - h(x - e, z)) / (2.0 * e), 1.0, -(h(x, z + e) - h(x, z - e)) / (2.0 * e)]);
            ([x, y, z], n, colf(x, z, y))
        };
        for i in 0..nx {
            for j in 0..nz {
                let (a, b, c, d) = (vert(i, j), vert(i + 1, j), vert(i + 1, j + 1), vert(i, j + 1));
                if a.0[1] <= base + 1e-4 && b.0[1] <= base + 1e-4 && c.0[1] <= base + 1e-4 && d.0[1] <= base + 1e-4 {
                    continue;
                }
                self.quad([a.0, d.0, c.0, b.0], [a.1, d.1, c.1, b.1], [a.2, d.2, c.2, b.2], false);
            }
        }
    }

    /// 草・わらの 1本(はじめの位置 p、向き dir、曲がり bend、長さ len、はば w)。両面
    pub(crate) fn blade(&mut self, p: P, dir: P, bend: f32, len: f32, w: f32, c: Color) {
        let d = norm(dir);
        let side = norm(cross(d, [0.0, 0.0, 1.0]));
        let side = if dot(side, side) < 0.01 { [1.0, 0.0, 0.0] } else { side };
        let segs = 3;
        let mut prev: Option<(P, P, f32)> = None;
        let mut cur = p;
        let mut dd = d;
        for s in 0..=segs {
            let t = s as f32 / segs as f32;
            let width = w * (1.0 - 0.8 * t);
            if s > 0 {
                cur = add(cur, mul(dd, len / segs as f32));
                dd = norm(add(dd, [0.0, -bend, 0.0]));
            }
            let l = add(cur, mul(side, -width));
            let r = add(cur, mul(side, width));
            if let Some((pl, pr, _)) = prev {
                let n = norm(cross(sub(r, l), sub(add(pl, [0.0; 3]), l)));
                let nn = if n[1] < 0.0 { mul(n, -1.0) } else { n };
                let cl = shadec(c, 0.85 + 0.25 * t);
                self.quad([pl, pr, r, l], [nn; 4], [shadec(c, 0.85 + 0.25 * (t - 0.3)); 4].map(|_| cl), true);
            }
            prev = Some((l, r, t));
        }
    }

    /// よこ(x)方向に 厚みのある 板(輪郭 poly は (z, y) の 凸多角形)。x0〜x1
    pub(crate) fn extrude_x(&mut self, poly: &[(f32, f32)], x0: f32, x1: f32, c: Color) {
        let n = poly.len();
        let (cz, cy) = (poly.iter().map(|p| p.0).sum::<f32>() / n as f32, poly.iter().map(|p| p.1).sum::<f32>() / n as f32);
        for i in 1..n - 1 {
            self.flat([[x1, poly[0].1, poly[0].0], [x1, poly[i].1, poly[i].0], [x1, poly[i + 1].1, poly[i + 1].0], [x1, poly[i + 1].1, poly[i + 1].0]], [1.0, 0.0, 0.0], c);
            self.flat([[x0, poly[0].1, poly[0].0], [x0, poly[i].1, poly[i].0], [x0, poly[i + 1].1, poly[i + 1].0], [x0, poly[i + 1].1, poly[i + 1].0]], [-1.0, 0.0, 0.0], shadec(c, 0.8));
        }
        for i in 0..n {
            let (a, b) = (poly[i], poly[(i + 1) % n]);
            let e = (b.0 - a.0, b.1 - a.1);
            // 辺の 外むきの 向き(中心から とおざかる ほう)
            let mut nn = (e.1, -e.0);
            let mid = ((a.0 + b.0) / 2.0 - cz, (a.1 + b.1) / 2.0 - cy);
            if nn.0 * mid.0 + nn.1 * mid.1 < 0.0 {
                nn = (-nn.0, -nn.1);
            }
            let l = (nn.0 * nn.0 + nn.1 * nn.1).sqrt().max(1e-6);
            self.flat([[x0, a.1, a.0], [x1, a.1, a.0], [x1, b.1, b.0], [x0, b.1, b.0]], [0.0, nn.1 / l, nn.0 / l], shadec(c, 1.02));
        }
    }

    /// 回転体(ろくろ): profile=(半径, 高さ)を 下から上へ。中心 (cx, cz)、高さの ずれ y0。colf(いちばん目の 番号, よこの 番号)=色
    pub(crate) fn lathe(&mut self, cx: f32, y0: f32, cz: f32, profile: &[(f32, f32)], sides: usize, colf: &dyn Fn(usize, usize) -> Color) {
        let n = profile.len();
        // 点ごとの 向き(となりあう 線分の 向きの 平均)
        let seg_n = |i: usize| -> (f32, f32) {
            let (a, b) = (profile[i], profile[i + 1]);
            let (dr, dy) = (b.0 - a.0, b.1 - a.1);
            let l = (dr * dr + dy * dy).sqrt().max(1e-6);
            (dy / l, -dr / l)
        };
        let pn: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                let a = if i > 0 { seg_n(i - 1) } else { seg_n(0) };
                let b = if i + 1 < n { seg_n(i) } else { seg_n(n - 2) };
                let (r, y) = (a.0 + b.0, a.1 + b.1);
                let l = (r * r + y * y).sqrt().max(1e-6);
                (r / l, y / l)
            })
            .collect();
        for i in 0..n - 1 {
            if (profile[i].0 - profile[i + 1].0).abs() < 1e-6 && (profile[i].1 - profile[i + 1].1).abs() < 1e-6 {
                continue;
            }
            for j in 0..sides {
                let a0 = j as f32 / sides as f32 * std::f32::consts::TAU;
                let a1 = (j + 1) as f32 / sides as f32 * std::f32::consts::TAU;
                let pt = |k: usize, a: f32| [cx + profile[k].0 * a.sin(), y0 + profile[k].1, cz + profile[k].0 * a.cos()];
                let nm = |k: usize, a: f32| norm([pn[k].0 * a.sin(), pn[k].1, pn[k].0 * a.cos()]);
                let c = colf(i, j);
                self.quad([pt(i, a0), pt(i, a1), pt(i + 1, a1), pt(i + 1, a0)], [nm(i, a0), nm(i, a1), nm(i + 1, a1), nm(i + 1, a0)], [c; 4], false);
            }
        }
    }

    /// ほのお: 外がわ(オレンジ)と 中(きいろ)の ゆらめく しずく形
    pub(crate) fn flame(&mut self, cx: f32, y0: f32, cz: f32, h: f32, w: f32) {
        let prof = |k: f32| -> Vec<(f32, f32)> { vec![(0.0, 0.0), (0.7 * w * k, 0.12 * h), (w * k, 0.35 * h), (0.75 * w * k, 0.62 * h), (0.35 * w * k, 0.85 * h), (0.0, h * (0.85 + 0.15 * k))] };
        self.lathe(cx, y0, cz, &prof(1.0), 10, &|i, _| mix(col(255, 120, 30), col(255, 190, 60), i as f32 / 5.0));
        self.lathe(cx, y0 + 0.01 * h, cz, &prof(0.55), 10, &|i, _| mix(col(255, 210, 80), col(255, 250, 190), i as f32 / 5.0));
    }

    /// 平らな わっか(ゆか・ふちの 輪)
    pub(crate) fn ring(&mut self, cx: f32, y: f32, cz: f32, r_out: f32, r_in: f32, sides: usize, c: Color) {
        self.lathe(cx, y, cz, &[(r_in, 0.0), (r_out, 0.0)], sides, &|_, _| c);
    }

    /// 垂れ布(ひだの ある たての 布)。x0〜x1、上 y_top から 下 y_bot、位置 z。folds=ひだの数
    pub(crate) fn drape(&mut self, x0: f32, x1: f32, y_top: f32, y_bot: f32, z: f32, folds: f32, front: bool, c: Color) {
        let n = 28;
        let nz = if front { 1.0 } else { -1.0 };
        let off = |x: f32| 0.02 * ((x - x0) / (x1 - x0) * folds * std::f32::consts::TAU).sin();
        for i in 0..n {
            let (t0, t1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
            let (xa, xb) = (x0 + (x1 - x0) * t0, x0 + (x1 - x0) * t1);
            let sl = |x: f32| ((x - x0) / (x1 - x0) * folds * std::f32::consts::TAU).cos() * 0.5;
            let (na, nb) = (norm([sl(xa) * -nz, 0.0, nz]), norm([sl(xb) * -nz, 0.0, nz]));
            let sh = |x: f32| 0.82 + 0.3 * (((x - x0) / (x1 - x0) * folds * std::f32::consts::TAU).cos() * 0.5 + 0.5);
            let (ca, cb) = (shadec(c, sh(xa)), shadec(c, sh(xb)));
            self.quad([[xa, y_bot, z + off(xa) * nz], [xb, y_bot, z + off(xb) * nz], [xb, y_top, z], [xa, y_top, z]], [na, nb, nb, na], [ca, cb, cb, ca], true);
        }
    }

    pub(crate) fn done(self) -> Model {
        Model { quads: self.q, hand: true, voxels: 0 }
    }
}

/// わらのベッド
pub fn straw_bed() -> Model {
    let mut m = Mb::new();
    let mut rnd = Rnd(7);
    let wood = col(128, 84, 46);
    let wood_d = col(94, 60, 32);
    let straw = col(228, 190, 86);

    // あし(すこし 細くなる まるい柱)と あし先
    for (x, z) in [(-0.43, -0.27), (0.43, -0.27), (-0.43, 0.27), (0.43, 0.27)] {
        m.tube([x, 0.0, z], [x, 0.15, z], 0.034, 0.046, 10, wood_d, true);
        m.tube([x, 0.0, z], [x, 0.02, z], 0.042, 0.042, 10, shadec(wood_d, 0.8), false);
    }
    // ゆか板(よこに ならべた 面とり板)
    for i in 0..9 {
        let x = -0.40 + i as f32 * 0.1;
        m.rbox(x, 0.13, 0.0, 0.088, 0.04, 0.60, 0.008, mix(wood, shadec(wood, 1.12), rnd.next()));
    }
    // 前・うしろの かまち(面とり)
    for z in [-0.30f32, 0.30] {
        m.rbox(0.0, 0.13, z, 0.94, 0.17, 0.055, 0.014, mix(wood, shadec(wood, 1.1), rnd.next()));
    }
    // ヘッドボード(左・高く 上が まるい)と フットボード(右・ひくい)
    let arch = |top: f32, rise: f32| -> Vec<(f32, f32)> {
        let mut v = vec![(-0.33, 0.12), (0.33, 0.12)];
        for k in 0..=12 {
            let a = k as f32 / 12.0;
            let z = 0.33 - 0.66 * a;
            v.push((z, top + rise * (a * std::f32::consts::PI).sin()));
        }
        v
    };
    m.extrude_x(&arch(0.50, 0.12), -0.505, -0.455, wood_d);
    m.extrude_x(&arch(0.27, 0.05), 0.455, 0.505, wood_d);
    for z in [-0.33f32, 0.33] {
        m.ellipsoid([-0.48, 0.52, z], [0.035, 0.035, 0.035], 6, 10, &|_, _| wood);
        m.tube([-0.48, 0.10, z], [-0.48, 0.52, z], 0.03, 0.03, 8, wood, false);
        m.tube([0.48, 0.10, z], [0.48, 0.28, z], 0.028, 0.028, 8, wood, false);
        m.ellipsoid([0.48, 0.29, z], [0.03, 0.03, 0.03], 6, 10, &|_, _| wood);
    }

    // わらの マットレス: なだらかな こんもりした 山
    let mat = |x: f32, z: f32| -> f32 {
        let fx = (x / 0.45).clamp(-1.0, 1.0);
        let fz = (z / 0.27).clamp(-1.0, 1.0);
        let dome = (1.0 - fx * fx).max(0.0).powf(0.55) * (1.0 - fz * fz).max(0.0).powf(0.6);
        0.20 + 0.12 * dome + 0.012 * noise(x * 14.0, z * 14.0, 1.0)
    };
    m.heightfield(-0.45, 0.45, -0.27, 0.27, 34, 16, 0.0, &mat, &|x, z, _| {
        let band = (noise(x * 40.0, z * 6.0, 3.0) * 0.5 + 0.5) * 0.35;
        let k = 0.86 + 0.28 * band + 0.06 * noise(x * 90.0, z * 90.0, 5.0);
        shadec(straw, k)
    });
    // わら: 上に つき出た 1本ずつ
    for _ in 0..320 {
        let x = rnd.range(-0.44, 0.44);
        let z = rnd.range(-0.26, 0.26);
        let a = rnd.range(0.0, std::f32::consts::TAU);
        let tilt = rnd.range(0.25, 0.85);
        let dir = [a.cos() * tilt, 1.0 - tilt * 0.3, a.sin() * tilt];
        let len = rnd.range(0.045, 0.11);
        let (bend, hue) = (rnd.range(0.2, 0.9), rnd.next());
        // 木枠・ヘッドボードに つき ささらない(先が 枠の 内がわに おさまる ものだけ)
        let d = norm(dir);
        let (tx, tz) = (x + d[0] * len * 0.9, z + d[2] * len * 0.9);
        if tz.abs() > 0.255 || tx.abs() > 0.435 {
            continue;
        }
        m.blade([x, mat(x, z) - 0.012, z], dir, bend, len, 0.0075, mix(col(236, 202, 100), col(210, 168, 70), hue));
    }
    // ふちの 内がわの わらを 外むきでなく 上むきに 少し 足す(木枠の 内がわに おさまる)
    for _ in 0..40 {
        let x = rnd.range(-0.42, 0.42);
        let z = rnd.range(-0.23, 0.23);
        let a = rnd.range(0.0, std::f32::consts::TAU);
        m.blade([x, mat(x, z) - 0.012, z], [a.cos() * 0.25, 1.0, a.sin() * 0.25], rnd.range(0.3, 0.8), rnd.range(0.04, 0.08), 0.008, mix(col(232, 196, 96), col(206, 162, 66), rnd.next()));
    }
    // まくら(頭の側。つぶれた だ円体)
    m.ellipsoid([-0.29, 0.355, 0.0], [0.14, 0.055, 0.19], 10, 18, &|th, _| shadec(col(240, 232, 208), 0.92 + 0.12 * (th / 3.1416)));
    // ぼろ布(足の側。マットレスの 上に かぶさる。枠の 内がわに おさまる)
    let blanket = |x: f32, z: f32| -> f32 {
        let edge = ((0.262 - z.abs()) / 0.05).clamp(0.0, 1.0);
        mat(x, z) + 0.004 + 0.022 * edge + 0.008 * edge * noise(x * 20.0, z * 9.0, 8.0)
    };
    m.heightfield(0.04, 0.44, -0.262, 0.262, 14, 18, 0.0, &blanket, &|x, z, _| {
        let stripe = if ((x * 22.0).floor() as i32) % 2 == 0 { 1.0 } else { 0.84 };
        shadec(col(158, 88, 64), stripe * (0.94 + 0.08 * noise(x * 40.0, z * 40.0, 2.0)))
    });
    m.done()
}

/// ゴミすてば
pub fn trash_dump() -> Model {
    let mut m = Mb::new();
    let mut rnd = Rnd(31);
    let wood = col(132, 88, 50);
    let iron = col(74, 70, 76);
    let palette = [col(122, 98, 68), col(92, 84, 76), col(146, 116, 72), col(84, 100, 72), col(150, 74, 58), col(112, 112, 118), col(170, 152, 112), col(70, 60, 54)];

    // ガラクタの 山(ノイズで でこぼこの 地面)
    let heap = |x: f32, z: f32| -> f32 {
        let e = (x / 0.47).powi(2) + (z / 0.36).powi(2);
        if e >= 1.0 {
            return 0.0;
        }
        let f = (1.0 - e).powf(0.75);
        (0.19 * f + 0.045 * f * (noise(x * 9.0, z * 9.0, 4.0) + 0.5 * noise(x * 22.0, z * 22.0, 6.0))).max(0.0)
    };
    m.heightfield(-0.47, 0.47, -0.36, 0.36, 40, 30, 0.0, &heap, &|x, z, y| {
        let n = noise(x * 13.0, z * 13.0, 9.0) * 0.5 + 0.5;
        let c = palette[((n * 7.99) as usize).min(7)];
        shadec(c, 0.82 + 0.3 * (y / 0.2).min(1.0) + 0.1 * noise(x * 60.0, z * 60.0, 1.0))
    });

    // たおれた たる(ふくらんだ 形・たが・ふた)。やや ななめに 山に のっている
    m.at([-0.12, 0.19, -0.05], 0.3, 0.0, 0.0, |m| {
        let (len, r0, segs, sides) = (0.40, 0.155, 14usize, 20usize);
        let rad = |t: f32| r0 * (1.0 + 0.11 * (1.0 - (2.0 * t - 1.0).powi(2)));
        for i in 0..segs {
            let (t0, t1) = (i as f32 / segs as f32, (i + 1) as f32 / segs as f32);
            let (x0, x1) = (-len / 2.0 + len * t0, -len / 2.0 + len * t1);
            let (r_a, r_b) = (rad(t0), rad(t1));
            let slope = (r_a - r_b) / (x1 - x0);
            for j in 0..sides {
                let (a0, a1) = (j as f32 / sides as f32 * std::f32::consts::TAU, (j + 1) as f32 / sides as f32 * std::f32::consts::TAU);
                let p = |x: f32, r: f32, a: f32| [x, r * a.sin(), r * a.cos()];
                let nrm = |a: f32| norm([slope, a.sin(), a.cos()]);
                let stave = if j % 2 == 0 { 1.0 } else { 0.88 };
                let cc = shadec(wood, stave * (0.92 + 0.12 * noise(i as f32 * 0.7, j as f32 * 2.3, 11.0)));
                m.quad([p(x0, r_a, a0), p(x0, r_a, a1), p(x1, r_b, a1), p(x1, r_b, a0)], [nrm(a0), nrm(a1), nrm(a1), nrm(a0)], [cc; 4], false);
            }
        }
        // たが(鉄の わっか)
        for t in [0.12f32, 0.34, 0.66, 0.88] {
            let x = -len / 2.0 + len * t;
            let r = rad(t) + 0.006;
            m.tube([x - 0.014, 0.0, 0.0], [x + 0.014, 0.0, 0.0], r, r, sides, iron, false);
        }
        // ふた(あいている がわは 暗く くぼむ)
        m.tube([len / 2.0 - 0.012, 0.0, 0.0], [len / 2.0, 0.0, 0.0], r0 * 0.9, r0 * 0.9, sides, col(46, 32, 22), true);
        m.tube([-len / 2.0, 0.0, 0.0], [-len / 2.0 + 0.012, 0.0, 0.0], r0 * 0.9, r0 * 0.9, sides, shadec(wood, 0.8), true);
    });

    // ふくろ(口を しばった)
    let sack = col(190, 164, 114);
    m.ellipsoid([0.27, 0.085, -0.12], [0.11, 0.09, 0.095], 10, 16, &|th, ph| shadec(sack, 0.9 + 0.1 * (ph * 3.0).sin() * th.sin()));
    m.ellipsoid([0.275, 0.17, -0.12], [0.05, 0.055, 0.05], 8, 12, &|_, _| shadec(sack, 0.95));
    m.tube([0.275, 0.14, -0.12], [0.275, 0.155, -0.12], 0.047, 0.047, 10, col(110, 80, 50), false);

    // かん(立っている・たおれている)
    let tin = col(150, 152, 160);
    m.tube([0.33, 0.0, 0.19], [0.33, 0.09, 0.19], 0.045, 0.045, 14, tin, true);
    m.tube([0.33, 0.085, 0.19], [0.33, 0.095, 0.19], 0.05, 0.05, 14, shadec(tin, 0.8), true);
    m.at([0.21, 0.045, 0.27], 0.8, 0.0, 1.45, |m| {
        m.tube([0.0, 0.0, 0.0], [0.0, 0.085, 0.0], 0.042, 0.042, 14, col(172, 96, 64), true);
    });
    // びん(横に ころがる。首が ほそい)
    m.at([-0.30, 0.04, 0.24], 0.4, 0.0, 0.0, |m| {
        let g = col(86, 144, 96);
        m.tube([-0.06, 0.0, 0.0], [0.03, 0.0, 0.0], 0.036, 0.036, 12, g, true);
        m.tube([0.03, 0.0, 0.0], [0.07, 0.0, 0.0], 0.036, 0.014, 12, g, false);
        m.tube([0.07, 0.0, 0.0], [0.115, 0.0, 0.0], 0.014, 0.014, 10, g, false);
        m.tube([0.115, 0.0, 0.0], [0.13, 0.0, 0.0], 0.016, 0.016, 10, col(150, 110, 60), true);
    });
    // 骨(ふとい 骨と 両はしの こぶ)
    let bone = col(238, 232, 216);
    for (p0, p1) in [([-0.08, 0.035, 0.29], [0.10, 0.05, 0.27]), ([0.02, 0.06, 0.12], [-0.12, 0.045, 0.20])] {
        m.tube(p0, p1, 0.014, 0.014, 8, bone, false);
        for p in [p0, p1] {
            m.ellipsoid(p, [0.024, 0.02, 0.022], 6, 10, &|_, _| bone);
        }
    }
    // 魚の ほね(せぼねと あばらと 頭)
    m.tube([0.0, 0.035, 0.16], [0.22, 0.04, 0.2], 0.007, 0.007, 6, shadec(bone, 0.95), false);
    for i in 0..8 {
        let t = i as f32 / 8.0;
        let c = [0.02 + t * 0.19, 0.04 + t * 0.004, 0.16 + t * 0.04];
        m.tube(add(c, [0.0, 0.0, -0.035 * (1.0 - t * 0.5)]), add(c, [0.0, 0.004, 0.035 * (1.0 - t * 0.5)]), 0.004, 0.004, 5, shadec(bone, 0.92), false);
    }
    m.ellipsoid([-0.012, 0.04, 0.158], [0.028, 0.018, 0.02], 6, 10, &|_, _| bone);
    // りんごの しん
    m.ellipsoid([-0.37, 0.045, -0.10], [0.038, 0.045, 0.038], 8, 12, &|th, _| mix(col(180, 56, 44), col(230, 220, 180), (th / 3.14).powi(3)));
    m.tube([-0.37, 0.085, -0.10], [-0.365, 0.115, -0.1], 0.005, 0.004, 5, col(80, 54, 28), false);
    // たおれかかった 板と ぼろ布
    m.at([0.12, 0.2, 0.02], 0.4, 0.0, 0.22, |m| m.rbox(0.0, 0.0, 0.0, 0.17, 0.028, 0.11, 0.006, col(152, 110, 66)));
    m.at([-0.28, 0.17, -0.20], -0.5, 0.0, -0.3, |m| m.rbox(0.0, 0.0, 0.0, 0.13, 0.026, 0.1, 0.006, col(138, 100, 60)));
    m.at([0.05, 0.19, 0.24], 0.9, 0.18, 0.1, |m| m.ellipsoid([0.0, 0.0, 0.0], [0.07, 0.012, 0.05], 5, 12, &|_, _| col(150, 170, 120)));
    // 散らばった 小さな ごみ
    for _ in 0..16 {
        let a = rnd.range(0.0, std::f32::consts::TAU);
        let rr = rnd.range(0.12, 0.40);
        let (x, z) = (a.cos() * rr * 1.1, a.sin() * rr * 0.8);
        let s = rnd.range(0.012, 0.03);
        let junk = mix(palette[(rnd.next() * 7.9) as usize], col(60, 50, 44), 0.2);
        m.at([x, heap(x, z) + s * 0.4, z], rnd.range(0.0, 3.0), rnd.range(-0.5, 0.5), rnd.range(-0.5, 0.5), |m| {
            m.ellipsoid([0.0, 0.0, 0.0], [s, s * 0.7, s * 1.2], 5, 8, &|_, _| junk);
        });
    }
    m.done()
}

/// 手で組んだモデルが あれば 返す(絵の名前で えらぶ)
pub fn hand_model(art_name: &str) -> Option<Model> {
    use crate::facilities::*;
    if art_name.starts_with("mon_") {
        return crate::creatures2::creature_model(art_name);
    }
    Some(match art_name {
        "room_わらのベッド" => straw_bed(),
        "room_ゴミすてば" => trash_dump(),
        "room_エサおけ" => feed_trough(),
        "room_てつのたからばこ" => chest(Metal::Iron, false),
        "room_てつのたからばこ_open" => chest(Metal::Iron, true),
        "room_ぎんのたからばこ" => chest(Metal::Silver, false),
        "room_ぎんのたからばこ_open" => chest(Metal::Silver, true),
        "room_おうごんチェスト" => chest(Metal::Gold, false),
        "room_おうごんチェスト_open" => chest(Metal::Gold, true),
        "room_さかば" => tavern(),
        "room_さいだん" => altar(),
        "room_いろり" => hearth(),
        "room_水のみば" => water_trough(),
        "room_りっぱなベッド" => canopy_bed(false),
        "room_ごうかなベッド" => canopy_bed(true),
        "room_だんろ" => fireplace(),
        "room_けっかいのへや" => barrier_room(),
        "room_ふんすい" => fountain(),
        "room_れいはいどう" => chapel(),
        "room_れいぞうこ" => fridge(),
        "room_まほうじん" => magic_circle(),
        "room_瘴気発生装置" => miasma_machine(),
        "room_はたけ" => crate::rooms3::farm(),
        "room_はなぞの" => crate::rooms3::flower_garden(),
        "room_どうじょう" => crate::rooms3::dojo(),
        "room_じっけんしつ" => crate::rooms3::lab(),
        "room_コロシアム" => crate::rooms3::arena(),
        _ => return None,
    })
}

/// 手組みモデルの ある 絵か
pub fn has_hand(art_name: &str) -> bool {
    if art_name.starts_with("mon_") {
        return true;
    }
    art_name.starts_with("room_") && !matches!(art_name, "room_ふんすい_open") && hand_names().contains(&art_name)
}

fn hand_names() -> &'static [&'static str] {
    &["room_わらのベッド", "room_ゴミすてば", "room_エサおけ", "room_てつのたからばこ", "room_てつのたからばこ_open", "room_ぎんのたからばこ", "room_ぎんのたからばこ_open", "room_おうごんチェスト", "room_おうごんチェスト_open", "room_さかば", "room_さいだん", "room_いろり", "room_水のみば", "room_りっぱなベッド", "room_ごうかなベッド", "room_だんろ", "room_けっかいのへや", "room_ふんすい", "room_れいはいどう", "room_れいぞうこ", "room_まほうじん", "room_瘴気発生装置", "room_はたけ", "room_はなぞの", "room_どうじょう", "room_じっけんしつ", "room_コロシアム"]
}

/// 何マスぶんの 大きさで 作ってあるか(1 か 3)
pub fn hand_cells(art_name: &str) -> usize {
    if matches!(art_name, "room_はたけ" | "room_はなぞの" | "room_どうじょう" | "room_じっけんしつ" | "room_コロシアム") {
        3
    } else {
        1
    }
}
