//! ボス級の魔物(20体)の モデル。ふつうの魔物と ひと目で 見分けがつくよう、
//!   (1) 体を 一回り 大きく 作り込み(とげ・冠・マント・つばさ・かざり)
//!   (2) 足もとに 石の だいざ(二段・金のふち)と 光る 魔法の輪
//!   (3) まわりに うかぶ 結晶(または ほのお)と、うしろへ のびる 光の すじ
//! を 足す。色は ボスごとに かえる。

use crate::creatures::*;
use crate::creatures2::creature_base;
use crate::model3d::Model;
use crate::modeling::*;
use macroquad::prelude::Color;
use std::f32::consts::{PI, TAU};

fn g(r: u8, gc: u8, b: u8) -> Color {
    col(r, gc, b)
}

/// ボス級の 絵の名前か
pub fn is_boss_art(name: &str) -> bool {
    matches!(
        name,
        "mon_lion" | "mon_dino" | "mon_sorcerer" | "mon_tetsu" | "mon_boss" | "mon_debrin" | "mon_octopus" | "mon_kinglion" | "mon_dragon" | "mon_devilprince" | "mon_dorunuma" | "mon_shogunboss" | "mon_devilprincess" | "mon_hitotsuoni" | "mon_flameflower" | "mon_kage" | "mon_mitsuoni" | "mon_elephantom" | "mon_devillord" | "mon_kingsnake"
    )
}

/// (光の色, ふちの色, 炎を 立てるか)
fn aura_of(name: &str) -> (Color, Color, bool) {
    match name {
        "mon_lion" | "mon_kinglion" => (g(255, 200, 70), g(240, 190, 60), false),
        "mon_dino" | "mon_debrin" | "mon_flameflower" | "mon_hitotsuoni" => (g(255, 110, 40), g(220, 90, 50), true),
        "mon_sorcerer" | "mon_octopus" => (g(90, 190, 255), g(120, 200, 240), false),
        "mon_tetsu" => (g(255, 170, 60), g(200, 200, 214), false),
        "mon_boss" | "mon_kingsnake" => (g(150, 255, 90), g(240, 200, 70), false),
        "mon_dragon" | "mon_dorunuma" | "mon_mitsuoni" => (g(120, 255, 150), g(180, 150, 90), false),
        "mon_devilprince" | "mon_devillord" => (g(255, 60, 90), g(240, 190, 60), true),
        "mon_devilprincess" => (g(230, 90, 255), g(240, 190, 60), false),
        "mon_shogunboss" => (g(200, 220, 255), g(230, 230, 240), false),
        "mon_kage" => (g(170, 90, 255), g(80, 50, 110), false),
        "mon_elephantom" => (g(120, 255, 190), g(180, 200, 200), false),
        _ => (g(255, 255, 255), g(240, 190, 60), false),
    }
}

/// 威厳の そうしょく(だいざ・光の輪・うかぶ結晶・光のすじ)を 足す
fn dress(mut base: Model, name: &str) -> Model {
    // ボスは 一回り 大きい(だいざの うえで 堂々と)
    let k = 1.28;
    for q in base.quads.iter_mut() {
        for c in q.c.iter_mut() {
            *c = [c[0] * k, c[1] * k, c[2] * k];
        }
    }
    let (glow, rim, fire) = aura_of(name);
    let mut m = Mb::from_model(base);
    let stone = g(70, 64, 78);
    // 二段の 石の だいざ
    m.lathe(0.0, 0.0, 0.0, &[(0.0, 0.0), (0.6, 0.0), (0.6, 0.045), (0.56, 0.06), (0.0, 0.06)], 28, &|i, j| shadec(stone, 0.85 + 0.12 * (j % 2) as f32 + 0.05 * i as f32));
    m.lathe(0.0, 0.06, 0.0, &[(0.0, 0.0), (0.5, 0.0), (0.5, 0.03), (0.46, 0.045), (0.0, 0.045)], 28, &|_, j| shadec(stone, 1.0 + 0.1 * (j % 2) as f32));
    m.ring(0.0, 0.003, 0.0, 0.6, 0.575, 40, rim);
    m.ring(0.0, 0.106, 0.0, 0.5, 0.47, 40, rim);
    // 光る 魔法の 輪と 星
    m.ring(0.0, 0.108, 0.0, 0.44, 0.415, 40, glow);
    m.ring(0.0, 0.108, 0.0, 0.3, 0.285, 32, glow);
    for i in 0..6 {
        let (a0, a1) = (i as f32 / 6.0 * TAU, ((i + 2) % 6) as f32 / 6.0 * TAU);
        m.tube([0.4 * a0.sin(), 0.112, 0.4 * a0.cos()], [0.4 * a1.sin(), 0.112, 0.4 * a1.cos()], 0.007, 0.007, 5, glow, false);
    }
    // だいざの ふちの 柱と かざり
    for k in 0..8 {
        let a = k as f32 / 8.0 * TAU + 0.2;
        let (x, z) = (0.55 * a.sin(), 0.55 * a.cos());
        m.tube([x, 0.06, z], [x, 0.2, z], 0.03, 0.02, 6, shadec(stone, 1.2), true);
        if fire {
            m.flame(x, 0.2, z, 0.2, 0.045);
        } else {
            m.ellipsoid([x, 0.24, z], [0.03, 0.055, 0.03], 5, 8, &|_, _| mix(glow, col(255, 255, 255), 0.4));
        }
    }
    // うかぶ 結晶(まわりを 回る)
    for k in 0..7 {
        let a = k as f32 / 7.0 * TAU;
        let y = 0.3 + 0.55 * ((k * 3 % 7) as f32 / 7.0);
        let r = 0.46 + 0.04 * (k % 2) as f32;
        m.at([r * a.sin(), y, r * a.cos()], a, 0.25, 0.3, |m| {
            m.ellipsoid([0.0, 0.0, 0.0], [0.028, 0.07, 0.028], 5, 8, &|th, _| mix(glow, col(255, 255, 255), (1.0 - th / PI).powi(2) * 0.6));
        });
    }
    // うしろへ のびる 光の すじ(威光)
    for k in 0..7 {
        let t = (k as f32 / 6.0 - 0.5) * 2.0;
        let x = t * 0.5;
        m.tube([x, 0.06, -0.52 + 0.1 * t.abs()], [x * 1.5, 1.25 - 0.25 * t.abs(), -0.6], 0.016, 0.003, 5, mix(glow, col(255, 255, 255), 0.35), false);
    }
    let mut model = m.done();
    model.hand = true;
    model
}

/// ボス級の モデル(なければ None)
pub fn boss_model(name: &str) -> Option<Model> {
    if !is_boss_art(name) {
        return None;
    }
    let h = |fur: Color, cloth: Color| hum(fur, cloth);
    let gold = g(240, 190, 60);
    let hum_model: Option<Model> = match name {
        "mon_lion" => Some(build_hum(&Hum { lean: -0.15, stance: 1.3, arm_pose: 2, mane: Some(g(214, 112, 40)), ear: Ear::Round, snout: Snout::Short, tails: 1, armor: Some(g(150, 100, 60)), hat: Hat::Crown, cape: Some(g(150, 36, 40)), spikes: true, trim_col: Some(gold), fangs: true, body: 1.55, head: 1.15, arm: 1.2, wp: Wp::Axe, wp_col: g(220, 220, 232), accent: gold, seed: 7, ..h(g(226, 160, 66), g(120, 80, 56)) })),
        "mon_dino" => Some(build_hum(&Hum { lean: 0.5, stance: 1.4, arm_pose: 2, snout: Snout::Short, horn: Horn::Demon, wings: Some(g(170, 36, 30)), spikes: true, tails: 2, tail_long: true, belly: 0.3, body: 1.65, head: 1.25, legs: 0.8, fangs: true, eyes: Eyes::Angry, armor: Some(g(150, 60, 40)), trim_col: Some(g(255, 190, 70)), seed: 17, ..h(g(210, 54, 42), g(214, 74, 52)) })),
        "mon_sorcerer" => Some(build_hum(&Hum { lean: 0.05, arm_pose: 1, ear: Ear::Round, snout: Snout::Short, tails: 1, tail_long: true, hat: Hat::Wizard, wp: Wp::Staff, accent: g(120, 210, 255), cape: Some(g(36, 70, 140)), armor: Some(g(60, 100, 170)), spikes: true, trim_col: Some(g(150, 220, 255)), body: 1.35, head: 1.1, seed: 18, ..h(g(136, 104, 72), g(40, 80, 150)) })),
        "mon_tetsu" => Some(build_hum(&Hum { lean: 0.1, stance: 1.45, arm_pose: 2, hat: Hat::Helm, horn: Horn::Bull, armor: Some(g(112, 114, 126)), spikes: true, plume: Some(g(220, 110, 40)), body: 2.0, head: 0.8, arm: 1.4, belly: 0.1, eyes: Eyes::Glow, eye: g(255, 190, 60), trim_col: Some(g(255, 160, 50)), shield: Some(g(100, 102, 114)), wp: Wp::Mace, seed: 19, ..h(g(92, 94, 104), g(76, 76, 86)) })),
        "mon_debrin" => Some(build_hum(&Hum { lean: 0.3, stance: 1.4, arm_pose: 2, horn: Horn::Demon, hat: Hat::Crown, ear: Ear::Point, fangs: true, eyes: Eyes::Angry, belly: 0.35, body: 1.5, head: 1.15, legs: 0.75, tails: 1, wp: Wp::Torch, wings: Some(g(180, 50, 36)), spikes: true, cape: Some(g(140, 30, 30)), armor: Some(g(130, 50, 40)), trim_col: Some(gold), seed: 34, ..h(g(220, 58, 42), g(172, 42, 38)) })),
        "mon_kinglion" => Some(build_hum(&Hum { lean: -0.2, stance: 1.3, arm_pose: 1, mane: Some(g(224, 160, 50)), ear: Ear::Round, snout: Snout::Short, hat: Hat::Crown, cape: Some(g(140, 30, 50)), wp: Wp::Spear, wp_col: g(240, 205, 100), armor: Some(g(236, 200, 90)), spikes: true, trim_col: Some(g(255, 240, 160)), accent: gold, body: 1.6, head: 1.15, tails: 1, fangs: true, seed: 35, ..h(g(224, 166, 70), g(150, 40, 52)) })),
        "mon_dragon" => Some(build_hum(&Hum { lean: 0.4, stance: 1.3, arm_pose: 1, horn: Horn::Demon, snout: Snout::Dog, tails: 2, tail_long: true, belly: 0.25, body: 1.55, head: 1.2, legs: 0.85, eyes: Eyes::Angry, wings: Some(g(70, 112, 60)), spikes: true, armor: Some(g(110, 90, 50)), fangs: true, trim_col: Some(g(220, 190, 80)), seed: 36, ..h(g(98, 134, 72), g(150, 112, 72)) })),
        "mon_devilprince" => Some(build_hum(&Hum { lean: -0.12, stance: 1.3, arm_pose: 2, horn: Horn::Demon, hat: Hat::Crown, ear: Ear::Point, armor: Some(g(80, 62, 96)), wings: Some(g(100, 50, 112)), cape: Some(g(120, 30, 80)), wp: Wp::FlameSword, wp_col: g(190, 90, 160), spikes: true, trim_col: Some(gold), eyes: Eyes::Glow, eye: g(255, 70, 60), body: 1.55, head: 1.1, seed: 37, ..h(g(72, 56, 84), g(44, 34, 56)) })),
        "mon_shogunboss" => Some(build_hum(&Hum { lean: -0.22, stance: 1.4, arm_pose: 1, ear: Ear::Dog, snout: Snout::Dog, armor: Some(g(232, 232, 240)), hat: Hat::Samurai, plume: None, cape: Some(g(60, 60, 110)), wp: Wp::Sword, wp_col: g(236, 240, 255), tails: 1, spikes: true, trim_col: Some(g(80, 120, 230)), shield: Some(g(220, 224, 236)), body: 1.5, head: 1.15, seed: 40, ..h(g(240, 240, 246), g(204, 204, 214)) })),
        "mon_devilprincess" => Some(build_hum(&Hum { lean: 0.05, stance: 1.1, arm_pose: 1, hair: Some(g(80, 40, 100)), horn: Horn::Demon, hat: Hat::Crown, ear: Ear::Point, wings: Some(g(120, 60, 140)), cape: Some(g(90, 30, 110)), armor: Some(g(80, 50, 100)), spikes: true, trim_col: Some(gold), wp: Wp::Staff, accent: g(240, 120, 255), legs: 0.9, eye: g(230, 50, 120), body: 1.3, head: 1.1, seed: 53, ..h(g(236, 216, 206), g(60, 40, 76)) })),
        "mon_hitotsuoni" => Some(build_hum(&Hum { lean: 0.4, stance: 1.35, arm_pose: 2, eyes: Eyes::One, horn: Horn::Bull, body: 1.9, belly: 0.3, arm: 1.5, head: 1.15, fangs: true, wp: Wp::Club, armor: Some(g(120, 76, 54)), spikes: true, cape: Some(g(110, 50, 40)), trim_col: Some(g(220, 190, 110)), seed: 54, ..h(g(176, 126, 100), g(142, 102, 62)) })),
        "mon_mitsuoni" => Some(build_hum(&Hum { lean: 0.35, stance: 1.35, arm_pose: 2, eyes: Eyes::Three, horn: Horn::Bull, armor: Some(g(118, 108, 84)), body: 1.85, arm: 1.4, fangs: true, wp: Wp::Mace, spikes: true, cape: Some(g(70, 70, 40)), trim_col: Some(g(210, 190, 90)), head: 1.15, seed: 56, ..h(g(100, 118, 72), g(82, 72, 52)) })),
        "mon_kage" => Some(build_hum(&Hum { lean: 0.4, stance: 0.9, arm_pose: 2, arm: 1.7, eyes: Eyes::Glow, eye: g(255, 255, 255), legs: 1.3, body: 1.0, horn: Horn::Demon, hat: Hat::Hood, cape: Some(g(36, 26, 60)), wings: Some(g(50, 36, 80)), armor: Some(g(34, 30, 50)), spikes: true, trim_col: Some(g(170, 90, 255)), seed: 55, ..h(g(20, 20, 26), g(24, 22, 36)) })),
        _ => None,
    };
    let base = match hum_model {
        Some(m) => m,
        None => creature_base(name)?,
    };
    Some(dress(base, name))
}
