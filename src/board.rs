//! クエストボード: メインクエストと、すべてのサブクエストをここにまとめる。
//! 受ける・届けるは街の「クエスト」からだけ。ダンジョンのメニューからは見るだけ。

use crate::data::*;
use crate::quest::main_quest;
use crate::town::comma;
use crate::ui::*;
use crate::App;
use macroquad::prelude::*;

pub struct BoardSt {
    pub menu: ListMenu,
    pub msg: String,
    /// 0=すすめる(受ける・報告) 1=達成ずみ
    pub tab: u8,
}

impl BoardSt {
    pub fn new() -> BoardSt {
        let mut menu = ListMenu::new();
        menu.no_page = true;
        BoardSt { menu, msg: String::new(), tab: 0 }
    }
}

/// 表示順: メインクエストが先
fn ordered() -> Vec<&'static Quest> {
    let mut v: Vec<&Quest> = QUESTS.iter().collect();
    v.sort_by_key(|q| !q.main);
    v
}

impl App {
    /// true を返したら閉じる
    pub fn board(&mut self, st: &mut BoardSt, interactive: bool) -> bool {
        // タブの切りかえ(← → / Q E / タブをクリック)
        let tab_rects = [Rect::new(14.0, SH + 4.0, 176.0, 32.0), Rect::new(198.0, SH + 4.0, 176.0, 32.0)];
        let mut want_tab = st.tab;
        if self.inp.left || self.inp.right || self.inp.l || self.inp.r {
            want_tab = 1 - st.tab;
        }
        for (i, r) in tab_rects.iter().enumerate() {
            if self.inp.clicked(*r) {
                want_tab = i as u8;
            }
        }
        if want_tab != st.tab {
            st.tab = want_tab;
            st.menu = ListMenu::new();
            st.menu.no_page = true;
            self.ui.audio.sfx("move");
        }
        // 今すすんでいる階までの、頼まれたクエスト(メインは 降りたとき自動で)だけ出す。今後 会話イベントで解放するクエストを入れたときは、条件が未達の物を ？？？ で出す
        let reached = self.g.reached;
        let all_visible: Vec<&Quest> = ordered().into_iter().filter(|q| q.min_floor <= reached && (q.main || self.g.quest_state(q.id) >= 1)).collect();
        let n_open = all_visible.iter().filter(|q| self.g.quest_state(q.id) != 2).count();
        let n_done = all_visible.len() - n_open;
        let qs: Vec<&Quest> = all_visible.into_iter().filter(|q| (self.g.quest_state(q.id) == 2) == (st.tab == 1)).collect();
        let mq = main_quest(&self.g);
        let ui = &self.ui;
        let g = &self.g;

        // ---- 上画面: メインクエストと、選んだクエストの詳細 ----
        parchment(8.0, 8.0, LW - 16.0, SH - 16.0);
        ui.text("クエストボード", 28.0, 20.0, 28.0, INK);
        chip(ui, LW - 200.0, 20.0, 176.0, 30.0, &format!("{} G", comma(g.gold)), 19.0);
        draw_rectangle(24.0, 58.0, LW - 48.0, 178.0, rgba(255, 244, 200, 255));
        draw_rectangle_lines(24.0, 58.0, LW - 48.0, 178.0, 2.5, rgb(190, 40, 30));
        ui.text(&format!("★ メインクエスト  {} / {}", mq.idx + 1, mq.total), 36.0, 63.0, 16.0, rgb(190, 40, 30));
        ui.text(&mq.title, 36.0, 82.0, 22.0, INK);
        let lines = ui.text_wrap(&mq.hint, 36.0, 112.0, LW - 80.0, 15.0, rgb(70, 50, 30), 19.0);
        // 項目が多い階(評価・倒した数・部屋3種 など)は 2列にして枠に収める
        let two_col = mq.progress.len() > 4;
        for (i, (name, a, b)) in mq.progress.iter().enumerate() {
            let (col, row) = if two_col { (i % 2, i / 2) } else { (0, i) };
            let x = 36.0 + col as f32 * 244.0;
            let y = 112.0 + lines as f32 * 19.0 + 4.0 + row as f32 * if two_col { 26.0 } else { 21.0 };
            let ok = a >= b;
            ui.text(&format!("{}  {} / {}", name, a, b), x, y, 15.0, if ok { rgb(40, 120, 40) } else { INK });
            if two_col {
                bar(x, y + 18.0, 200.0, *a as f32 / (*b).max(1) as f32, if ok { rgb(90, 190, 80) } else { rgb(210, 150, 60) });
            } else {
                bar(300.0, y + 7.0, 200.0, *a as f32 / (*b).max(1) as f32, if ok { rgb(90, 190, 80) } else { rgb(210, 150, 60) });
            }
        }

        if qs.is_empty() {
            ui.text(if st.tab == 1 { "まだ 達成したクエストは ありません" } else { "いま受けられるクエストは ありません" }, 36.0, 262.0, 18.0, INK);
        }
        if !qs.is_empty() {
        let cur = st.menu.cursor.min(qs.len().saturating_sub(1));
        let q = qs[cur];
        let avail = g.quest_available(q);
        let state = g.quest_state(q.id);
        ui.text(if avail { q.title } else { "？？？" }, 36.0, 244.0, 20.0, INK);
        if avail {
            let giver = match q.giver {
                Giver::Mayor => "村長",
                Giver::Magic => "魔法屋のおじさん",
                Giver::Armor => "武具屋のおねえさん",
            };
            ui.text_right(&format!("依頼者: {}{}", giver, if q.main { "  (メイン)" } else { "" }), LW - 36.0, 248.0, 15.0, rgb(120, 80, 40));
            ui.text_wrap(&q.text.replace('\n', ""), 36.0, 272.0, LW - 80.0, 16.0, INK, 20.0);
            let have = g.count(q.need);
            ui.text(&format!("ひつよう: {} x{}  (もっている {})", q.need, q.count, have), 36.0, 318.0, 16.0, if have >= q.count { rgb(40, 120, 40) } else { INK });
            let mut reward = vec![];
            if q.reward_gold > 0 {
                reward.push(format!("{}G", q.reward_gold));
            }
            if let Some(it) = q.reward_item {
                reward.push(it.to_string());
            }
            if q.prepay > 0 {
                reward.push(format!("(前払い {}G)", q.prepay));
            }
            ui.text(&format!("ほうしゅう: {}{}", if reward.is_empty() { "なし".to_string() } else { reward.join(" + ") }, if state == 2 { "   ★ かんりょう" } else { "" }), 36.0, 340.0, 16.0, INK);
        } else {
            ui.text_wrap(&format!("地下{}階に降りると 解放される。", q.min_floor), 36.0, 272.0, LW - 80.0, 16.0, INK, 20.0);
        }
        }
        if !st.msg.is_empty() {
            msgbox(24.0, 366.0, LW - 48.0, 52.0);
            ui.text_wrap(&st.msg, 36.0, 372.0, LW - 72.0, 16.0, CREAM, 20.0);
        }

        // ---- 下画面: 一覧 ----
        panel_wood(0.0, SH, LW, SH);
        for (i, r) in tab_rects.iter().enumerate() {
            let on = st.tab as usize == i;
            draw_rectangle(r.x, r.y, r.w, r.h, if on { rgb(250, 232, 170) } else { rgb(96, 62, 36) });
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.5, if on { rgb(190, 40, 30) } else { rgb(52, 30, 14) });
            let label = if i == 0 { format!("すすめる {}", n_open) } else { format!("達成ずみ {}", n_done) };
            ui.text_center(&label, r.x + r.w / 2.0, r.y + 5.0, 20.0, if on { INK } else { CREAM });
        }
        ui.text(if interactive { "Z:うける/報告  ←→:タブ" } else { "見るだけ  ←→:タブ" }, 390.0, SH + 12.0, 16.0, CREAM);
        let rows: Vec<Row> = qs
            .iter()
            .map(|q| {
                let avail = g.quest_available(q);
                let state = g.quest_state(q.id);
                let have = g.count(q.need);
                let mark = if q.main { "★ " } else { "" };
                if !avail {
                    return Row::new("？？？").right(format!("地下{}階で解放", q.min_floor)).enabled(false);
                }
                let (tag, en) = match state {
                    2 => ("かんりょう", false),
                    1 if have >= q.count => ("報告", true),
                    0 if q.main && have >= q.count => ("報告", true),
                    1 => (if q.main { "あつめる" } else { "あつめる" }, false),
                    _ => ("うける", true),
                };
                Row::new(format!("{}{}", mark, q.title)).right(tag).enabled(en).badge(tag == "報告")
            })
            .collect();
        let ev = st.menu.run(ui, &self.inp, &rows, Rect::new(10.0, SH + 38.0, LW - 20.0, 372.0), 50.0, 21.0, Style::Wood, true);
        match ev {
            MenuEv::Cancel => return true,
            MenuEv::Confirm(i) if interactive => {
                let q = qs[i];
                let state = self.g.quest_state(q.id);
                if state == 0 && !(q.main && self.g.count(q.need) >= q.count) {
                    self.g.accept_quest(q.id);
                    st.msg = format!("「{}」を受けた!{}", q.title, if q.prepay > 0 { format!(" 前払い {}G をもらった。", q.prepay) } else { String::new() });
                } else {
                    match self.g.deliver_quest(q.id) {
                        Ok(m) => {
                            self.ui.audio.sfx("quest");
                            if interactive {
                                // 達成の会話(依頼者とアレンたち)。終わったら 町へ もどる
                                let (bg, who) = crate::script::quest_giver_scene(q);
                                let lines = crate::script::quest_done_lines(q, self.g.party[2].joined, &m);
                                let t = crate::talk::TalkSt::new(bg, lines, crate::Scr::Town(crate::town::TownSt::new(None))).speaker(who).faces();
                                self.scr = crate::Scr::Talk(Box::new(t));
                                return true;
                            }
                            st.msg = m
                        }
                        Err(e) => st.msg = e,
                    }
                }
            }
            _ => {}
        }
        false
    }
}
