//! SVG素材をビルド時に埋め込み、起動時にラスタライズしてテクスチャにする。
//! assets/img/<name>.png があればそちらを優先(qwen-image などで差し替え用)。

use macroquad::prelude::*;
use std::collections::HashMap;

macro_rules! svgs {
    ($($name:literal),* $(,)?) => {
        const SVGS: &[(&str, &str)] = &[
            $(($name, include_str!(concat!("../assets/svg/", $name, ".svg")))),*
        ];
    };
}

svgs!(
    "tile_floor", "tile_rock", "tile_bedrock", "tile_barrier", "tile_stairs_up", "tile_stairs_down", "tile_portal",
    "room_エサおけ", "room_ゴミすてば", "room_わらのベッド", "room_てつのたからばこ", "room_てつのたからばこ_open",
    "hero_map", "portrait_a", "portrait_j", "portrait_m",
    "mon_bat", "mon_pig", "mon_cat", "mon_gob", "mon_slime", "mon_boss",
    "mon_wan", "mon_gobl", "mon_ghost", "mon_potato", "mon_lion",
    "room_さかば", "room_さいだん", "room_はたけ",
    "scene_town", "scene_armor", "scene_magic", "scene_furn", "scene_plaza", "scene_mayor", "scene_home",
    "bg_battle", "scene_title",
);

/// SVG の仮素材を持たない絵(assets/img の PNG だけ)
const PNG_ONLY: &[&str] = &[
    "mon_anagerilla", "mon_lumberduck", "mon_daruma", "mon_samuraiwolf", "mon_shogunwolf", "stand_mayor", "scene_walk", "scene_site", "mon_shogunboss", "mon_needleworm", "mon_hinezumi", "mon_shieldmaiden", "mon_detaramedice", "mon_killercock", "mon_hidaruma", "mon_maidoll", "mon_yukidaruma", "mon_mushroomlizard", "mon_warbeetle", "mon_flamemaster", "mon_highmaster", "mon_detarame", "mon_caterpillar", "mon_killerflower", "mon_bonewizard", "mon_magicdonkey", "mon_devildice", "mon_hellscream", "mon_magicshield", "mon_magicbow", "mon_magicsword", "mon_dorunuma", "mon_devilprincess", "mon_hitotsuoni", "mon_flameflower", "mon_kage", "mon_mitsuoni", "mon_elephantom", "mon_devillord", "mon_kingsnake", "room_コロシアム", "room_れいぞうこ", "room_まほうじん",
    "mon_chikaraushi", "mon_powerwan", "mon_flamewar", "mon_icewar", "mon_masterwar", "mon_hotjigoku", "mon_icejigoku", "mon_buffalo", "mon_penguin", "mon_pepper", "mon_nekomata2", "mon_nekomata3", "mon_zombie", "mon_platypus", "mon_valkyrie", "mon_catqueen", "mon_emperorwan", "mon_poisonlime", "mon_demonarmor", "mon_debrin", "mon_octopus", "mon_kinglion", "mon_dragon", "mon_devilprince", "room_だんろ", "room_けっかいのへや", "room_ふんすい", "room_ごうかなベッド", "room_れいはいどう", "room_じっけんしつ",
    "room_ぎんのたからばこ", "room_ぎんのたからばこ_open", "room_おうごんチェスト", "room_おうごんチェスト_open",
    "mon_hibuta", "mon_ningen", "mon_firelime", "mon_hotghost", "mon_nekomata", "mon_waterlime", "mon_coldghost", "mon_otter", "mon_carrot",
    "mon_crowcat", "mon_blackcat", "mon_wizdrill", "mon_blackwar", "mon_bone", "mon_dino", "mon_sorcerer", "mon_tetsu",
    "stand_a", "stand_m", "stand_j",
    "room_瘴気発生装置", "room_いろり", "room_水のみば", "room_りっぱなベッド", "room_はなぞの", "room_どうじょう",
];

pub struct Art {
    tex: HashMap<String, Texture2D>,
}

/// 絵として登録されている名前か(テスト用)
#[cfg(test)]
pub fn declared(name: &str) -> bool {
    SVGS.iter().any(|(n, _)| *n == name) || PNG_ONLY.contains(&name)
}

/// PNG だけで持つ絵の名前(テスト用)
#[cfg(test)]
pub fn png_only() -> &'static [&'static str] {
    PNG_ONLY
}

fn raster_scale(name: &str) -> f32 {
    if name.starts_with("tile_") || name.starts_with("room_") || name.starts_with("hero_") || name.starts_with("portrait_") {
        2.0
    } else {
        1.0
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn rasterize(svg: &str, scale: f32) -> Option<Texture2D> {
    let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default()).ok()?;
    let size = tree.size();
    let w = (size.width() * scale).ceil() as u32;
    let h = (size.height() * scale).ceil() as u32;
    let mut pm = resvg::tiny_skia::Pixmap::new(w, h)?;
    resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(scale, scale), &mut pm.as_mut());
    // tiny-skia は乗算済みアルファ。macroquad は通常のアルファ
    let mut bytes = pm.take();
    for px in bytes.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a > 0 && a < 255 {
            for c in px.iter_mut().take(3) {
                *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    let img = Image { bytes, width: w as u16, height: h as u16 };
    let t = Texture2D::from_image(&img);
    t.set_filter(FilterMode::Linear);
    Some(t)
}

impl Art {
    pub async fn load() -> Art {
        let mut tex = HashMap::new();
        for (name, svg) in SVGS {
            let t = if let Some(bytes) = crate::assets::read(&format!("assets/img/{}.png", name)).await {
                let t = Texture2D::from_file_with_format(&bytes, None);
                t.set_filter(FilterMode::Linear);
                Some(t)
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    rasterize(svg, raster_scale(name))
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = (svg, raster_scale(name));
                    None
                }
            };
            if let Some(t) = t {
                tex.insert(name.to_string(), t);
            }
        }
        for name in PNG_ONLY {
            if let Some(bytes) = crate::assets::read(&format!("assets/img/{}.png", name)).await {
                let t = Texture2D::from_file_with_format(&bytes, None);
                t.set_filter(FilterMode::Linear);
                tex.insert(name.to_string(), t);
            }
        }
        Art { tex }
    }

    pub fn get(&self, name: &str) -> Option<&Texture2D> {
        self.tex.get(name)
    }

    /// 指定矩形いっぱいに描く
    pub fn draw(&self, name: &str, x: f32, y: f32, w: f32, h: f32) {
        if let Some(t) = self.tex.get(name) {
            draw_texture_ex(t, x, y, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
        } else {
            draw_rectangle(x, y, w, h, Color::new(0.8, 0.2, 0.8, 1.0));
        }
    }

    pub fn draw_tinted(&self, name: &str, x: f32, y: f32, w: f32, h: f32, c: Color) {
        if let Some(t) = self.tex.get(name) {
            draw_texture_ex(t, x, y, c, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
        }
    }
}
