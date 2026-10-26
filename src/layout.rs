//! 部屋のならべ方(ボットのテストとデバッグのプリセットで共通)。
//! 3F〜10F は 9x9 のときに作った並べ方(data::comb_layout)を入口に合わせてずらす。
//! 11F 以降は 階の広さ(floor_size)に合わせて、魔穴の条件の部屋数ぶんを作り出す。

use crate::data::*;
use crate::sim::*;

/// 1手: 掘る(room=None)か部屋を置く。stand に立って face の向きで front を掘る/置く
#[derive(Clone)]
pub struct Step {
    pub front: (usize, usize),
    pub room: Option<&'static str>,
    pub stand: (usize, usize),
    pub face: Dir,
}

fn corridor(x: usize, y: usize) -> Step {
    Step { front: (x, y), room: None, stand: (x, y + 1), face: Dir::Up }
}

/// 横に掘る(stand から face の向きへ)
fn dig_side(x: usize, y: usize, face: Dir) -> Step {
    let stand = if face == Dir::Left { (x + 1, y) } else { (x - 1, y) };
    Step { front: (x, y), room: None, stand, face }
}

/// 回廊の (cx, y) に立って 左右に置く。front は 部屋の入口側の1マス(大きな部屋ならその縁)
fn side(kind: &'static str, front_x: usize, cx: usize, y: usize) -> Step {
    Step { front: (front_x, y), room: Some(kind), stand: (cx, y), face: if front_x < cx { Dir::Left } else { Dir::Right } }
}

fn shifted(s: &Step, dx: i32, dy: i32) -> Step {
    let f = |p: (usize, usize)| ((p.0 as i32 + dx) as usize, (p.1 as i32 + dy) as usize);
    Step { front: f(s.front), room: s.room, stand: f(s.stand), face: s.face }
}

/// 3F〜10F: 9x9 の並べ方を、その階の入口にずらしたもの
fn comb_steps(floor: usize) -> Vec<Step> {
    let c = comb_layout(floor);
    let mut v = vec![];
    for y in (5..=11).rev() {
        v.push(corridor(8, y));
        if (5..=10).contains(&y) {
            v.push(side(c.right[10 - y], 9, 8, y));
        }
        for (yc, kind) in c.left {
            if *yc == y {
                v.push(side(kind, 7, 8, y));
            }
        }
    }
    v.push(Step { front: (8, 4), room: Some(c.cap), stand: (8, 5), face: Dir::Up });
    let e = Dungeon::with_size(floor, floor_size(floor)).entrance();
    let (dx, dy) = (e.0 as i32 - ENTRANCE.0 as i32, e.1 as i32 - ENTRANCE.1 as i32);
    v.iter().map(|s| shifted(s, dx, dy)).collect()
}

/// 密度をかせぐための 安い部屋(どの階にも出現表がある)
const FILLERS: [&str; 4] = ["りっぱなベッド", "いろり", "水のみば", "さいだん"];
/// 1x1 の部屋は 最低これだけ置く(評価の目安のため)
const MIN_SINGLES: usize = 14;

/// 11F〜: 横の通路(入口の1つ上)から 縦の通路を何本か伸ばし、その両側に部屋を並べる。
/// [大 大 大][通路][大 大 大] / [大 大 大][通路][1x1] / [1x1][通路][1x1] を左から並べる
fn generated_steps(floor: usize) -> Vec<Step> {
    let s = floor_size(floor);
    let lo = INNER_MIN;
    let hi = INNER_MIN + s - 1;
    let ex = INNER_MIN + s / 2;
    let yt = hi - 1; // 横の通路の行
    let rows = yt - lo; // 縦の通路の左右に部屋を置ける行の数
    let per = rows / 3; // 1列に並べられる大きな部屋の数
    let rule = boss_rule(floor);
    let mut bigs: Vec<&'static str> = vec![];
    let mut singles: Vec<&'static str> = vec![];
    for (k, n) in rule.rooms {
        for _ in 0..*n {
            if room_def(k).w > 1 {
                bigs.push(k);
            } else {
                singles.push(k);
            }
        }
    }
    let mut i = 0;
    while singles.len() < MIN_SINGLES {
        singles.push(FILLERS[i % FILLERS.len()]);
        i += 1;
    }
    // ---- 通路の列を決める ----
    struct Col {
        cx: usize,
        left_bigs: Vec<&'static str>,
        right_bigs: Vec<&'static str>,
        left_singles: Vec<&'static str>,
        right_singles: Vec<&'static str>,
    }
    let mut cols: Vec<Col> = vec![];
    let mut x = lo;
    let mut big_it = bigs.into_iter();
    let mut sing_it = singles.into_iter();
    loop {
        let rest: Vec<&'static str> = big_it.clone().collect();
        if rest.is_empty() {
            break;
        }
        let nl = rest.len().min(per);
        let left: Vec<&'static str> = big_it.by_ref().take(nl).collect();
        if rest.len() > per {
            let right: Vec<&'static str> = big_it.by_ref().take(per).collect();
            cols.push(Col { cx: x + 3, left_bigs: left, right_bigs: right, left_singles: vec![], right_singles: vec![] });
            x += 7;
        } else {
            let rs: Vec<&'static str> = sing_it.by_ref().take(rows).collect();
            cols.push(Col { cx: x + 3, left_bigs: left, right_bigs: vec![], left_singles: vec![], right_singles: rs });
            x += 5;
        }
    }
    loop {
        let rest: Vec<&'static str> = sing_it.clone().collect();
        if rest.is_empty() {
            break;
        }
        // 左右が同じ行数になるように 交互に
        let mut l = vec![];
        let mut r = vec![];
        for (k, kind) in sing_it.by_ref().take(rows * 2).enumerate() {
            if k % 2 == 0 {
                l.push(kind);
            } else {
                r.push(kind);
            }
        }
        cols.push(Col { cx: x + 1, left_bigs: vec![], right_bigs: vec![], left_singles: l, right_singles: r });
        x += 3;
    }
    assert!(x <= hi + 1, "{}F: 部屋が広さ {} に収まらない", floor, s);
    // ---- 手順にする ----
    let mut v = vec![];
    v.push(corridor(ex, yt)); // 入口の1つ上
    let mut order: Vec<usize> = (0..cols.len()).collect();
    order.sort_by_key(|&k| (cols[k].cx as i32 - ex as i32).abs());
    let (mut tl, mut tr) = (ex, ex); // 横の通路が 掘れている範囲
    for k in order {
        let c = &cols[k];
        while c.cx < tl {
            tl -= 1;
            v.push(dig_side(tl, yt, Dir::Left));
        }
        while c.cx > tr {
            tr += 1;
            v.push(dig_side(tr, yt, Dir::Right));
        }
        let (nl, nr) = (c.left_singles.len(), c.right_singles.len());
        let top_rows = nl.max(nr).max(c.left_bigs.len().max(c.right_bigs.len()) * 3);
        let mut li = 0;
        let mut ri = 0;
        for step in 0..top_rows {
            let y = yt - 1 - step;
            v.push(corridor(c.cx, y));
            if li < nl {
                v.push(side(c.left_singles[li], c.cx - 1, c.cx, y));
                li += 1;
            }
            if ri < nr {
                v.push(side(c.right_singles[ri], c.cx + 1, c.cx, y));
                ri += 1;
            }
            // 大きな部屋は 3行ごと(中心の行で置く)
            if step % 3 == 1 {
                let n = step / 3;
                if let Some(kind) = c.left_bigs.get(n) {
                    v.push(side(kind, c.cx - 1, c.cx, y));
                }
                if let Some(kind) = c.right_bigs.get(n) {
                    v.push(side(kind, c.cx + 1, c.cx, y));
                }
            }
        }
    }
    v
}

/// その階の 部屋のならべ方(3F〜最上階)
pub fn plan_steps(floor: usize) -> Vec<Step> {
    if floor <= 10 {
        comb_steps(floor)
    } else {
        generated_steps(floor)
    }
}
