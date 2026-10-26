//! 店・広場・村長: 売り買い、クエスト、魔法、料理のヒント。

use crate::data::*;
use crate::sim::Game;
use crate::town::{comma, item_help, TownSt};
use crate::ui::*;
use crate::{App, Scr};
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShopKind {
    Armor,
    Magic,
    Furniture,
    Plaza,
    Mayor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Main,
    Buy,
    Sell,
    Spell,
    Show,
    Peddler,
}

pub struct ShopSt {
    pub kind: ShopKind,
    pub page: Page,
    pub menu: ListMenu,
    pub msg: String,
    /// まとめ買い・まとめ売りの個数(QTY の添字)
    pub qi: usize,
    /// 店の人の話(1行ずつ。決定キーで進む)
    pub talk: std::collections::VecDeque<String>,
    talk_init: bool,
    /// 話が終わったら「かう」を開いて ナイフに カーソルを合わせる(はじめての武具屋)
    goto_buy: bool,
}

const QTY: [i32; 3] = [1, 5, 10];

enum Act {
    Peddler(String),
    Go(Page),
    Buy(String),
    Sell(String),
    SellLoot,
    Spell(&'static str),
    Show(String),
    Hint,
    Leave,
    None,
}

impl ShopSt {
    pub fn new(kind: ShopKind, g: &Game) -> ShopSt {
        let msg = match kind {
            ShopKind::Armor => "いらっしゃい! 武具屋へようこそ。",
            ShopKind::Magic => "おお、アレンか。魔法屋へようこそ。",
            ShopKind::Furniture => "へい、らっしゃい! 部屋ならまかせとけ!",
            ShopKind::Plaza => if g.peddler_here() { "広場へようこそ! 今日は 行商人が 来ているよ!" } else { "広場へようこそ! 野菜はいかが?" },
            ShopKind::Mayor => "おお、アレン。ダンジョンの調子はどうじゃ?",
        };
        let _ = g;
        ShopSt { kind, page: Page::Main, menu: ListMenu::new(), msg: msg.to_string(), qi: 0, talk: Default::default(), talk_init: false, goto_buy: false }
    }
}

fn key(kind: ShopKind) -> &'static str {
    match kind {
        ShopKind::Armor => "armor",
        ShopKind::Magic => "magic",
        ShopKind::Furniture => "furn",
        ShopKind::Plaza => "plaza",
        ShopKind::Mayor => "",
    }
}

fn scene(kind: ShopKind) -> &'static str {
    match kind {
        ShopKind::Armor => "scene_armor",
        ShopKind::Magic => "scene_magic",
        ShopKind::Furniture => "scene_furn",
        ShopKind::Plaza => "scene_plaza",
        ShopKind::Mayor => "scene_mayor",
    }
}

fn title(kind: ShopKind, page: Page) -> &'static str {
    match (kind, page) {
        (_, Page::Buy) => "かう",
        (_, Page::Sell) => "うる",
        (_, Page::Spell) => "まほう",
        (_, Page::Show) => "しょくざいを みせる",
        (_, Page::Peddler) => "行商人",
        (ShopKind::Armor, _) => "武具屋",
        (ShopKind::Magic, _) => "魔法屋",
        (ShopKind::Furniture, _) => "家具屋",
        (ShopKind::Plaza, _) => "広場",
        (ShopKind::Mayor, _) => "村長の家",
    }
}

fn buy_list(kind: ShopKind, g: &Game) -> Vec<String> {
    match kind {
        ShopKind::Armor => ITEMS.iter().filter(|i| g.armor_stock.contains(i.name)).map(|i| i.name.to_string()).collect(),
        ShopKind::Magic => g.magic_stock().iter().map(|s| s.to_string()).collect(),
        ShopKind::Furniture => g.furniture_stock().iter().map(|s| s.to_string()).collect(),
        ShopKind::Plaza => ITEMS.iter().filter(|i| g.veg_stock.contains(i.name)).map(|i| i.name.to_string()).collect(),
        ShopKind::Mayor => vec![],
    }
}

fn sell_ok(kind: ShopKind, k: Kind) -> bool {
    match kind {
        ShopKind::Armor => k.is_equip(),
        ShopKind::Magic => matches!(k, Kind::Potion | Kind::Bomb | Kind::Loot),
        ShopKind::Furniture => k == Kind::Room,
        ShopKind::Plaza => matches!(k, Kind::Veg | Kind::Food),
        ShopKind::Mayor => false,
    }
}

fn sell_list(kind: ShopKind, g: &Game) -> Vec<String> {
    g.owned_defs().into_iter().filter(|i| i.sell > 0 && sell_ok(kind, i.kind)).map(|i| i.name.to_string()).collect()
}

impl App {
    pub fn shop(&mut self, st: &mut ShopSt) -> Option<Scr> {
        if st.page == Page::Main {
            st.qi = 0;
        }
        // 着いたとき: 頼みごと・新しい部屋の説明があれば 話してもらう
        if !st.talk_init {
            st.talk_init = true;
            let lines = crate::script::on_enter(&mut self.g, st.kind);
            // はじめての武具屋: 武器を持っていなければ、話のあと そのまま 買えるようにする
            if st.kind == ShopKind::Armor && crate::script::tour_on(&self.g) && !lines.is_empty() && self.g.party[0].equip[0].is_none() && self.g.count("ナイフ") == 0 {
                st.goto_buy = true;
            }
            st.talk.extend(lines);
        }
        let talking = !st.talk.is_empty();
        if let Some(l) = st.talk.front() {
            st.msg = l.clone();
        }
        let kind = st.kind;
        let g = &self.g;
        let ui = &self.ui;
        // ---- 行の組み立て ----
        let mut rows: Vec<(Row, Act)> = vec![];
        let mut help = String::new();
        match st.page {
            Page::Main => {
                match kind {
                    ShopKind::Mayor => {
                        rows.push((Row::new("いまの目標を きく"), Act::Hint));
                    }
                    _ => {
                        if kind != ShopKind::Mayor {
                            rows.push((Row::new("かう").badge(g.shop_news(key(kind)).iter().any(|n| !n.starts_with("魔法:"))), Act::Go(Page::Buy)));
                        }
                        if kind == ShopKind::Magic {
                            rows.push((Row::new("まほうを おぼえる").badge(g.shop_news("magic").iter().any(|n| n.starts_with("魔法:"))), Act::Go(Page::Spell)));
                        }
                        if kind == ShopKind::Plaza && g.peddler_here() {
                            rows.push((Row::new("行商人").right("きょうは 来ているよ!").badge(!g.peddler_wares().is_empty()), Act::Go(Page::Peddler)));
                        }
                        rows.push((Row::new("うる"), Act::Go(Page::Sell)));
                        if kind == ShopKind::Magic {
                            rows.push((Row::new("しょくざいを みせる").badge(!g.teachable().is_empty()), Act::Go(Page::Show)));
                        }
                        if kind == ShopKind::Magic {
                            let total: i32 = ITEMS.iter().filter(|d| d.kind == Kind::Loot).map(|d| d.sell * g.count(d.name)).sum();
                            rows.push((Row::new("換金アイテムを全部売る").right(if total > 0 { format!("{}G", total) } else { String::new() }).enabled(total > 0), Act::SellLoot));
                        }
                    }
                }
                rows.push((Row::new("でる"), Act::Leave));
            }
            Page::Buy => {
                for n in buy_list(kind, g) {
                    let d = item(&n).unwrap();
                    let own = g.count(&n);
                    let label = if own > 0 { format!("{}  (もっている {})", n, own) } else { n.clone() };
                    rows.push((Row::new(label).right(format!("{}G", d.buy)).enabled(g.gold >= d.buy).badge(!g.seen.contains(&n)), Act::Buy(n)));
                }
            }
            Page::Sell => {
                for n in sell_list(kind, g) {
                    let d = item(&n).unwrap();
                    rows.push((Row::new(format!("{}  x{}", n, g.count(&n))).right(format!("{}G", d.sell)), Act::Sell(n)));
                }
            }
            Page::Spell => {
                for s in SPELLS.iter().filter(|s| g.spell_for_sale(s)) {
                    rows.push((
                        Row::new(format!("{}  (MP{})", s.name, s.mp)).right(format!("{}G", s.price)).enabled(g.gold >= s.price).badge(!g.seen.contains(&format!("魔法:{}", s.name))),
                        Act::Spell(s.id),
                    ));
                }
            }
            Page::Peddler => {
                for (n, p) in g.peddler_wares() {
                    let boss = food_origin(&n).map(|x| x.1).unwrap_or(false);
                    rows.push((Row::new(format!("{}{}", if boss { "★ " } else { "" }, n)).right(format!("{}G", p)).enabled(g.gold >= p), Act::Peddler(n)));
                }
            }
            Page::Show => {
                for i in ITEMS.iter().filter(|i| matches!(i.kind, Kind::Food | Kind::Veg) && g.count(i.name) > 0) {
                    rows.push((Row::new(format!("{}  x{}", i.name, g.count(i.name))).badge(g.teachable().contains(&i.name)), Act::Show(i.name.to_string())));
                }
            }
        }
        if rows.is_empty() && st.page != Page::Main {
            rows.push((Row::new("(なにもない)").enabled(false), Act::None));
        }
        // 説明文
        if let Some((_, act)) = rows.get(st.menu.cursor.min(rows.len().saturating_sub(1))) {
            help = match act {
                Act::Buy(n) | Act::Sell(n) | Act::Show(n) | Act::Peddler(n) => {
                    let mut h = item_help(n);
                    if let Some((a0, a1, d0, d1)) = g.preview(0, n) {
                        h += &format!("
アレン: ATK {}→{}  DEF {}→{}", a0, a1, d0, d1);
                    }
                    h
                }
                Act::Spell(id) => spell(id).desc.to_string(),
                _ => String::new(),
            };
        }

        // ---- 描画 ----
        ui.art.draw(scene(kind), 0.0, 0.0, LW, SH);
        chip(ui, LW - 180.0, 12.0, 168.0, 32.0, &format!("{} G", comma(g.gold)), 20.0);
        // 店の人の 一言は いつも 名札つき(「＊」の行は システムの文)
        let keeper = match kind {
            ShopKind::Mayor => Some("村長"),
            ShopKind::Magic => Some("魔法屋"),
            ShopKind::Armor => Some("武具屋"),
            ShopKind::Furniture => Some("家具屋"),
            ShopKind::Plaza => Some(if st.page == Page::Peddler { "行商人" } else { "広場の子" }),
        };
        crate::stage::draw_talk_face(ui, &st.msg, keeper, 336.0, 84.0);

        panel_wood(0.0, SH, LW, SH);
        let rows_only: Vec<Row> = rows.iter().map(|(r, _)| Row { label: r.label.clone(), right: r.right.clone(), enabled: r.enabled, badge: r.badge }).collect();
        // 見出し
        ui.text(title(kind, st.page), 18.0, SH + 8.0, 24.0, CREAM);
        if matches!(st.page, Page::Buy | Page::Sell | Page::Spell) {
            ui.text_right("ねだん", LW - 24.0, SH + 12.0, 18.0, CREAM);
        }
        let list_rect = Rect::new(10.0, SH + 40.0, LW - 20.0, 262.0);
        if matches!(st.page, Page::Buy | Page::Sell) {
            // ←→ でまとめ買い・まとめ売りの個数を変える
            st.menu.no_page = true;
            if self.inp.right {
                st.qi = (st.qi + 1).min(QTY.len() - 1);
            }
            if self.inp.left {
                st.qi = st.qi.saturating_sub(1);
            }
            ui.text(&format!("←→ で個数: x{}", QTY[st.qi]), 220.0, SH + 12.0, 18.0, rgb(255, 230, 140));
        } else {
            st.menu.no_page = false;
        }
        let ev = st.menu.run(ui, &self.inp, &rows_only, list_rect, 42.0, 22.0, Style::Wood, !talking);
        // 説明ボックス
        msgbox(10.0, SH + 312.0, LW - 20.0, 106.0);
        if !help.is_empty() {
            ui.text_wrap(&help, 26.0, SH + 322.0, LW - 52.0, 19.0, CREAM, 25.0);
        } else if st.page == Page::Main {
            ui.text("矢印キーとEnterで えらぶ   Escで もどる", 26.0, SH + 322.0, 19.0, rgb(220, 205, 170));
        }
        // ---- 話の途中: 決定キー/クリックで 次の行へ(メニューは止まっている) ----
        if talking {
            ui.text("▼ Zで つぎへ", LW - 170.0, 396.0, 16.0, rgb(255, 230, 140));
            if self.inp.ok || self.inp.click {
                st.talk.pop_front();
                self.ui.audio.sfx("ok");
                if st.talk.is_empty() {
                    st.msg = crate::script::closing(kind).to_string();
                    if st.goto_buy {
                        st.goto_buy = false;
                        st.page = Page::Buy;
                        let list = buy_list(kind, &self.g);
                        st.menu = ListMenu::new();
                        st.menu.cursor = list.iter().position(|n| n == "ナイフ").unwrap_or(0);
                        st.msg = "さあ、ナイフを 買っていってね!".to_string();
                    }
                }
            }
            return None;
        }
        // ---- 確定処理 ----
        match ev {
            MenuEv::Cancel => {
                // 新入荷の一覧を見終わったら、赤い!を消す
                match st.page {
                    Page::Buy => self.g.mark_seen(key(kind), false),
                    Page::Spell => self.g.mark_seen(key(kind), true),
                    _ => {}
                }
                if st.page == Page::Main {
                    return Some(Scr::Town(TownSt::new(None)));
                }
                st.page = Page::Main;
                st.menu = ListMenu::new();
            }
            MenuEv::Confirm(i) => {
                let act = std::mem::replace(&mut rows[i].1, Act::None);
                match act {
                    Act::Go(p) => {
                        st.page = p;
                        st.menu = ListMenu::new();
                    }
                    Act::Leave => return Some(Scr::Town(TownSt::new(None))),
                    Act::Buy(n) => {
                        let want = QTY[st.qi];
                        let mut bought = 0;
                        let mut err = String::new();
                        for _ in 0..want {
                            match self.g.buy(&n) {
                                Ok(()) => bought += 1,
                                Err(e) => {
                                    err = e;
                                    break;
                                }
                            }
                        }
                        self.ui.audio.sfx(if bought > 0 { "buy" } else { "blocked" });
                        let peddler = false;
                        st.msg = if bought > 0 { crate::script::shop_line(kind, peddler, crate::script::ShopEv::Bought(&n, bought)) } else { crate::script::shop_error(kind, peddler, &err) };
                        // 武器を持っていないアレンが 武器を買ったら そのまま そうびする
                        if bought > 0 && kind == ShopKind::Armor && self.g.party[0].equip[0].is_none() && item(&n).map(|d| d.kind == Kind::Weapon).unwrap_or(false) && self.g.equip(0, &n).is_ok() {
                            st.msg = crate::script::shop_line(kind, false, crate::script::ShopEv::BoughtEquipped(&n));
                        }
                    }
                    Act::Sell(n) => {
                        let want = QTY[st.qi];
                        let mut sold = 0;
                        let mut total = 0;
                        for _ in 0..want {
                            match self.g.sell(&n) {
                                Ok(v) => {
                                    sold += 1;
                                    total += v;
                                }
                                Err(_) => break,
                            }
                        }
                        self.ui.audio.sfx(if sold > 0 { "sell" } else { "blocked" });
                        st.msg = if sold > 0 { crate::script::shop_line(kind, false, crate::script::ShopEv::Sold(&n, sold, total)) } else { crate::script::shop_line(kind, false, crate::script::ShopEv::CantSell) };
                    }
                    Act::SellLoot => {
                        let (n, gold) = self.g.sell_loot();
                        self.ui.audio.sfx(if n > 0 { "sell" } else { "blocked" });
                        st.msg = if n > 0 { crate::script::shop_line(kind, false, crate::script::ShopEv::SoldLoot(n as i32, gold)) } else { crate::script::shop_line(kind, false, crate::script::ShopEv::NothingToSell) };
                    }
                    Act::Spell(id) => match self.g.buy_spell(id) {
                        Ok(()) => {
                            self.ui.audio.sfx("buy");
                            st.msg = crate::script::shop_line(kind, false, crate::script::ShopEv::SpellLearned(spell(id).name))
                        }
                        Err(e) => st.msg = crate::script::shop_error(kind, false, &e),
                    },
                    Act::Peddler(n) => match self.g.peddler_buy(&n) {
                        Ok(p) => {
                            self.ui.audio.sfx("buy");
                            st.msg = crate::script::shop_line(kind, true, crate::script::ShopEv::PeddlerBought(&n, p));
                        }
                        Err(e) => {
                            self.ui.audio.sfx("blocked");
                            st.msg = crate::script::shop_error(kind, true, &e);
                        }
                    },
                    Act::Show(n) => {
                        let before = self.g.spells_open.clone();
                        match self.g.show_ingredient(&n) {
                            Ok(m) => {
                                st.msg = m;
                                // 新しく入荷した魔法の 話
                                let opened: Vec<String> = self.g.spells_open.difference(&before).cloned().collect();
                                for id in opened {
                                    self.g.talks_seen.insert(format!("magic:spell:{}", id));
                                    let mei = self.g.party[2].joined;
                                    st.talk.extend(crate::script::clean(crate::script::spell_talk(&id), mei));
                                }
                            }
                            Err(e) => st.msg = crate::script::shop_error(kind, false, &e),
                        }
                    }
                    Act::Hint => {
                        st.talk.extend(crate::script::mayor_goal(&self.g));
                    }
                    Act::None => {}
                }
            }
            MenuEv::None => {}
        }
        None
    }
}

