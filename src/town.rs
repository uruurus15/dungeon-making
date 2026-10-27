//! 街: 一枚絵 + 施設選択。クエストは「クエスト」ひとつにまとめてある。

use crate::data::*;
use crate::overlays::Overlay;
use crate::quest::main_quest;
use crate::shop::{ShopKind, ShopSt};
use crate::ui::*;
use crate::{App, Scr};
use macroquad::prelude::*;

static LAST_CURSOR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub struct TownSt {
    pub menu: ListMenu,
    pub msg: Option<String>,
    /// 「ダンジョンへ」で階を選ぶとき
    pub floor_pick: Option<ListMenu>,
}

impl TownSt {
    pub fn new(msg: Option<String>) -> TownSt {
        let mut menu = ListMenu::new();
        menu.cursor = LAST_CURSOR.load(std::sync::atomic::Ordering::Relaxed);
        TownSt { menu, msg, floor_pick: None }
    }
}

pub struct Spot {
    pub label: &'static str,
    pub caption: &'static str,
    pub rect: Rect,
}

pub const SPOTS: [Spot; 8] = [
    Spot { label: "クエスト", caption: "クエストボード", rect: Rect { x: 402.0, y: 306.0, w: 82.0, h: 96.0 } },
    Spot { label: "武具屋", caption: "武具屋  ― おねえさん", rect: Rect { x: 96.0, y: 84.0, w: 168.0, h: 168.0 } },
    Spot { label: "魔法屋", caption: "魔法屋  ― おじさん", rect: Rect { x: 286.0, y: 55.0, w: 166.0, h: 182.0 } },
    Spot { label: "家具屋", caption: "家具屋  ― だいくさん", rect: Rect { x: 428.0, y: 92.0, w: 148.0, h: 212.0 } },
    Spot { label: "広場", caption: "広場  ― むらの女の子", rect: Rect { x: 204.0, y: 284.0, w: 130.0, h: 112.0 } },
    Spot { label: "村長の家", caption: "村長  ― そんちょう", rect: Rect { x: 92.0, y: 292.0, w: 80.0, h: 92.0 } },
    Spot { label: "じたく", caption: "アレンの家", rect: Rect { x: 0.0, y: 112.0, w: 120.0, h: 190.0 } },
    Spot { label: "ダンジョンへ", caption: "ダンジョンの入口", rect: Rect { x: 0.0, y: 0.0, w: 150.0, h: 112.0 } },
];

/// メニューの行番号: 0..=7 = SPOTS(0=クエストボード)、8=換金アイテムを全部売る、9=メニュー
const ROW_QUEST: usize = 0;
/// 新入荷の赤い!を出す施設(SPOTS の 1..=4: 武具屋・魔法屋・家具屋・広場)
/// 話を聞くことがある施設(SPOTS の番号 → 店の種類)
fn talk_kind(i: usize) -> Option<ShopKind> {
    match i {
        1 => Some(ShopKind::Armor),
        2 => Some(ShopKind::Magic),
        3 => Some(ShopKind::Furniture),
        4 => Some(ShopKind::Plaza),
        5 => Some(ShopKind::Mayor),
        _ => None,
    }
}

const NEWS_KEYS: [&str; 4] = ["armor", "magic", "furn", "plaza"];
const ROW_SELL: usize = 8;

impl App {
    pub fn town(&mut self, st: &mut TownSt) -> Option<Scr> {
        // はじめて 町に戻ってきたとき: あいさつまわりの はじまり
        if crate::script::tour_on(&self.g) && !self.g.talks_seen.contains("tour:town") {
            self.g.talks_seen.insert("tour:town".to_string());
            let t = crate::talk::TalkSt::new("scene_town", crate::script::tour_town_lines(), Scr::Town(TownSt::new(None)));
            return Some(Scr::Talk(Box::new(t)));
        }
        let mq = main_quest(&self.g);
        let loot_total: i32 = ITEMS.iter().filter(|d| d.kind == Kind::Loot).map(|d| d.sell * self.g.count(d.name)).sum();
        let deliver = self.g.reportable_quests();
        let ui = &self.ui;
        ui.art.draw("scene_town", 0.0, 0.0, LW, SH);
        // 上画面: 選択中の施設を光らせる
        let cur = st.menu.cursor;
        if cur < SPOTS.len() && st.floor_pick.is_none() {
            let sp = &SPOTS[cur];
            let r = sp.rect;
            let p = pulse(ui.t);
            draw_rectangle(r.x, r.y, r.w, r.h, rgba(255, 240, 120, (30.0 + 40.0 * p) as u8));
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 3.0, rgba(255, 230, 90, (150.0 + 100.0 * p) as u8));
            let w = ui.measure(sp.caption, 20.0) + 28.0;
            draw_rectangle(12.0, 12.0, w, 36.0, rgba(24, 18, 10, 220));
            draw_rectangle_lines(12.0, 12.0, w, 36.0, 2.0, GOLDEN);
            ui.text(sp.caption, 26.0, 19.0, 20.0, CREAM);
        }
        // 新入荷・新しい食材のある家の上に、赤い!
        for (i, sp) in SPOTS.iter().enumerate() {
            let news = if (1..=4).contains(&i) { self.g.has_news(NEWS_KEYS[i - 1]) || (i == 4 && self.g.peddler_here()) } else { i == 0 && deliver > 0 } || talk_kind(i).map(|k| crate::script::has_talk(&self.g, k)).unwrap_or(false);
            if news {
                let bob = (ui.t * 5.0 + i as f32).sin() * 3.0;
                draw_bang(sp.rect.x + sp.rect.w / 2.0, sp.rect.y + 16.0 + bob, 14.0, ui.t, ui);
            }
        }
        chip(ui, LW - 200.0, 12.0, 188.0, 34.0, &format!("{} G", comma(self.g.gold)), 22.0);
        chip(ui, LW - 200.0, 52.0, 188.0, 30.0, &format!("{}日目", self.g.day), 20.0);
        // ひとこと + いまの目標(メインクエスト)
        let unarmed = crate::script::tour_on(&self.g) && self.g.party[0].equip[0].is_none() && self.g.count("ナイフ") == 0;
        let msg = st.msg.clone().unwrap_or_else(|| {
            if crate::script::tour_left(&self.g) > 0 {
                "アレン「みんなに あいさつ まわりを しよう。赤い「!」の ある 家へ 行ってみよう。」".to_string()
            } else if unarmed {
                "アレン「武器が ないと 心細いな…。武具屋で ナイフを 買っておこう。」".to_string()
            } else {
                "どこへ行く?".to_string()
            }
        });
        // 下のメッセージ欄: 1行目にいまの目標(メインクエスト)、2行目にひとこと
        msgbox(70.0, 366.0, 420.0, 62.0);
        let goal = format!("▶ {}", mq.title);
        let mut gs = 15.0;
        while gs > 11.0 && ui.measure(&goal, gs) > 396.0 {
            gs -= 1.0;
        }
        ui.text(&goal, 84.0, 371.0, gs, rgb(255, 230, 140));
        ui.text_wrap(&msg, 84.0, 392.0, 396.0, 15.0, CREAM, 18.0);

        // 下画面
        panel_wood(0.0, SH, LW, SH);
        let mut next: Option<Scr> = None;
        if let Some(fm) = &mut st.floor_pick {
            ui.text("どの階から はじめる?", 18.0, SH + 10.0, 22.0, CREAM);
            let rows: Vec<Row> = (1..=self.g.reached).map(|f| Row::new(format!("地下{}階", f)).right(if f == self.g.floor_no() { "いま居る階" } else { "" })).collect();
            match fm.run(ui, &self.inp, &rows, Rect::new(10.0, SH + 50.0, 290.0, 200.0), 48.0, 22.0, Style::Wood, true) {
                MenuEv::Confirm(i) => {
                    self.g.undo.clear();
                    self.g.warp(i + 1);
                    self.g.return_to_entrance();
                    next = Some(Scr::Dungeon(crate::dungeon::DunSt::new()));
                }
                MenuEv::Cancel => st.floor_pick = None,
                MenuEv::None => {}
            }
        } else {
            ui.text("まちの みんな", 18.0, SH + 8.0, 22.0, CREAM);
            let mut rows: Vec<Row> = SPOTS
                .iter()
                .enumerate()
                .map(|(i, s)| match i {
                    0 => Row::new(s.label).right(if deliver > 0 { "報告できる" } else { "" }).badge(deliver > 0),
                    1..=4 => Row::new(s.label).badge(self.g.has_news(NEWS_KEYS[i - 1]) || (i == 4 && self.g.peddler_here()) || talk_kind(i).map(|k| crate::script::has_talk(&self.g, k)).unwrap_or(false)),
                    5 => Row::new(s.label).badge(talk_kind(i).map(|k| crate::script::has_talk(&self.g, k)).unwrap_or(false)),
                    _ => Row::new(s.label),
                })
                .collect();
            rows.push(Row::new("換金アイテムを全部売る").enabled(loot_total > 0));
            rows.push(Row::new("メニュー"));
            let ev = st.menu.run(ui, &self.inp, &rows, Rect::new(10.0, SH + 34.0, 300.0, 10.0 * 38.0 + 8.0), 38.0, 21.0, Style::Wood, true);
            gold_box(318.0, SH + 40.0, 248.0, 366.0);
            ui.text("パーティ", 332.0, SH + 46.0, 20.0, GOLDEN);
            for (k, &i) in self.g.members().iter().enumerate() {
                let m = &self.g.party[i];
                let y = SH + 76.0 + k as f32 * 68.0;
                ui.text(&m.name, 332.0, y, 19.0, CREAM);
                ui.text_right(m.status_text(), 552.0, y + 2.0, 15.0, if m.poison { rgb(200, 120, 255) } else { rgb(190, 230, 190) });
                ui.text(&format!("HP {}/{}", m.hp, m.hp_max), 332.0, y + 24.0, 16.0, rgb(150, 235, 120));
                bar(332.0, y + 44.0, 220.0, m.hp as f32 / m.hp_max as f32, rgb(110, 210, 90));
                ui.text(&format!("MP {}/{}", m.mp, m.mp_max), 440.0, y + 24.0, 16.0, rgb(214, 160, 255));
            }
            ui.text(&format!("シャベルMP {}/{}", self.g.mp, self.g.mp_max), 332.0, SH + 284.0, 17.0, CREAM);
            ui.text(&format!("評価 {}  ごはん {}/{}人", self.g.rating(), self.g.members().iter().filter(|&&i| self.g.ate[i]).count(), self.g.members().len()), 332.0, SH + 308.0, 17.0, CREAM);
            ui.text(&format!("B{}Fで倒した数 {}", self.g.floor_no(), self.g.dungeon.kills), 332.0, SH + 332.0, 17.0, CREAM);
            ui.text(&format!("メイン {}/{}", mq.idx + 1, mq.total), 332.0, SH + 360.0, 17.0, rgb(255, 230, 140));

            if self.inp.click {
                for (i, s) in SPOTS.iter().enumerate() {
                    if s.rect.contains(self.inp.mouse) {
                        st.menu.cursor = i;
                        return self.town_go(i, st);
                    }
                }
            }
            if let MenuEv::Confirm(i) = ev {
                next = self.town_go(i, st);
            }
        }
        next
    }

    fn town_go(&mut self, i: usize, st: &mut TownSt) -> Option<Scr> {
        LAST_CURSOR.store(i, std::sync::atomic::Ordering::Relaxed);
        match i {
            ROW_QUEST => {
                self.ov = Some(Overlay::Board(crate::board::BoardSt::new(), true));
                None
            }
            1 => Some(Scr::Shop(ShopSt::new(ShopKind::Armor, &self.g))),
            2 => Some(Scr::Shop(ShopSt::new(ShopKind::Magic, &self.g))),
            3 => Some(Scr::Shop(ShopSt::new(ShopKind::Furniture, &self.g))),
            4 => Some(Scr::Shop(ShopSt::new(ShopKind::Plaza, &self.g))),
            5 => Some(Scr::Shop(ShopSt::new(ShopKind::Mayor, &self.g))),
            6 => Some(Scr::Home(crate::home::HomeSt::new(&self.g))),
            7 => {
                self.g.undo.clear();
                if self.g.reached > 1 {
                    // 行ったことのある階を選べる
                    let mut m = ListMenu::new();
                    m.cursor = self.g.floor_no() - 1;
                    st.floor_pick = Some(m);
                    None
                } else {
                    self.g.return_to_entrance();
                    Some(Scr::Dungeon(crate::dungeon::DunSt::new()))
                }
            }
            ROW_SELL => {
                let (n, gold) = self.g.sell_loot();
                self.ui.audio.sfx(if n > 0 { "sell" } else { "blocked" });
                st.msg = Some(if n > 0 { format!("換金アイテム {}個を まとめて売った! {}G", n, gold) } else { "売る物が ない".to_string() });
                None
            }
            _ => {
                self.ov = Some(Overlay::menu());
                None
            }
        }
    }
}

pub fn comma(n: i32) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 {
        out.push('-');
    }
    out.chars().rev().collect()
}

pub fn item_help(name: &str) -> String {
    match item(name) {
        Some(d) => {
            let extra = match d.kind {
                Kind::Weapon => format!("  ATK+{}", d.power),
                Kind::Shield | Kind::Head | Kind::Body | Kind::Feet => format!("  DEF+{}", d.power),
                _ => String::new(),
            };
            format!("{}{}", d.desc, extra)
        }
        None => String::new(),
    }
}
