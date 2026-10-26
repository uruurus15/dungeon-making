//! テストプレイ用のデバッグ機能。`--debug` で起動したときだけ有効(debug.cmd)。
//! セーブも別ファイル(savedata_debug.json)なので、本番のセーブは上書きしない。F1 でパネルを開く。

use crate::data::*;
use crate::sim::*;
use crate::town::comma;
use crate::ui::*;
use crate::App;
use macroquad::prelude::*;

// ---------------- ゲーム状態のいじり方 ----------------

impl Game {
    /// 前のマスまで掘って、部屋を置く(MPは消費しない)
    fn dbg_room(&mut self, kind: &str, front: (usize, usize), stand: (usize, usize), face: Dir) {
        let def = room_def(kind);
        if def.w == 1 {
            if self.dungeon.tile(front.0, front.1) == Tile::Rock {
                self.dungeon.set(front.0, front.1, Tile::Floor);
            }
        } else if let Some(c) = pos_add(front, face) {
            // 大きな部屋は 掘りずみの床が必要なので、先に3x3を床にしておく
            for dy in 0..def.h {
                for dx in 0..def.w {
                    self.dungeon.set(c.0 - 1 + dx, c.1 - 1 + dy, Tile::Floor);
                }
            }
        }
        self.pos = stand;
        self.facing = face;
        self.mp = self.mp.max(9);
        self.add_item(kind, 1);
        let _ = self.place_room(kind);
        self.mp = self.mp_max;
        self.undo.clear();
    }

    /// 3D表示の 確認用: 通路の 西がわに その部屋を 1つ 置いて、魔物を すみつかせ、前に 立つ
    pub fn dbg_showroom(&mut self, kind: &str, open_chest: bool) {
        *self = Game::new(5);
        self.dbg_corridor(8, 5, 11);
        self.dbg_room(kind, (7, 9), (8, 9), Dir::Left);
        self.dbg_fill_monsters();
        if open_chest {
            if let Some(r) = self.dungeon.rooms.iter_mut().find(|r| r.kind == kind) {
                r.chest_open = true;
            }
        }
        self.pos = (8, 9);
        self.facing = Dir::Left;
        self.dungeon.roamers.clear();
        self.map3d = true;
    }

    /// 瘴気発生装置を n個(最大5) 置いた 1F(確認用)
    pub fn dbg_miasma(&mut self, n: usize) {
        *self = Game::new(5);
        self.dbg_corridor(8, 3, 11);
        let spots = [(7usize, 9usize, Dir::Left), (9, 9, Dir::Right), (7, 7, Dir::Left), (9, 7, Dir::Right), (7, 5, Dir::Left)];
        for (x, y, d) in spots.iter().take(n) {
            self.dbg_room(MIASMA_ROOM, (*x, *y), (8, *y), *d);
        }
        self.dbg_room("エサおけ", (9, 5), (8, 5), Dir::Right);
        self.dbg_fill_monsters();
        self.pos = (8, 11);
        self.facing = Dir::Up;
        self.dungeon.roamers.clear();
        self.map3d = true;
    }

    fn dbg_corridor(&mut self, x: usize, y0: usize, y1: usize) {
        for y in y0..=y1 {
            self.dungeon.set(x, y, Tile::Floor);
        }
    }

    pub fn dbg_boost(&mut self, level: i32) {
        for (i, m) in self.party.iter_mut().enumerate() {
            m.hp_max += 12 * level;
            m.mp_max += 3 * level;
            m.str_ += 3 * level;
            m.con += 3 * level;
            m.wis += 2 * level;
            m.spd += level;
            m.hp = m.hp_max;
            m.mp = m.mp_max;
            let _ = i;
        }
    }

    /// 1Fを「ボスの条件ぎりぎり」まで作る
    fn dbg_build_floor1(&mut self) {
        self.dbg_corridor(8, 5, 11);
        let kinds = ["わらのベッド", "ゴミすてば", "エサおけ", "わらのベッド", "ゴミすてば", "エサおけ"];
        let mut k = 0;
        for y in [9usize, 7, 5] {
            self.dbg_room(kinds[k], (7, y), (8, y), Dir::Left);
            self.dbg_room(kinds[k + 1], (9, y), (8, y), Dir::Right);
            k += 2;
        }
        self.dbg_room("てつのたからばこ", (8, 4), (8, 5), Dir::Up);
        self.dungeon.kills = BOSS_KILLS - 1;
    }

    fn dbg_build_floor2(&mut self) {
        self.dbg_corridor(8, 5, 11);
        self.dbg_room("さかば", (9, 10), (8, 10), Dir::Right);
        self.dbg_room("はたけ", (7, 9), (8, 9), Dir::Left);
        self.dbg_room("さかば", (9, 8), (8, 8), Dir::Right);
        self.dbg_room("さかば", (9, 7), (8, 7), Dir::Right);
        self.dbg_room("エサおけ", (7, 7), (8, 7), Dir::Left);
        self.dbg_room("さかば", (9, 6), (8, 6), Dir::Right);
        self.dbg_room("わらのベッド", (7, 6), (8, 6), Dir::Left);
        self.dbg_room("さいだん", (8, 4), (8, 5), Dir::Up);
        self.dungeon.kills = boss_rule(2).kills - 1;
    }

    /// 部屋にだけ魔物を入れる(魔穴は開かない)
    fn dbg_fill_monsters(&mut self) {
        let floor = self.dungeon.floor;
        for i in 0..self.dungeon.rooms.len() {
            let kind = self.dungeon.rooms[i].kind.clone();
            let table = room_spawns(floor, &kind);
            if table.is_empty() {
                continue;
            }
            for k in 0..self.dungeon.rooms[i].mons.len().min(4) {
                let m = self.rng.pick(table);
                self.dungeon.rooms[i].mons[k] = Some(m.to_string());
            }
        }
    }

    /// 夜の処理だけを実行(日付は進めない): 部屋に魔物を入れて、条件が揃えば魔穴を開く
    fn dbg_fill_and_portal(&mut self) -> NightReport {
        let mut rep = NightReport::default();
        self.night_here(&mut rep);
        rep
    }

    /// 3F〜最上階 の部屋のならべ方(layout::plan_steps。ボットのテストと同じ)
    fn dbg_build_comb(&mut self, floor: usize) {
        for st in crate::layout::plan_steps(floor) {
            match st.room {
                None => self.dungeon.set(st.front.0, st.front.1, Tile::Floor),
                Some(kind) => self.dbg_room(kind, st.front, st.stand, st.face),
            }
        }
        self.undo.clear();
    }

    fn dbg_build_any(&mut self, floor: usize) {
        match floor {
            1 => self.dbg_build_floor1(),
            2 => self.dbg_build_floor2(),
            f => self.dbg_build_comb(f),
        }
        self.dungeon.kills = boss_rule(floor).kills;
        self.dbg_fill_monsters();
    }

    /// 3F〜最上階 の開始/ボス直前: 手前の階はぜんぶ攻略ずみにしてから、その階まで降りる
    fn preset_deep(&mut self, floor: usize, ready: bool) -> String {
        self.gold = 30000;
        self.dbg_boost(floor as i32);
        self.add_item("てつのカギ", 3);
        self.add_item("かいふくやく", 10);
        self.add_item("まほうのくすり", 3);
        self.party[0].spells.push("heal".to_string());
        for f in 1..floor {
            self.dbg_build_any(f);
            let rep = self.dbg_fill_and_portal();
            if rep.portals.is_empty() {
                self.dbg_open_portal();
            }
            self.dungeon.boss_defeated = true;
            if let Some(p) = self.dungeon.portal {
                self.dungeon.set(p.0, p.1, Tile::StairsDown);
            }
            let q = quest(FLOOR_MAIN_QUEST[f - 1]);
            self.add_item(q.need, q.count);
            self.deliver_quest(q.id).ok();
            while self.count("魔法のオリーブオイル") > 0 {
                self.shovel_polish().ok();
            }
            for (fl, k) in FURNITURE_UNLOCK.iter().filter(|(fl, _)| *fl <= f + 1) {
                let _ = fl;
                self.add_item(k, 1);
            }
            self.descend();
        }
        self.mp = self.mp_max;
        if ready {
            self.dbg_build_any(floor);
            let rep = self.dbg_fill_and_portal();
            if rep.portals.is_empty() {
                self.dbg_open_portal();
            }
            return format!("{}F ボス直前(魔穴が開いた状態)。評価{}", floor, self.rating());
        }
        for (_, k) in FURNITURE_UNLOCK.iter().filter(|(fl, _)| *fl <= floor) {
            self.add_item(k, 2);
        }
        format!("{}F 開始(手前の階は攻略ずみ)", floor)
    }

    pub fn preset(&mut self, which: usize) -> String {
        let seed = self.rng.next() as u64 + 1;
        *self = Game::new(seed);
        self.mp = 99;
        self.mp_max = 99;
        match which {
            w if (5..5 + 2 * (MAX_FLOOR - 2)).contains(&w) => {
                let floor = 3 + (which - 5) / 2;
                let ready = (which - 5) % 2 == 1;
                self.preset_deep(floor, ready)
            }
            0 => {
                self.mp = START_MP;
                self.mp_max = START_MP;
                "1F 初日からやりなおし".to_string()
            }
            1 | 2 => {
                self.gold = 3000;
                self.add_item("てつのカギ", 3);
                self.dbg_boost(1);
                self.dbg_build_floor1();
                self.dungeon.kills = BOSS_KILLS;
                self.dbg_fill_monsters();
                if which == 2 {
                    let rep = self.dbg_fill_and_portal();
                    self.mp = START_MP;
                    self.mp_max = START_MP;
                    return format!("1F ボス直前(魔穴: {})。評価{}", if rep.portals.is_empty() { "まだ" } else { "開いた" }, self.rating());
                }
                self.mp = START_MP;
                self.mp_max = START_MP;
                format!("1F 今夜 魔穴が開く直前(評価{}・倒した数{})。寝ると開く", self.rating(), self.dungeon.kills)
            }
            _ => {
                self.gold = 6000;
                self.dbg_boost(2);
                self.dbg_build_floor1();
                self.dungeon.kills = BOSS_KILLS;
                self.dbg_fill_and_portal();
                self.dungeon.boss_defeated = true;
                if let Some(p) = self.dungeon.portal {
                    self.dungeon.set(p.0, p.1, Tile::StairsDown);
                }
                self.add_item("ヘビのはっぱ", 1);
                self.deliver_quest("snake").ok();
                self.shovel_polish().ok();
                self.descend();
                self.mp = 99;
                for (_, k) in FURNITURE_UNLOCK.iter().filter(|(f, _)| *f <= 2) {
                    self.add_item(k, 2);
                }
                self.add_item("てつのカギ", 3);
                if which == 4 {
                    self.dbg_boost(1);
                    self.dbg_build_floor2();
                    self.dungeon.kills = boss_rule(2).kills;
                    let rep = self.dbg_fill_and_portal();
                    self.mp = 20;
                    self.mp_max = 20;
                    return format!("2F ボス直前(魔穴: {})。評価{}", if rep.portals.is_empty() { "まだ" } else { "開いた" }, self.rating());
                }
                self.mp = 20;
                self.mp_max = 20;
                "2F 開始".to_string()
            }
        }
    }

    /// 操作性の計測用: 部屋のある2日目(部屋6つに魔物入り・売る物つき)
    pub fn dbg_day2(&mut self) {
        *self = Game::new(5);
        // キー操作の検証スクリプト(tools/day_new.txt)は 2D の操作(矢印=むき・すすむ)で書いてある
        self.map3d = false;
        self.dbg_corridor(8, 5, 11);
        let kinds = ["わらのベッド", "ゴミすてば", "エサおけ", "わらのベッド", "ゴミすてば", "エサおけ"];
        let mut k = 0;
        for y in [9usize, 7, 5] {
            self.dbg_room(kinds[k], (7, y), (8, y), Dir::Left);
            self.dbg_room(kinds[k + 1], (9, y), (8, y), Dir::Right);
            k += 2;
        }
        self.dbg_fill_monsters();
        self.add_item("イノシシのけがわ", 3);
        self.add_item("シルバーコイン", 2);
        self.pos = self.dungeon.entrance();
        self.facing = Dir::Up;
        self.mp = START_MP;
        self.undo.clear();
    }

    pub fn dbg_all_rooms_spawn(&mut self) -> usize {
        let mut rep = NightReport::default();
        self.night_here(&mut rep);
        rep.spawned
    }

    pub fn dbg_collect_hidden(&mut self) -> usize {
        let items: Vec<(usize, usize, String)> = self.dungeon.hidden.drain(..).collect();
        let n = items.len();
        for (_, _, name) in items {
            if let Some(g) = name.strip_prefix('G') {
                self.gold += g.parse::<i32>().unwrap_or(0);
            } else {
                self.add_item(&name, 1);
            }
        }
        n
    }

    pub fn dbg_open_portal(&mut self) -> bool {
        if self.dungeon.portal.is_some() || self.dungeon.boss_defeated {
            return false;
        }
        for t in self.dungeon.tiles.iter_mut() {
            if *t == Tile::Barrier {
                *t = Tile::Rock;
            }
        }
        self.dungeon.barrier_open = true;
        let p = (15usize, 8usize);
        self.dungeon.set(p.0, p.1, Tile::Portal);
        self.dungeon.portal = Some(p);
        true
    }

    pub fn dbg_quests(&mut self) -> usize {
        let mut n = 0;
        for q in QUESTS {
            if self.reached < q.min_floor {
                continue;
            }
            if self.quest_state(q.id) == 0 {
                self.accept_quest(q.id);
            }
            if self.quest_state(q.id) == 1 {
                self.add_item(q.need, q.count);
                if self.deliver_quest(q.id).is_ok() {
                    n += 1;
                }
            }
        }
        n
    }
}

// ---------------- パネル ----------------

#[derive(PartialEq, Clone, Copy)]
pub enum DPage {
    Main,
    Items,
    Floor,
    Preset,
}

pub struct DebugSt {
    pub page: DPage,
    pub menu: ListMenu,
    pub msg: String,
}

impl DebugSt {
    pub fn new() -> DebugSt {
        DebugSt { page: DPage::Main, menu: ListMenu::new(), msg: "F1 / Esc で とじる".to_string() }
    }
}

fn onoff(b: bool) -> &'static str {
    if b {
        "ON"
    } else {
        "OFF"
    }
}

impl App {
    /// true を返したら閉じる
    pub fn debug_panel(&mut self, st: &mut DebugSt) -> bool {
        let in_battle = matches!(self.scr, crate::Scr::Battle(_));
        let g = &self.g;
        let ui = &self.ui;
        // 上画面: 状態
        draw_rectangle(0.0, 0.0, LW, SH, rgba(10, 10, 20, 235));
        ui.text("DEBUG", 16.0, 10.0, 28.0, rgb(255, 120, 120));
        ui.text(&format!("{}日目  B{}F  座標{:?}  {}G  MP {}/{}", g.day, g.floor_no(), g.pos, comma(g.gold), g.mp, g.mp_max), 16.0, 48.0, 17.0, CREAM);
        let (r, k, j, c, p) = g.rating_parts();
        ui.text(&format!("評価 {} = 部屋{} + 種類{} + 分かれ道{} + 曲がり角{} - 掘りすぎ{}", g.rating(), r, k, j, c, p), 16.0, 76.0, 17.0, CREAM);
        ui.text(&format!("到達階 {}   全滅 {}回   魔穴 {:?}   barrier {}", g.reached, g.total_wipes, g.dungeon.portal, onoff(g.dungeon.barrier_open)), 16.0, 104.0, 17.0, CREAM);
        ui.text("ボス出現の条件:", 16.0, 140.0, 18.0, GOLDEN);
        for (i, (name, have, need)) in g.boss_progress().iter().enumerate() {
            let ok = have >= need;
            ui.text(&format!("  {}  {} / {}  {}", name, have, need, if ok { "OK" } else { "" }), 16.0, 168.0 + i as f32 * 26.0, 18.0, if ok { rgb(150, 235, 120) } else { CREAM });
        }
        let rooms = g.dungeon.rooms.len();
        let mons: usize = g.dungeon.rooms.iter().map(|r| r.alive()).sum::<usize>() + g.dungeon.roamers.len();
        ui.text(&format!("部屋 {}  いる魔物 {}  掘った床 {}  もどす履歴 {}", rooms, mons, g.dungeon.floor_count(), g.undo.len()), 16.0, 262.0, 17.0, CREAM);
        ui.text(&format!("むてき {}   ワンパン {}   ごはん {:?}", onoff(g.dbg.god), onoff(g.dbg.onehit), g.ate), 16.0, 290.0, 17.0, CREAM);
        ui.text(&st.msg, 16.0, 380.0, 18.0, rgb(255, 230, 140));

        // 下画面: メニュー
        panel_wood(0.0, SH, LW, SH);
        let mut rows: Vec<Row> = vec![];
        match st.page {
            DPage::Main => {
                for l in [
                    "おかね +1000G",
                    "おかね +10000G",
                    "アイテムを ふやす…",
                    "部屋を ぜんぶ x5",
                    "シャベルMPを ぜんかいふく",
                    "シャベルMPの上限を 99に",
                    "魔法のオリーブオイル x3",
                    "パーティを ぜんかいふく",
                    "むてき",
                    "ワンパン",
                    "夜を すごす(寝る)",
                    "いまの階の部屋に 魔物を入れる",
                    "魔穴を いま 開く",
                    "隠しアイテムを ぜんぶ 手に入れる",
                    "クエストを ぜんぶ 達成",
                    "階を いどう…",
                    "プリセット…",
                    "とじる",
                ] {
                    let risky = matches!(l, "夜を すごす(寝る)" | "階を いどう…" | "プリセット…");
                    let right = match l {
                        "むてき" => onoff(g.dbg.god).to_string(),
                        "ワンパン" => onoff(g.dbg.onehit).to_string(),
                        _ if risky && in_battle => "戦闘中は使えない".to_string(),
                        _ => String::new(),
                    };
                    rows.push(Row::new(l).right(right).enabled(!(risky && in_battle)));
                }
            }
            DPage::Items => {
                for d in ITEMS {
                    rows.push(Row::new(d.name).right(format!("x{}", g.count(d.name))));
                }
            }
            DPage::Floor => {
                for f in 1..=MAX_FLOOR {
                    rows.push(Row::new(format!("B{}F の入口へ", f)).right(if f == g.floor_no() { "いま居る階" } else { "" }));
                }
            }
            DPage::Preset => {
                let mut labels: Vec<String> = ["1F 初日からやりなおし", "1F 今夜 魔穴が開く直前(寝ると開く)", "1F ボス直前(魔穴が開いた状態)", "2F 開始(1Fは攻略ずみ)", "2F ボス直前(魔穴が開いた状態)"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                for f in 3..=MAX_FLOOR {
                    labels.push(format!("{}F 開始", f));
                    labels.push(format!("{}F ボス直前", f));
                }
                for l in labels {
                    rows.push(Row::new(l));
                }
            }
        }
        let ev = st.menu.run(&self.ui, &self.inp, &rows, Rect::new(8.0, SH + 8.0, LW - 16.0, SH - 16.0), 40.0, 20.0, Style::Wood, true);
        match ev {
            MenuEv::Cancel => {
                if st.page == DPage::Main {
                    return true;
                }
                st.page = DPage::Main;
                st.menu = ListMenu::new();
            }
            MenuEv::Confirm(i) => match st.page {
                DPage::Main => match i {
                    0 => {
                        self.g.gold += 1000;
                        st.msg = "+1000G".into();
                    }
                    1 => {
                        self.g.gold += 10000;
                        st.msg = "+10000G".into();
                    }
                    2 => {
                        st.page = DPage::Items;
                        st.menu = ListMenu::new();
                    }
                    3 => {
                        for d in ITEMS.iter().filter(|d| d.kind == Kind::Room) {
                            self.g.add_item(d.name, 5);
                        }
                        st.msg = "部屋を 全種類 5つずつ 追加".into();
                    }
                    4 => {
                        self.g.mp = self.g.mp_max;
                        st.msg = "シャベルMP 全回復".into();
                    }
                    5 => {
                        self.g.mp_max = 99;
                        self.g.mp = 99;
                        st.msg = "シャベルMP上限 99".into();
                    }
                    6 => {
                        self.g.add_item("魔法のオリーブオイル", 3);
                        st.msg = "オイル x3(自宅でシャベルを磨く)".into();
                    }
                    7 => {
                        for m in self.g.party.iter_mut() {
                            m.hp = m.hp_max;
                            m.mp = m.mp_max;
                            m.poison = false;
                            m.stun = false;
                        }
                        st.msg = "パーティ全回復".into();
                    }
                    8 => {
                        self.g.dbg.god = !self.g.dbg.god;
                        st.msg = format!("むてき {}", onoff(self.g.dbg.god));
                    }
                    9 => {
                        self.g.dbg.onehit = !self.g.dbg.onehit;
                        st.msg = format!("ワンパン {}", onoff(self.g.dbg.onehit));
                    }
                    10 => {
                        let rep = self.g.sleep();
                        st.msg = format!("夜をすごした。{}日目。魔物 +{} / 通路 +{} / 魔穴 {:?}", self.g.day, rep.spawned, rep.roamers, rep.portals);
                    }
                    11 => {
                        let n = self.g.dbg_all_rooms_spawn();
                        st.msg = format!("魔物を {} ひき 入れた", n);
                    }
                    12 => {
                        st.msg = if self.g.dbg_open_portal() { "魔穴を開いた(東)".into() } else { "もう開いている/倒しずみ".into() };
                    }
                    13 => {
                        let n = self.g.dbg_collect_hidden();
                        st.msg = format!("隠しアイテム {} 個 回収", n);
                    }
                    14 => {
                        let n = self.g.dbg_quests();
                        st.msg = format!("クエスト {} 件 達成", n);
                    }
                    15 => {
                        st.page = DPage::Floor;
                        st.menu = ListMenu::new();
                    }
                    16 => {
                        st.page = DPage::Preset;
                        st.menu = ListMenu::new();
                    }
                    _ => return true,
                },
                DPage::Items => {
                    let d = &ITEMS[i];
                    let n = if d.kind.is_equip() || d.kind == Kind::Quest || d.kind == Kind::Oil { 1 } else { 5 };
                    self.g.add_item(d.name, n);
                    st.msg = format!("{} x{}", d.name, n);
                }
                DPage::Floor => {
                    let f = i + 1;
                    self.g.reached = self.g.reached.max(f);
                    self.g.change_floor(f);
                    self.g.pos = self.g.dungeon.entrance();
                    st.msg = format!("B{}F へ", f);
                }
                DPage::Preset => {
                    st.msg = self.g.preset(i);
                    st.page = DPage::Main;
                    st.menu = ListMenu::new();
                }
            },
            MenuEv::None => {}
        }
        false
    }
}
