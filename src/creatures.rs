//! 魔物の 手組みモデル。2頭身の かわいい 体つき(頭が大きい)を 土台に、
//! 人型(Hum: 耳・口もと・しっぽ・つの・ぼうし・よろい・ぶき・たて・つばさ・マント の 組み合わせ)と、
//! ぷるぷる(スライム)・おばけ・やさい/くだもの・どうぶつ などの 型で 作る。
//! 座標は 足もとが y=0、まえが +z。できあがりは 高さ 1 に そろえる(fit)。

use crate::model3d::Model;
use crate::modeling::*;
use macroquad::prelude::Color;
use std::f32::consts::{PI, TAU};

// ---------------------------------------------------------------- 人型

#[derive(Clone, Copy, PartialEq)]
pub enum Ear {
    No,
    Point,
    Cat,
    Dog,
    Round,
    Long,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Snout {
    No,
    Short,
    Dog,
    Pig,
    Beak,
    Bill,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Horn {
    No,
    Small,
    Bull,
    Demon,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Hat {
    No,
    Helm,
    Wizard,
    Top,
    Crown,
    Hood,
    Samurai,
    Cap,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Wp {
    No,
    Club,
    Sword,
    Dagger,
    Staff,
    Spear,
    Axe,
    Bow,
    Mace,
    Torch,
    Trident,
    FlameSword,
    IceSword,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Eyes {
    Normal,
    Glow,
    One,
    Three,
    Angry,
    Dot,
}

#[derive(Clone, Copy)]
pub struct Hum {
    pub fur: Color,
    pub cloth: Color,
    pub trim: Color,
    pub hair: Option<Color>,
    pub eye: Color,
    pub eyes: Eyes,
    pub head: f32,
    pub body: f32,
    pub belly: f32,
    pub legs: f32,
    pub arm: f32,
    pub ear: Ear,
    pub snout: Snout,
    pub tails: u8,
    pub tail_long: bool,
    pub horn: Horn,
    pub hat: Hat,
    pub armor: Option<Color>,
    pub cape: Option<Color>,
    pub wp: Wp,
    pub wp_col: Color,
    pub shield: Option<Color>,
    pub wings: Option<Color>,
    pub mane: Option<Color>,
    pub fangs: bool,
    pub skele: bool,
    pub accent: Color,
    pub pants: Option<Color>,
    pub seed: u32,
    /// 頭を 前に かたむける(ラジアン。ねこ背・ずるい 体つき)
    pub lean: f32,
    /// 足の ひらき(1=ふつう)
    pub stance: f32,
    /// うでの ポーズ: 0 たらす / 1 ひじを はって 前に かまえる / 2 かかげる / 3 むねの 前で 手を あわせる / 4 ぶらんと 長く たらす(ゴブリン・オニ)
    pub arm_pose: u8,
    pub tusks: bool,
    /// かたの とげ(よろいの 肩)
    pub spikes: bool,
    /// かぶとの かざり羽
    pub plume: Option<Color>,
    pub trim_col: Option<Color>,
}

pub fn hum(fur: Color, cloth: Color) -> Hum {
    Hum {
        fur,
        cloth,
        trim: shadec(cloth, 0.7),
        hair: None,
        eye: col(30, 24, 20),
        eyes: Eyes::Normal,
        head: 1.0,
        body: 1.0,
        belly: 0.0,
        legs: 1.0,
        arm: 1.0,
        ear: Ear::No,
        snout: Snout::No,
        tails: 0,
        tail_long: false,
        horn: Horn::No,
        hat: Hat::No,
        armor: None,
        cape: None,
        wp: Wp::No,
        wp_col: col(200, 204, 214),
        shield: None,
        wings: None,
        mane: None,
        fangs: false,
        skele: false,
        accent: col(236, 190, 70),
        pants: None,
        seed: 1,
        lean: 0.0,
        stance: 1.0,
        arm_pose: 0,
        tusks: false,
        spikes: false,
        plume: None,
        trim_col: None,
    }
}

fn sph(m: &mut Mb, c: [f32; 3], r: [f32; 3], color: Color) {
    m.ellipsoid(c, r, 8, 12, &|th, _| shadec(color, 0.9 + 0.14 * (1.0 - th / PI)));
}

fn glow_sph(m: &mut Mb, c: [f32; 3], r: f32, color: Color) {
    m.ellipsoid(c, [r, r, r * 0.6], 6, 8, &|_, _| color);
}

/// コウモリの はね(こまかい ほねと まく)
pub fn bat_wing(m: &mut Mb, side: f32, root: [f32; 3], span: f32, color: Color, bone: Color) {
    let s = side;
    // 横から 見ても 消えないように、ひじで 後ろに まがった 奥行きと 太い ほねを もたせる
    let elbow = [s * span * 0.42, 0.26 * span, -0.2 * span];
    let tips = [[s * span, 0.34 * span, -0.34 * span], [s * span * 0.96, 0.1 * span, -0.38 * span], [s * span * 0.7, -0.14 * span, -0.3 * span], [s * span * 0.36, -0.22 * span, -0.2 * span]];
    let r = |p: [f32; 3]| add(root, p);
    for i in 0..3 {
        let (a, b) = (tips[i], tips[i + 1]);
        let bulge = add(mul(add(a, b), 0.5), [0.0, 0.0, -0.03 * span]);
        m.quad([r(elbow), r(a), r(bulge), r(b)], [[0.0, 0.0, 1.0]; 4], [color, shadec(color, 1.1), shadec(color, 0.9), shadec(color, 1.1)], true);
    }
    m.quad([r([0.0; 3]), r(elbow), r(tips[3]), r(tips[3])], [[0.0, 0.0, 1.0]; 4], [color; 4], true);
    let bt = (0.03 * span / 0.6).max(0.014);
    m.tube(r([0.0; 3]), r(elbow), bt * 1.3, bt, 6, bone, true);
    m.tube(r(elbow), r(tips[0]), bt, bt * 0.5, 6, bone, true);
    for t in &tips[1..] {
        m.tube(r(elbow), r(*t), bt * 0.6, bt * 0.3, 5, bone, false);
    }
    m.ellipsoid(r([0.0; 3]), [bt * 1.6, bt * 1.6, bt * 1.6], 5, 8, &|_, _| bone);
}

/// 羽の つばさ(重ねた 羽)
pub fn feather_wing(m: &mut Mb, side: f32, root: [f32; 3], span: f32, color: Color) {
    for row in 0..3 {
        for i in 0..5 {
            let t = i as f32 / 4.0;
            let len = span * (0.55 + 0.45 * (1.0 - t * 0.4)) * (1.0 - 0.18 * row as f32);
            let a = 0.2 + t * 1.0 + row as f32 * 0.12;
            let dir = [side * a.sin().max(0.2), a.cos() * 0.5 + 0.35 - 0.2 * row as f32, -0.05];
            let tip = add(root, mul(norm(dir), len));
            m.tube(add(root, [0.0, -0.02 * row as f32, -0.01 * row as f32]), tip, 0.022 - 0.004 * row as f32, 0.004, 6, shadec(color, 0.95 + 0.05 * row as f32), false);
        }
    }
}

pub fn build_hum(h: &Hum) -> Model {
    let mut m = Mb::new();
    let mut rnd = Rnd(h.seed.max(1));
    let sk = h.skele;
    let ly = 0.30 * h.legs;
    let (bw, bh, bd) = (0.15 * h.body + h.belly * 0.3, 0.16 * (1.0 + (h.body - 1.0).max(0.0) * 0.55) + h.belly * 0.1, 0.12 * h.body + h.belly * 0.3);
    let ty = ly + 0.14;
    let top = ty + bh;
    let hr = 0.17 * h.head;
    let hy = top + hr * 0.78;
    let bone = col(236, 230, 212);
    let pants = h.pants.unwrap_or(h.fur);

    // 脚と 足
    for sx in [-1.0f32, 1.0] {
        let hip = [sx * 0.075 * h.body, ly, 0.0];
        let foot = [sx * 0.085 * h.body * h.stance, 0.03, 0.01];
        if sk {
            m.tube(hip, foot, 0.016, 0.014, 6, bone, false);
            sph(&mut m, [foot[0], 0.026, 0.04], [0.04, 0.022, 0.07], bone);
        } else {
            m.tube(hip, foot, 0.048, 0.042, 8, pants, true);
            sph(&mut m, [foot[0], 0.03, 0.04], [0.058, 0.032, 0.085], if h.pants.is_some() { shadec(pants, 0.8) } else { h.fur });
        }
    }
    // しっぽ
    for k in 0..h.tails {
        let off = if h.tails == 2 { if k == 0 { -0.06 } else { 0.06 } } else { 0.0 };
        let mut prev = [off, ly + 0.02, -bd * 0.9];
        let len = if h.tail_long { 0.12 } else { 0.08 };
        for s in 1..=5 {
            let t = s as f32 / 5.0;
            let p = [off * (1.0 + t) + 0.03 * (t * 5.0).sin(), ly + 0.02 + 0.22 * t * t + 0.02 * t, -bd * 0.9 - len * 2.0 * t - 0.04];
            m.tube(prev, p, 0.034 * (1.0 - 0.45 * t), 0.034 * (1.0 - 0.45 * (t + 0.2)), 8, h.fur, s == 5);
            prev = p;
        }
    }
    // 胴
    if sk {
        m.tube([0.0, ly + 0.02, 0.0], [0.0, top - 0.02, -0.04], 0.015, 0.015, 6, bone, false);
        for k in 0..5 {
            let y = ty - 0.1 + k as f32 * 0.055;
            let r = 0.11 * h.body * (1.0 - ((k as f32 - 2.0) / 3.0).powi(2) * 0.6);
            m.lathe(0.0, y, 0.0, &[(r - 0.008, 0.0), (r, 0.0), (r, 0.012), (r - 0.008, 0.012)], 14, &|_, _| bone);
        }
        sph(&mut m, [0.0, ly + 0.02, 0.0], [0.1, 0.04, 0.06], bone);
    } else {
        m.ellipsoid([0.0, ty, 0.0], [bw, bh, bd], 10, 16, &|th, _| shadec(h.cloth, 0.86 + 0.18 * (1.0 - th / PI)));
        if h.belly > 0.0 {
            sph(&mut m, [0.0, ty - 0.03, bd * 0.5], [bw * 0.8, bh * 0.8, bd * 0.7], shadec(h.fur, 1.05));
        }
        if let Some(a) = h.armor {
            sph(&mut m, [0.0, ty + 0.03, 0.012], [bw * 1.08, bh * 0.78, bd * 1.1], a);
            m.lathe(0.0, ty - 0.12, 0.0, &[(bw * 0.78, 0.0), (bw * 0.86, 0.0), (bw * 0.86, 0.025), (bw * 0.78, 0.025)], 16, &|_, _| h.accent);
            if let Some(tc) = h.trim_col {
                // 金の むなあてと 肩かざり
                m.lathe(0.0, ty + 0.1, 0.0, &[(bw * 1.1, 0.0), (bw * 1.12, 0.0), (bw * 1.12, 0.03), (bw * 1.1, 0.03)], 18, &|_, _| tc);
                m.ellipsoid([0.0, ty + 0.03, bd * 1.1], [0.05, 0.06, 0.02], 6, 8, &|_, _| tc);
            }
            for sx in [-1.0f32, 1.0] {
                sph(&mut m, [sx * (bw + 0.0), top - 0.03, 0.0], [0.075, 0.06, 0.075], shadec(a, 1.08));
                if h.spikes {
                    for k in 0..3 {
                        let ang = 0.5 + 0.35 * k as f32;
                        m.tube([sx * (bw + 0.02), top - 0.01, 0.0], [sx * (bw + 0.02 + 0.1 * ang.cos()), top + 0.04 + 0.12 * ang.sin(), -0.02 * k as f32], 0.03, 0.0, 6, shadec(a, 0.7), true);
                    }
                }
            }
        }
        // こしまわり(ベルト)
        m.lathe(0.0, ty - 0.09, 0.0, &[(bw * 0.86, 0.0), (bw * 0.9, 0.0), (bw * 0.9, 0.025), (bw * 0.86, 0.025)], 18, &|_, _| h.trim_col.unwrap_or(h.trim));
    }
    // 首・頭(頭だけ 前に かたむけられる)
    let head_tf = m.push([0.0, top - 0.02, 0.0], 0.0, h.lean, 0.0);
    let head_tf2 = m.push([0.0, -(top - 0.02), 0.0], 0.0, 0.0, 0.0);
    if !sk {
        m.tube([0.0, top - 0.03, 0.0], [0.0, hy - hr * 0.6, 0.0], 0.06, 0.055, 8, h.fur, false);
    }
    let head_col = if sk { bone } else { h.fur };
    m.ellipsoid([0.0, hy, 0.0], [hr, hr * 0.95, hr * 0.95], 12, 18, &|th, _| shadec(head_col, 0.88 + 0.16 * (1.0 - th / PI)));
    // 目
    let ez = hr * 0.84;
    match h.eyes {
        Eyes::Normal | Eyes::Angry | Eyes::Dot => {
            for sx in [-1.0f32, 1.0] {
                let ec = [sx * 0.36 * hr, hy + 0.1 * hr, ez];
                if h.eyes == Eyes::Dot {
                    m.ellipsoid(ec, [0.1 * hr, 0.14 * hr, 0.08 * hr], 6, 8, &|_, _| h.eye);
                } else {
                    m.ellipsoid(ec, [0.2 * hr, 0.22 * hr, 0.12 * hr], 7, 10, &|_, _| col(250, 248, 240));
                    m.ellipsoid([ec[0], ec[1] - 0.01, ec[2] + 0.01], [0.12 * hr, 0.14 * hr, 0.1 * hr], 6, 8, &|_, _| h.eye);
                    if h.eyes == Eyes::Angry {
                        m.tube([ec[0] - sx * 0.22 * hr, ec[1] + 0.26 * hr, ec[2]], [ec[0] + sx * 0.2 * hr, ec[1] + 0.12 * hr, ec[2]], 0.02 * hr, 0.02 * hr, 5, shadec(head_col, 0.5), false);
                    }
                }
            }
        }
        Eyes::Glow => {
            for sx in [-1.0f32, 1.0] {
                glow_sph(&mut m, [sx * 0.34 * hr, hy + 0.1 * hr, ez], 0.17 * hr, h.eye);
            }
        }
        Eyes::One => {
            m.ellipsoid([0.0, hy + 0.1 * hr, ez], [0.36 * hr, 0.34 * hr, 0.14 * hr], 8, 10, &|_, _| col(250, 248, 236));
            m.ellipsoid([0.0, hy + 0.1 * hr, ez + 0.03], [0.2 * hr, 0.22 * hr, 0.1 * hr], 6, 8, &|_, _| h.eye);
        }
        Eyes::Three => {
            for (x, y, r) in [(-0.4f32, 0.0f32, 0.16f32), (0.4, 0.0, 0.16), (0.0, 0.42, 0.18)] {
                m.ellipsoid([x * hr, hy + (0.1 + y) * hr, ez], [r * hr, r * hr * 1.1, 0.1 * hr], 6, 8, &|_, _| col(250, 240, 200));
                m.ellipsoid([x * hr, hy + (0.1 + y) * hr, ez + 0.02], [r * 0.55 * hr, r * 0.7 * hr, 0.08 * hr], 5, 6, &|_, _| h.eye);
            }
        }
    }
    // 口もと
    let (mz, my) = (hr * 0.86, hy - 0.4 * hr);
    match h.snout {
        Snout::No => {}
        Snout::Short => {
            sph(&mut m, [0.0, hy - 0.2 * hr, hr * 0.78], [0.42 * hr, 0.32 * hr, 0.4 * hr], shadec(head_col, 1.08));
            sph(&mut m, [0.0, hy - 0.05 * hr, hr * 1.12], [0.1 * hr, 0.07 * hr, 0.07 * hr], col(40, 30, 30));
        }
        Snout::Dog => {
            sph(&mut m, [0.0, hy - 0.22 * hr, hr * 0.95], [0.38 * hr, 0.3 * hr, 0.58 * hr], shadec(head_col, 1.1));
            sph(&mut m, [0.0, hy - 0.05 * hr, hr * 1.5], [0.12 * hr, 0.09 * hr, 0.08 * hr], col(30, 24, 24));
        }
        Snout::Pig => {
            m.tube([0.0, hy - 0.2 * hr, hr * 0.85], [0.0, hy - 0.2 * hr, hr * 1.18], 0.36 * hr, 0.32 * hr, 12, shadec(head_col, 1.12), true);
            if h.tusks {
                for sx in [-1.0f32, 1.0] {
                    // 下あごから 外に ひらいて 上に そる きば
                    let mut prev = [sx * 0.2 * hr, hy - 0.4 * hr, hr * 1.0];
                    for s in 1..=6 {
                        let t = s as f32 / 6.0;
                        let p = [sx * (0.2 + 0.26 * t) * hr, hy - (0.4 - 0.7 * t * t) * hr, hr * (1.0 + 0.5 * t)];
                        m.tube(prev, p, 0.09 * hr * (1.0 - 0.7 * t) + 0.004, 0.09 * hr * (1.0 - 0.7 * (t + 0.17)) + 0.003, 7, col(246, 240, 220), s == 6);
                        prev = p;
                    }
                }
            }
            for sx in [-1.0f32, 1.0] {
                sph(&mut m, [sx * 0.12 * hr, hy - 0.2 * hr, hr * 1.2], [0.06 * hr, 0.08 * hr, 0.03 * hr], col(60, 30, 30));
            }
        }
        Snout::Beak => {
            m.tube([0.0, hy - 0.12 * hr, hr * 0.8], [0.0, hy - 0.2 * hr, hr * 1.55], 0.3 * hr, 0.0, 8, h.accent, false);
        }
        Snout::Bill => {
            sph(&mut m, [0.0, hy - 0.25 * hr, hr * 1.0], [0.62 * hr, 0.14 * hr, 0.6 * hr], h.accent);
        }
    }
    if h.snout != Snout::Beak {
        m.ellipsoid([0.0, my, mz + if h.snout == Snout::Dog { 0.25 * hr } else { 0.0 }], [0.2 * hr, 0.035 * hr, 0.05 * hr], 4, 8, &|_, _| col(56, 24, 24));
    }
    if h.fangs {
        for sx in [-1.0f32, 1.0] {
            m.tube([sx * 0.14 * hr, my + 0.02, mz + 0.02], [sx * 0.14 * hr, my - 0.06 * hr, mz + 0.03], 0.03 * hr, 0.005, 6, col(250, 248, 240), false);
        }
    }
    // 耳
    let ear_in = |c: Color| shadec(mix(c, col(240, 150, 150), 0.5), 1.0);
    for sx in [-1.0f32, 1.0] {
        match h.ear {
            Ear::No => {}
            Ear::Point => {
                m.tube([sx * 0.85 * hr, hy + 0.15 * hr, 0.0], [sx * 1.75 * hr, hy + 0.5 * hr, -0.1 * hr], 0.22 * hr, 0.0, 8, head_col, true);
            }
            Ear::Cat => {
                m.tube([sx * 0.55 * hr, hy + 0.7 * hr, 0.0], [sx * 0.7 * hr, hy + 1.5 * hr, -0.1 * hr], 0.32 * hr, 0.0, 8, head_col, true);
                m.tube([sx * 0.55 * hr, hy + 0.72 * hr, 0.04 * hr], [sx * 0.68 * hr, hy + 1.35 * hr, -0.04 * hr], 0.2 * hr, 0.0, 8, ear_in(head_col), true);
            }
            Ear::Dog => {
                m.tube([sx * 0.6 * hr, hy + 0.75 * hr, 0.0], [sx * 0.95 * hr, hy + 1.45 * hr, -0.15 * hr], 0.3 * hr, 0.08 * hr, 8, shadec(head_col, 0.92), true);
            }
            Ear::Round => {
                sph(&mut m, [sx * 0.75 * hr, hy + 0.78 * hr, 0.0], [0.3 * hr, 0.3 * hr, 0.14 * hr], head_col);
                sph(&mut m, [sx * 0.75 * hr, hy + 0.78 * hr, 0.1 * hr], [0.18 * hr, 0.18 * hr, 0.06 * hr], ear_in(head_col));
            }
            Ear::Long => {
                m.tube([sx * 0.35 * hr, hy + 0.85 * hr, 0.0], [sx * 0.7 * hr, hy + 2.0 * hr, -0.1 * hr], 0.2 * hr, 0.14 * hr, 8, head_col, true);
            }
        }
    }
    // たてがみ・かみ
    if let Some(mc) = h.mane {
        // たてがみ: 顔のまわりと うしろに ふさふさの 玉
        for k in 0..14 {
            let a = k as f32 / 14.0 * TAU;
            let r = 1.0 * hr;
            sph(&mut m, [a.sin() * r, hy + a.cos() * 0.9 * hr, -0.18 * hr], [0.36 * hr, 0.38 * hr, 0.3 * hr], shadec(mc, 0.85 + 0.25 * rnd.next()));
        }
        for k in 0..8 {
            let a = k as f32 / 8.0 * TAU;
            sph(&mut m, [a.sin() * 0.7 * hr, hy + a.cos() * 0.65 * hr, -0.55 * hr], [0.4 * hr, 0.4 * hr, 0.3 * hr], shadec(mc, 0.8 + 0.2 * rnd.next()));
        }
    }
    if let Some(hc) = h.hair {
        m.ellipsoid([0.0, hy + 0.18 * hr, -0.1 * hr], [1.05 * hr, 0.92 * hr, 1.0 * hr], 10, 14, &|th, _| shadec(hc, 0.85 + 0.2 * (1.0 - th / PI)));
        sph(&mut m, [0.0, hy + 0.55 * hr, 0.55 * hr], [0.9 * hr, 0.3 * hr, 0.35 * hr], hc);
    }
    // つの
    for sx in [-1.0f32, 1.0] {
        match h.horn {
            Horn::No => {}
            Horn::Small => {
                m.tube([sx * 0.4 * hr, hy + 0.85 * hr, 0.1 * hr], [sx * 0.5 * hr, hy + 1.4 * hr, 0.1 * hr], 0.14 * hr, 0.0, 6, col(240, 230, 210), true);
            }
            Horn::Bull => {
                m.tube([sx * 0.7 * hr, hy + 0.55 * hr, 0.0], [sx * 1.3 * hr, hy + 0.6 * hr, 0.05 * hr], 0.2 * hr, 0.16 * hr, 8, col(240, 230, 200), false);
                m.tube([sx * 1.3 * hr, hy + 0.6 * hr, 0.05 * hr], [sx * 1.45 * hr, hy + 1.3 * hr, 0.18 * hr], 0.16 * hr, 0.0, 8, col(240, 230, 200), true);
            }
            Horn::Demon => {
                m.tube([sx * 0.5 * hr, hy + 0.8 * hr, 0.0], [sx * 0.8 * hr, hy + 1.5 * hr, -0.2 * hr], 0.16 * hr, 0.1 * hr, 8, col(60, 44, 54), false);
                m.tube([sx * 0.8 * hr, hy + 1.5 * hr, -0.2 * hr], [sx * 0.55 * hr, hy + 1.95 * hr, -0.45 * hr], 0.1 * hr, 0.0, 8, col(60, 44, 54), true);
            }
        }
    }
    // ぼうし
    match h.hat {
        Hat::No => {}
        Hat::Helm => {
            let a = h.armor.unwrap_or(col(150, 150, 160));
            m.ellipsoid([0.0, hy + 0.2 * hr, 0.0], [1.08 * hr, 0.9 * hr, 1.08 * hr], 10, 16, &|th, _| if th < 1.5 { shadec(a, 0.9 + 0.2 * (1.0 - th / PI)) } else { shadec(a, 0.7) });
            m.rbox(0.0, hy - 0.25 * hr, hr * 0.95, 0.14 * hr, 0.7 * hr, 0.1 * hr, 0.01, shadec(a, 0.85));
            m.lathe(0.0, hy + 0.1 * hr, 0.0, &[(1.04 * hr, 0.0), (1.12 * hr, 0.0), (1.12 * hr, 0.1 * hr), (1.04 * hr, 0.1 * hr)], 16, &|_, _| shadec(a, 1.1));
            if let Some(pc) = h.plume {
                // かざり羽(あたまの うえから うしろに なびく)
                let mut prev = [0.0, hy + 1.0 * hr, 0.1 * hr];
                for s in 1..=8 {
                    let t = s as f32 / 8.0;
                    let p = [0.0, hy + (1.0 + 0.55 * (t * PI).sin()) * hr + 0.1 * hr - 0.5 * t * hr, 0.1 * hr - 1.4 * t * hr];
                    m.tube(prev, p, 0.2 * hr * (1.0 - 0.5 * t), 0.2 * hr * (1.0 - 0.5 * (t + 0.12)), 8, shadec(pc, 0.9 + 0.2 * t), s == 8);
                    prev = p;
                }
            }
        }
        Hat::Wizard => {
            let c = h.cloth;
            m.lathe(0.0, hy + 0.55 * hr, 0.0, &[(1.5 * hr, 0.0), (1.55 * hr, 0.06 * hr), (0.9 * hr, 0.12 * hr)], 18, &|_, _| shadec(c, 0.85));
            m.tube([0.0, hy + 0.6 * hr, 0.0], [0.15 * hr, hy + 2.3 * hr, -0.1 * hr], 0.88 * hr, 0.0, 14, c, true);
            m.lathe(0.0, hy + 0.7 * hr, 0.0, &[(0.84 * hr, 0.0), (0.9 * hr, 0.0), (0.9 * hr, 0.14 * hr), (0.84 * hr, 0.14 * hr)], 16, &|_, _| h.accent);
        }
        Hat::Top => {
            m.lathe(0.0, hy + 0.65 * hr, 0.0, &[(1.3 * hr, 0.0), (1.3 * hr, 0.05 * hr), (0.0, 0.05 * hr)], 18, &|_, _| col(30, 28, 34));
            m.tube([0.0, hy + 0.7 * hr, 0.0], [0.0, hy + 1.7 * hr, 0.0], 0.75 * hr, 0.72 * hr, 16, col(36, 34, 42), true);
            m.lathe(0.0, hy + 0.72 * hr, 0.0, &[(0.75 * hr, 0.0), (0.76 * hr, 0.22 * hr)], 16, &|_, _| col(150, 50, 60));
        }
        Hat::Crown => {
            m.lathe(0.0, hy + 0.7 * hr, 0.0, &[(0.82 * hr, 0.0), (0.82 * hr, 0.35 * hr)], 16, &|_, _| h.accent);
            for k in 0..6 {
                let a = k as f32 / 6.0 * TAU;
                m.tube([a.sin() * 0.78 * hr, hy + 1.0 * hr, a.cos() * 0.78 * hr], [a.sin() * 0.8 * hr, hy + 1.45 * hr, a.cos() * 0.8 * hr], 0.16 * hr, 0.0, 6, h.accent, true);
                sph(&mut m, [a.sin() * 0.8 * hr, hy + 1.47 * hr, a.cos() * 0.8 * hr], [0.07 * hr, 0.07 * hr, 0.07 * hr], col(220, 60, 70));
            }
        }
        Hat::Hood => {
            m.ellipsoid([0.0, hy + 0.1 * hr, -0.18 * hr], [1.15 * hr, 1.08 * hr, 1.1 * hr], 10, 16, &|th, _| shadec(h.cloth, 0.8 + 0.2 * (1.0 - th / PI)));
            m.tube([0.0, hy + 0.9 * hr, -0.2 * hr], [0.0, hy + 1.1 * hr, -0.55 * hr], 0.45 * hr, 0.0, 8, h.cloth, true);
            // 顔の あな(くらい)
            m.ellipsoid([0.0, hy - 0.05 * hr, hr * 0.52], [0.8 * hr, 0.8 * hr, 0.45 * hr], 8, 12, &|_, _| col(28, 22, 30));
            for sx in [-1.0f32, 1.0] {
                glow_sph(&mut m, [sx * 0.32 * hr, hy + 0.05 * hr, hr * 0.92], 0.14 * hr, h.eye);
            }
        }
        Hat::Samurai => {
            let a = h.armor.unwrap_or(col(180, 180, 190));
            m.ellipsoid([0.0, hy + 0.25 * hr, 0.0], [1.1 * hr, 0.85 * hr, 1.1 * hr], 10, 16, &|th, _| shadec(a, 0.9 + 0.2 * (1.0 - th / PI)));
            for sx in [-1.0f32, 1.0] {
                m.tube([sx * 0.2 * hr, hy + 0.9 * hr, hr * 0.9], [sx * 1.0 * hr, hy + 1.7 * hr, hr * 0.7], 0.08 * hr, 0.02 * hr, 6, h.accent, true);
            }
            m.lathe(0.0, hy + 0.0, 0.0, &[(1.05 * hr, 0.0), (1.4 * hr, -0.1 * hr), (1.45 * hr, -0.05 * hr)], 16, &|_, _| shadec(a, 0.85));
        }
        Hat::Cap => {
            m.ellipsoid([0.0, hy + 0.5 * hr, 0.0], [1.02 * hr, 0.55 * hr, 1.02 * hr], 8, 14, &|_, _| h.cloth);
            m.lathe(0.0, hy + 0.45 * hr, hr * 0.3, &[(0.0, 0.0), (0.7 * hr, 0.0), (0.7 * hr, 0.04 * hr)], 12, &|_, _| shadec(h.cloth, 0.8));
        }
    }
    m.pop(head_tf2);
    m.pop(head_tf);
    // うで・手
    for sx in [-1.0f32, 1.0] {
        let shoulder = [sx * (bw + 0.01), ty + bh * 0.55, 0.0];
        let right = sx > 0.0;
        let fwd = if right && h.wp != Wp::No { 0.1 } else { 0.03 };
        let mut hand = [sx * (bw + 0.07 * h.arm), ty - 0.1 * h.arm - if right && h.wp != Wp::No { 0.0 } else { 0.04 }, fwd + 0.02];
        let mut elbow = [sx * (bw + 0.06), (shoulder[1] + hand[1]) / 2.0, 0.0];
        match h.arm_pose {
            1 => {
                elbow = [sx * (bw + 0.1), ty + 0.0, -0.02];
                hand = [sx * (bw * 0.7), ty + 0.03, 0.15];
            }
            2 => {
                elbow = [sx * (bw + 0.12), top - 0.02, 0.0];
                hand = [sx * (bw + 0.1), top + 0.16, 0.05];
            }
            3 => {
                elbow = [sx * (bw + 0.02), ty - 0.05, 0.05];
                hand = [sx * 0.035, ty + 0.0, 0.14];
            }
            4 => {
                hand = [sx * (bw + 0.1), ty - 0.2 * h.arm, 0.04];
                elbow = [sx * (bw + 0.08), (shoulder[1] + hand[1]) / 2.0, 0.0];
            }
            _ => {}
        }
        if sk {
            m.tube(shoulder, elbow, 0.014, 0.013, 6, bone, false);
            m.tube(elbow, hand, 0.013, 0.012, 6, bone, false);
            sph(&mut m, hand, [0.03, 0.03, 0.03], bone);
        } else {
            m.tube(shoulder, elbow, 0.046, 0.04, 8, h.cloth, true);
            m.tube(elbow, hand, 0.04, 0.034, 8, h.fur, true);
            sph(&mut m, hand, [0.048, 0.048, 0.048], h.fur);
        }
        if let (Some(a), false) = (h.armor, sk) {
            sph(&mut m, elbow, [0.055, 0.055, 0.055], a);
        }
        if right && h.wp != Wp::No {
            m.at(hand, 0.0, -0.35, -0.1, |m| weapon(m, h));
        }
        if !right {
            if let Some(sc) = h.shield {
                m.at([hand[0] - 0.02, hand[1] + 0.02, hand[2] + 0.05], 0.4, PI / 2.0, 0.0, |m| {
                    m.lathe(0.0, 0.0, 0.0, &[(0.0, 0.035), (0.08, 0.03), (0.15, 0.012), (0.17, 0.0), (0.17, -0.02), (0.0, -0.02)], 20, &|i, _| if i < 3 { sc } else { shadec(sc, 0.6) });
                    m.lathe(0.0, 0.0, 0.0, &[(0.0, 0.05), (0.04, 0.04), (0.05, 0.03)], 10, &|_, _| h.accent);
                });
            }
        }
    }
    if let Some(c) = h.cape {
        m.drape(-bw - 0.02, bw + 0.02, top - 0.02, ly - 0.06, -bd - 0.02, 4.0, false, c);
    }
    if let Some(w) = h.wings {
        for s in [-1.0f32, 1.0] {
            bat_wing(&mut m, s, [s * 0.06, ty + 0.1, -bd], 0.36 + (h.body - 1.0).max(0.0) * 0.4, w, shadec(w, 0.6));
        }
    }
    finish(m, 1.0)
}

fn weapon(m: &mut Mb, h: &Hum) {
    let steel = h.wp_col;
    let wood = col(124, 84, 46);
    match h.wp {
        Wp::No => {}
        Wp::Club => {
            m.tube([0.0, -0.04, 0.0], [0.0, 0.3, 0.0], 0.026, 0.05, 8, wood, true);
            for k in 0..5 {
                let a = k as f32 / 5.0 * TAU;
                m.tube([a.sin() * 0.04, 0.24 + 0.04 * (k % 2) as f32, a.cos() * 0.04], [a.sin() * 0.06, 0.25 + 0.04 * (k % 2) as f32, a.cos() * 0.06], 0.012, 0.0, 5, col(90, 90, 96), true);
            }
        }
        Wp::Sword | Wp::FlameSword | Wp::IceSword => {
            let blade = match h.wp {
                Wp::FlameSword => col(255, 130, 50),
                Wp::IceSword => col(170, 230, 255),
                _ => steel,
            };
            m.rbox(0.0, 0.03, 0.0, 0.03, 0.46, 0.008, 0.003, blade);
            m.tube([0.0, 0.49, 0.0], [0.0, 0.54, 0.0], 0.015, 0.0, 6, blade, true);
            m.rbox(0.0, 0.0, 0.0, 0.1, 0.025, 0.03, 0.005, h.accent);
            m.tube([0.0, -0.08, 0.0], [0.0, 0.0, 0.0], 0.014, 0.014, 6, col(90, 56, 34), true);
            if h.wp == Wp::FlameSword {
                m.flame(0.0, 0.1, 0.0, 0.22, 0.04);
            }
        }
        Wp::Dagger => {
            m.rbox(0.0, 0.02, 0.0, 0.026, 0.2, 0.008, 0.002, steel);
            m.tube([0.0, 0.22, 0.0], [0.0, 0.26, 0.0], 0.013, 0.0, 6, steel, true);
            m.rbox(0.0, 0.0, 0.0, 0.07, 0.02, 0.025, 0.004, h.accent);
        }
        Wp::Staff => {
            m.tube([0.0, -0.3, 0.0], [0.0, 0.5, 0.0], 0.016, 0.02, 8, wood, true);
            m.ellipsoid([0.0, 0.58, 0.0], [0.05, 0.06, 0.05], 8, 12, &|th, _| mix(h.accent, col(255, 255, 255), (1.0 - th / PI).powi(2) * 0.5));
            for k in 0..3 {
                let a = k as f32 / 3.0 * TAU;
                m.tube([a.sin() * 0.05, 0.5, a.cos() * 0.05], [a.sin() * 0.03, 0.6, a.cos() * 0.03], 0.008, 0.005, 5, wood, false);
            }
        }
        Wp::Spear => {
            m.tube([0.0, -0.4, 0.0], [0.0, 0.62, 0.0], 0.014, 0.014, 6, wood, true);
            m.tube([0.0, 0.62, 0.0], [0.0, 0.84, 0.0], 0.034, 0.0, 6, steel, true);
        }
        Wp::Axe => {
            m.tube([0.0, -0.1, 0.0], [0.0, 0.46, 0.0], 0.02, 0.02, 7, wood, true);
            m.at([0.0, 0.4, 0.0], 0.0, 0.0, 0.0, |m| m.ellipsoid([0.06, 0.0, 0.0], [0.11, 0.1, 0.012], 6, 10, &|_, _| steel));
        }
        Wp::Bow => {
            let mut prev = [0.0, -0.3, 0.0];
            for s in 1..=10 {
                let t = s as f32 / 10.0;
                let a = (t - 0.5) * PI * 0.95;
                let p = [0.0, a.sin() * 0.3, -0.14 * a.cos() + 0.14];
                m.tube(prev, p, 0.014, 0.014, 5, wood, false);
                prev = p;
            }
            m.tube([0.0, -0.3, 0.0], [0.0, 0.3, 0.0], 0.003, 0.003, 4, col(240, 236, 220), false);
        }
        Wp::Mace => {
            m.tube([0.0, -0.1, 0.0], [0.0, 0.32, 0.0], 0.02, 0.02, 7, wood, true);
            m.ellipsoid([0.0, 0.4, 0.0], [0.07, 0.07, 0.07], 8, 12, &|_, _| col(110, 110, 120));
            for k in 0..8 {
                let a = k as f32 / 8.0 * TAU;
                m.tube([a.sin() * 0.06, 0.4, a.cos() * 0.06], [a.sin() * 0.11, 0.4, a.cos() * 0.11], 0.015, 0.0, 5, col(150, 150, 160), true);
            }
        }
        Wp::Torch => {
            m.tube([0.0, -0.1, 0.0], [0.0, 0.34, 0.0], 0.022, 0.03, 7, wood, true);
            m.flame(0.0, 0.34, 0.0, 0.22, 0.06);
        }
        Wp::Trident => {
            m.tube([0.0, -0.4, 0.0], [0.0, 0.6, 0.0], 0.014, 0.014, 6, wood, true);
            for dx in [-0.06f32, 0.0, 0.06] {
                m.tube([dx, 0.6, 0.0], [dx, 0.8, 0.0], 0.012, 0.0, 5, steel, true);
            }
            m.tube([-0.06, 0.6, 0.0], [0.06, 0.6, 0.0], 0.012, 0.012, 5, steel, false);
        }
    }
}

/// 高さを 1 に そろえる(はみ出す ものは 縮める)。fit=そろえる 高さ
pub fn finish(m: Mb, fit: f32) -> Model {
    let mut model = m.done();
    let (mut y1, mut xm, mut zm) = (0.0f32, 0.0f32, 0.0f32);
    for q in &model.quads {
        for c in &q.c {
            y1 = y1.max(c[1]);
            xm = xm.max(c[0].abs());
            zm = zm.max(c[2].abs());
        }
    }
    let k = (fit / y1.max(0.05)).min(0.62 / xm.max(0.05)).min(0.62 / zm.max(0.05));
    for q in model.quads.iter_mut() {
        for c in q.c.iter_mut() {
            *c = [c[0] * k, c[1] * k, c[2] * k];
        }
    }
    model.hand = true;
    model
}
