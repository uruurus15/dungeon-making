//! 戦闘画面: 上=敵とメッセージ、下=敵HP行・コマンド・パーティHP/MP行(原作のレイアウトに合わせた)。

use crate::data::*;
use crate::dungeon::DunSt;
use crate::sim::*;
use crate::town::TownSt;
use crate::ui::*;
use crate::{App, Scr};
use macroquad::prelude::*;
use std::collections::VecDeque;

pub enum TKind {
    Attack,
    Spell(&'static str),
    Item(String),
}

pub enum Phase {
    Intro,
    Run,
    Cmd(usize),
    Magic(usize),
    Items(usize),
    Target(usize, TKind),
    Finish(Outcome),
}

pub struct BattleSt {
    pub b: Battle,
    pub prev: (usize, usize),
    pub phase: Phase,
    pub queue: VecDeque<String>,
    pub cur: Option<(String, f32)>,
    pub menu: ListMenu,
    pub menu2: ListMenu,
    pub log_last: String,
    /// 「おまかせ」中(Xかescで中止)
    pub auto: bool,
    /// 仲間ごとの前回の対象(敵の番号)
    pub last_t: [usize; 3],
    pub tinit: bool,
}

impl BattleSt {
    pub fn new(g: &Game, srcs: &[Source], prev: (usize, usize)) -> BattleSt {
        let b = Battle::new(g, srcs);
        let mut menu = ListMenu::new();
        menu.cursor = 1; // たたかう
        BattleSt { b, prev, phase: Phase::Intro, queue: VecDeque::new(), cur: None, menu, menu2: ListMenu::new(), log_last: String::new(), auto: false, last_t: [0; 3], tinit: false }
    }
}

const MSG_TIME: f32 = 0.55;

fn slot_x(n: usize, i: usize) -> f32 {
    match n {
        1 => LW / 2.0,
        2 => LW / 2.0 + if i == 0 { -110.0 } else { 110.0 },
        _ => LW / 2.0 + (i as f32 - 1.0) * 188.0,
    }
}

impl App {
    fn draw_battle_top(&self, st: &BattleSt, msg: &str, actor: Option<usize>) {
        let ui = &self.ui;
        ui.art.draw("bg_battle", 0.0, 0.0, LW, SH);
        let n = st.b.enemies.len();
        for (i, e) in st.b.enemies.iter().enumerate() {
            if e.hp <= 0 || e.fled {
                continue;
            }
            let size = if e.def.boss { 340.0 } else if n >= 3 { 170.0 } else { 210.0 };
            let x = slot_x(n, i);
            let bob = (ui.t * 2.5 + i as f32).sin() * 3.0;
            let tint = if e.sleep > 0 { rgb(150, 160, 230) } else { WHITE };
            ui.art.draw_tinted(e.def.sprite, x - size / 2.0, 312.0 - size + bob, size, size, tint);
            if e.sleep > 0 {
                ui.text_shadow("Zzz", x + size / 4.0, 306.0 - size + 10.0, 24.0, WHITE);
            }
        }
        self.draw_haze(0.0, SH);
        if let Some(a) = actor {
            let name = &self.g.party[a].name;
            let w = ui.measure(name, 22.0) + 56.0;
            draw_rectangle(LW - w - 12.0, 286.0, w, 36.0, rgb(86, 56, 32));
            draw_rectangle_lines(LW - w - 12.0, 286.0, w, 36.0, 2.5, rgb(232, 217, 168));
            draw_circle(LW - w + 4.0, 304.0, 7.0, rgb(200, 60, 50));
            ui.text(name, LW - w + 22.0, 292.0, 22.0, CREAM);
        }
        msgbox(16.0, 328.0, 544.0, 94.0);
        if let Some(a) = actor {
            // 行動を選んでいるキャラの顔(だれの番か 文字を読まなくてもわかるように)
            let port = ["portrait_a", "portrait_j", "portrait_m"][a.min(2)];
            draw_rectangle(26.0, 334.0, 82.0, 82.0, rgb(52, 36, 22));
            ui.art.draw(port, 27.0, 335.0, 80.0, 80.0);
            draw_rectangle_lines(26.0, 334.0, 82.0, 82.0, 2.5, rgb(232, 217, 168));
            ui.text_wrap(msg, 124.0, 338.0, 424.0, 21.0, CREAM, 27.0);
        } else {
            ui.text_wrap(msg, 32.0, 338.0, 512.0, 21.0, CREAM, 27.0);
        }
    }

    fn draw_battle_bottom(&self, st: &BattleSt) {
        let ui = &self.ui;
        panel_brick(0.0, SH, LW, SH);
        let n = st.b.enemies.len();
        for (i, e) in st.b.enemies.iter().enumerate() {
            let w = 176.0;
            let x = 10.0 + i as f32 * (w + 6.0);
            if i >= 3 {
                break;
            }
            gold_box(x, SH + 10.0, w, 74.0);
            ui.text_center(e.def.name, x + w / 2.0, SH + 16.0, 19.0, if e.hp > 0 && !e.fled { CREAM } else { rgb(120, 110, 100) });
            if e.fled {
                ui.text_center("にげた", x + w / 2.0, SH + 46.0, 18.0, rgb(150, 140, 130));
            } else if e.hp > 0 {
                ui.text(&format!("HP {}/{}", e.hp, e.max_hp()), x + 12.0, SH + 46.0, 18.0, rgb(150, 235, 120));
                bar(x + 12.0, SH + 72.0, w - 24.0, e.hp as f32 / e.max_hp() as f32, rgb(110, 210, 90));
            } else {
                ui.text_center("たおした", x + w / 2.0, SH + 46.0, 18.0, rgb(150, 140, 130));
            }
        }
        let _ = n;
        for (k, &i) in self.g.members().iter().enumerate() {
            let m = &self.g.party[i];
            let w = 182.0;
            let x = 8.0 + k as f32 * (w + 6.0);
            let y = SH + 318.0;
            gold_box(x, y, w, 104.0);
            ui.text(&m.name, x + 10.0, y + 6.0, 19.0, CREAM);
            ui.text_right(m.status_text(), x + w - 10.0, y + 9.0, 14.0, if m.poison { rgb(214, 150, 255) } else { rgb(190, 235, 190) });
            ui.text(&format!("HP {}/{}", m.hp, m.hp_max), x + 10.0, y + 34.0, 17.0, rgb(150, 235, 120));
            bar(x + 10.0, y + 56.0, w - 20.0, m.hp as f32 / m.hp_max as f32, rgb(110, 210, 90));
            ui.text(&format!("MP {}/{}", m.mp, m.mp_max), x + 10.0, y + 66.0, 17.0, rgb(214, 160, 255));
            bar(x + 10.0, y + 90.0, w - 20.0, m.mp as f32 / m.mp_max.max(1) as f32, rgb(180, 120, 240));
        }
    }

    pub fn battle(&mut self, st: &mut BattleSt) -> Option<Scr> {
        let menu_rect = Rect::new(120.0, SH + 92.0, 336.0, 224.0);
        let phase_actor = match &st.phase {
            Phase::Cmd(i) | Phase::Magic(i) | Phase::Items(i) | Phase::Target(i, _) => Some(*i),
            _ => None,
        };
        let shown = st.cur.as_ref().map(|c| c.0.clone()).unwrap_or_else(|| match &st.phase {
            Phase::Cmd(i) => format!("{} は どうする?", self.g.party[*i].name),
            Phase::Magic(_) => "どの まほうを つかう?".to_string(),
            Phase::Items(_) => "どの どうぐを つかう?".to_string(),
            Phase::Target(_, k) => match k {
                _ if is_ally_target(k) => "だれに つかう?".to_string(),
                _ => "どの てきを ねらう?".to_string(),
            },
            _ => st.log_last.clone(),
        });
        self.draw_battle_top(st, &shown, phase_actor);
        if st.auto {
            ui_auto_badge(&self.ui);
        }
        self.draw_battle_bottom(st);

        match &mut st.phase {
            Phase::Intro => {
                if self.inp.scripted {
                    println!("BATTLE_START used={}", self.inp.used);
                }
                let names: Vec<&str> = st.b.enemies.iter().map(|e| e.def.name).collect();
                if st.b.boss {
                    self.ui.audio.sfx("boss");
                }
                let m = if st.b.boss { format!("{} が あらわれた!!", names[0]) } else { format!("{} が あらわれた!", names.join("、")) };
                st.queue.push_back(m);
                st.phase = Phase::Run;
            }
            Phase::Run => {
                if st.auto && self.inp.cancel {
                    st.auto = false;
                    st.queue.push_back("おまかせを やめた".to_string());
                }
                let manual = self.g.manual_log && !st.auto;
                if let Some((_, t)) = &mut st.cur {
                    *t -= get_frame_time();
                    if (*t <= 0.0 && !manual) || self.inp.ok || self.inp.click || (!manual && self.inp.fast) {
                        st.log_last = st.cur.take().map(|c| c.0).unwrap_or_default();
                    }
                }
                if st.cur.is_none() {
                    if let Some(m) = st.queue.pop_front() {
                        let names: Vec<String> = self.g.party.iter().map(|p| p.name.clone()).collect();
                        if let Some(s) = crate::audio::sfx_for_message(&m, &names) {
                            self.ui.audio.sfx(s);
                        }
                        st.cur = Some((m, MSG_TIME));
                    } else {
                        match st.b.advance(&mut self.g) {
                            Advance::NeedCmd(i) => {
                                if st.auto && self.inp.cancel {
                                    st.auto = false;
                                    st.queue.push_back("おまかせを やめた".to_string());
                                }
                                if st.auto {
                                    let cmd = st.b.auto_cmd(&self.g, i);
                                    let msgs = st.b.command(&mut self.g, i, cmd);
                                    st.queue.extend(msgs);
                                } else {
                                    st.phase = Phase::Cmd(i);
                                }
                            }
                            Advance::Msgs(v) => st.queue.extend(v),
                            Advance::End(o) => {
                                match o {
                                    Outcome::Won => {
                                        let mut lines = vec![];
                                        if st.b.boss {
                                            lines.push(format!("{} を たおした!!", st.b.enemies[0].def.name));
                                        } else {
                                            lines.push("モンスターを たおした!".to_string());
                                        }
                                        lines.push(format!("{} ギルを てにいれた!", st.b.gold));
                                        for d in &st.b.got {
                                            lines.push(format!("{} を てにいれた!", d));
                                        }
                                        if st.b.boss {
                                            let f = self.g.dungeon.floor;
                                            if f < MAX_FLOOR {
                                                lines.push(format!("ダンジョンの奥に 地下{}階への階段が あらわれた!", f + 1));
                                            } else {
                                                lines.push(format!("地下{}階の ボスを たおした! ぼうけんは まだ つづく…", f));
                                            }
                                        }
                                        st.queue.extend(lines);
                                    }
                                    Outcome::Lost => {
                                        st.queue.push_back("パーティは ぜんめつした…".to_string());
                                        st.queue.push_back("気がつくと 町の入口に ころがっていた。おかねを すこし おとしてしまった…".to_string());
                                    }
                                    Outcome::Fled => {}
                                }
                                match o {
                                    Outcome::Won => self.ui.audio.jingle("win", 3.0),
                                    Outcome::Lost => self.ui.audio.jingle("lose", 3.0),
                                    Outcome::Fled => {}
                                }
                                st.phase = Phase::Finish(o);
                            }
                        }
                    }
                }
            }
            Phase::Finish(o) => {
                let manual = self.g.manual_log;
                if let Some((_, t)) = &mut st.cur {
                    *t -= get_frame_time();
                    if (*t <= 0.0 && !manual) || self.inp.ok || self.inp.click || (!manual && self.inp.fast) {
                        st.log_last = st.cur.take().map(|c| c.0).unwrap_or_default();
                    }
                }
                if st.cur.is_none() {
                    if let Some(m) = st.queue.pop_front() {
                        st.cur = Some((m, MSG_TIME + 0.2));
                    } else {
                        if self.inp.scripted {
                            println!("BATTLE_END used={} outcome={:?}", self.inp.used, o);
                        }
                        return Some(match o {
                            Outcome::Won if st.b.joined.contains(&1) => {
                                // ジェリーが 仲間になる場面
                                let mei = self.g.party[2].joined;
                                {
                                    let mut t = crate::talk::TalkSt::new("bg_battle", crate::script::jerry_join_scene(mei), Scr::Dungeon(DunSt::new())).with_cast("ジェリー");
                                    if mei {
                                        t = t.with_cast("メイミ");
                                    }
                                    Scr::Talk(Box::new(t))
                                }
                            }
                            Outcome::Won => Scr::Dungeon(DunSt::new()),
                            Outcome::Fled => {
                                self.g.pos = st.prev;
                                Scr::Dungeon(DunSt::new())
                            }
                            Outcome::Lost => {
                                self.g.wipe();
                                Scr::Town(TownSt::new(Some("ぜんめつして 町へ もどされた。まずは休んで 作戦を立てなおそう。".to_string())))
                            }
                        });
                    }
                }
            }
            Phase::Cmd(who) => {
                let who = *who;
                let rows = vec![Row::new("おまかせ").right("Xで中止"), Row::new("たたかう"), Row::new("まほう"), Row::new("どうぐ"), Row::new("ぼうぎょ"), Row::new("にげる")];
                gold_box(menu_rect.x - 6.0, menu_rect.y - 6.0, menu_rect.w + 12.0, menu_rect.h + 12.0);
                match st.menu.run(&self.ui, &self.inp, &rows, menu_rect, 36.0, 21.0, Style::Dark, true) {
                    MenuEv::Confirm(0) => {
                        st.auto = true;
                        let cmd = st.b.auto_cmd(&self.g, who);
                        let msgs = st.b.command(&mut self.g, who, cmd);
                        st.queue.extend(msgs);
                        st.phase = Phase::Run;
                    }
                    MenuEv::Confirm(1) => {
                        let alive = st.b.alive_enemies();
                        if alive.len() == 1 {
                            let t = alive[0];
                            let msgs = st.b.command(&mut self.g, who, Cmd::Attack(t));
                            st.queue.extend(msgs);
                            st.phase = Phase::Run;
                        } else {
                            st.menu2 = ListMenu::new();
                            st.tinit = false;
                        st.phase = Phase::Target(who, TKind::Attack);
                        }
                    }
                    MenuEv::Confirm(2) => {
                        st.menu2 = ListMenu::new();
                        st.phase = Phase::Magic(who);
                    }
                    MenuEv::Confirm(3) => {
                        st.menu2 = ListMenu::new();
                        st.phase = Phase::Items(who);
                    }
                    MenuEv::Confirm(4) => {
                        let msgs = st.b.command(&mut self.g, who, Cmd::Defend);
                        st.queue.extend(msgs);
                        st.phase = Phase::Run;
                    }
                    MenuEv::Confirm(_) => {
                        let msgs = st.b.command(&mut self.g, who, Cmd::Flee);
                        st.queue.extend(msgs);
                        st.phase = Phase::Run;
                    }
                    _ => {}
                }
            }
            Phase::Magic(who) => {
                let who = *who;
                let known: Vec<&'static Spell> = self.g.party[who].spells.iter().map(|s| spell(s)).collect();
                let rows: Vec<Row> = if known.is_empty() {
                    vec![Row::new("おぼえている まほうが ない").enabled(false)]
                } else {
                    let sealed = self.g.party[who].seal > 0;
                    known.iter().map(|s| Row::new(s.name).right(if sealed { "ふういん".to_string() } else { format!("MP{}", s.mp) }).enabled(self.g.party[who].mp >= s.mp && !sealed)).collect()
                };
                gold_box(menu_rect.x - 6.0, menu_rect.y - 6.0, menu_rect.w + 12.0, menu_rect.h + 12.0);
                match st.menu2.run(&self.ui, &self.inp, &rows, menu_rect, 31.0, 19.0, Style::Dark, true) {
                    MenuEv::Confirm(i) => {
                        let s = known[i];
                        match s.tgt {
                            Tgt::FoeAll | Tgt::AllyAll => {
                                let first = st.b.alive_enemies().first().copied().unwrap_or(0);
                                let msgs = st.b.command(&mut self.g, who, Cmd::Spell(s.id, if s.tgt == Tgt::FoeAll { first } else { who }));
                                st.queue.extend(msgs);
                                st.phase = Phase::Run;
                            }
                            Tgt::Ally => {
                                st.menu2 = ListMenu::new();
                                st.tinit = false;
                                st.phase = Phase::Target(who, TKind::Spell(s.id));
                            }
                            Tgt::Foe => {
                                let alive = st.b.alive_enemies();
                                if alive.len() == 1 {
                                    let msgs = st.b.command(&mut self.g, who, Cmd::Spell(s.id, alive[0]));
                                    st.queue.extend(msgs);
                                    st.phase = Phase::Run;
                                } else {
                                    st.menu2 = ListMenu::new();
                                    st.tinit = false;
                                    st.phase = Phase::Target(who, TKind::Spell(s.id));
                                }
                            }
                        }
                    }
                    MenuEv::Cancel => st.phase = Phase::Cmd(who),
                    _ => {}
                }
            }
            Phase::Items(who) => {
                let who = *who;
                let list: Vec<&'static ItemDef> = ITEMS.iter().filter(|d| matches!(d.kind, Kind::Potion | Kind::Bomb) && self.g.count(d.name) > 0).collect();
                let rows: Vec<Row> = if list.is_empty() {
                    vec![Row::new("つかえる どうぐが ない").enabled(false)]
                } else {
                    list.iter().map(|d| Row::new(d.name).right(format!("x{}", self.g.count(d.name)))).collect()
                };
                gold_box(menu_rect.x - 6.0, menu_rect.y - 6.0, menu_rect.w + 12.0, menu_rect.h + 12.0);
                match st.menu2.run(&self.ui, &self.inp, &rows, menu_rect, 38.0, 21.0, Style::Dark, true) {
                    MenuEv::Confirm(i) => {
                        let d = list[i];
                        if d.name == "炎バクダン" {
                            let msgs = st.b.command(&mut self.g, who, Cmd::Item(d.name.to_string(), 0));
                            st.queue.extend(msgs);
                            st.phase = Phase::Run;
                        } else {
                            st.menu2 = ListMenu::new();
                            st.tinit = false;
                        st.phase = Phase::Target(who, TKind::Item(d.name.to_string()));
                        }
                    }
                    MenuEv::Cancel => st.phase = Phase::Cmd(who),
                    _ => {}
                }
            }
            Phase::Target(who, kind) => {
                let who = *who;
                let ally = is_ally_target(kind);
                let (rows, idxs): (Vec<Row>, Vec<usize>) = if ally {
                    let v: Vec<usize> = self.g.members();
                    (v.iter().map(|&i| Row::new(self.g.party[i].name.clone()).right(format!("HP{}/{}", self.g.party[i].hp, self.g.party[i].hp_max))).collect(), v)
                } else {
                    let v = st.b.alive_enemies();
                    (v.iter().map(|&i| Row::new(st.b.enemies[i].def.name).right(format!("HP{}", st.b.enemies[i].hp))).collect(), v)
                };
                gold_box(menu_rect.x - 6.0, menu_rect.y - 6.0, menu_rect.w + 12.0, menu_rect.h + 12.0);
                if !st.tinit {
                    st.tinit = true;
                    if !ally {
                        if let Some(pos) = idxs.iter().position(|&e| e == st.last_t[who]) {
                            st.menu2.cursor = pos;
                        }
                    }
                }
                // 選択中の敵に矢印
                if !ally {
                    if let Some(&ei) = idxs.get(st.menu2.cursor) {
                        let x = slot_x(st.b.enemies.len(), ei);
                        draw_triangle(vec2(x - 14.0, 40.0), vec2(x + 14.0, 40.0), vec2(x, 62.0 + 6.0 * (self.ui.t * 8.0).sin()), rgb(255, 230, 90));
                    }
                }
                // 画面では横にならんでいるので、左右キーでも選べる
                let n = idxs.len().max(1);
                if self.inp.left {
                    st.menu2.cursor = (st.menu2.cursor + n - 1) % n;
                }
                if self.inp.right {
                    st.menu2.cursor = (st.menu2.cursor + 1) % n;
                }
                match st.menu2.run(&self.ui, &self.inp, &rows, menu_rect, 38.0, 21.0, Style::Dark, true) {
                    MenuEv::Confirm(i) => {
                        let t = idxs[i];
                        if !ally {
                            st.last_t[who] = t;
                        }
                        let cmd = match kind {
                            TKind::Attack => Cmd::Attack(t),
                            TKind::Spell(id) => Cmd::Spell(id, t),
                            TKind::Item(n) => Cmd::Item(n.clone(), t),
                        };
                        let msgs = st.b.command(&mut self.g, who, cmd);
                        st.queue.extend(msgs);
                        st.phase = Phase::Run;
                    }
                    MenuEv::Cancel => st.phase = Phase::Cmd(who),
                    _ => {}
                }
            }
        }
        None
    }
}

/// 味方を選ぶ使い方か(薬・回復の魔法・つよくなれ など)
fn is_ally_target(kind: &TKind) -> bool {
    match kind {
        TKind::Spell(id) => spell(id).tgt == Tgt::Ally,
        TKind::Item(n) => n == "かいふくやく" || n == "まほうのくすり" || n == "ばんのうやく",
        _ => false,
    }
}

fn ui_auto_badge(ui: &Ui) {
    draw_rectangle(12.0, 12.0, 250.0, 34.0, rgba(20, 30, 40, 220));
    draw_rectangle_lines(12.0, 12.0, 250.0, 34.0, 2.0, rgb(150, 190, 210));
    ui.text("おまかせ中(Xで中止)", 22.0, 18.0, 19.0, rgb(255, 230, 140));
}
