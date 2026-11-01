//! 店・広場・村長: 売り買い、クエスト、魔法、料理のヒント。

use crate::data::*;
use crate::sim::Game;
use crate::town::{comma, equip_effects, item_help, TownSt};
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
    /// 武具屋のお姉さんの 武器合成(エレメントで 開く)
    Forge,
    FuseMain,
    FuseSlot,
    FuseDonor,
    PlusMain,
    PlusMat,
    StripMain,
    StripSlot,
    Confirm,
}

/// 武器合成の 確認待ち
#[derive(Clone)]
pub enum Pending {
    /// (本体, 枠, 素材)
    Ability(String, usize, String),
    /// (本体, 素材)
    Plus(String, String),
    /// (本体, 枠)
    Strip(String, usize),
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
    /// 武具屋の タブ(0=ぶき 1=たて 2=あたま 3=からだ 4=あし 5=ゆびわ)。店を出るまで おぼえておく
    pub cat: usize,
    /// 武器合成: 選んだ 本体・枠・素材
    pub fmain: String,
    pub fslot: usize,
    pub fmat: String,
    pub pend: Option<Pending>,
}

/// 武器合成の 画面を もどる 先
fn parent(p: Page) -> Page {
    match p {
        Page::FuseSlot => Page::FuseMain,
        Page::FuseDonor => Page::FuseSlot,
        Page::PlusMat => Page::PlusMain,
        Page::StripSlot => Page::StripMain,
        Page::FuseMain | Page::PlusMain | Page::StripMain | Page::Confirm => Page::Forge,
        _ => Page::Main,
    }
}

/// 合成で 本体に できる 武器(持っている 武器すべて)
fn forge_weapons(g: &Game) -> Vec<String> {
    g.forge_weapon_names()
}

/// 一覧の 右がわ: 持ち物の 数、持ち物に なく そうび中なら 「そうび中」
fn qty(g: &Game, n: &str) -> String {
    if g.count(n) > 0 {
        format!("x{}", g.count(n))
    } else {
        "そうび中".to_string()
    }
}

/// 能力の 素材に できる 武器(合成ずみでなく、特殊な 能力が あり、本体と ちがう 武器)
fn donor_weapons(g: &Game, main: &str) -> Vec<String> {
    forge_weapons(g).into_iter().filter(|n| fuse_split(n).1.iter().all(|s| s.is_none()) && fuse_donor_ok(base_name(n)) && base_name(n) != base_name(main)).collect()
}

/// 強化合成の 相手に なれる 武器(おなじ武器で おなじ +。本体と おなじ名前なら 2本 ひつよう)
fn plus_mats(g: &Game, main: &str) -> Vec<String> {
    forge_weapons(g).into_iter().filter(|n| g.plus_fuse_plan(main, n).is_ok()).collect()
}

/// 確認画面の 説明(できあがる 武器と 費用)
fn pend_text(g: &Game, p: &Pending) -> (String, i32) {
    match p {
        Pending::Ability(m, s, d) => {
            let (mm, mut slots) = fuse_split(m);
            let db = base_name(d);
            slots[*s] = Some(db);
            let new = fuse_join(mm, slots);
            let _ = g;
            (format!("{} の {}枠目に {} の能力を つける。{} は なくなる。\nできあがり: {}\n効果: {}", disp_name(m), s + 1, db, d, disp_name(&new), equip_effects(&new)), FUSE_COST[*s])
        }
        Pending::Plus(m, mat) => match g.plus_fuse_plan(m, mat) {
            Ok((new, cost)) => (format!("{} と {} を あわせて {} に する。\n本体の 合成枠は のこり、素材の 枠は きえる。", disp_name(m), disp_name(mat), disp_name(&new)), cost),
            Err(e) => (e, 0),
        },
        Pending::Strip(m, s) => {
            let (mm, mut slots) = fuse_split(m);
            let was = slots[*s].unwrap_or("");
            slots[*s] = None;
            (format!("{} の {}枠目({} の能力)を はずす。ただで できるが、素材は もどらない。\nできあがり: {}", disp_name(m), s + 1, was, disp_name(&fuse_join(mm, slots))), 0)
        }
    }
}

/// 武具屋の 種類わけ(Kind::slot の 番号と おなじ)
pub const CATS: [&str; 6] = ["ぶき", "たて", "あたま", "からだ", "あし", "ゆびわ"];
const TAB_Y: f32 = 40.0;
const TAB_H: f32 = 30.0;

fn cat_of(n: &str) -> Option<usize> {
    item(n).and_then(|d| d.kind.slot())
}

/// 武具屋の 買う・売る 画面で タブを つかうか
fn has_tabs(kind: ShopKind, page: Page) -> bool {
    kind == ShopKind::Armor && matches!(page, Page::Buy | Page::Sell)
}

fn tab_rect(i: usize) -> Rect {
    let w = (LW - 20.0) / CATS.len() as f32;
    Rect::new(10.0 + w * i as f32, SH + TAB_Y, w - 3.0, TAB_H)
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
    ForgeUnlock,
    FMain(String),
    FSlot(usize),
    FDonor(String),
    PMain(String),
    PMat(String),
    SMain(String),
    SSlot(usize),
    Do,
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
        ShopSt { kind, page: Page::Main, menu: ListMenu::new(), msg: msg.to_string(), qi: 0, talk: Default::default(), talk_init: false, goto_buy: false, cat: 0, fmain: String::new(), fslot: 0, fmat: String::new(), pend: None }
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
        (_, Page::Forge) => "ごうせい",
        (_, Page::FuseMain) => "ごうせい: どの武器に?",
        (_, Page::FuseSlot) => "ごうせい: どの枠に?",
        (_, Page::FuseDonor) => "ごうせい: どの能力を?",
        (_, Page::PlusMain) => "強化合成: どの武器を?",
        (_, Page::PlusMat) => "強化合成: あわせる 武器は?",
        (_, Page::StripMain) => "能力を はずす: どの武器?",
        (_, Page::StripSlot) => "能力を はずす: どの枠?",
        (_, Page::Confirm) => "この内容で いい?",
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
        // ---- 武具屋のタブ: Q・E か クリックで きりかえ ----
        if has_tabs(kind, st.page) && !talking {
            let mut to = st.cat;
            if self.inp.r {
                to = (st.cat + 1) % CATS.len();
            }
            if self.inp.l {
                to = (st.cat + CATS.len() - 1) % CATS.len();
            }
            if self.inp.click {
                if let Some(i) = (0..CATS.len()).find(|&i| tab_rect(i).contains(self.inp.mouse)) {
                    to = i;
                }
            }
            if to != st.cat {
                self.leave_tab(st);
                st.cat = to;
                st.menu = ListMenu::new();
                self.ui.audio.sfx("ok");
            }
        }
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
                        if kind == ShopKind::Armor && g.reached >= FUSE_FLOOR && (g.forge_told() || g.forge_open()) {
                            let r = Row::new("ごうせい");
                            rows.push((if g.forge_open() { r.right("武器の能力・+を つよくする") } else { r.right(format!("エレメント {}/{}", g.elements_have(), ELEMENTS.len())).badge(g.elements_have() == ELEMENTS.len()) }, Act::Go(Page::Forge)));
                        }
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
                for n in buy_list(kind, g).into_iter().filter(|n| kind != ShopKind::Armor || cat_of(n) == Some(st.cat)) {
                    let d = item(&n).unwrap();
                    let own = g.count(&n);
                    let label = if own > 0 { format!("{}  (もっている {})", n, own) } else { n.clone() };
                    rows.push((Row::new(label).right(format!("{}G", d.buy)).enabled(g.gold >= d.buy).badge(!g.seen.contains(&n)), Act::Buy(n)));
                }
            }
            Page::Sell => {
                for n in sell_list(kind, g).into_iter().filter(|n| kind != ShopKind::Armor || cat_of(n) == Some(st.cat)) {
                    let d = item(&n).unwrap();
                    rows.push((Row::new(format!("{}  x{}", disp_name(&n), g.count(&n))).right(format!("{}G", d.sell)), Act::Sell(n)));
                }
            }
            Page::Spell => {
                for s in SPELLS.iter().filter(|s| g.spell_for_sale(s)) {
                    rows.push((
                        Row::new(format!("{}  ({}・MP{})", s.name, g.party[s.user].name, s.mp)).right(format!("{}G", s.price)).enabled(g.gold >= s.price).badge(!g.seen.contains(&format!("魔法:{}", s.name))),
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
            Page::Forge if !g.forge_open() => {
                let all = g.elements_have() == ELEMENTS.len();
                rows.push((Row::new("エレメントを わたして ごうせいを ひらく").enabled(all), Act::ForgeUnlock));
                for e in ELEMENTS {
                    let have = g.count(e) > 0;
                    rows.push((Row::new(e.to_string()).right(if have { "もっている" } else { "まだ" }).enabled(false), Act::None));
                }
            }
            Page::Forge => {
                rows.push((Row::new("のうりょくを あわせる").right("30000G / 50000G"), Act::Go(Page::FuseMain)));
                rows.push((Row::new("+を つよくする(強化合成)").right("+5から +9まで"), Act::Go(Page::PlusMain)));
                rows.push((Row::new("のうりょくを はずす").right("ただ"), Act::Go(Page::StripMain)));
            }
            Page::FuseMain => {
                for n in forge_weapons(g) {
                    rows.push((Row::new(disp_name(&n)).right(qty(g, &n)), Act::FMain(n)));
                }
            }
            Page::FuseSlot => {
                let slots = fuse_split(&st.fmain).1;
                for i in 0..FUSE_SLOTS {
                    let cur = slots[i].map(|s| format!("{}の能力", s)).unwrap_or_else(|| "あき".to_string());
                    rows.push((Row::new(format!("{}枠目  ({})", i + 1, cur)).right(format!("{}G", FUSE_COST[i])).enabled(g.gold >= FUSE_COST[i]), Act::FSlot(i)));
                }
            }
            Page::FuseDonor => {
                for n in donor_weapons(g, &st.fmain) {
                    rows.push((Row::new(disp_name(&n)).right(qty(g, &n)), Act::FDonor(n)));
                }
            }
            Page::PlusMain => {
                for n in forge_weapons(g).into_iter().filter(|n| !plus_mats(g, n).is_empty()) {
                    let cost = g.plus_fuse_plan(&n, &plus_mats(g, &n)[0]).map(|x| x.1).unwrap_or(0);
                    rows.push((Row::new(disp_name(&n)).right(format!("{}G", cost)).enabled(g.gold >= cost), Act::PMain(n)));
                }
            }
            Page::PlusMat => {
                for n in plus_mats(g, &st.fmain) {
                    rows.push((Row::new(disp_name(&n)).right(qty(g, &n)), Act::PMat(n)));
                }
            }
            Page::StripMain => {
                for n in forge_weapons(g).into_iter().filter(|n| fuse_split(n).1.iter().any(|s| s.is_some())) {
                    rows.push((Row::new(disp_name(&n)).right(qty(g, &n)), Act::SMain(n)));
                }
            }
            Page::StripSlot => {
                let slots = fuse_split(&st.fmain).1;
                for i in 0..FUSE_SLOTS {
                    if let Some(s) = slots[i] {
                        rows.push((Row::new(format!("{}枠目  ({}の能力)", i + 1, s)), Act::SSlot(i)));
                    }
                }
            }
            Page::Confirm => {
                if let Some(p) = &st.pend {
                    let (_, cost) = pend_text(g, p);
                    rows.push((Row::new("あわせる").right(if cost > 0 { format!("{}G", cost) } else { String::new() }).enabled(g.gold >= cost), Act::Do));
                    rows.push((Row::new("やめる"), Act::Go(Page::Forge)));
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
                Act::FMain(n) | Act::PMain(n) | Act::SMain(n) | Act::FDonor(n) | Act::PMat(n) => item_help(n),
                Act::FSlot(i) => format!("{}枠目に 能力を つける。{}G。すでに ついている 能力は 上書きされる。素材の武器は なくなる。", i + 1, FUSE_COST[*i]),
                Act::SSlot(i) => format!("{}枠目の 能力を はずす(ただ)。", i + 1),
                Act::ForgeUnlock => "1F〜9F の ボスの エレメントを 9つ そろえて わたすと、武器合成が 開く。".to_string(),
                Act::Go(Page::Forge) if kind == ShopKind::Armor => "エレメントを あつめて 武器合成を ひらこう。瘴気発生装置を5個置いた階のボスだけが 落とす。11Fから 家具屋に ならぶ「けっかいじん(簡易)」で ボスを 呼びなおせる。".to_string(),
                Act::Go(Page::FuseMain) => "武器に ほかの武器の 特殊な能力を つける(2枠まで)。1枠目 30000G、2枠目 50000G。素材の武器は なくなる。".to_string(),
                Act::Go(Page::PlusMain) => "おなじ武器の +N と +N を あわせて +(N+1) に。+5〜+8の 武器が 2本 そろうと できる。最大 +9。".to_string(),
                Act::Go(Page::StripMain) => "つけた 能力を 枠ごとに はずす(ただ)。素材は もどらない。".to_string(),
                Act::Do => st.pend.as_ref().map(|p| pend_text(g, p).0).unwrap_or_default(),
                Act::None if st.page == Page::Forge => "どんな 武器にも 合う ように、エレメントが 力を かしてくれる。".to_string(),
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
        let tabs = has_tabs(kind, st.page);
        let list_rect = if tabs { Rect::new(10.0, SH + TAB_Y + TAB_H + 4.0, LW - 20.0, 228.0) } else { Rect::new(10.0, SH + 40.0, LW - 20.0, 262.0) };
        if tabs {
            // 種類タブ(空の タブは うすく。新しい品が あるタブには 赤い!)
            let full: Vec<String> = if st.page == Page::Buy { buy_list(kind, g) } else { sell_list(kind, g) };
            for (i, name) in CATS.iter().enumerate() {
                let r = tab_rect(i);
                let in_tab: Vec<&String> = full.iter().filter(|n| cat_of(n) == Some(i)).collect();
                let cur = i == st.cat;
                let hover = r.contains(self.inp.mouse);
                let base = if cur { rgb(150, 100, 52) } else if hover { rgb(112, 76, 42) } else { rgb(84, 56, 32) };
                draw_rectangle(r.x, r.y, r.w, r.h, base);
                draw_rectangle_lines(r.x, r.y, r.w, r.h, if cur { 3.0 } else { 1.5 }, if cur { rgb(255, 230, 140) } else { rgb(190, 170, 120) });
                let col = if in_tab.is_empty() { rgb(160, 140, 110) } else if cur { rgb(255, 245, 200) } else { CREAM };
                ui.text_center(&format!("{} {}", name, in_tab.len()), r.x + r.w / 2.0 - 3.0, r.y + 6.0, 16.0, col);
                if st.page == Page::Buy && in_tab.iter().any(|n| !g.seen.contains(*n)) {
                    draw_bang(r.x + r.w - 6.0, r.y + 4.0, 5.0, ui.t, ui);
                }
            }
        }
        if matches!(st.page, Page::Buy | Page::Sell) {
            // ←→ でまとめ買い・まとめ売りの個数を変える
            st.menu.no_page = true;
            if self.inp.right {
                st.qi = (st.qi + 1).min(QTY.len() - 1);
            }
            if self.inp.left {
                st.qi = st.qi.saturating_sub(1);
            }
            ui.text(&format!("←→ で個数: x{}", QTY[st.qi]), if tabs { 96.0 } else { 220.0 }, SH + 12.0, if tabs { 16.0 } else { 18.0 }, rgb(255, 230, 140));
            if tabs {
                ui.text("Q・E で しゅるい", 250.0, SH + 12.0, 16.0, rgb(255, 230, 140));
            }
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
                        // ナイフは ぶきタブ
                        st.cat = 0;
                        let list: Vec<String> = buy_list(kind, &self.g).into_iter().filter(|n| cat_of(n) == Some(0)).collect();
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
                    Page::Buy if kind == ShopKind::Armor => self.leave_tab(st),
                    Page::Buy => self.g.mark_seen(key(kind), false),
                    Page::Spell => self.g.mark_seen(key(kind), true),
                    _ => {}
                }
                if st.page == Page::Main {
                    return Some(Scr::Town(TownSt::new(None)));
                }
                st.page = parent(st.page);
                st.menu = ListMenu::new();
            }
            MenuEv::Confirm(i) => {
                let act = std::mem::replace(&mut rows[i].1, Act::None);
                match act {
                    Act::Go(p) => {
                        st.page = p;
                        st.menu = ListMenu::new();
                        // 武具屋: 今のタブが からっぽなら、品のある いちばん左のタブへ
                        if has_tabs(kind, p) {
                            let full = if p == Page::Buy { buy_list(kind, &self.g) } else { sell_list(kind, &self.g) };
                            if !full.iter().any(|n| cat_of(n) == Some(st.cat)) {
                                if let Some(c) = (0..CATS.len()).find(|&c| full.iter().any(|n| cat_of(n) == Some(c))) {
                                    st.cat = c;
                                }
                            }
                        }
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
                    Act::ForgeUnlock => match self.g.forge_unlock() {
                        Ok(()) => {
                            self.ui.audio.sfx("buy");
                            st.talk.extend(crate::script::clean(crate::script::forge_open_lines(), self.g.party[2].joined));
                            st.menu = ListMenu::new();
                        }
                        Err(e) => {
                            self.ui.audio.sfx("blocked");
                            st.msg = e;
                        }
                    },
                    Act::FMain(n) => {
                        st.fmain = n;
                        st.page = Page::FuseSlot;
                        st.menu = ListMenu::new();
                    }
                    Act::FSlot(i) => {
                        st.fslot = i;
                        if donor_weapons(&self.g, &st.fmain).is_empty() {
                            self.ui.audio.sfx("blocked");
                            st.msg = "素材に できる 武器が ないわ(特殊な能力の ある 別の武器が 要るの)".to_string();
                        } else {
                            st.page = Page::FuseDonor;
                            st.menu = ListMenu::new();
                        }
                    }
                    Act::FDonor(d) => {
                        st.pend = Some(Pending::Ability(st.fmain.clone(), st.fslot, d));
                        st.page = Page::Confirm;
                        st.menu = ListMenu::new();
                    }
                    Act::PMain(n) => {
                        st.fmain = n;
                        st.page = Page::PlusMat;
                        st.menu = ListMenu::new();
                    }
                    Act::PMat(m) => {
                        st.pend = Some(Pending::Plus(st.fmain.clone(), m));
                        st.page = Page::Confirm;
                        st.menu = ListMenu::new();
                    }
                    Act::SMain(n) => {
                        st.fmain = n;
                        st.page = Page::StripSlot;
                        st.menu = ListMenu::new();
                    }
                    Act::SSlot(i) => {
                        st.pend = Some(Pending::Strip(st.fmain.clone(), i));
                        st.page = Page::Confirm;
                        st.menu = ListMenu::new();
                    }
                    Act::Do => {
                        if let Some(p) = st.pend.take() {
                            let r = match &p {
                                Pending::Ability(m, s, d) => self.g.fuse_ability(m, *s, d),
                                Pending::Plus(m, mat) => self.g.fuse_plus(m, mat),
                                Pending::Strip(m, s) => self.g.fuse_remove(m, *s),
                            };
                            match r {
                                Ok(new) => {
                                    self.ui.audio.sfx("buy");
                                    st.msg = format!("できたわ! 「{}」の 完成よ!", disp_name(&new));
                                }
                                Err(e) => {
                                    self.ui.audio.sfx("blocked");
                                    st.msg = e;
                                }
                            }
                            st.page = Page::Forge;
                            st.menu = ListMenu::new();
                        }
                    }
                    Act::None => {}
                }
            }
            MenuEv::None => {}
        }
        None
    }

    /// 武具屋の タブを はなれる とき: そのタブの 新入荷を 見たことにする
    fn leave_tab(&mut self, st: &ShopSt) {
        if st.page == Page::Buy && st.kind == ShopKind::Armor {
            for n in buy_list(st.kind, &self.g) {
                if cat_of(&n) == Some(st.cat) {
                    self.g.seen.insert(n);
                }
            }
        }
    }
}
