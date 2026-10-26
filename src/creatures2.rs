//! 魔物の 手組みモデル(その2): 人型でない もの(スライム・おばけ・やさい・どうぶつ など)と、全魔物の 表。

use crate::creatures::*;
use crate::model3d::Model;
use crate::modeling::*;
use macroquad::prelude::Color;
use std::f32::consts::{PI, TAU};

fn sph(m: &mut Mb, c: [f32; 3], r: [f32; 3], color: Color) {
    m.ellipsoid(c, r, 8, 12, &|th, _| shadec(color, 0.88 + 0.16 * (1.0 - th / PI)));
}

/// まるい 目(白目+ひとみ)
fn eyes2(m: &mut Mb, y: f32, z: f32, dx: f32, r: f32, pupil: Color) {
    for sx in [-1.0f32, 1.0] {
        m.ellipsoid([sx * dx, y, z], [r, r * 1.15, r * 0.6], 7, 10, &|_, _| col(252, 250, 244));
        m.ellipsoid([sx * dx, y - r * 0.1, z + r * 0.2], [r * 0.6, r * 0.75, r * 0.5], 6, 8, &|_, _| pupil);
    }
}

/// 回転体の 輪郭(半径, 高さ)から、ある 高さの 半径を とる
fn rad_at(prof: &[(f32, f32)], y: f32) -> f32 {
    for w in prof.windows(2) {
        let (a, b) = (w[0], w[1]);
        if y >= a.1 && y <= b.1 && b.1 > a.1 {
            let t = (y - a.1) / (b.1 - a.1);
            return a.0 + (b.0 - a.0) * t;
        }
    }
    0.0
}

fn legs_feet(m: &mut Mb, x: f32, hip_y: f32, z: f32, r: f32, c: Color) {
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * x, hip_y, z], [sx * x, 0.03, z], r, r * 0.9, 8, c, true);
        sph(m, [sx * x, 0.03, z + 0.04], [r * 1.3, r * 0.7, r * 1.9], shadec(c, 0.85));
    }
}

fn arms(m: &mut Mb, x: f32, y: f32, len: f32, r: f32, c: Color) {
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * x, y, 0.0], [sx * (x + len * 0.5), y - len * 0.6, 0.05], r, r * 0.8, 8, c, true);
        sph(m, [sx * (x + len * 0.5), y - len * 0.6, 0.05], [r * 1.2, r * 1.2, r * 1.2], c);
    }
}

/// スライム(しずく形の ぷるぷる)。k: 0 ふつう 1 ほのお 2 みず 3 どく
fn slime(color: Color, hi: Color, kind: u8) -> Model {
    let mut m = Mb::new();
    // なめらかな しずく形(kind 2 は ひくい ドーム)
    let (h, w, p) = if kind == 2 { (0.56f32, 0.5f32, 0.55f32) } else { (0.9, 0.46, 0.65) };
    let prof: Vec<(f32, f32)> = (0..=22)
        .map(|i| {
            let t = i as f32 / 22.0;
            let r = if kind == 2 {
                w * (1.0 - t.powf(2.4)).max(0.0).sqrt() * (t / 0.05).min(1.0).powf(0.5)
            } else {
                // 下が まるく ふくらみ、上が とがる しずく
                w * (PI * t.powf(0.6)).sin().max(0.0).powf(p)
            };
            (if i == 22 { 0.0 } else { r }, h * t)
        })
        .collect();
    let top = prof.last().unwrap().1;
    m.lathe(0.0, 0.0, 0.0, &prof, 20, &|i, j| mix(shadec(color, 0.82), hi, (i as f32 / prof.len() as f32) * 0.5 + 0.06 * (j % 2) as f32));
    // 顔: 体の 表面の すこし 手前に のせる(半径から 求める)
    let ey = top * if kind == 2 { 0.5 } else { 0.36 };
    let rz = |y: f32, dx: f32| (rad_at(&prof, y).powi(2) - dx * dx).max(0.0).sqrt();
    for sx in [-1.0f32, 1.0] {
        let dx = sx * 0.12;
        let z = rz(ey, dx.abs());
        m.ellipsoid([dx, ey, z], [0.07, 0.08, 0.035], 7, 10, &|_, _| col(252, 250, 244));
        m.ellipsoid([dx, ey - 0.008, z + 0.02], [0.042, 0.055, 0.03], 6, 8, &|_, _| col(30, 24, 20));
    }
    let my = ey - 0.1;
    m.ellipsoid([0.0, my, rz(my, 0.0)], [0.07, 0.026, 0.02], 4, 8, &|_, _| col(60, 30, 30));
    // つや
    let hy = top * 0.64;
    m.ellipsoid([-0.15, hy, rz(hy, 0.15) + 0.005], [0.04, 0.09, 0.02], 6, 8, &|_, _| col(255, 255, 255));
    match kind {
        1 => {
            for (x, z, h) in [(0.0f32, 0.0f32, 0.34f32), (0.12, 0.05, 0.22), (-0.12, 0.0, 0.26), (0.0, -0.1, 0.2)] {
                m.flame(x, top - 0.08, z, h, 0.1);
            }
        }
        3 => {
            let sy = top * 0.6;
            let z0 = rad_at(&prof, sy);
            sph(&mut m, [0.0, sy, z0 * 0.7], [0.15, 0.17, 0.12], col(240, 236, 220));
            for sx in [-1.0f32, 1.0] {
                sph(&mut m, [sx * 0.055, sy + 0.02, z0 * 0.7 + 0.1], [0.034, 0.042, 0.02], col(40, 24, 50));
            }
            for sx in [-0.28f32, 0.0, 0.28] {
                sph(&mut m, [sx, 0.02, 0.38], [0.04, 0.02, 0.04], shadec(color, 0.8));
            }
        }
        _ => {}
    }
    finish(m, 1.0)
}

/// おばけ(すそが 波うつ ローブ状)。k: 0 ふつう 1 ひ 2 こおり
fn ghost(color: Color, glow: Color, kind: u8) -> Model {
    let mut m = Mb::new();
    let prof: Vec<(f32, f32)> = (0..=20)
        .map(|i| {
            let t = i as f32 / 20.0;
            // すそが ひろがり、頭が まるい ローブ形
            let r = 0.26 + 0.12 * (1.0 - t).powf(2.0);
            let dome = (1.0 - ((t - 0.74) / 0.26).max(0.0).powi(2)).max(0.0).sqrt();
            (if i == 20 { 0.0 } else { r * dome.max(0.02) }, 0.8 * t)
        })
        .collect();
    m.lathe(0.0, 0.12, 0.0, &prof, 28, &|i, j| shadec(color, 0.9 + 0.1 * ((j as f32 * 1.7).sin()) - 0.02 * i as f32));
    // ひだの すそ
    for k in 0..10 {
        let a = k as f32 / 10.0 * TAU;
        m.tube([a.sin() * 0.3, 0.2, a.cos() * 0.3], [a.sin() * 0.34, 0.06 + 0.04 * (k % 2) as f32, a.cos() * 0.34], 0.05, 0.02, 6, shadec(color, 0.95), true);
    }
    // 顔(輪郭の 半径から 表面に のせる)
    let fy = 0.7;
    let gz = |y: f32, dx: f32| (rad_at(&prof.iter().map(|p| (p.0, p.1 + 0.12)).collect::<Vec<_>>(), y).powi(2) - dx * dx).max(0.0).sqrt();
    for sx in [-1.0f32, 1.0] {
        m.ellipsoid([sx * 0.1, fy, gz(fy, 0.1)], [0.06, 0.085, 0.035], 6, 8, &|_, _| if kind == 0 { col(40, 36, 40) } else { glow });
    }
    m.ellipsoid([0.0, fy - 0.15, gz(fy - 0.15, 0.0)], [0.11, 0.07, 0.03], 6, 10, &|_, _| col(40, 24, 30));
    // うで
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * 0.22, 0.5, 0.05], [sx * 0.4, 0.4, 0.14], 0.05, 0.035, 8, color, true);
        sph(&mut m, [sx * 0.4, 0.4, 0.14], [0.05, 0.04, 0.05], shadec(color, 0.95));
    }
    match kind {
        1 => {
            for (x, h) in [(0.0f32, 0.3f32), (0.12, 0.2), (-0.12, 0.22)] {
                m.flame(x, 0.85, -0.02, h, 0.09);
            }
        }
        2 => {
            for k in 0..9 {
                let a = k as f32 / 9.0 * TAU;
                m.tube([a.sin() * 0.2, 0.7, a.cos() * 0.2 - 0.03], [a.sin() * 0.32, 0.88 + 0.08 * (k % 3) as f32, a.cos() * 0.3 - 0.03], 0.05, 0.0, 6, col(210, 240, 255), true);
            }
        }
        _ => {}
    }
    finish(m, 1.0)
}

fn bat() -> Model {
    let mut m = Mb::new();
    let black = col(46, 42, 52);
    sph(&mut m, [0.0, 0.42, 0.0], [0.2, 0.22, 0.18], black);
    sph(&mut m, [0.0, 0.7, 0.04], [0.15, 0.14, 0.14], shadec(black, 1.05));
    m.tube([0.0, 0.7, 0.14], [0.0, 0.66, 0.28], 0.06, 0.0, 8, col(236, 186, 70), true);
    eyes2(&mut m, 0.74, 0.14, 0.07, 0.035, col(30, 24, 20));
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * 0.08, 0.26, 0.0], [sx * 0.09, 0.04, 0.03], 0.016, 0.012, 6, col(200, 170, 80), false);
        sph(&mut m, [sx * 0.09, 0.03, 0.06], [0.03, 0.015, 0.05], col(200, 170, 80));
        bat_wing(&mut m, sx, [sx * 0.17, 0.5, -0.02], 0.6, col(60, 54, 70), col(40, 36, 46));
    }
    m.tube([0.0, 0.34, -0.12], [0.0, 0.16, -0.26], 0.06, 0.03, 8, black, true);
    finish(m, 1.0)
}

fn potato() -> Model {
    let mut m = Mb::new();
    let skin = col(176, 140, 84);
    m.ellipsoid([0.0, 0.46, 0.0], [0.3, 0.36, 0.26], 10, 16, &|th, ph| mix(shadec(skin, 0.85), skin, (1.0 - th / PI) * 0.7 + 0.1 * (ph * 5.0).sin()));
    for k in 0..9 {
        let a = k as f32 * 2.1;
        sph(&mut m, [a.sin() * 0.22, 0.4 + 0.2 * (a * 1.3).cos(), 0.2 * a.cos().abs() + 0.08], [0.03, 0.025, 0.02], col(120, 90, 54));
    }
    eyes2(&mut m, 0.54, 0.22, 0.11, 0.06, col(40, 24, 20));
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * 0.2, 0.68, 0.24], [sx * 0.04, 0.62, 0.26], 0.025, 0.025, 6, col(70, 44, 30), false);
    }
    m.ellipsoid([0.0, 0.36, 0.25], [0.1, 0.04, 0.03], 5, 8, &|_, _| col(70, 30, 30));
    legs_feet(&mut m, 0.12, 0.2, 0.0, 0.05, col(150, 110, 70));
    arms(&mut m, 0.28, 0.42, 0.2, 0.04, col(176, 140, 84));
    // 導火線
    m.tube([0.0, 0.8, 0.0], [0.04, 0.95, 0.0], 0.014, 0.012, 6, col(80, 56, 36), false);
    m.flame(0.04, 0.94, 0.0, 0.1, 0.03);
    finish(m, 1.0)
}

fn carrot() -> Model {
    let mut m = Mb::new();
    m.tube([0.0, 0.2, 0.0], [0.0, 0.8, 0.0], 0.0, 0.2, 16, col(240, 130, 40), false);
    m.lathe(0.0, 0.8, 0.0, &[(0.2, 0.0), (0.18, 0.04), (0.0, 0.05)], 16, &|_, _| col(250, 150, 60));
    for k in 0..7 {
        let a = k as f32 / 7.0 * TAU;
        m.blade([a.sin() * 0.08, 0.84, a.cos() * 0.08], [a.sin() * 0.4, 1.0, a.cos() * 0.4], 0.5, 0.2, 0.04, col(70, 160, 70));
    }
    for y in [0.4f32, 0.55, 0.68] {
        m.lathe(0.0, y, 0.0, &[(0.12 + (y - 0.2) * 0.14, 0.0), (0.125 + (y - 0.2) * 0.14, 0.012)], 16, &|_, _| col(210, 100, 30));
    }
    eyes2(&mut m, 0.66, 0.17, 0.08, 0.05, col(30, 24, 20));
    m.ellipsoid([0.0, 0.54, 0.19], [0.08, 0.03, 0.03], 5, 8, &|_, _| col(80, 30, 30));
    arms(&mut m, 0.2, 0.55, 0.2, 0.035, col(240, 130, 40));
    legs_feet(&mut m, 0.06, 0.22, 0.0, 0.04, col(240, 130, 40));
    finish(m, 1.0)
}

fn pepper() -> Model {
    let mut m = Mb::new();
    let g = col(70, 150, 60);
    for (x, z) in [(-0.14f32, 0.0f32), (0.14, 0.0), (0.0, -0.12), (0.0, 0.12)] {
        sph(&mut m, [x, 0.42, z], [0.19, 0.3, 0.19], shadec(g, 0.92 + 0.1 * x));
    }
    sph(&mut m, [0.0, 0.42, 0.0], [0.24, 0.32, 0.24], g);
    m.tube([0.0, 0.7, 0.0], [0.03, 0.86, 0.0], 0.04, 0.025, 8, col(80, 120, 50), true);
    eyes2(&mut m, 0.5, 0.22, 0.1, 0.055, col(40, 20, 20));
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * 0.18, 0.62, 0.2], [sx * 0.04, 0.56, 0.26], 0.022, 0.022, 5, col(30, 70, 30), false);
    }
    // ぎざぎざの 口
    m.ellipsoid([0.0, 0.3, 0.26], [0.14, 0.07, 0.03], 5, 10, &|_, _| col(40, 16, 24));
    for k in 0..5 {
        let x = -0.1 + k as f32 * 0.05;
        m.tube([x, 0.36, 0.28], [x, 0.31, 0.29], 0.014, 0.0, 5, col(250, 248, 240), true);
    }
    legs_feet(&mut m, 0.09, 0.2, 0.0, 0.04, col(60, 120, 50));
    arms(&mut m, 0.25, 0.42, 0.16, 0.03, col(60, 120, 50));
    finish(m, 1.0)
}

fn apple(king: bool) -> Model {
    let mut m = Mb::new();
    let red = col(210, 44, 40);
    m.ellipsoid([0.0, 0.4, 0.0], [0.3, 0.3, 0.3], 12, 18, &|th, ph| mix(shadec(red, 0.82), col(240, 90, 70), (1.0 - th / PI) * 0.6 + 0.04 * (ph * 3.0).sin()));
    m.tube([0.0, 0.66, 0.0], [0.02, 0.8, 0.0], 0.025, 0.02, 6, col(80, 56, 30), true);
    m.blade([0.02, 0.76, 0.0], [1.0, 0.3, 0.0], 0.6, 0.18, 0.05, col(80, 160, 60));
    eyes2(&mut m, 0.46, 0.27, 0.1, 0.06, col(40, 20, 20));
    m.ellipsoid([0.0, 0.34, 0.29], [0.1, 0.04, 0.03], 5, 8, &|_, _| col(60, 24, 24));
    // へび(まわりに まきついている)
    let sn = col(90, 130, 56);
    let turns = if king { 2.4 } else { 1.7 };
    let mut prev = [0.3, 0.16, 0.0];
    for s in 1..=36 {
        let t = s as f32 / 36.0;
        let a = t * turns * TAU;
        let r = 0.33 - 0.05 * t;
        let p = [a.cos() * r, 0.14 + 0.52 * t, a.sin() * r];
        m.tube(prev, p, 0.034 - 0.006 * t, 0.034 - 0.006 * (t + 0.03), 7, if s % 2 == 0 { sn } else { shadec(sn, 0.85) }, false);
        prev = p;
    }
    sph(&mut m, add(prev, [0.0, 0.03, 0.0]), [0.05, 0.04, 0.06], sn);
    for sx in [-1.0f32, 1.0] {
        sph(&mut m, add(prev, [sx * 0.025, 0.05, 0.03]), [0.012, 0.012, 0.012], col(255, 220, 60));
    }
    m.tube(add(prev, [0.0, 0.03, 0.05]), add(prev, [0.0, 0.03, 0.1]), 0.005, 0.003, 4, col(220, 40, 60), false);
    if king {
        m.lathe(0.0, 0.72, 0.0, &[(0.14, 0.0), (0.15, 0.12)], 14, &|_, _| col(240, 200, 70));
        for k in 0..6 {
            let a = k as f32 / 6.0 * TAU;
            m.tube([a.sin() * 0.14, 0.82, a.cos() * 0.14], [a.sin() * 0.15, 0.96, a.cos() * 0.15], 0.04, 0.0, 6, col(240, 200, 70), true);
        }
    }
    legs_feet(&mut m, 0.1, 0.16, 0.02, 0.04, col(150, 40, 36));
    finish(m, 1.0)
}

fn killer_flower(fire: bool) -> Model {
    let mut m = Mb::new();
    let stem = col(80, 130, 60);
    m.tube([0.0, 0.0, 0.0], [0.0, 0.52, 0.0], 0.05, 0.04, 8, stem, false);
    for (s, y) in [(-1.0f32, 0.2f32), (1.0, 0.3)] {
        m.blade([0.0, y, 0.0], [s, 0.5, 0.0], 0.4, 0.28, 0.07, col(70, 140, 60));
    }
    let (pc, ic) = if fire { (col(240, 70, 40), col(255, 190, 60)) } else { (col(240, 120, 170), col(120, 30, 50)) };
    for k in 0..9 {
        let a = k as f32 / 9.0 * TAU;
        // はなびらは 上に そりあがる(横から 見ても ×に ならない)
        let tilt = if fire { -0.95 } else { -0.55 };
        m.at([0.0, 0.6, 0.0], -a, tilt, 0.0, |m| m.ellipsoid([0.0, 0.0, 0.24], [0.1, 0.05, 0.2], 6, 10, &|th, _| shadec(pc, 0.85 + 0.2 * (1.0 - th / PI))));
    }
    m.at([0.0, 0.6, 0.0], 0.0, -PI / 2.0 + 0.3, 0.0, |m| {
        m.lathe(0.0, 0.0, 0.0, &[(0.0, -0.06), (0.2, -0.04), (0.26, 0.0), (0.2, 0.06), (0.0, 0.08)], 16, &|_, _| shadec(pc, 0.95));
        m.lathe(0.0, 0.0, 0.0, &[(0.0, -0.05), (0.16, -0.03), (0.2, 0.01)], 16, &|_, _| ic);
        for k in 0..12 {
            let a = k as f32 / 12.0 * TAU;
            m.tube([a.sin() * 0.19, 0.0, a.cos() * 0.19], [a.sin() * 0.17, 0.0 + 0.13, a.cos() * 0.17], 0.025, 0.0, 5, col(250, 248, 240), true);
        }
    });
    if fire {
        for (x, h) in [(0.0f32, 0.3f32), (0.12, 0.2), (-0.12, 0.2)] {
            m.flame(x, 0.7, -0.05, h, 0.08);
        }
    }
    eyes2(&mut m, 0.7, 0.14, 0.12, 0.05, col(30, 20, 20));
    finish(m, 1.0)
}

fn caterpillar() -> Model {
    let mut m = Mb::new();
    let g = col(130, 190, 80);
    for i in 0..5 {
        let t = i as f32 / 4.0;
        let (z, y) = (-0.35 + 0.7 * t, 0.2 + 0.08 * (1.0 - (t - 0.5).abs() * 2.0));
        sph(&mut m, [0.0, y, -z], [0.2 - 0.02 * (i == 0) as i32 as f32, 0.2, 0.17], mix(g, col(200, 230, 120), (i % 2) as f32 * 0.4));
        for sx in [-1.0f32, 1.0] {
            m.tube([sx * 0.16, y - 0.12, -z], [sx * 0.19, 0.02, -z], 0.02, 0.016, 6, col(90, 140, 60), false);
        }
    }
    sph(&mut m, [0.0, 0.34, 0.4], [0.22, 0.22, 0.2], g);
    eyes2(&mut m, 0.38, 0.55, 0.08, 0.06, col(30, 24, 20));
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * 0.06, 0.52, 0.4], [sx * 0.12, 0.7, 0.42], 0.01, 0.01, 4, col(60, 100, 50), false);
        sph(&mut m, [sx * 0.12, 0.71, 0.42], [0.024, 0.024, 0.024], col(220, 60, 80));
    }
    finish(m, 1.0)
}

/// ニードルワーム: とげの ぼうぼうの まるっこい いも虫(体を おこして 前を にらむ)
fn needleworm() -> Model {
    let mut m = Mb::new();
    let skin = col(150, 182, 100);
    let spine = col(236, 232, 196);
    let mut rnd = Rnd(91);
    // 体: 下から 上へ まがりながら たちあがる 5つの ふし
    let segs: Vec<([f32; 3], f32)> = (0..6)
        .map(|i| {
            let t = i as f32 / 5.0;
            ([0.0, 0.14 + 0.62 * t, -0.28 * (1.0 - t).powi(2) + 0.05 * t], 0.24 - 0.025 * i as f32 + if i == 5 { 0.03 } else { 0.0 })
        })
        .collect();
    for (i, (c, r)) in segs.iter().enumerate() {
        m.ellipsoid(*c, [*r, *r * 0.9, *r], 9, 14, &|th, _| mix(shadec(skin, 0.82), shadec(skin, 1.12), 1.0 - th / PI + 0.05 * (i % 2) as f32));
        // 体の まわりの とげの わ
        for k in 0..9 {
            let a = k as f32 / 9.0 * TAU + i as f32 * 0.4;
            let (dx, dz) = (a.sin(), a.cos());
            let base = [c[0] + dx * r * 0.9, c[1] + 0.02, c[2] + dz * r * 0.9];
            let tip = [c[0] + dx * (r + 0.13), c[1] + 0.06 + 0.04 * rnd.next(), c[2] + dz * (r + 0.13)];
            m.tube(base, tip, 0.022, 0.0, 5, spine, true);
        }
    }
    // せなかの おおきな とげ
    for (i, (c, r)) in segs.iter().enumerate().skip(1) {
        m.tube([c[0], c[1] + r * 0.7, c[2] - r * 0.6], [c[0], c[1] + r + 0.12, c[2] - r - 0.08], 0.035, 0.0, 6, shadec(spine, 0.95), true);
        let _ = i;
    }
    let (hc, hr) = segs[5];
    // 顔: おこった 目と 口
    for sx in [-1.0f32, 1.0] {
        let z = hc[2] + (hr * hr - (0.09f32).powi(2)).max(0.0).sqrt() * 0.96;
        m.ellipsoid([sx * 0.09, hc[1] + 0.04, z], [0.05, 0.05, 0.03], 7, 10, &|_, _| col(252, 250, 240));
        m.ellipsoid([sx * 0.09, hc[1] + 0.035, z + 0.02], [0.028, 0.034, 0.022], 6, 8, &|_, _| col(30, 20, 20));
        m.tube([sx * 0.15, hc[1] + 0.12, z], [sx * 0.04, hc[1] + 0.085, z + 0.01], 0.014, 0.014, 5, shadec(skin, 0.5), false);
    }
    m.ellipsoid([0.0, hc[1] - 0.07, hc[2] + hr * 0.96], [0.08, 0.025, 0.02], 5, 8, &|_, _| col(70, 30, 30));
    // おなかの 足(ちいさな いぼ)
    for i in 0..3 {
        for sx in [-1.0f32, 1.0] {
            m.ellipsoid([sx * 0.1, 0.04 + 0.02 * i as f32, 0.12 - 0.12 * i as f32], [0.04, 0.03, 0.05], 5, 6, &|_, _| shadec(skin, 0.7));
        }
    }
    finish(m, 1.0)
}

/// ヒネズミ: 四つあしの 赤い ねずみ。せなかで たき火のように ほのおが 燃える
fn hinezumi() -> Model {
    let mut m = Mb::new();
    let fur = col(206, 78, 40);
    // からだ(よこ ながの まるい)
    m.ellipsoid([0.0, 0.3, -0.02], [0.24, 0.22, 0.38], 12, 18, &|th, ph| mix(shadec(fur, 0.82), shadec(fur, 1.14), 1.0 - th / PI + 0.05 * (ph * 6.0).sin()));
    // あたま・はな
    m.ellipsoid([0.0, 0.28, 0.4], [0.16, 0.15, 0.2], 10, 14, &|th, _| shadec(fur, 0.9 + 0.14 * (1.0 - th / PI)));
    m.tube([0.0, 0.27, 0.52], [0.0, 0.25, 0.68], 0.1, 0.03, 10, shadec(fur, 1.1), false);
    m.ellipsoid([0.0, 0.25, 0.69], [0.035, 0.03, 0.03], 6, 8, &|_, _| col(60, 28, 30));
    for sx in [-1.0f32, 1.0] {
        // 目
        m.ellipsoid([sx * 0.09, 0.34, 0.52], [0.03, 0.04, 0.025], 6, 8, &|_, _| col(26, 18, 18));
        m.ellipsoid([sx * 0.09 + sx * 0.004, 0.35, 0.538], [0.01, 0.012, 0.008], 4, 6, &|_, _| col(255, 255, 255));
        // 大きな まるい 耳
        m.at([sx * 0.13, 0.43, 0.36], 0.0, 0.0, sx * 0.4, |m| {
            m.ellipsoid([0.0, 0.08, 0.0], [0.09, 0.11, 0.025], 8, 12, &|th, _| shadec(fur, 0.9 + 0.1 * (1.0 - th / PI)));
            m.ellipsoid([0.0, 0.08, 0.02], [0.06, 0.08, 0.012], 6, 10, &|_, _| col(240, 160, 150));
        });
        // ひげ
        for k in 0..3 {
            let dy = (k as f32 - 1.0) * 0.03;
            m.tube([sx * 0.04, 0.26 + dy, 0.62], [sx * 0.2, 0.28 + dy * 2.0, 0.62], 0.004, 0.002, 4, col(240, 230, 220), false);
        }
        // まえあし・うしろあし
        for (z, big) in [(0.2f32, 0.0f32), (-0.22, 1.0)] {
            m.tube([sx * 0.16, 0.2, z], [sx * 0.17, 0.04, z + 0.03], 0.055 + 0.01 * big, 0.04, 8, shadec(fur, 0.9), true);
            m.ellipsoid([sx * 0.17, 0.025, z + 0.07], [0.05, 0.02, 0.08], 6, 8, &|_, _| col(236, 160, 150));
        }
    }
    // 長い しっぽ(うしろに のびて ゆるく まく)
    let mut prev = [0.0, 0.22, -0.36];
    for s2 in 1..=10 {
        let t = s2 as f32 / 10.0;
        let p = [0.12 * (t * 5.0).sin() * t, 0.2 + 0.02 * t - 0.14 * t * t, -0.36 - 0.38 * t];
        m.tube(prev, p, 0.03 * (1.0 - 0.7 * t) + 0.006, 0.03 * (1.0 - 0.7 * (t + 0.1)) + 0.005, 6, col(236, 150, 140), s2 == 10);
        prev = p;
    }
    // せなかの たき火(まきと 大きな ほのお)
    for (x, z, a, l) in [(-0.06f32, -0.05f32, 0.5f32, 0.18f32), (0.05, 0.05, -0.6, 0.16), (0.0, -0.15, 1.6, 0.16)] {
        m.at([x, 0.5, z], a, 0.0, 1.3, |m| m.tube([-l / 2.0, 0.0, 0.0], [l / 2.0, 0.0, 0.0], 0.022, 0.022, 7, col(96, 62, 36), true));
    }
    for (x, z, h, w) in [(0.0f32, 0.0f32, 0.46f32, 0.12f32), (0.08, -0.06, 0.3, 0.09), (-0.08, 0.02, 0.34, 0.1), (0.0, -0.14, 0.26, 0.08), (0.05, 0.1, 0.22, 0.07), (-0.05, -0.1, 0.4, 0.09)] {
        m.flame(x, 0.46, z, h, w);
    }
    for (x, z) in [(0.1f32, 0.0f32), (-0.1, -0.1), (0.0, 0.1), (0.06, -0.15)] {
        m.ellipsoid([x, 0.5, z], [0.025, 0.012, 0.025], 4, 6, &|_, _| col(255, 90, 40));
    }
    finish(m, 1.0)
}

/// でたらめサイコロ: 面ごとに ちがう 色の サイコロ(目も でたらめ)
fn detarame_dice() -> Model {
    let mut m = Mb::new();
    let cols = [col(220, 56, 52), col(60, 110, 220), col(70, 180, 90), col(240, 200, 60), col(170, 90, 210), col(240, 140, 50)];
    let (lo, hi, c0) = (0.18f32, 0.7f32, 0.0f32);
    let h = (hi - lo) / 2.0;
    let cy = lo + h;
    let f = |m: &mut Mb, corners: [[f32; 3]; 4], n: [f32; 3], c: Color| m.quad(corners, [n; 4], [c; 4], false);
    let (x0, x1, z0, z1) = (c0 - h, c0 + h, c0 - h, c0 + h);
    f(&mut m, [[x0, lo, z1], [x1, lo, z1], [x1, hi, z1], [x0, hi, z1]], [0.0, 0.0, 1.0], cols[0]);
    f(&mut m, [[x1, lo, z0], [x0, lo, z0], [x0, hi, z0], [x1, hi, z0]], [0.0, 0.0, -1.0], cols[1]);
    f(&mut m, [[x1, lo, z1], [x1, lo, z0], [x1, hi, z0], [x1, hi, z1]], [1.0, 0.0, 0.0], cols[2]);
    f(&mut m, [[x0, lo, z0], [x0, lo, z1], [x0, hi, z1], [x0, hi, z0]], [-1.0, 0.0, 0.0], cols[3]);
    f(&mut m, [[x0, hi, z1], [x1, hi, z1], [x1, hi, z0], [x0, hi, z0]], [0.0, 1.0, 0.0], cols[4]);
    f(&mut m, [[x0, lo, z0], [x1, lo, z0], [x1, lo, z1], [x0, lo, z1]], [0.0, -1.0, 0.0], cols[5]);
    eyes2(&mut m, cy + 0.08, h + 0.005, 0.12, 0.06, col(40, 20, 20));
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * 0.2, cy + 0.2, h + 0.01], [sx * 0.06, cy + 0.15, h + 0.01], 0.018, 0.018, 5, col(40, 20, 20), false);
    }
    m.ellipsoid([0.0, cy - 0.1, h + 0.005], [0.15, 0.05, 0.02], 5, 10, &|_, _| col(50, 16, 24));
    for k in 0..5 {
        let x = -0.1 + k as f32 * 0.05;
        m.tube([x, cy - 0.065, h + 0.012], [x, cy - 0.12, h + 0.014], 0.014, 0.0, 5, col(250, 248, 240), true);
    }
    for (x, z) in [(-0.1f32, -0.1f32), (0.1, 0.1), (0.0, 0.0), (-0.1, 0.1), (0.1, -0.1)] {
        m.ellipsoid([x, hi + 0.003, z], [0.04, 0.008, 0.04], 5, 8, &|_, _| col(250, 250, 244));
    }
    for (y, z) in [(cy + 0.08, -0.08f32), (cy - 0.08, 0.08)] {
        m.ellipsoid([h + 0.003, y, z], [0.008, 0.04, 0.04], 5, 8, &|_, _| col(250, 250, 244));
    }
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * h, cy - 0.05, 0.0], [sx * (h + 0.1), cy - 0.2, 0.05], 0.03, 0.025, 7, col(120, 60, 70), true);
    }
    legs_feet(&mut m, 0.12, lo + 0.02, 0.0, 0.04, col(120, 60, 70));
    finish(m, 1.0)
}

fn daruma(color: Color, kind: u8) -> Model {
    let mut m = Mb::new();
    let snow = kind == 2;
    let body = if snow { col(214, 238, 252) } else { color };
    m.ellipsoid([0.0, 0.42, 0.0], [0.36, 0.4, 0.34], 12, 18, &|th, ph| shadec(body, 0.86 + 0.18 * (1.0 - th / PI) + if snow { 0.04 * (ph * 9.0).sin() } else { 0.0 }));
    if snow {
        for k in 0..30 {
            let a = k as f32 * 2.4;
            let y = 0.12 + 0.6 * ((k as f32 * 0.37) % 1.0);
            m.blade([a.sin() * 0.33, y, a.cos() * 0.31], [a.sin(), 0.2, a.cos()], 0.3, 0.08, 0.025, col(236, 248, 255));
        }
        eyes2(&mut m, 0.55, 0.3, 0.12, 0.06, col(30, 40, 70));
        m.tube([0.0, 0.46, 0.32], [0.0, 0.43, 0.5], 0.05, 0.0, 8, col(250, 140, 50), true);
        m.ellipsoid([0.0, 0.34, 0.31], [0.08, 0.03, 0.03], 5, 8, &|_, _| col(70, 60, 90));
    } else {
        // 白い 顔の お面
        m.ellipsoid([0.0, 0.54, 0.22], [0.25, 0.22, 0.16], 10, 14, &|_, _| col(252, 244, 232));
        for sx in [-1.0f32, 1.0] {
            m.ellipsoid([sx * 0.1, 0.58, 0.34], [0.07, 0.045, 0.03], 5, 8, &|_, _| col(30, 20, 20));
            m.tube([sx * 0.18, 0.7, 0.33], [sx * 0.04, 0.63, 0.35], 0.03, 0.02, 6, col(30, 20, 20), false);
            m.tube([sx * 0.12, 0.66, 0.33], [sx * 0.16, 0.52, 0.34], 0.018, 0.012, 5, col(30, 20, 20), false);
        }
        m.ellipsoid([0.0, 0.42, 0.36], [0.1, 0.035, 0.03], 5, 8, &|_, _| col(160, 40, 40));
        m.ellipsoid([0.0, 0.2, 0.3], [0.16, 0.06, 0.05], 5, 10, &|_, _| col(236, 200, 70));
        if kind == 1 {
            for (x, h) in [(0.0f32, 0.34f32), (0.14, 0.22), (-0.14, 0.22), (0.07, 0.28)] {
                m.flame(x, 0.74, 0.0, h, 0.1);
            }
        }
    }
    finish(m, 1.0)
}

fn octopus() -> Model {
    let mut m = Mb::new();
    let c = col(70, 110, 190);
    m.ellipsoid([0.0, 0.6, 0.0], [0.3, 0.32, 0.3], 12, 16, &|th, ph| mix(shadec(c, 0.8), col(130, 170, 240), (1.0 - th / PI) * 0.6 + 0.04 * (ph * 7.0).sin()));
    eyes2(&mut m, 0.56, 0.26, 0.12, 0.07, col(30, 40, 20));
    m.ellipsoid([0.0, 0.42, 0.28], [0.1, 0.03, 0.03], 5, 8, &|_, _| col(40, 30, 50));
    for k in 0..8 {
        let a = k as f32 / 8.0 * TAU + 0.2;
        let mut prev = [a.sin() * 0.14, 0.4, a.cos() * 0.14];
        for s in 1..=7 {
            let t = s as f32 / 7.0;
            let r = 0.14 + 0.3 * t + 0.05 * (t * 5.0).sin();
            let p = [a.sin() * r, 0.4 - 0.34 * t + 0.1 * (t * PI).sin(), a.cos() * r];
            m.tube(prev, p, 0.06 * (1.0 - 0.7 * t) + 0.01, 0.06 * (1.0 - 0.7 * (t + 0.15)) + 0.01, 8, mix(c, col(170, 120, 190), t * 0.4), s == 7);
            prev = p;
        }
    }
    finish(m, 1.0)
}

fn elephantom() -> Model {
    let mut m = Mb::new();
    let c = col(176, 186, 190);
    sph(&mut m, [0.0, 0.5, 0.0], [0.3, 0.26, 0.4], c);
    for (x, z) in [(-0.16f32, -0.24f32), (0.16, -0.24), (-0.16, 0.24), (0.16, 0.24)] {
        m.tube([x, 0.4, z], [x, 0.04, z], 0.09, 0.08, 10, shadec(c, 0.9), true);
    }
    sph(&mut m, [0.0, 0.62, 0.38], [0.2, 0.22, 0.2], c);
    for sx in [-1.0f32, 1.0] {
        m.at([sx * 0.22, 0.66, 0.34], sx * 0.7, 0.0, sx * 0.1, |m| m.ellipsoid([sx * 0.1, 0.0, 0.0], [0.05, 0.22, 0.2], 8, 12, &|_, _| shadec(col(190, 176, 184), 0.95)));
    }
    let mut prev = [0.0, 0.58, 0.54];
    for s in 1..=8 {
        let t = s as f32 / 8.0;
        let p = [0.0, 0.58 - 0.5 * t * t, 0.54 + 0.18 * (t * PI * 0.6).sin() + 0.04 * t];
        m.tube(prev, p, 0.07 * (1.0 - 0.5 * t), 0.07 * (1.0 - 0.5 * (t + 0.12)), 8, c, false);
        prev = p;
    }
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * 0.1, 0.52, 0.5], [sx * 0.14, 0.34, 0.7], 0.03, 0.0, 6, col(248, 240, 220), true);
        glow(&mut m, [sx * 0.1, 0.68, 0.54], 0.04);
    }
    m.tube([0.0, 0.5, -0.4], [0.0, 0.22, -0.46], 0.03, 0.015, 6, c, true);
    // 緑の 炎(おばけ)
    for (x, z) in [(-0.1f32, 0.0f32), (0.12, -0.12)] {
        m.flame(x, 0.7, z, 0.18, 0.06);
    }
    finish(m, 1.0)
}

fn glow(m: &mut Mb, c: [f32; 3], r: f32) {
    m.ellipsoid(c, [r, r, r * 0.6], 5, 8, &|_, _| col(120, 255, 170));
}

fn buffalo() -> Model {
    let mut m = Mb::new();
    let c = col(88, 62, 44);
    sph(&mut m, [0.0, 0.5, -0.02], [0.3, 0.28, 0.42], c);
    sph(&mut m, [0.0, 0.66, 0.16], [0.28, 0.24, 0.28], shadec(c, 0.92));
    for (x, z) in [(-0.14f32, -0.26f32), (0.14, -0.26), (-0.14, 0.26), (0.14, 0.26)] {
        m.tube([x, 0.4, z], [x, 0.04, z], 0.08, 0.06, 9, shadec(c, 0.9), true);
        sph(&mut m, [x, 0.03, z + 0.02], [0.08, 0.04, 0.09], col(40, 30, 26));
    }
    sph(&mut m, [0.0, 0.48, 0.46], [0.2, 0.22, 0.22], shadec(c, 1.08));
    sph(&mut m, [0.0, 0.4, 0.62], [0.12, 0.1, 0.1], col(130, 100, 84));
    for sx in [-1.0f32, 1.0] {
        sph(&mut m, [sx * 0.04, 0.4, 0.69], [0.025, 0.03, 0.02], col(30, 20, 20));
        m.tube([sx * 0.16, 0.6, 0.44], [sx * 0.34, 0.66, 0.44], 0.05, 0.04, 8, col(238, 226, 196), false);
        m.tube([sx * 0.34, 0.66, 0.44], [sx * 0.4, 0.86, 0.5], 0.04, 0.0, 8, col(238, 226, 196), true);
        sph(&mut m, [sx * 0.1, 0.55, 0.62], [0.035, 0.04, 0.03], col(250, 240, 220));
        sph(&mut m, [sx * 0.1, 0.55, 0.64], [0.02, 0.025, 0.02], col(20, 14, 12));
        sph(&mut m, [sx * 0.2, 0.52, 0.4], [0.06, 0.04, 0.1], shadec(c, 0.8));
    }
    m.tube([0.0, 0.58, -0.4], [0.0, 0.3, -0.46], 0.025, 0.02, 6, c, true);
    finish(m, 1.0)
}

fn penguin() -> Model {
    let mut m = Mb::new();
    let (blk, wht) = (col(40, 44, 60), col(248, 248, 244));
    sph(&mut m, [0.0, 0.42, 0.0], [0.28, 0.4, 0.24], blk);
    sph(&mut m, [0.0, 0.4, 0.07], [0.22, 0.34, 0.2], wht);
    sph(&mut m, [0.0, 0.8, 0.0], [0.2, 0.18, 0.18], blk);
    sph(&mut m, [0.0, 0.76, 0.1], [0.15, 0.14, 0.1], wht);
    m.tube([0.0, 0.76, 0.17], [0.0, 0.72, 0.34], 0.06, 0.0, 8, col(244, 160, 40), true);
    eyes2(&mut m, 0.84, 0.15, 0.08, 0.04, col(20, 20, 24));
    for sx in [-1.0f32, 1.0] {
        m.at([sx * 0.27, 0.5, 0.0], 0.0, 0.0, sx * 0.5, |m| m.ellipsoid([sx * 0.04, -0.12, 0.0], [0.03, 0.17, 0.07], 6, 8, &|_, _| col(34, 38, 52)));
        sph(&mut m, [sx * 0.1, 0.03, 0.1], [0.07, 0.025, 0.1], col(244, 160, 40));
    }
    m.at([0.3, 0.3, 0.12], 0.0, -0.3, -0.1, |m| {
        m.tube([0.0, -0.3, 0.0], [0.0, 0.56, 0.0], 0.014, 0.014, 6, col(124, 84, 46), true);
        for dx in [-0.06f32, 0.0, 0.06] {
            m.tube([dx, 0.56, 0.0], [dx, 0.76, 0.0], 0.012, 0.0, 5, col(200, 204, 214), true);
        }
        m.tube([-0.06, 0.56, 0.0], [0.06, 0.56, 0.0], 0.012, 0.012, 5, col(200, 204, 214), false);
    });
    m.at([-0.34, 0.4, 0.12], 0.4, PI / 2.0, 0.0, |m| m.lathe(0.0, 0.0, 0.0, &[(0.0, 0.03), (0.15, 0.01), (0.17, 0.0), (0.17, -0.02), (0.0, -0.02)], 18, &|_, _| col(130, 120, 100)));
    finish(m, 1.0)
}

fn platypus() -> Model {
    let mut m = Mb::new();
    let c = col(108, 82, 62);
    sph(&mut m, [0.0, 0.3, 0.0], [0.32, 0.24, 0.4], c);
    sph(&mut m, [0.0, 0.42, 0.34], [0.2, 0.18, 0.18], shadec(c, 1.05));
    sph(&mut m, [0.0, 0.34, 0.56], [0.2, 0.04, 0.2], col(60, 50, 56));
    eyes2(&mut m, 0.5, 0.42, 0.1, 0.04, col(20, 16, 16));
    m.ellipsoid([0.0, 0.2, -0.46], [0.16, 0.025, 0.24], 5, 10, &|_, _| shadec(c, 0.8));
    for (x, z) in [(-0.2f32, -0.2f32), (0.2, -0.2), (-0.2, 0.24), (0.2, 0.24)] {
        m.tube([x, 0.18, z], [x * 1.1, 0.03, z], 0.05, 0.04, 8, shadec(c, 0.9), true);
        sph(&mut m, [x * 1.1, 0.025, z + 0.04], [0.07, 0.02, 0.09], col(60, 50, 56));
    }
    // バンダナ
    m.lathe(0.0, 0.46, 0.32, &[(0.14, 0.0), (0.16, 0.0), (0.16, 0.06), (0.14, 0.06)], 14, &|_, _| col(120, 30, 40));
    m.tube([0.0, 0.5, 0.2], [0.0, 0.46, 0.08], 0.03, 0.0, 5, col(120, 30, 40), true);
    m.at([0.28, 0.4, 0.1], 0.0, -0.2, 0.0, |m| m.rbox(0.0, 0.0, 0.0, 0.02, 0.2, 0.01, 0.002, col(200, 204, 214)));
    finish(m, 1.0)
}

fn mole() -> Model {
    let mut m = Mb::new();
    let c = col(122, 98, 74);
    sph(&mut m, [0.0, 0.4, 0.0], [0.36, 0.38, 0.34], c);
    sph(&mut m, [0.0, 0.52, 0.22], [0.2, 0.2, 0.2], shadec(c, 1.05));
    m.tube([0.0, 0.5, 0.3], [0.0, 0.46, 0.48], 0.13, 0.05, 10, col(236, 176, 170), true);
    sph(&mut m, [0.0, 0.46, 0.5], [0.05, 0.04, 0.04], col(240, 150, 150));
    for sx in [-1.0f32, 1.0] {
        sph(&mut m, [sx * 0.1, 0.6, 0.34], [0.02, 0.025, 0.015], col(20, 16, 16));
        sph(&mut m, [sx * 0.18, 0.64, 0.16], [0.05, 0.06, 0.03], shadec(c, 0.85));
        m.tube([sx * 0.3, 0.42, 0.04], [sx * 0.44, 0.18, 0.2], 0.08, 0.07, 9, col(236, 176, 170), true);
        for k in -1..=1 {
            m.tube([sx * 0.44 + k as f32 * 0.04, 0.16, 0.22], [sx * 0.46 + k as f32 * 0.05, 0.08, 0.36], 0.02, 0.0, 5, col(250, 246, 230), true);
        }
        sph(&mut m, [sx * 0.14, 0.04, 0.0], [0.1, 0.04, 0.14], col(236, 176, 170));
    }
    // ヘルメット
    m.ellipsoid([0.0, 0.66, 0.16], [0.2, 0.12, 0.2], 8, 12, &|_, _| col(216, 190, 80));
    m.flame(0.0, 0.74, 0.2, 0.08, 0.025);
    finish(m, 1.0)
}

fn dice() -> Model {
    let mut m = Mb::new();
    let c = col(214, 44, 48);
    m.rbox(0.0, 0.2, 0.0, 0.5, 0.5, 0.5, 0.06, c);
    m.bx(0.0, 0.2, 0.0, 0.5, 0.5, 0.5, c, true);
    for (x, y) in [(-0.12f32, 0.56f32), (0.12, 0.56), (0.0, 0.46)] {
        sph(&mut m, [x, y, 0.255], [0.045, 0.045, 0.01], col(250, 248, 240));
    }
    eyes2(&mut m, 0.5, 0.26, 0.12, 0.06, col(255, 220, 60));
    m.ellipsoid([0.0, 0.34, 0.26], [0.14, 0.05, 0.02], 5, 10, &|_, _| col(40, 16, 24));
    for k in 0..4 {
        let x = -0.09 + k as f32 * 0.06;
        m.tube([x, 0.39, 0.27], [x, 0.33, 0.275], 0.014, 0.0, 5, col(250, 248, 240), true);
    }
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * 0.18, 0.7, 0.1], [sx * 0.28, 0.9, 0.0], 0.06, 0.0, 6, col(60, 40, 50), true);
        bat_wing(&mut m, sx, [sx * 0.25, 0.5, -0.1], 0.4, col(130, 30, 60), col(60, 20, 30));
    }
    legs_feet(&mut m, 0.12, 0.22, 0.0, 0.04, col(60, 40, 50));
    finish(m, 1.0)
}

fn magic_shield() -> Model {
    let mut m = Mb::new();
    m.at([0.0, 0.5, 0.0], 0.0, PI / 2.0, 0.0, |m| {
        // あつみの ある まるい たて(ドーム + 太い ふち + うらがわ)
        m.lathe(0.0, 0.0, 0.0, &[(0.0, 0.17), (0.1, 0.165), (0.2, 0.14), (0.3, 0.09), (0.36, 0.04), (0.38, 0.0), (0.36, -0.07), (0.2, -0.06), (0.0, -0.05)], 28, &|i, _| if i < 5 { col(216, 200, 170) } else { col(120, 100, 76) });
        m.lathe(0.0, 0.0, 0.0, &[(0.37, 0.05), (0.39, 0.02), (0.39, -0.05), (0.36, -0.08), (0.34, -0.03), (0.34, 0.05)], 28, &|_, _| col(236, 190, 70));
        m.lathe(0.0, 0.0, 0.0, &[(0.0, 0.18), (0.06, 0.175), (0.08, 0.16), (0.0, 0.16)], 14, &|_, _| col(236, 190, 70));
        m.lathe(0.0, 0.0, 0.0, &[(0.19, 0.145), (0.215, 0.135), (0.215, 0.125), (0.19, 0.13)], 28, &|_, _| col(236, 190, 70));
    });
    // 顔
    eyes2(&mut m, 0.54, 0.2, 0.1, 0.06, col(60, 40, 120));
    m.ellipsoid([0.0, 0.4, 0.2], [0.1, 0.035, 0.02], 5, 10, &|_, _| col(150, 50, 60));
    for sx in [-1.0f32, 1.0] {
        m.tube([sx * 0.38, 0.5, 0.0], [sx * 0.52, 0.7, -0.06], 0.04, 0.0, 6, col(236, 190, 70), true);
    }
    legs_feet(&mut m, 0.1, 0.18, 0.0, 0.04, col(120, 100, 76));
    arms(&mut m, 0.36, 0.4, 0.18, 0.03, col(120, 100, 76));
    finish(m, 1.0)
}

fn dorunuma() -> Model {
    let mut m = Mb::new();
    let c = col(92, 70, 44);
    sph(&mut m, [0.0, 0.45, 0.0], [0.4, 0.42, 0.36], c);
    let mut rnd = Rnd(77);
    for _ in 0..140 {
        let a = rnd.range(0.0, TAU);
        let b = rnd.range(0.2, PI * 0.62);
        let p = [0.4 * b.sin() * a.cos(), 0.45 + 0.42 * b.cos(), 0.36 * b.sin() * a.sin()];
        let d = norm([p[0], p[1] - 0.45, p[2]]);
        m.blade(p, add(d, [0.0, 0.2, 0.0]), rnd.range(0.2, 0.7), rnd.range(0.1, 0.2), 0.02, mix(col(120, 90, 56), col(70, 52, 34), rnd.next()));
    }
    for sx in [-1.0f32, 1.0] {
        m.ellipsoid([sx * 0.13, 0.55, 0.34], [0.07, 0.07, 0.04], 6, 8, &|_, _| col(255, 230, 70));
        m.tube([sx * 0.36, 0.45, 0.1], [sx * 0.5, 0.2, 0.22], 0.08, 0.07, 8, c, true);
        sph(&mut m, [sx * 0.5, 0.2, 0.22], [0.09, 0.08, 0.09], shadec(c, 0.85));
    }
    m.ellipsoid([0.0, 0.36, 0.36], [0.14, 0.05, 0.03], 5, 10, &|_, _| col(30, 20, 16));
    legs_feet(&mut m, 0.16, 0.14, 0.0, 0.07, shadec(c, 0.8));
    finish(m, 1.0)
}

/// 魔物の 絵の名前から 手組みモデルを 作る(ないものは None)
/// 魔物の モデル(ボス級は 威厳の そうしょく つき)
pub fn creature_model(name: &str) -> Option<Model> {
    crate::bosses::boss_model(name).or_else(|| creature_base(name))
}

/// ボスの そうしょくを つける 前の モデル
pub fn creature_base(name: &str) -> Option<Model> {
    // ショーグンウルフ(ボス)の ふつうの形は 通常個体と同じ
    let name = if name == "mon_shogunboss" { "mon_shogunwolf" } else { name };
    let g = col;
    let h = |fur: Color, cloth: Color| hum(fur, cloth);
    let flame = g(255, 140, 40);
    Some(match name {
        "mon_bat" => bat(),
        "mon_detaramedice" => detarame_dice(),
        "mon_shieldmaiden" => build_hum(&Hum { hat: Hat::Helm, armor: Some(g(196, 200, 212)), wp: Wp::Sword, shield: Some(g(176, 180, 196)), hair: Some(g(60, 46, 36)), spikes: false, lean: -0.1, stance: 1.3, arm_pose: 1, body: 1.1, accent: g(236, 200, 90), trim_col: Some(g(236, 200, 90)), seed: 71, ..hum(g(246, 224, 204), g(176, 180, 196)) }),
        "mon_needleworm" => needleworm(),
        "mon_hinezumi" => hinezumi(),
        "mon_pig" => build_hum(&Hum { lean: 0.3, stance: 1.35, arm_pose: 4, tusks: true, arm: 0.8, snout: Snout::Pig, ear: Ear::Dog, body: 1.55, belly: 0.3, head: 1.2, legs: 0.6, fangs: false, eyes: Eyes::Angry, mane: Some(g(96, 62, 38)), seed: 2, ..h(g(150, 96, 60), g(138, 88, 54)) }),
        "mon_cat" => build_hum(&Hum { lean: 0.35, stance: 0.9, arm_pose: 1, ear: Ear::Cat, snout: Snout::Short, tails: 1, wp: Wp::Dagger, seed: 3, ..h(g(168, 144, 108), g(112, 80, 140)) }),
        "mon_gob" => build_hum(&Hum { lean: 0.5, stance: 1.35, arm_pose: 4, arm: 1.5, ear: Ear::Point, hat: Hat::Helm, armor: Some(g(130, 120, 100)), wp: Wp::Club, fangs: true, head: 1.15, legs: 0.8, eyes: Eyes::Angry, seed: 4, ..h(g(108, 150, 70), g(110, 90, 60)) }),
        "mon_slime" => slime(g(120, 190, 60), g(200, 240, 110), 0),
        "mon_wan" => build_hum(&Hum { lean: -0.12, stance: 1.3, arm_pose: 1, ear: Ear::Dog, snout: Snout::Dog, tails: 1, wp: Wp::Sword, shield: Some(g(120, 90, 60)), seed: 5, ..h(g(184, 124, 72), g(120, 84, 56)) }),
        "mon_gobl" => build_hum(&Hum { lean: 0.4, stance: 1.3, arm_pose: 4, arm: 1.4, ear: Ear::Point, hat: Hat::Helm, armor: Some(g(96, 86, 76)), wp: Wp::Mace, fangs: true, body: 1.2, eyes: Eyes::Angry, seed: 6, ..h(g(96, 130, 70), g(84, 70, 56)) }),
        "mon_ghost" => ghost(g(246, 246, 250), g(40, 36, 40), 0),
        "mon_potato" => potato(),
        "mon_lion" => build_hum(&Hum { lean: -0.1, stance: 1.2, arm_pose: 2, mane: Some(g(200, 100, 40)), ear: Ear::Round, snout: Snout::Short, tails: 1, armor: Some(g(120, 84, 60)), body: 1.2, fangs: true, seed: 7, ..h(g(222, 156, 66), g(120, 80, 56)) }),
        "mon_hibuta" => build_hum(&Hum { lean: 0.3, stance: 1.2, arm_pose: 1, snout: Snout::Pig, ear: Ear::Round, mane: Some(flame), belly: 0.18, body: 1.1, fangs: true, seed: 8, ..h(g(234, 142, 50), g(228, 130, 46)) }),
        "mon_ningen" => build_hum(&Hum { lean: 0.55, stance: 1.1, arm_pose: 4, snout: Snout::Dog, tails: 1, tail_long: true, eyes: Eyes::Dot, belly: 0.1, wp: Wp::No, seed: 9, ..h(g(120, 152, 90), g(150, 124, 80)) }),
        "mon_firelime" => slime(g(240, 70, 40), g(255, 170, 70), 1),
        "mon_hotghost" => ghost(g(250, 120, 80), g(255, 230, 90), 1),
        "mon_nekomata" => build_hum(&Hum { lean: 0.2, stance: 0.9, arm_pose: 3, ear: Ear::Cat, snout: Snout::Short, tails: 2, wp: Wp::Bow, seed: 10, ..h(g(222, 208, 182), g(112, 82, 142)) }),
        "mon_waterlime" => slime(g(60, 140, 230), g(150, 220, 255), 2),
        "mon_coldghost" => ghost(g(190, 226, 250), g(230, 250, 255), 2),
        "mon_otter" => build_hum(&Hum { lean: 0.1, stance: 1.0, arm_pose: 1, ear: Ear::Round, snout: Snout::Short, tails: 1, tail_long: true, armor: Some(g(70, 100, 160)), wp: Wp::Sword, shield: Some(g(90, 110, 150)), seed: 11, ..h(g(132, 100, 70), g(60, 90, 150)) }),
        "mon_carrot" => carrot(),
        "mon_crowcat" => build_hum(&Hum { lean: 0.3, stance: 0.9, arm_pose: 3, ear: Ear::Cat, snout: Snout::Short, tails: 1, wings: Some(g(70, 64, 82)), eyes: Eyes::Glow, eye: g(255, 220, 70), seed: 12, ..h(g(46, 44, 52), g(54, 50, 62)) }),
        "mon_blackcat" => build_hum(&Hum { lean: 0.45, stance: 0.85, arm_pose: 1, ear: Ear::Cat, snout: Snout::Short, tails: 1, wp: Wp::Dagger, eyes: Eyes::Glow, eye: g(240, 210, 60), seed: 13, ..h(g(40, 40, 46), g(48, 46, 58)) }),
        "mon_wizdrill" => build_hum(&Hum { lean: 0.15, arm_pose: 3, ear: Ear::Point, hat: Hat::Wizard, wp: Wp::Bow, body: 0.9, head: 1.1, seed: 14, ..h(g(196, 184, 150), g(160, 150, 118)) }),
        "mon_blackwar" => build_hum(&Hum { lean: 0.05, stance: 1.25, arm_pose: 1, spikes: true, hat: Hat::Helm, armor: Some(g(52, 52, 62)), horn: Horn::Demon, wp: Wp::Sword, wp_col: g(120, 120, 134), eyes: Eyes::Glow, eye: g(255, 200, 60), body: 1.25, seed: 15, ..h(g(40, 40, 48), g(46, 46, 54)) }),
        "mon_bone" => build_hum(&Hum { lean: -0.05, stance: 0.9, arm_pose: 1, skele: true, wp: Wp::Sword, shield: Some(g(120, 96, 70)), eyes: Eyes::Dot, eye: g(20, 16, 16), head: 1.05, seed: 16, ..h(g(238, 232, 214), g(200, 194, 176)) }),
        "mon_dino" => build_hum(&Hum { lean: 0.5, stance: 1.3, arm_pose: 2, snout: Snout::Short, horn: Horn::Small, tails: 1, tail_long: true, belly: 0.25, body: 1.3, head: 1.15, legs: 0.8, fangs: true, eyes: Eyes::Angry, seed: 17, ..h(g(204, 52, 42), g(214, 74, 52)) }),
        "mon_sorcerer" => build_hum(&Hum { lean: 0.1, arm_pose: 3, ear: Ear::Round, snout: Snout::Short, tails: 1, tail_long: true, wp: Wp::Staff, accent: g(120, 200, 255), cape: Some(g(40, 70, 130)), seed: 18, ..h(g(132, 100, 70), g(50, 80, 140)) }),
        "mon_tetsu" => build_hum(&Hum { lean: 0.1, stance: 1.35, arm_pose: 4, hat: Hat::Helm, armor: Some(g(98, 100, 110)), body: 1.65, head: 0.8, arm: 1.3, belly: 0.1, eyes: Eyes::Glow, eye: g(255, 190, 60), seed: 19, ..h(g(88, 90, 100), g(72, 72, 82)) }),
        "mon_boss" => apple(false),
        "mon_chikaraushi" => build_hum(&Hum { lean: 0.3, stance: 1.3, arm_pose: 2, horn: Horn::Bull, ear: Ear::Dog, snout: Snout::Pig, body: 1.5, arm: 1.3, eyes: Eyes::Angry, seed: 20, ..h(g(142, 102, 70), g(132, 98, 66)) }),
        "mon_powerwan" => build_hum(&Hum { lean: -0.15, stance: 1.4, arm_pose: 1, ear: Ear::No, snout: Snout::Dog, hat: Hat::Helm, armor: Some(g(112, 112, 126)), shield: Some(g(100, 100, 116)), wp: Wp::Sword, tails: 1, body: 1.3, seed: 21, ..h(g(166, 112, 72), g(92, 92, 102)) }),
        "mon_flamewar" => build_hum(&Hum { lean: 0.05, stance: 1.2, arm_pose: 1, armor: Some(g(176, 50, 30)), wp: Wp::FlameSword, mane: Some(g(255, 150, 40)), eyes: Eyes::Glow, eye: g(255, 230, 100), body: 1.1, seed: 22, ..h(g(232, 66, 42), g(204, 60, 32)) }),
        "mon_icewar" => build_hum(&Hum { lean: -0.05, stance: 1.2, arm_pose: 1, armor: Some(g(190, 230, 255)), hat: Hat::Helm, wp: Wp::IceSword, eyes: Eyes::Glow, eye: g(255, 255, 255), body: 1.1, seed: 23, ..h(g(180, 220, 255), g(150, 200, 240)) }),
        "mon_masterwar" => build_hum(&Hum { lean: -0.12, stance: 1.2, arm_pose: 1, plume: Some(g(200, 36, 44)), accent: g(240, 200, 80), hat: Hat::Helm, armor: Some(g(62, 62, 74)), cape: Some(g(130, 30, 40)), wp: Wp::Sword, wp_col: g(150, 150, 164), eyes: Eyes::Glow, eye: g(255, 80, 60), body: 1.2, seed: 24, ..h(g(40, 40, 48), g(50, 50, 60)) }),
        "mon_hotjigoku" => build_hum(&Hum { lean: 0.3, stance: 1.3, arm_pose: 2, horn: Horn::Small, ear: Ear::Point, fangs: true, mane: Some(g(255, 150, 40)), eyes: Eyes::Glow, eye: g(255, 230, 80), legs: 0.7, body: 0.9, head: 1.2, seed: 25, ..h(g(232, 52, 42), g(204, 42, 32)) }),
        "mon_icejigoku" => build_hum(&Hum { lean: 0.25, stance: 1.2, arm_pose: 2, horn: Horn::Demon, ear: Ear::Point, mane: Some(g(190, 236, 255)), eyes: Eyes::Glow, eye: g(255, 255, 255), legs: 0.75, head: 1.2, wings: Some(g(130, 200, 250)), seed: 26, ..h(g(110, 190, 250), g(80, 150, 230)) }),
        "mon_buffalo" => buffalo(),
        "mon_penguin" => penguin(),
        "mon_pepper" => pepper(),
        "mon_nekomata2" => build_hum(&Hum { lean: 0.4, stance: 0.9, arm_pose: 1, ear: Ear::Cat, snout: Snout::Short, tails: 2, wp: Wp::Dagger, eyes: Eyes::Angry, seed: 27, ..h(g(152, 132, 112), g(44, 40, 52)) }),
        "mon_nekomata3" => build_hum(&Hum { lean: 0.1, stance: 0.9, arm_pose: 3, ear: Ear::Cat, snout: Snout::Short, tails: 2, hat: Hat::Crown, wp: Wp::Staff, seed: 28, ..h(g(242, 238, 226), g(232, 222, 202)) }),
        "mon_zombie" => build_hum(&Hum { lean: 0.5, stance: 0.9, arm_pose: 4, hat: Hat::Top, eyes: Eyes::Dot, legs: 0.9, body: 0.85, seed: 29, ..h(g(190, 202, 170), g(36, 34, 40)) }),
        "mon_platypus" => platypus(),
        "mon_valkyrie" => build_hum(&Hum { lean: -0.1, arm_pose: 1, hair: Some(g(238, 228, 204)), ear: Ear::Point, armor: Some(g(224, 214, 184)), wings: Some(g(252, 250, 244)), wp: Wp::Spear, seed: 30, ..h(g(246, 224, 204), g(226, 216, 190)) }),
        "mon_catqueen" => build_hum(&Hum { lean: -0.05, stance: 0.9, arm_pose: 3, ear: Ear::Cat, snout: Snout::Short, tails: 1, hat: Hat::Crown, cape: Some(g(104, 30, 52)), seed: 31, ..h(g(112, 86, 62), g(114, 32, 52)) }),
        "mon_emperorwan" => build_hum(&Hum { lean: -0.2, stance: 1.4, arm_pose: 2, ear: Ear::Dog, snout: Snout::Dog, armor: Some(g(228, 186, 72)), hat: Hat::Crown, cape: Some(g(120, 30, 40)), wp: Wp::Sword, wp_col: g(236, 200, 90), shield: Some(g(232, 192, 84)), tails: 1, body: 1.2, seed: 32, ..h(g(192, 142, 72), g(150, 100, 52)) }),
        "mon_poisonlime" => slime(g(130, 70, 170), g(200, 140, 230), 3),
        "mon_demonarmor" => build_hum(&Hum { lean: 0.2, stance: 1.35, arm_pose: 2, spikes: true, accent: g(170, 70, 230), shield: Some(g(80, 60, 96)), hat: Hat::Helm, armor: Some(g(58, 52, 66)), horn: Horn::Demon, wp: Wp::Sword, wp_col: g(110, 70, 90), eyes: Eyes::Glow, eye: g(255, 60, 50), body: 1.2, seed: 33, ..h(g(32, 32, 38), g(44, 40, 52)) }),
        "mon_debrin" => build_hum(&Hum { lean: 0.3, stance: 1.3, arm_pose: 2, horn: Horn::Small, ear: Ear::Point, fangs: true, eyes: Eyes::Angry, belly: 0.3, body: 1.2, head: 1.1, legs: 0.75, tails: 1, wp: Wp::Torch, seed: 34, ..h(g(214, 56, 42), g(172, 42, 38)) }),
        "mon_octopus" => octopus(),
        "mon_kinglion" => build_hum(&Hum { lean: -0.15, stance: 1.2, arm_pose: 1, mane: Some(g(214, 150, 50)), ear: Ear::Round, snout: Snout::Short, hat: Hat::Crown, cape: Some(g(130, 40, 50)), wp: Wp::Spear, wp_col: g(236, 200, 90), body: 1.2, seed: 35, ..h(g(218, 160, 70), g(132, 42, 52)) }),
        "mon_dragon" => build_hum(&Hum { lean: 0.4, stance: 1.2, arm_pose: 1, horn: Horn::Small, snout: Snout::Dog, tails: 1, tail_long: true, belly: 0.2, body: 1.15, legs: 0.85, eyes: Eyes::Dot, seed: 36, ..h(g(98, 130, 72), g(150, 112, 72)) }),
        "mon_devilprince" => build_hum(&Hum { lean: -0.1, stance: 1.2, arm_pose: 1, horn: Horn::Demon, ear: Ear::Point, armor: Some(g(60, 50, 74)), wings: Some(g(72, 44, 82)), cape: Some(g(80, 30, 62)), wp: Wp::Sword, wp_col: g(150, 80, 130), eyes: Eyes::Glow, eye: g(255, 70, 60), seed: 37, ..h(g(64, 52, 72), g(40, 32, 48)) }),
        "mon_anagerilla" => mole(),
        "mon_lumberduck" => build_hum(&Hum { lean: 0.15, stance: 1.2, arm_pose: 1, snout: Snout::Bill, accent: g(242, 172, 50), hat: Hat::Cap, wp: Wp::Axe, body: 1.1, seed: 38, ..h(g(238, 234, 220), g(170, 60, 60)) }),
        "mon_daruma" => daruma(g(206, 36, 40), 0),
        "mon_samuraiwolf" => build_hum(&Hum { lean: -0.1, stance: 1.25, arm_pose: 1, ear: Ear::Dog, snout: Snout::Dog, armor: Some(g(84, 84, 94)), hat: Hat::Samurai, wp: Wp::Sword, tails: 1, seed: 39, ..h(g(98, 98, 108), g(62, 62, 72)) }),
        "mon_shogunwolf" => build_hum(&Hum { lean: -0.2, stance: 1.3, arm_pose: 1, ear: Ear::Dog, snout: Snout::Dog, armor: Some(g(222, 222, 228)), hat: Hat::Samurai, cape: Some(g(60, 60, 84)), wp: Wp::Sword, tails: 1, body: 1.1, seed: 40, ..h(g(238, 238, 242), g(204, 204, 214)) }),
        "mon_killercock" => build_hum(&Hum { lean: 0.3, stance: 1.1, arm_pose: 2, snout: Snout::Beak, accent: g(242, 170, 50), hair: Some(g(220, 50, 50)), tails: 1, tail_long: true, eyes: Eyes::Angry, legs: 0.9, seed: 41, ..h(g(242, 232, 212), g(242, 232, 212)) }),
        "mon_hidaruma" => daruma(g(226, 80, 40), 1),
        "mon_maidoll" => build_hum(&Hum { lean: 0.05, arm_pose: 3, hair: Some(g(182, 132, 92)), hat: Hat::Cap, wp: Wp::Club, wp_col: g(150, 100, 60), seed: 42, ..h(g(250, 228, 204), g(60, 60, 78)) }),
        "mon_yukidaruma" => daruma(g(214, 238, 252), 2),
        "mon_mushroomlizard" => build_hum(&Hum { lean: 0.35, stance: 1.0, arm_pose: 4, snout: Snout::Dog, tails: 1, tail_long: true, hat: Hat::Cap, eyes: Eyes::Dot, seed: 43, ..h(g(112, 162, 92), g(112, 162, 92)) }),
        "mon_warbeetle" => build_hum(&Hum { lean: 0.3, stance: 1.3, arm_pose: 2, horn: Horn::Small, armor: Some(g(112, 64, 44)), cape: Some(g(90, 50, 36)), body: 1.3, arm: 1.2, eyes: Eyes::Glow, eye: g(255, 200, 60), seed: 44, ..h(g(92, 52, 38), g(98, 58, 42)) }),
        "mon_flamemaster" => build_hum(&Hum { lean: 0.1, arm_pose: 1, hat: Hat::Hood, wp: Wp::FlameSword, eyes: Eyes::Glow, eye: g(255, 220, 100), seed: 45, ..h(g(204, 60, 50), g(156, 40, 40)) }),
        "mon_highmaster" => build_hum(&Hum { lean: -0.05, arm_pose: 1, hair: Some(g(240, 240, 248)), cape: Some(g(36, 34, 44)), wp: Wp::Sword, wp_col: g(190, 200, 230), seed: 46, ..h(g(242, 232, 222), g(30, 30, 38)) }),
        "mon_detarame" => build_hum(&Hum { lean: 0.2, stance: 1.1, arm_pose: 2, ear: Ear::Point, hat: Hat::Cap, accent: g(80, 110, 160), seed: 47, ..h(g(232, 212, 182), g(204, 152, 62)) }),
        "mon_caterpillar" => caterpillar(),
        "mon_killerflower" => killer_flower(false),
        "mon_bonewizard" => build_hum(&Hum { lean: 0.2, arm_pose: 3, skele: true, hat: Hat::Hood, wp: Wp::Staff, accent: g(120, 255, 170), eyes: Eyes::Dot, seed: 48, ..h(g(236, 230, 212), g(60, 52, 70)) }),
        "mon_magicdonkey" => build_hum(&Hum { lean: 0.15, arm_pose: 3, ear: Ear::Long, snout: Snout::Short, tails: 1, hat: Hat::Wizard, seed: 49, ..h(g(146, 116, 94), g(92, 60, 100)) }),
        "mon_devildice" => dice(),
        "mon_hellscream" => build_hum(&Hum { lean: 0.35, stance: 1.2, arm_pose: 2, hair: Some(g(60, 40, 50)), fangs: true, eyes: Eyes::Dot, arm: 1.4, legs: 0.9, seed: 50, ..h(g(196, 186, 204), g(112, 102, 92)) }),
        "mon_magicshield" => magic_shield(),
        "mon_magicbow" => build_hum(&Hum { lean: 0.1, arm_pose: 1, hair: Some(g(232, 204, 134)), ear: Ear::Point, wp: Wp::Bow, seed: 51, ..h(g(248, 228, 204), g(200, 172, 122)) }),
        "mon_magicsword" => build_hum(&Hum { lean: -0.05, arm_pose: 1, hair: Some(g(222, 194, 124)), wp: Wp::IceSword, seed: 52, ..h(g(248, 228, 204), g(152, 132, 92)) }),
        "mon_dorunuma" => dorunuma(),
        "mon_devilprincess" => build_hum(&Hum { lean: 0.1, arm_pose: 3, hair: Some(g(60, 40, 70)), horn: Horn::Small, ear: Ear::Point, wings: Some(g(92, 58, 102)), legs: 0.8, eye: g(200, 40, 80), seed: 53, ..h(g(232, 212, 202), g(52, 38, 62)) }),
        "mon_hitotsuoni" => build_hum(&Hum { lean: 0.4, stance: 1.2, arm_pose: 4, eyes: Eyes::One, horn: Horn::Small, body: 1.5, belly: 0.25, arm: 1.3, head: 1.1, fangs: true, wp: Wp::Club, seed: 54, ..h(g(168, 122, 98), g(142, 102, 62)) }),
        "mon_flameflower" => killer_flower(true),
        "mon_kage" => build_hum(&Hum { lean: 0.4, stance: 0.8, arm_pose: 4, arm: 1.5, eyes: Eyes::Glow, eye: g(255, 255, 255), legs: 1.2, body: 0.8, seed: 55, ..h(g(20, 20, 26), g(24, 24, 30)) }),
        "mon_mitsuoni" => build_hum(&Hum { lean: 0.35, stance: 1.2, arm_pose: 4, eyes: Eyes::Three, horn: Horn::Small, armor: Some(g(112, 102, 82)), body: 1.4, arm: 1.2, fangs: true, wp: Wp::Mace, seed: 56, ..h(g(98, 112, 72), g(82, 72, 52)) }),
        "mon_elephantom" => elephantom(),
        "mon_devillord" => build_hum(&Hum { hat: Hat::Crown, horn: Horn::Demon, armor: Some(g(78, 66, 96)), cape: Some(g(168, 30, 56)), wings: Some(g(130, 30, 60)), trim_col: Some(g(240, 190, 60)), wp: Wp::FlameSword, wp_col: g(190, 70, 80), eyes: Eyes::Glow, eye: g(255, 90, 50), body: 1.75, head: 1.1, arm: 1.4, legs: 1.15, lean: -0.1, stance: 1.35, arm_pose: 2, spikes: true, accent: g(240, 190, 60), seed: 57, ..h(g(30, 28, 36), g(44, 36, 56)) }),
        "mon_kingsnake" => apple(true),
        _ => return None,
    })
}
