//! 自宅: 食事(1日1回)でステータスUP・シャベル磨き・就寝。

use crate::data::*;
use crate::overlays::Overlay;
use crate::town::TownSt;
use crate::ui::*;
use crate::{App, Scr};
use macroquad::prelude::*;

pub enum HomePage {
    Main,
    Meal,
    Who(&'static str),
    ConfirmSleep,
}

pub struct HomeSt {
    pub page: HomePage,
    pub menu: ListMenu,
    pub msg: String,
}

impl HomeSt {
    pub fn new(g: &crate::sim::Game) -> HomeSt {
        let mq = crate::quest::main_quest(g);
        HomeSt { page: HomePage::Main, menu: ListMenu::new(), msg: format!("アレン「ただいま! ごはんを 食べて、ゆっくり 休もう。」\n▶ {}", mq.title) }
    }
}

impl App {
    pub fn home(&mut self, st: &mut HomeSt) -> Option<Scr> {
        self.ui.art.draw("scene_home", 0.0, 0.0, LW, SH);
        let ui = &self.ui;
        chip(ui, LW - 200.0, 12.0, 188.0, 34.0, &format!("{} G", crate::town::comma(self.g.gold)), 22.0);
        chip(ui, LW - 200.0, 52.0, 188.0, 30.0, &format!("{}日目", self.g.day), 20.0);
        // アレンなどの 台詞の ときは 顔と 名札つきで 出す
        crate::stage::draw_talk_face(ui, &st.msg, None, 336.0, 84.0);
        panel_wood(0.0, SH, LW, SH);

        let list_rect = Rect::new(10.0, SH + 40.0, LW - 20.0, 300.0);
        let mut next: Option<Scr> = None;
        match st.page {
            HomePage::Main => {
                ui.text("じたく", 18.0, SH + 8.0, 24.0, CREAM);
                let has_oil = self.g.count("魔法のオリーブオイル") > 0;
                let rows = vec![
                    Row::new("ごはんを たべる").right(if self.g.ate.iter().all(|a| *a) { "みんな 食べた" } else { "ひとり1日1回" }).enabled(!self.g.ate.iter().all(|a| *a)),
                    Row::new("シャベルを みがく").right(if has_oil { "オイルあり" } else { "" }).enabled(has_oil),
                    Row::new("ステータス・そうび・どうぐ"),
                    Row::new("ねる"),
                    Row::new("でる"),
                ];
                match st.menu.run(ui, &self.inp, &rows, list_rect, 52.0, 24.0, Style::Wood, true) {
                    MenuEv::Confirm(0) => {
                        st.page = HomePage::Meal;
                        st.menu = ListMenu::new();
                        st.msg = "アレン「なにを 作ろうかな。作れる 料理が 光ってるよ。」".to_string();
                    }
                    MenuEv::Confirm(1) => match self.g.shovel_polish() {
                        Ok(m) => {
                            self.ui.audio.sfx("equip");
                            st.msg = m
                        }
                        Err(e) => st.msg = e,
                    },
                    MenuEv::Confirm(2) => self.ov = Some(Overlay::menu()),
                    MenuEv::Confirm(3) => {
                        st.page = HomePage::ConfirmSleep;
                        st.menu = ListMenu::new();
                        st.msg = if self.g.ate.iter().all(|a| *a) {
                            "アレン「もう 寝ようか。寝ている あいだに 魔物が ダンジョンに 住みつくんだって。」".to_string()
                        } else {
                            // まだ 食べていない 仲間の 名前を 出して、アレンが 注意する
                            let hungry: Vec<String> = self.g.members().into_iter().filter(|&i| !self.g.ate[i]).map(|i| if i == 0 { "僕".to_string() } else { self.g.party[i].name.clone() }).collect();
                            format!("アレン「ちょっと待って。{}は まだ ごはんを 食べてないよ。このまま ねちゃって いいのかな?」", hungry.join("と"))
                        };
                    }
                    MenuEv::Confirm(_) | MenuEv::Cancel => next = Some(Scr::Town(TownSt::new(None))),
                    MenuEv::None => {}
                }
            }
            HomePage::Meal => {
                ui.text("りょうり", 18.0, SH + 8.0, 24.0, CREAM);
                let mut rows = vec![Row::new("おまかせ(まだ食べてない人みんな)").right("1回で全員").enabled(!self.g.ate.iter().all(|a| *a))];
                for r in RECIPES {
                    let known = self.g.recipes.contains(r.name);
                    let can = self.g.can_cook(r);
                    let label = if known { r.name.to_string() } else { "？？？".to_string() };
                    let right = if known { gain_text(r) } else { "".to_string() };
                    rows.push(Row::new(label).right(right).enabled(can));
                }
                let ev = st.menu.run(ui, &self.inp, &rows, Rect::new(10.0, SH + 40.0, LW - 20.0, 250.0), 44.0, 21.0, Style::Wood, true);
                // 材料の説明
                msgbox(10.0, SH + 300.0, LW - 20.0, 118.0);
                let r = &RECIPES[st.menu.cursor.saturating_sub(1).min(RECIPES.len() - 1)];
                if st.menu.cursor == 0 {
                    ui.text_wrap("まだ食べていない人それぞれに、作れる中でいちばん伸びる料理を食べさせるよ。", 26.0, SH + 310.0, LW - 52.0, 19.0, CREAM, 25.0);
                } else if self.g.recipes.contains(r.name) {
                    let needs: Vec<String> = r.needs.iter().map(|n| format!("{}({})", n, self.g.count(n))).collect();
                    ui.text_wrap(&format!("ざいりょう: {}", needs.join(" + ")), 26.0, SH + 310.0, LW - 52.0, 19.0, CREAM, 25.0);
                    ui.text(&format!("食べた人: {}", gain_text(r)), 26.0, SH + 372.0, 19.0, rgb(255, 230, 140));
                } else {
                    ui.text_wrap("魔法屋のおじさんに食材をみせると、新しい料理を教えてもらえるよ。", 26.0, SH + 310.0, LW - 52.0, 19.0, CREAM, 25.0);
                }
                match ev {
                    MenuEv::Confirm(0) => {
                        let msgs = self.g.auto_meals();
                        self.ui.audio.sfx("eat");
                        st.msg = msgs.join("\n");
                        st.page = HomePage::Main;
                        st.menu = ListMenu::new();
                    }
                    MenuEv::Confirm(i) => {
                        let i = i - 1;
                        st.page = HomePage::Who(RECIPES[i].name);
                        st.menu = ListMenu::new();
                        st.msg = "だれが 食べる?".to_string();
                    }
                    MenuEv::Cancel => {
                        st.page = HomePage::Main;
                        st.menu = ListMenu::new();
                    }
                    MenuEv::None => {}
                }
            }
            HomePage::Who(name) => {
                ui.text(&format!("{} を だれが たべる?", name), 18.0, SH + 8.0, 24.0, CREAM);
                let mem = self.g.members();
                let rows: Vec<Row> = mem
                    .iter()
                    .map(|&i| (i, &self.g.party[i]))
                    .map(|(i, m)| Row::new(m.name.clone()).right(if self.g.ate[i] { "食べた".to_string() } else { format!("HP{} MP{} つよ{} すば{} じょ{} かし{}", m.hp_max, m.mp_max, m.str_, m.spd, m.con, m.wis) }).enabled(!self.g.ate[i]))
                    .collect();
                match st.menu.run(ui, &self.inp, &rows, Rect::new(10.0, SH + 50.0, LW - 20.0, 190.0), 56.0, 20.0, Style::Wood, true) {
                    MenuEv::Confirm(i) => {
                        let i = mem[i];
                        match self.g.cook(name, i) {
                            Ok(m) => {
                                self.ui.audio.sfx("eat");
                                st.msg = m
                            }
                            Err(e) => st.msg = e,
                        }
                        st.page = if self.g.ate.iter().all(|a| *a) { HomePage::Main } else { HomePage::Meal };
                        st.menu = ListMenu::new();
                    }
                    MenuEv::Cancel => {
                        st.page = HomePage::Meal;
                        st.menu = ListMenu::new();
                    }
                    MenuEv::None => {}
                }
            }
            HomePage::ConfirmSleep => {
                ui.text("ねる?", 18.0, SH + 8.0, 24.0, CREAM);
                let rows = vec![Row::new("ねる"), Row::new("やめる")];
                match st.menu.run(ui, &self.inp, &rows, Rect::new(80.0, SH + 80.0, 416.0, 130.0), 56.0, 26.0, Style::Wood, true) {
                    MenuEv::Confirm(0) => next = Some(self.do_sleep()),
                    MenuEv::Confirm(_) | MenuEv::Cancel => {
                        st.page = HomePage::Main;
                        st.menu = ListMenu::new();
                    }
                    MenuEv::None => {}
                }
            }
        }
        next
    }

    pub fn do_sleep(&mut self) -> Scr {
        let before_day = self.g.day;
        let rep = self.g.sleep();
        self.ui.audio.sfx("sleep");
        if !rep.portals.is_empty() {
            self.ui.audio.sfx("portal");
        }
        crate::assets::save_text(crate::assets::save_name(self.debug), &self.g.save_text());
        let mut lines = vec![
            "ぐっすり ねむった。HPとMPが ぜんかいふくした!".to_string(),
            format!("夜のあいだに {} ひきの魔物が部屋にすみついた。", rep.spawned),
            if rep.roamers > 0 { format!("通路にも {} ひきの魔物が うろついている。", rep.roamers) } else { "通路は静かだ。".to_string() },
        ];
        lines.push(format!("ダンジョンの評価: {}   シャベルMP: {}", rep.rating, self.g.mp_max));
        for f in &rep.portals {
            lines.push(format!("地下{}階に「魔穴」が開いた! ボスがやってくる…", f));
            lines.push("(赤い点線の壁がきえて、外周が掘れるようになった)".to_string());
        }
        let _ = before_day;
        // ボスを倒した次の朝: 村長から 次のボスの知らせ(手紙)。3日めの朝は メイミの場面が先
        let mei = self.g.party[2].joined;
        let mut next = Scr::Morning(lines);
        for &f in rep.news.iter().rev() {
            next = Scr::Talk(Box::new(crate::talk::TalkSt::new("scene_home", crate::script::next_boss_scene(f, mei), next).speaker(crate::script::LETTER_FROM)));
        }
        if rep.joined.contains(&2) {
            next = Scr::Talk(Box::new(crate::talk::TalkSt::new("scene_home", crate::script::mei_join_scene(), next)));
        }
        next
    }
}

fn gain_text(r: &Recipe) -> String {
    let labels = ["HP", "MP", "つよさ", "すばやさ", "じょうぶさ", "かしこさ"];
    r.gain.iter().enumerate().filter(|(_, v)| **v != 0).map(|(i, v)| format!("{}+{}", labels[i], v)).collect::<Vec<_>>().join(" ")
}
