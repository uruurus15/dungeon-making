//! 全画面オーバーレイ: ステータス・そうび・どうぐ・簡易メッセージ。

use crate::data::*;
use crate::sim::Game;
use crate::town::{comma, item_help};
use crate::ui::*;
use crate::App;
use macroquad::prelude::*;

pub enum Overlay {
    Menu(ListMenu),
    Status(usize),
    Equip { who: usize, menu: ListMenu, sub: Option<(usize, ListMenu, Vec<Option<String>>)> },
    Items { menu: ListMenu, pick: Option<(String, ListMenu)>, msg: String },
    /// ジェリーの ものまね(部位, 候補の一覧)
    Mimic { menu: ListMenu, sub: Option<(usize, ListMenu, Vec<Option<&'static str>>)>, msg: String },
    Debug(crate::debug::DebugSt),
    /// オプション(音量・戦闘ログ・画面)
    Options(ListMenu),
    /// クエストボード(true=受ける・届ける、false=見るだけ)
    Board(crate::board::BoardSt, bool),
    /// タイトルへ もどる かくにん(寝ていない データは セーブされない)
    TitleConfirm(ListMenu),
}

impl Overlay {
    pub fn menu() -> Overlay {
        Overlay::Menu(ListMenu::new())
    }
}

const LABELS: [&str; 6] = ["ATK", "DEF", "つよさ", "すばやさ", "じょうぶさ", "かしこさ"];

fn card(ui: &Ui, g: &Game, idx: usize, x: f32, y: f32, w: f32, h: f32) {
    let m = &g.party[idx];
    parchment(x, y, w, h);
    let port = ["portrait_a", "portrait_j", "portrait_m"][idx];
    ui.art.draw(port, x + 18.0, y + 18.0, 96.0, 96.0);
    ui.text(&m.name, x + 130.0, y + 16.0, 32.0, INK);
    ui.text(m.status_text(), x + 130.0, y + 58.0, 20.0, rgb(120, 60, 40));
    ui.text(&format!("HP {} / {}", m.hp, m.hp_max), x + 130.0, y + 86.0, 20.0, rgb(40, 110, 30));
    ui.text(&format!("MP {} / {}", m.mp, m.mp_max), x + 300.0, y + 86.0, 20.0, rgb(110, 50, 150));
    let vals = [m.atk(), m.def(), m.str_total(), m.spd_total(), m.con_total(), m.wis_total()];
    for (i, (l, v)) in LABELS.iter().zip(vals).enumerate() {
        let cx = x + 24.0 + (i % 2) as f32 * 250.0;
        let cy = y + 136.0 + (i / 2) as f32 * 34.0;
        ui.text(l, cx, cy, 20.0, INK);
        ui.text_right(&v.to_string(), cx + 220.0, cy, 22.0, INK);
        draw_line(cx, cy + 28.0, cx + 220.0, cy + 28.0, 1.0, rgb(200, 170, 120));
    }
    // 装備
    ui.text("そうび", x + 24.0, y + 244.0, 18.0, rgb(150, 70, 30));
    for (i, s) in SLOT_NAMES.iter().enumerate() {
        let cx = x + 24.0 + (i % 2) as f32 * 250.0;
        let cy = y + 270.0 + (i / 2) as f32 * 26.0;
        let name = m.equip[i].as_deref().unwrap_or("―");
        ui.text(&format!("{} {}", s, name), cx, cy, 17.0, INK);
    }
    // 魔法
    let sp: Vec<String> = m.spells.iter().map(|s| spell(s).name.to_string()).collect();
    ui.text(&format!("まほう: {}", if sp.is_empty() { "なし".to_string() } else { sp.join("・") }), x + 24.0, y + 352.0, 17.0, INK);
    if idx == 1 {
        ui.text("(ジェリーは ものまねした部分にだけ そうびできる)", x + 24.0, y + 378.0, 15.0, rgb(130, 100, 70));
    }
}

/// 仲間のうち 次・前の人(←→で なかまを きりかえ)
fn next_member(g: &Game, cur: usize) -> usize {
    let m = g.members();
    let p = m.iter().position(|&i| i == cur).unwrap_or(0);
    m[(p + 1) % m.len()]
}

fn prev_member(g: &Game, cur: usize) -> usize {
    let m = g.members();
    let p = m.iter().position(|&i| i == cur).unwrap_or(0);
    m[(p + m.len() - 1) % m.len()]
}

fn summary(ui: &Ui, g: &Game, y0: f32) {
    panel_wood(0.0, y0, LW, SH);
    chip(ui, 14.0, y0 + 10.0, 150.0, 30.0, &format!("{}日目", g.day), 20.0);
    chip(ui, 176.0, y0 + 10.0, 190.0, 30.0, &format!("{} G", comma(g.gold)), 20.0);
    chip(ui, 378.0, y0 + 10.0, 184.0, 30.0, &format!("シャベルMP {}", g.mp_max), 18.0);
    for (k, &i) in g.members().iter().enumerate() {
        let m = &g.party[i];
        let x = 10.0 + k as f32 * 187.0;
        gold_box(x, y0 + 54.0, 180.0, 310.0);
        ui.text(&m.name, x + 10.0, y0 + 62.0, 20.0, CREAM);
        ui.text(m.status_text(), x + 10.0, y0 + 88.0, 16.0, rgb(190, 230, 190));
        ui.text(&format!("HP {}/{}", m.hp, m.hp_max), x + 10.0, y0 + 112.0, 17.0, rgb(150, 235, 120));
        ui.text(&format!("MP {}/{}", m.mp, m.mp_max), x + 10.0, y0 + 136.0, 17.0, rgb(214, 160, 255));
        let vals = [m.atk(), m.def(), m.str_total(), m.spd_total(), m.con_total(), m.wis_total()];
        for (k, (l, v)) in LABELS.iter().zip(vals).enumerate() {
            let yy = y0 + 170.0 + k as f32 * 30.0;
            ui.text(l, x + 10.0, yy, 17.0, CREAM);
            ui.text_right(&v.to_string(), x + 168.0, yy, 18.0, CREAM);
        }
    }
}

impl App {
    pub fn overlay(&mut self) {
        let Some(mut ov) = self.ov.take() else { return };
        let mut keep = true;
        let mut new_ov: Option<Overlay> = None;
        match &mut ov {
            Overlay::Debug(st) => {
                if self.debug_panel(st) {
                    keep = false;
                }
            }
            Overlay::Board(st, interactive) => {
                if self.board(st, *interactive) {
                    keep = false;
                }
            }
            Overlay::Menu(menu) => {
                // 下画面に小さなメニュー(上画面は現在の画面のまま)
                let r = Rect::new(120.0, SH + 60.0, 336.0, 300.0);
                parchment(r.x - 8.0, r.y - 8.0, r.w + 16.0, r.h + 16.0);
                // ジェリーが仲間になったら 「ものまね」が出る
                let mut labels = vec!["ステータス", "そうび", "どうぐ"];
                if self.g.party[1].joined {
                    labels.push("ものまね");
                }
                labels.extend(["オプション", "タイトルへ もどる", "とじる"]);
                let rows: Vec<Row> = labels.iter().map(|l| Row::new(*l)).collect();
                match menu.run(&self.ui, &self.inp, &rows, r, 46.0, 24.0, Style::Parchment, true) {
                    MenuEv::Confirm(i) if labels[i] == "ステータス" => new_ov = Some(Overlay::Status(0)),
                    MenuEv::Confirm(i) if labels[i] == "そうび" => new_ov = Some(Overlay::Equip { who: 0, menu: ListMenu::new(), sub: None }),
                    MenuEv::Confirm(i) if labels[i] == "どうぐ" => new_ov = Some(Overlay::Items { menu: ListMenu::new(), pick: None, msg: String::new() }),
                    MenuEv::Confirm(i) if labels[i] == "ものまね" => new_ov = Some(Overlay::Mimic { menu: ListMenu::new(), sub: None, msg: String::new() }),
                    MenuEv::Confirm(i) if labels[i] == "オプション" => new_ov = Some(Overlay::Options(ListMenu::new())),
                    MenuEv::Confirm(i) if labels[i] == "タイトルへ もどる" => new_ov = Some(Overlay::TitleConfirm(ListMenu::new())),
                    MenuEv::Confirm(_) | MenuEv::Cancel => keep = false,
                    MenuEv::None => {}
                }
            }
            Overlay::TitleConfirm(menu) => {
                let r = Rect::new(80.0, SH + 150.0, 416.0, 130.0);
                parchment(40.0, SH + 20.0, LW - 80.0, 280.0);
                self.ui.text_center("タイトルへ もどりますか?", LW / 2.0, SH + 36.0, 24.0, INK);
                self.ui.text_center("寝ていない データは セーブされません。", LW / 2.0, SH + 76.0, 21.0, rgb(160, 40, 30));
                self.ui.text_center("(セーブは 家で 寝たときに されます)", LW / 2.0, SH + 108.0, 16.0, INK);
                let rows = vec![Row::new("やめる"), Row::new("タイトルへ もどる")];
                menu.no_page = true;
                match menu.run(&self.ui, &self.inp, &rows, r, 52.0, 24.0, Style::Parchment, true) {
                    MenuEv::Confirm(1) => {
                        keep = false;
                        self.scr = crate::Scr::Title(ListMenu::new());
                        self.ui.audio.sfx("ok");
                    }
                    MenuEv::Confirm(_) | MenuEv::Cancel => new_ov = Some(Overlay::menu()),
                    MenuEv::None => {}
                }
            }
            Overlay::Options(menu) => {
                let r = Rect::new(60.0, SH + 36.0, 456.0, 300.0);
                parchment(r.x - 8.0, r.y - 8.0, r.w + 16.0, r.h + 16.0);
                self.ui.text_center("オプション", LW / 2.0, SH + 4.0, 24.0, INK);
                let bar = |v: u8| format!("{}{} {}", "■".repeat(v as usize), "□".repeat(10 - v as usize), v);
                let log_mode = if self.g.manual_log { "押して進む" } else { "自動で流れる" };
                let view_mode = ["オート", "スマホ縦", "PC横"][self.g.view_mode as usize % 3];
                let mute = if self.g.sound_mode == 0 { "" } else { "(いま ミュート中)" };
                let rows = vec![
                    Row::new("音楽の音量").right(bar(self.g.bgm_vol)),
                    Row::new("効果音の音量").right(bar(self.g.sfx_vol)),
                    Row::new("戦闘ログ").right(log_mode),
                    Row::new("あるく速さ").right(["ふつう(最速)", "すこし遅い", "遅い"][self.g.walk_speed as usize % 3]),
                    Row::new("画面").right(view_mode),
                    Row::new("もどる"),
                ];
                // ← → で 音量・設定を変える(Z でも 1つずつ進む)
                let cur = menu.cursor.min(rows.len() - 1);
                let dir: i32 = if self.inp.right { 1 } else if self.inp.left { -1 } else { 0 };
                let mut changed = dir != 0;
                match cur {
                    0 if dir != 0 => self.g.bgm_vol = (self.g.bgm_vol as i32 + dir).clamp(0, 10) as u8,
                    1 if dir != 0 => self.g.sfx_vol = (self.g.sfx_vol as i32 + dir).clamp(0, 10) as u8,
                    2 if dir != 0 => self.g.manual_log = !self.g.manual_log,
                    3 if dir != 0 => self.g.walk_speed = ((self.g.walk_speed as i32 + dir + 3) % 3) as u8,
                    4 if dir != 0 => self.g.view_mode = ((self.g.view_mode as i32 + dir + 3) % 3) as u8,
                    _ => changed = false,
                }
                if changed && cur == 1 {
                    self.ui.audio.sfx_vol.set(self.g.sfx_vol as f32 / 10.0);
                    self.ui.audio.sfx("hit");
                }
                menu.no_page = true;
                match menu.run(&self.ui, &self.inp, &rows, r, 48.0, 22.0, Style::Parchment, true) {
                    MenuEv::Confirm(0) => self.g.bgm_vol = (self.g.bgm_vol + 1) % 11,
                    MenuEv::Confirm(1) => {
                        self.g.sfx_vol = (self.g.sfx_vol + 1) % 11;
                        self.ui.audio.sfx_vol.set(self.g.sfx_vol as f32 / 10.0);
                        self.ui.audio.sfx("hit");
                    }
                    MenuEv::Confirm(2) => self.g.manual_log = !self.g.manual_log,
                    MenuEv::Confirm(3) => self.g.walk_speed = (self.g.walk_speed + 1) % 3,
                    MenuEv::Confirm(4) => self.g.view_mode = (self.g.view_mode + 1) % 3,
                    MenuEv::Confirm(_) | MenuEv::Cancel => keep = false,
                    MenuEv::None => {}
                }
                self.ui.text_center(&format!("← →:かえる  Z:ひとつ進める  M:音のオン/オフ {}", mute), LW / 2.0, SH + 348.0, 15.0, CREAM);
            }
            Overlay::Status(idx) => {
                card(&self.ui, &self.g, *idx, 8.0, 8.0, LW - 16.0, SH - 16.0);
                summary(&self.ui, &self.g, SH);
                self.ui.text_center("Q/E か ← →: なかまを きりかえ   Esc: もどる", LW / 2.0, SH + 394.0, 17.0, CREAM);
                if self.inp.left || self.inp.l {
                    *idx = prev_member(&self.g, *idx);
                }
                if self.inp.right || self.inp.r {
                    *idx = next_member(&self.g, *idx);
                }
                if self.inp.cancel || self.inp.ok || self.inp.click {
                    new_ov = Some(Overlay::menu());
                }
            }
            Overlay::Equip { who, menu, sub } => {
                card(&self.ui, &self.g, *who, 8.0, 8.0, LW - 16.0, SH - 16.0);
                panel_wood(0.0, SH, LW, SH);
                let name = self.g.party[*who].name.clone();
                self.ui.text(&format!("{} の そうび   (←→: なかまを きりかえ)", name), 18.0, SH + 8.0, 22.0, CREAM);
                if let Some((slot, m2, cands)) = sub {
                    let rows: Vec<Row> = cands
                        .iter()
                        .map(|c| match c {
                            None => Row::new("はずす"),
                            Some(n) => {
                                let d = item(n).unwrap();
                                Row::new(format!("{}  x{}", n, self.g.count(n))).right(if d.kind == Kind::Ring { crate::town::equip_effects(n) } else if d.kind == Kind::Weapon { format!("ATK+{}", d.power) } else { format!("DEF+{}", d.power) })
                            }
                        })
                        .collect();
                    self.ui.text(&format!("{} を えらぶ", SLOT_NAMES[*slot]), 18.0, SH + 38.0, 20.0, rgb(255, 230, 140));
                    // いま えらんでいる 装備の 説明(特殊な 能力も)
                    let cur = m2.cursor.min(cands.len().saturating_sub(1));
                    if let Some(Some(n)) = cands.get(cur) {
                        draw_rectangle(10.0, SH + 322.0, LW - 20.0, 92.0, rgba(24, 18, 10, 235));
                        draw_rectangle_lines(10.0, SH + 322.0, LW - 20.0, 92.0, 2.0, rgb(232, 192, 64));
                        self.ui.text_wrap(&format!("{}\n{}", n, item_help(n)), 20.0, SH + 328.0, LW - 40.0, 17.0, CREAM, 21.0);
                    }
                    match m2.run(&self.ui, &self.inp, &rows, Rect::new(10.0, SH + 68.0, LW - 20.0, 250.0), 46.0, 22.0, Style::Wood, true) {
                        MenuEv::Confirm(i) => {
                            match &cands[i] {
                                None => self.g.unequip(*who, *slot),
                                Some(n) => {
                                    let _ = self.g.equip(*who, n);
                                    self.ui.audio.sfx("equip");
                                }
                            }
                            *sub = None;
                        }
                        MenuEv::Cancel => *sub = None,
                        MenuEv::None => {}
                    }
                } else {
                    let m = &self.g.party[*who];
                    let mut rows: Vec<Row> = SLOT_NAMES.iter().enumerate().map(|(i, s)| Row::new(*s).right(m.equip[i].clone().unwrap_or_else(|| "―".to_string())).enabled(m.can_wear(i))).collect();
                    rows.push(Row::new("さいきょう そうび(おまかせ)").enabled((0..6).any(|i| m.can_wear(i))));
                    menu.no_page = true;
                    match menu.run(&self.ui, &self.inp, &rows, Rect::new(10.0, SH + 40.0, LW - 20.0, 340.0), 46.0, 22.0, Style::Wood, true) {
                        MenuEv::Confirm(6) => {
                            let n = self.g.best_equip(*who);
                            self.say(if n > 0 { format!("{} つ そうびを かえた!", n) } else { "もう いちばん強い そうびだ".to_string() });
                        }
                        MenuEv::Confirm(i) => {
                            let mut c: Vec<Option<String>> = vec![None];
                            for d in self.g.owned_defs().into_iter().filter(|d| d.kind.slot() == Some(i)) {
                                c.push(Some(d.name.to_string()));
                            }
                            *sub = Some((i, ListMenu::new(), c));
                        }
                        MenuEv::Cancel => new_ov = Some(Overlay::menu()),
                        MenuEv::None => {}
                    }
                    // いま つけている 装備の 説明(カーソルの スロット)
                    let slot_cur = menu.cursor.min(5);
                    if let Some(n) = self.g.party[*who].equip[slot_cur].clone() {
                        draw_rectangle(10.0, SH + 322.0, LW - 20.0, 52.0, rgba(24, 18, 10, 235));
                        draw_rectangle_lines(10.0, SH + 322.0, LW - 20.0, 52.0, 2.0, rgb(232, 192, 64));
                        self.ui.text_wrap(&format!("{}: {}", n, item_help(&n)), 20.0, SH + 326.0, LW - 40.0, 16.0, CREAM, 20.0);
                    }
                    if self.g.party[*who].is_slime {
                        self.ui.text("ジェリーは ものまねした部分にだけ そうびできる(メニューの「ものまね」)", 18.0, SH + 380.0, 16.0, rgb(255, 200, 160));
                    }
                    if self.inp.l || self.inp.left {
                        *who = prev_member(&self.g, *who);
                    }
                    if self.inp.r || self.inp.right {
                        *who = next_member(&self.g, *who);
                    }
                }
            }
            Overlay::Mimic { menu, sub, msg } => {
                // ジェリーの ものまね
                card(&self.ui, &self.g, 1, 8.0, 8.0, LW - 16.0, SH - 16.0);
                panel_wood(0.0, SH, LW, SH);
                self.ui.text("ジェリーの ものまね", 18.0, SH + 8.0, 22.0, CREAM);
                if let Some((part, m2, cands)) = sub {
                    let rows: Vec<Row> = cands
                        .iter()
                        .map(|c| match c {
                            None => Row::new("もとの スライム"),
                            Some(id) => Row::new(mimic_label(&self.g, id)),
                        })
                        .collect();
                    self.ui.text(&format!("{} を マネする魔物を えらぶ", crate::mimic::PARTS[*part]), 18.0, SH + 38.0, 18.0, rgb(255, 230, 140));
                    let cur = m2.cursor.min(cands.len().saturating_sub(1));
                    m2.no_page = true;
                    // 説明(いま選んでいる魔物)
                    if let Some(Some(id)) = cands.get(cur) {
                        self.ui.text_wrap(&crate::mimic::describe(*part, id, self.g.party[1].mimic_plus.get(*id).copied().unwrap_or(0)), 18.0, SH + 350.0, LW - 36.0, 17.0, CREAM, 22.0);
                    }
                    match m2.run(&self.ui, &self.inp, &rows, Rect::new(10.0, SH + 64.0, LW - 20.0, 280.0), 34.0, 21.0, Style::Wood, true) {
                        MenuEv::Confirm(i) => {
                            let id = cands[i];
                            *msg = match self.g.set_mimic(1, *part, id) {
                                Ok(m) => m,
                                Err(e) => e,
                            };
                            self.ui.audio.sfx("equip");
                            *sub = None;
                        }
                        MenuEv::Cancel => *sub = None,
                        MenuEv::None => {}
                    }
                } else {
                    let m = &self.g.party[1];
                    let mut rows: Vec<Row> = (0..4)
                        .map(|p| {
                            let cur = m.mimic[p].as_deref().map(|id| mimic_label(&self.g, id)).unwrap_or_else(|| "もとの スライム".to_string());
                            Row::new(crate::mimic::PARTS[p]).right(cur)
                        })
                        .collect();
                    rows.push(Row::new("おまかせ(いちばん強くなる組み合わせ)").enabled(!self.g.mimic_choices().is_empty()));
                    menu.no_page = true;
                    if !msg.is_empty() {
                        self.ui.text_wrap(msg, 18.0, SH + 300.0, LW - 36.0, 18.0, rgb(255, 230, 140), 24.0);
                    }
                    self.ui.text_wrap("たおした魔物の あたま・うで・からだ・あしを マネすると、能力・そうび・特技・魔法が かわる。", 18.0, SH + 346.0, LW - 36.0, 16.0, CREAM, 22.0);
                    match menu.run(&self.ui, &self.inp, &rows, Rect::new(10.0, SH + 40.0, LW - 20.0, 250.0), 46.0, 21.0, Style::Wood, true) {
                        MenuEv::Confirm(4) => {
                            let n = self.g.auto_mimic(1);
                            *msg = if n > 0 { format!("{} か所 マネしなおした!", n) } else { "もう いちばん強い組み合わせだ".to_string() };
                        }
                        MenuEv::Confirm(p) => {
                            let mut c: Vec<Option<&'static str>> = vec![None];
                            c.extend(self.g.mimic_choices().into_iter().map(Some));
                            *sub = Some((p, ListMenu::new(), c));
                        }
                        MenuEv::Cancel => new_ov = Some(Overlay::menu()),
                        MenuEv::None => {}
                    }
                }
            }
            Overlay::Items { menu, pick, msg } => {
                // 上画面: 説明
                parchment(8.0, 8.0, LW - 16.0, SH - 16.0);
                self.ui.text("どうぐ", 28.0, 22.0, 28.0, INK);
                chip(&self.ui, LW - 200.0, 20.0, 176.0, 32.0, &format!("{} G", comma(self.g.gold)), 20.0);
                let names: Vec<String> = self.g.owned_defs().into_iter().map(|d| d.name.to_string()).collect();
                if let Some((n, m2)) = pick {
                    let mem = self.g.members();
                    let rows: Vec<Row> = mem.iter().map(|&k| &self.g.party[k]).map(|m| Row::new(m.name.clone()).right(format!("HP{}/{}  MP{}/{}", m.hp, m.hp_max, m.mp, m.mp_max))).collect();
                    panel_wood(0.0, SH, LW, SH);
                    self.ui.text(&format!("{} を だれに つかう?", n), 18.0, SH + 10.0, 22.0, CREAM);
                    match m2.run(&self.ui, &self.inp, &rows, Rect::new(10.0, SH + 50.0, LW - 20.0, 200.0), 56.0, 22.0, Style::Wood, true) {
                        MenuEv::Confirm(i) => {
                            let i = mem[i];
                            let d = item(n).unwrap();
                            let ok = self.g.party[i].alive() || d.kind != Kind::Potion;
                            if ok && self.g.take_item(n, 1) {
                                let m = &mut self.g.party[i];
                                if n == "かいふくやく" {
                                    m.hp = (m.hp + d.power).min(m.hp_max);
                                } else if n == "ばんのうやく" {
                                    for m in self.g.party.iter_mut().filter(|m| m.alive()) {
                                        m.hp = (m.hp + d.power).min(m.hp_max);
                                    }
                                } else {
                                    m.mp = (m.mp + d.power).min(m.mp_max);
                                }
                                *msg = format!("{} は {} を つかった!", self.g.party[i].name, n);
                            }
                            *pick = None;
                        }
                        MenuEv::Cancel => *pick = None,
                        MenuEv::None => {}
                    }
                } else {
                    panel_wood(0.0, SH, LW, SH);
                    let rows: Vec<Row> = names.iter().map(|n| Row::new(format!("{}  x{}", n, self.g.count(n)))).collect();
                    let cur = menu.cursor.min(names.len().saturating_sub(1));
                    if let Some(n) = names.get(cur) {
                        self.ui.text_wrap(&item_help(n), 28.0, 70.0, LW - 60.0, 22.0, INK, 30.0);
                    }
                    if !msg.is_empty() {
                        self.ui.text(msg, 28.0, 350.0, 20.0, rgb(150, 50, 30));
                    }
                    if names.is_empty() {
                        self.ui.text("もちものは ない", 28.0, 70.0, 22.0, INK);
                    }
                    match menu.run(&self.ui, &self.inp, &rows, Rect::new(10.0, SH + 10.0, LW - 20.0, 410.0), 42.0, 21.0, Style::Wood, true) {
                        MenuEv::Confirm(i) => {
                            let n = names[i].clone();
                            let d = item(&n).unwrap();
                            if d.kind == Kind::Potion {
                                *pick = Some((n, ListMenu::new()));
                                msg.clear();
                            } else {
                                *msg = "それは ここでは つかえない".to_string();
                            }
                        }
                        MenuEv::Cancel => new_ov = Some(Overlay::menu()),
                        MenuEv::None => {}
                    }
                }
            }
        }
        if let Some(n) = new_ov {
            ov = n;
        }
        if keep {
            self.ov = Some(ov);
        }
    }
}

/// ものまねの 魔物の 名前(瘴気で 強くなった ぶんは 「ショーグンウルフ+3」のように)
fn mimic_label(g: &Game, id: &str) -> String {
    let plus = g.party[1].mimic_plus.get(id).copied().unwrap_or(0);
    if plus > 0 {
        format!("{}+{}", mon(id).name, plus)
    } else {
        mon(id).name.to_string()
    }
}
