mod art;
mod assets;
mod audio;
mod battle;
mod board;
mod layout;
mod mimic;
mod model3d;
mod modeling;
mod facilities;
mod creatures;
mod creatures2;
mod bosses;
mod rooms3;
mod models;
mod script;
mod talk;
mod data;
mod debug;
mod dungeon;
mod home;
mod overlays;
mod paths;
mod quest;
mod shop;
mod stage;
mod sim;
#[cfg(test)]
mod tests;
mod town;
mod ui;
mod view3d;

use macroquad::prelude::*;
use sim::Game;
use ui::*;

pub enum Scr {
    None,
    Title(ListMenu),
    Town(town::TownSt),
    Shop(shop::ShopSt),
    Home(home::HomeSt),
    Dungeon(dungeon::DunSt),
    Battle(Box<battle::BattleSt>),
    Morning(Vec<String>),
    Talk(Box<talk::TalkSt>),
    Models(Box<models::ModelsSt>),
}

pub struct App {
    pub g: Game,
    pub ui: Ui,
    pub inp: Input,
    pub view: View,
    pub scr: Scr,
    pub ov: Option<overlays::Overlay>,
    pub toast: Option<(String, f32)>,
    pub quit: bool,
    pub debug: bool,
    /// 3Dのカメラ: [x, y, 向き, 階, 初期化ずみ]
    pub cam3: std::cell::Cell<[f32; 6]>,
    /// 絵から作った 3Dモデルの ため置き
    pub models: std::cell::RefCell<std::collections::HashMap<(String, u8), std::rc::Rc<model3d::Model>>>,
    /// F4 で モデル一覧を 開く前の画面
    pub prev_scr: Option<Scr>,
    /// 台詞の整理に 最後に 知らせた「ジェリーが 仲間か」
    pub jerry_synced: Option<bool>,
}

impl App {
    pub fn say(&mut self, s: impl Into<String>) {
        self.toast = Some((s.into(), 2.4));
    }

    fn frame(&mut self) {
        // ジェリーが 仲間に なったときだけ 台詞の整理に 知らせる
        let jerry = self.g.party.len() > 1 && self.g.party[1].joined;
        if self.jerry_synced != Some(jerry) {
            script::sync_party(&self.g);
            self.jerry_synced = Some(jerry);
        }
        // オーバーレイ表示中はベース画面に空の入力を渡す(描画だけ行う)
        let had_ov = self.ov.is_some();
        let real = if self.ov.is_some() { Some(std::mem::take(&mut self.inp)) } else { None };
        let mut scr = std::mem::replace(&mut self.scr, Scr::None);
        let next = match &mut scr {
            Scr::Title(m) => self.title(m),
            Scr::Town(st) => self.town(st),
            Scr::Shop(st) => self.shop(st),
            Scr::Home(st) => self.home(st),
            Scr::Dungeon(st) => self.dungeon(st),
            Scr::Battle(st) => self.battle(st),
            Scr::Morning(lines) => self.morning(lines),
            Scr::Talk(st) => self.talk(st),
            Scr::Models(st) => self.models_screen(st),
            Scr::None => None,
        };
        self.scr = next.unwrap_or(scr);
        if let Some(r) = real {
            self.inp = r;
        }
        // オーバーレイ(ステータス等)は最前面。開いたフレームは入力の二重処理を避けて描かない
        if had_ov {
            self.overlay();
        }
        // トースト
        if let Some((s, t)) = &mut self.toast {
            *t -= get_frame_time();
            let w = self.ui.measure(s, 20.0) + 36.0;
            let x = (LW - w) / 2.0;
            draw_rectangle(x, 54.0, w, 38.0, rgba(20, 16, 10, 220));
            draw_rectangle_lines(x, 54.0, w, 38.0, 2.0, GOLDEN);
            self.ui.text_center(s, LW / 2.0, 62.0, 20.0, CREAM);
            if *t <= 0.0 {
                self.toast = None;
            }
        }
    }

    /// 場面に合う音楽(町・ダンジョンなど=のどかな曲、戦闘、ボス戦)
    fn update_music(&mut self) {
        let want = match &self.scr {
            Scr::Battle(st) => Some(if st.b.boss { "boss" } else { "battle" }),
            _ => Some("town"),
        };
        self.ui.audio.bgm(want);
    }

    /// はじめからの場面: 朝の自宅 → 村長の家(シャベルを押しつけられる)→ ダンジョンの入口へ放り出される
    fn opening(&self) -> Scr {
        use talk::TalkSt;
        let dungeon = Scr::Dungeon(dungeon::DunSt::new());
        let outside = Scr::Talk(Box::new(TalkSt::new("bg_battle", script::intro_dungeon(), dungeon)));
        let site = Scr::Talk(Box::new(TalkSt::new("scene_site", script::intro_site(), outside).speaker("村長").with_cast("アレン").with_cast("村長")));
        let walk = Scr::Talk(Box::new(TalkSt::new("scene_walk", script::intro_walk(), site).speaker("村長").with_cast("アレン").with_cast("村長").walking()));
        let mayor = Scr::Talk(Box::new(TalkSt::new("scene_mayor", script::intro_mayor(), walk).speaker("村長").faces()));
        Scr::Talk(Box::new(TalkSt::new("scene_home", script::intro_home(), mayor)))
    }

    fn title(&mut self, m: &mut ListMenu) -> Option<Scr> {
        self.ui.art.draw("scene_title", 0.0, 0.0, LW, SH);
        self.ui.text_center("Dungeon Making", LW / 2.0 + 2.0, 72.0, 56.0, rgba(255, 245, 210, 200));
        self.ui.text_center("Dungeon Making", LW / 2.0, 70.0, 56.0, rgb(84, 50, 18));
        if self.debug {
            self.ui.text_center("DEBUG MODE(セーブは別ファイル)", LW / 2.0, 140.0, 24.0, rgb(255, 120, 120));
        }
        panel_wood(0.0, SH, LW, SH);
        let save_name = assets::save_name(self.debug);
        let has_save = assets::load_text(save_name).is_some();
        let rows = vec![
            Row::new("はじめから"),
            Row::new("つづきから").enabled(has_save),
            Row::new("オプション"),
            Row::new("おわる"),
        ];
        let ev = m.run(&self.ui, &self.inp, &rows, Rect::new(120.0, SH + 70.0, 336.0, 240.0), 56.0, 28.0, Style::Wood, true);
        self.ui.text_center("矢印/WASD・マウス:えらぶ  Z/Enter:けってい  X:もどる", LW / 2.0, SH + 370.0, 15.0, CREAM);
        self.ui.text_center("F2: 3D表示の 動作確認(12F・セーブには かかわらない)", LW / 2.0, SH + 346.0, 14.0, rgb(200, 190, 160));
        self.ui.text_center("ダンジョン Z:ほる X:へや V:まほう U:もどす WASD:むき・すすむ", LW / 2.0, SH + 396.0, 15.0, CREAM);
        // F2: 3D表示の 動作確認用(12Fの 部屋と魔物が たくさん ある ダンジョン)
        if is_key_pressed(KeyCode::F2) {
            self.debug = true;
            self.g.preset(24);
            self.g.map3d = true;
            return Some(Scr::Dungeon(dungeon::DunSt::new()));
        }
        // タイトルで変えた設定(音量など)は はじめる ゲームに引きつぐ
        let prefs = Some(self.g.settings());
        match ev {
            MenuEv::Confirm(0) => {
                self.g = Game::start_story((macroquad::miniquad::date::now() * 1000.0) as u64);
                if let Some(p) = prefs {
                    self.g.set_settings(p);
                }
                self.g.return_to_entrance();
                return Some(self.opening());
            }
            MenuEv::Confirm(1) => {
                if let Some(s) = assets::load_text(save_name) {
                    if let Some(g) = Game::load_text(&s) {
                        self.g = g;
                        if let Some(p) = prefs {
                            self.g.set_settings(p);
                        }
                        return Some(Scr::Town(town::TownSt::new(Some("つづきから はじめた".into()))));
                    }
                }
                self.say("セーブデータを読めなかった(古い版のセーブは読めません。はじめからどうぞ)");
            }
            MenuEv::Confirm(2) => self.ov = Some(overlays::Overlay::Options(ListMenu::new())),
            MenuEv::Confirm(3) => self.quit = true,
            _ => {}
        }
        None
    }

    fn morning(&mut self, lines: &mut Vec<String>) -> Option<Scr> {
        self.ui.art.draw("scene_home", 0.0, 0.0, LW, SH);
        draw_rectangle(0.0, 0.0, LW, SH, rgba(255, 240, 200, 40));
        msgbox(16.0, 316.0, 544.0, 104.0);
        for (i, l) in lines.iter().take(3).enumerate() {
            self.ui.text(l, 32.0, 326.0 + i as f32 * 30.0, 22.0, CREAM);
        }
        panel_wood(0.0, SH, LW, SH);
        self.ui.text_center(&format!("{}日目の あさ", self.g.day), LW / 2.0, SH + 40.0, 34.0, CREAM);
        for (i, l) in lines.iter().skip(3).enumerate() {
            self.ui.text_center(l, LW / 2.0, SH + 110.0 + i as f32 * 34.0, 22.0, CREAM);
        }
        let goal = format!("▶ いまの目標: {}", quest::main_quest(&self.g).title);
        self.ui.text_center(&goal, LW / 2.0, SH + 300.0, 20.0, rgb(255, 230, 140));
        let blink = if (self.ui.t * 2.0) as i32 % 2 == 0 { "▼ Zでつづける" } else { "" };
        self.ui.text_center(blink, LW / 2.0, SH + 380.0, 20.0, CREAM);
        if self.inp.ok || self.inp.click {
            self.ui.audio.sfx("morning");
            return Some(Scr::Town(town::TownSt::new(Some("おはよう! 今日もダンジョンを育てよう".into()))));
        }
        None
    }
}

impl App {
    /// 検証用: --screen town|shop|home|dungeon|battle|status で直接その画面を開く
    fn debug_screen(&mut self, name: &str) {
        self.g = Game::new(7);
        match name {
            "town" => self.scr = Scr::Town(town::TownSt::new(None)),
            "shop" => self.scr = Scr::Shop(shop::ShopSt::new(shop::ShopKind::Magic, &self.g)),
            "armor" => self.scr = Scr::Shop(shop::ShopSt::new(shop::ShopKind::Armor, &self.g)),
            "home" => self.scr = Scr::Home(home::HomeSt::new(&self.g)),
            "morning" => self.scr = Scr::Morning(vec!["ぐっすり ねむった".into(), "魔物が3びき すみついた".into(), "通路は静かだ".into(), "ひょうか 12".into()]),
            "dungeon" | "battle" => {
                self.g.mp = 6;
                for (x, y) in [(8, 11), (8, 10), (8, 9), (7, 9), (9, 9), (8, 8)] {
                    self.g.dungeon.set(x, y, sim::Tile::Floor);
                }
                self.g.pos = (8, 10);
                self.g.facing = sim::Dir::Up;
                let rooms = [(7usize, 9usize, "わらのベッド", sim::Dir::Right, "cat"), (9, 9, "エサおけ", sim::Dir::Left, "pig"), (8, 8, "ゴミすてば", sim::Dir::Down, "slime")];
                for (x, y, k, d, m) in rooms {
                    self.g.dungeon.rooms.push(sim::Room { x, y, w: 1, h: 1, front: (x, y), kind: k.into(), door: d, open4: false, mons: vec![Some(m.into())], chest_open: false, refill: 0 });
                }
                self.g.dungeon.roamers.push(sim::Roamer { x: 8, y: 11, mon: "bat".into() });
                if name == "battle" {
                    self.g.pos = (8, 9);
                    let srcs = vec![sim::Source::Room(0, 0), sim::Source::Room(1, 0), sim::Source::Room(2, 0)];
                    self.scr = Scr::Battle(Box::new(battle::BattleSt::new(&self.g, &srcs, (8, 10))));
                } else {
                    self.scr = Scr::Dungeon(dungeon::DunSt::new());
                }
            }
            n if n.starts_with('p') && n[1..].parse::<usize>().is_ok() => {
                self.debug = true;
                let n = name[1..].parse::<usize>().unwrap_or(1) - 1;
                self.g.preset(n);
                self.g.pos = self.g.dungeon.entrance();
                self.scr = Scr::Dungeon(dungeon::DunSt::new());
            }
            "furn2" | "mayor2" | "magic2" => {
                self.debug = true;
                self.g.preset(3);
                let k = match name {
                    "furn2" => shop::ShopKind::Furniture,
                    "mayor2" => shop::ShopKind::Mayor,
                    _ => shop::ShopKind::Magic,
                };
                self.scr = Scr::Shop(shop::ShopSt::new(k, &self.g));
            }
            "day2" => {
                self.debug = true;
                self.g.dbg_day2();
                self.scr = Scr::Town(town::TownSt::new(None));
            }
            "town2" | "board" => {
                self.debug = true;
                self.g.preset(3);
                self.g.add_item("イノシシのけがわ", 2);
                self.g.add_item("シルバーコイン", 1);
                self.g.add_item("ヘビのはっぱ", 0);
                self.scr = Scr::Town(town::TownSt::new(None));
                if name == "board" {
                    self.g.add_item("リオンのたてがみ", 1);
                    self.ov = Some(overlays::Overlay::Board(board::BoardSt::new(), true));
                }
            }
            "stairs" => {
                // 1Fのボス撃破ずみ。階段のとなりに立つ
                self.debug = true;
                self.g.preset(3);
                self.g.warp(1);
                let p = self.g.dungeon.portal.unwrap_or((15, 8));
                let (stand, face) = if p.0 == 15 { ((14, 8), sim::Dir::Right) } else if p.0 == 1 { ((2, 8), sim::Dir::Left) } else { ((8, 2), sim::Dir::Up) };
                self.g.dungeon.set(stand.0, stand.1, sim::Tile::Floor);
                self.g.pos = stand;
                self.g.facing = face;
                self.scr = Scr::Dungeon(dungeon::DunSt::new());
            }
            n if n.starts_with("boss") && n.len() <= 6 && n[4..].parse::<usize>().map(|f| (3..=data::MAX_FLOOR).contains(&f)).unwrap_or(false) => {
                self.debug = true;
                let f = n[4..].parse::<usize>().unwrap();
                self.g.preset(5 + (f - 3) * 2 + 1);
                // DM_ONEHIT=1 なら 1撃で倒す(撃破メッセージの確認用)
                let onehit = std::env::var("DM_ONEHIT").is_ok();
                self.g.dbg.onehit = onehit;
                let mut b = battle::BattleSt::new(&self.g, &[sim::Source::Boss], (8, 11));
                b.auto = onehit;
                self.scr = Scr::Battle(Box::new(b));
            }
            n if n.starts_with("board") && n[5..].parse::<usize>().map(|f| (3..=data::MAX_FLOOR).contains(&f)).unwrap_or(false) => {
                self.debug = true;
                let f = n[5..].parse::<usize>().unwrap();
                self.g.preset(5 + (f - 3) * 2);
                self.scr = Scr::Town(town::TownSt::new(None));
                self.ov = Some(overlays::Overlay::Board(board::BoardSt::new(), true));
            }
            "boss2" => {
                self.debug = true;
                self.g.preset(4);
                self.scr = Scr::Battle(Box::new(battle::BattleSt::new(&self.g, &[sim::Source::Boss], (8, 11))));
            }
            "panel" => {
                self.debug = true;
                self.g.preset(4);
                self.scr = Scr::Dungeon(dungeon::DunSt::new());
                self.ov = Some(overlays::Overlay::Debug(debug::DebugSt::new()));
            }
            "boss" => {
                self.g.pos = (8, 11);
                self.g.dungeon.set(15, 8, sim::Tile::Portal);
                self.g.dungeon.portal = Some((15, 8));
                self.scr = Scr::Battle(Box::new(battle::BattleSt::new(&self.g, &[sim::Source::Boss], (8, 11))));
            }
            "mimic" | "mimic2" => {
                // ジェリーの ものまね画面(検証用)
                self.debug = true;
                self.g.preset(3);
                for id in ["bat", "pig", "cat", "gobl", "wan", "ghost", "nekomata", "ningen", "powerwan", "icewar", "demonarmor"] {
                    self.g.mimic_known.insert(id.to_string());
                }
                let _ = self.g.set_mimic(1, 1, Some("powerwan"));
                let _ = self.g.set_mimic(1, 0, Some("ghost"));
                let _ = self.g.set_mimic(1, 2, Some("ningen"));
                let sub = if name == "mimic2" {
                    let mut c: Vec<Option<&'static str>> = vec![None];
                    c.extend(self.g.mimic_choices().into_iter().map(Some));
                    Some((1usize, ListMenu::new(), c))
                } else {
                    None
                };
                self.scr = Scr::Dungeon(dungeon::DunSt::new());
                self.ov = Some(overlays::Overlay::Mimic { menu: ListMenu::new(), sub, msg: String::new() });
            }
            "battle2" => {
                // アレンとメイミだけの戦闘(ジェリーはまだ いない)
                self.debug = true;
                self.g = Game::start_story(7);
                self.g.party[2].joined = true;
                self.g.dungeon.set(8, 11, sim::Tile::Floor);
                self.g.dungeon.set(8, 10, sim::Tile::Floor);
                self.g.dungeon.roamers.push(sim::Roamer { x: 8, y: 10, mon: "bat".into() });
                self.g.dungeon.roamers.push(sim::Roamer { x: 8, y: 11, mon: "slime".into() });
                self.scr = Scr::Battle(Box::new(battle::BattleSt::new(&self.g, &[sim::Source::Roamer(0), sim::Source::Roamer(1)], (8, 12))));
            }
            n if n.starts_with("d3_") => {
                // 3D表示の確認用: d3_<x>_<y>_<u|d|l|r>(プリセット3の階。通路の魔物は消す)
                self.debug = true;
                self.g.preset(2);
                self.g.dungeon.roamers.clear();
                let v: Vec<&str> = n[3..].split('_').collect();
                if v.len() == 3 {
                    self.g.pos = (v[0].parse().unwrap_or(8), v[1].parse().unwrap_or(12));
                    self.g.facing = match v[2] { "d" => sim::Dir::Down, "l" => sim::Dir::Left, "r" => sim::Dir::Right, _ => sim::Dir::Up };
                }
                self.scr = Scr::Dungeon(dungeon::DunSt::new());
            }
            "models" => self.scr = Scr::Models(Box::new(models::ModelsSt::new())),
            n if n.starts_with("models_") => {
                let mut st = models::ModelsSt::new();
                // models_<ページ>[_<えらぶ番号>[_b]]
                let v: Vec<&str> = n[7..].split('_').collect();
                st.page = v[0].parse().unwrap_or(0);
                st.sel = v.get(1).and_then(|x| x.parse().ok()).unwrap_or(0);
                st.big = v.get(2) == Some(&"b");
                if let Some(a) = v.get(3).and_then(|x| x.parse::<f32>().ok()) {
                    st.angle = a * 0.5;
                }
                st.spin = false;
                self.scr = Scr::Models(Box::new(st));
            }
            n if n.starts_with("room3d_") => {
                // room3d_<部屋の名前>[_o]: その部屋を 西に 置いた 3D表示
                self.debug = true;
                let v: Vec<&str> = n[7..].split('_').collect();
                self.g.dbg_showroom(v[0], v.get(1) == Some(&"o"));
                self.scr = Scr::Dungeon(dungeon::DunSt::new());
            }
            n if n.starts_with("miasma") => {
                self.debug = true;
                let k: usize = n[6..].parse().unwrap_or(3);
                self.g.dbg_miasma(k);
                if std::env::var("DM_2D").is_ok() {
                    self.g.map3d = false;
                }
                self.scr = Scr::Dungeon(dungeon::DunSt::new());
            }
            "shop_tour" => {
                self.debug = true;
                self.g = Game::start_story(7);
                self.scr = Scr::Shop(shop::ShopSt::new(shop::ShopKind::Magic, &self.g));
            }
            n if n.starts_with("room3dat_") => {
                // room3dat_<部屋>_<x>_<y>_<u|d|l|r>: 部屋を 置いた 1Fで その位置・向きに 立つ
                self.debug = true;
                let v: Vec<&str> = n[9..].split('_').collect();
                self.g.dbg_showroom(v[0], false);
                self.g.pos = (v[1].parse().unwrap_or(8), v[2].parse().unwrap_or(9));
                self.g.facing = match v.get(3).copied().unwrap_or("u") { "d" => sim::Dir::Down, "l" => sim::Dir::Left, "r" => sim::Dir::Right, _ => sim::Dir::Up };
                self.scr = Scr::Dungeon(dungeon::DunSt::new());
            }
            "tour" => {
                // はじめからで はじめて町に戻ったところ(あいさつまわり)
                self.debug = true;
                self.g = Game::start_story(7);
                self.scr = Scr::Town(town::TownSt::new(None));
            }
            "scene_quest" | "scene_quest2" => {
                // クエスト達成の会話(依頼者とアレンたち)
                self.debug = true;
                self.g = Game::start_story(7);
                self.g.party[2].joined = true;
                let q = data::quest(if name == "scene_quest" { "halter" } else { "dino" });
                let (bg, who) = script::quest_giver_scene(q);
                let mut lines = script::quest_done_lines(q, true, "「ワーネコのホルター」を達成! 400G もらった。");
                if let Ok(t) = std::env::var("DM_LINES") {
                    lines = t.split('|').map(|l| l.to_string()).collect();
                }
                self.scr = Scr::Talk(Box::new(talk::TalkSt::new(bg, lines, Scr::None).speaker(who).faces()));
            }
            "scene_intro" | "scene_intro2" | "scene_intro3" | "scene_intro4" | "scene_intro5" => {
                self.debug = true;
                self.g = Game::start_story(7);
                let n = ["scene_intro", "scene_intro2", "scene_intro3", "scene_intro4", "scene_intro5"].iter().position(|x| *x == name).unwrap_or(0);
                let mut sc = self.opening();
                for _ in 0..n {
                    if let Scr::Talk(t) = sc {
                        sc = t.then.unwrap();
                    }
                }
                self.scr = sc;
            }
            "scene_next3" | "scene_next19" | "scene_next20" => {
                // 村長からの手紙(ボスを倒した次の朝)。メイミあり
                self.debug = true;
                self.g = Game::start_story(7);
                self.g.party[2].joined = true;
                let f = match name { "scene_next3" => 3, "scene_next19" => 19, _ => 20 };
                self.scr = Scr::Talk(Box::new(talk::TalkSt::new("scene_home", script::next_boss_scene(f, true), Scr::None).speaker(script::LETTER_FROM)));
            }
            "scene_mei" => self.scr = Scr::Talk(Box::new(talk::TalkSt::new("scene_home", script::mei_join_scene(), Scr::None))),
            "scene_jerry" => self.scr = Scr::Talk(Box::new(talk::TalkSt::new("bg_battle", script::jerry_join_scene(true), Scr::None).with_cast("ジェリー").with_cast("メイミ"))),
            "status" => {
                self.scr = Scr::Town(town::TownSt::new(None));
                self.ov = Some(overlays::Overlay::Status(0));
            }
            _ => {}
        }
    }
}

thread_local! {
    static FULLSCREEN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// --window 幅x高さ で 起動時の窓の大きさを決める(スクリーンショットの検証用)。ふだんは PC 向けの横長
fn window_size() -> (i32, i32) {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--window") {
        if let Some((w, h)) = args.get(i + 1).and_then(|s| s.split_once('x')) {
            if let (Ok(w), Ok(h)) = (w.parse(), h.parse()) {
                return (w, h);
            }
        }
    }
    (1280, 720)
}

fn conf() -> Conf {
    let (ww, wh) = window_size();
    let small: [u8; 16 * 16 * 4] = include_bytes!("../assets/icon/icon_16.rgba").to_owned();
    let medium: [u8; 32 * 32 * 4] = include_bytes!("../assets/icon/icon_32.rgba").to_owned();
    let big: [u8; 64 * 64 * 4] = include_bytes!("../assets/icon/icon_64.rgba").to_owned();
    Conf {
        window_title: "Dungeon Making".to_string(),
        window_width: ww,
        window_height: wh,
        window_resizable: true,
        icon: Some(miniquad::conf::Icon { small, medium, big }),
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    // 最初に 「よみこみちゅう」を 出す(Web では 素材を まとめて ダウンロードするので 少し かかる)
    clear_background(Color::new(0.1, 0.08, 0.12, 1.0));
    draw_text("Loading... (first time only: about 15MB)", 40.0, 80.0, 32.0, WHITE);
    next_frame().await;
    assets::init().await;
    let art = art::Art::load().await;
    let font = assets::font_bytes().await;
    let ui = Ui::new(art, font);
    let mut app = App {
        g: Game::new(1),
        ui,
        inp: Input::default(),
        view: View::new(),
        scr: Scr::Title(ListMenu::new()),
        ov: None,
        toast: None,
        quit: false,
        debug: false,
        cam3: std::cell::Cell::new([0.0; 6]),
        models: Default::default(),
        prev_scr: None,
        jerry_synced: None,
    };
    let args: Vec<String> = std::env::args().collect();
    app.debug = args.iter().any(|a| a == "--debug");
    if let Some(i) = args.iter().position(|a| a == "--screen") {
        if let Some(name) = args.get(i + 1) {
            app.debug_screen(name);
        }
    }
    if let Some(i) = args.iter().position(|a| a == "--keys") {
        if let Some(p) = args.get(i + 1) {
            if let Ok(s) = std::fs::read_to_string(p) {
                app.inp.script = s.lines().filter(|l| !l.trim_start().starts_with('#')).flat_map(|l| l.split_whitespace()).map(|t| t.to_string()).collect();
                app.inp.scripted = true;
            }
        }
    }
    // 音: --shot / --keys のときと --mute では 鳴らさない(検証を静かに、デバイスが無くても止まらないように)
    let mute = std::env::var("DM_MUTE").is_ok() || args.iter().any(|a| a == "--mute" || a == "--shot" || a == "--keys");
    if !mute {
        app.ui.audio = audio::Audio::load().await;
    }
    // 設定(音量・画面など)は settings.json に保存して、次の起動でも使う。検証用の起動(--screen/--keys/--shot)では 読み書きしない
    let persist = !args.iter().any(|a| a == "--screen" || a == "--keys" || a == "--shot" || a == "--debug");
    if persist {
        if let Some(st) = assets::load_text("settings").and_then(|t| serde_json::from_str::<(u8, u8, u8, u8, bool, bool)>(&t).ok()) {
            app.g.set_settings(st);
        }
    }
    let mut saved_settings = app.g.settings();
    let rt = render_target((LW * RT_K) as u32, (LH * RT_K) as u32);
    rt.texture.set_filter(FilterMode::Linear);
    let mut frames = 0u32;
    loop {
        if persist && app.g.settings() != saved_settings {
            saved_settings = app.g.settings();
            if let Ok(t) = serde_json::to_string(&saved_settings) {
                assets::save_text("settings", &t);
            }
        }
        app.view.mode = app.g.view_mode;
        app.ui.audio.mode.set(app.g.sound_mode);
        app.ui.audio.bgm_vol.set(app.g.bgm_vol as f32 / 10.0);
        app.ui.audio.sfx_vol.set(app.g.sfx_vol as f32 / 10.0);
        if is_key_pressed(KeyCode::M) {
            // Mで 音のオン/オフ
            app.g.sound_mode = if app.g.sound_mode == 0 { 2 } else { 0 };
        }
        app.view.update();
        app.ui.t += get_frame_time();
        if is_key_pressed(KeyCode::F11) {
            FULLSCREEN.with(|f| {
                let on = !f.get();
                f.set(on);
                set_fullscreen(on);
            });
        }
        if is_key_pressed(KeyCode::Tab) {
            // Tab で ダンジョンのマップを 3D / 2D に切りかえ
            app.g.map3d = !app.g.map3d;
        }
        if is_key_pressed(KeyCode::F4) {
            // F4 で 3Dモデル一覧を 開く/とじる
            if matches!(app.scr, Scr::Models(_)) {
                if let Some(p) = app.prev_scr.take() {
                    app.scr = p;
                }
            } else {
                let p = std::mem::replace(&mut app.scr, Scr::Models(Box::new(models::ModelsSt::new())));
                app.prev_scr = Some(p);
            }
        }
        if is_key_pressed(KeyCode::F9) {
            app.g.view_mode = (app.g.view_mode + 1) % 3;
        }
        if app.debug && is_key_pressed(KeyCode::F1) {
            app.ov = if matches!(app.ov, Some(overlays::Overlay::Debug(_))) { None } else { Some(overlays::Overlay::Debug(debug::DebugSt::new())) };
        }
        app.inp.poll(&app.view);
        set_camera(&Camera2D { render_target: Some(rt.clone()), ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, LW, LH)) });
        clear_background(BLACK);
        app.frame();
        app.update_music();
        set_default_camera();
        app.view.tip = match &app.scr {
            Scr::Dungeon(st) if matches!(st.sub, dungeon::DSub::Play) => {
                if app.g.map3d {
                    ui::TIP_3D
                } else {
                    ui::TIP_2D
                }
            }
            _ => ui::TIP_DEFAULT,
        };
        app.view.draw(&rt.texture, &app.ui);
        if let Some(p) = app.inp.shot.take() {
            get_screen_data().export_png(&p);
        }
        if app.inp.script.is_empty() && app.inp.script_wait == 0 && args.iter().any(|a| a == "--keys") {
            frames += 1;
            if frames > 20 && args.iter().any(|a| a == "--exit-after-keys") {
                let g = &app.g;
                println!(
                    "SUMMARY used={} day={} gold={} floor={} rooms={} floor_tiles={} kills={} ate={:?} loot_left={}",
                    app.inp.used,
                    g.day,
                    g.gold,
                    g.floor_no(),
                    g.dungeon.rooms.len(),
                    g.dungeon.floor_count(),
                    g.dungeon.kills,
                    g.ate,
                    g.count("イノシシのけがわ") + g.count("シルバーコイン")
                );
                break;
            }
        }
        frames += 1;
        if let Some(i) = args.iter().position(|a| a == "--shot") {
            // --shot <frames> <path>: 指定フレーム後にスクリーンショットを保存して終了
            let n: u32 = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(10);
            if frames == n {
                if let Some(p) = args.get(i + 2) {
                    get_screen_data().export_png(p);
                }
                break;
            }
        }
        if app.quit || is_key_pressed(KeyCode::F10) {
            break;
        }
        next_frame().await;
    }
}
