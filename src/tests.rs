//! ルールのテストと、数十日ぶんを自動で遊ぶ bot(1F → 2F ボス撃破まで)。

use crate::data::*;
use crate::sim::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

// ---------------- bot の部品 ----------------

use crate::layout::Step;

fn corridor(x: usize, y: usize) -> Step {
    Step { front: (x, y), room: None, stand: (x, y + 1), face: Dir::Up }
}

fn side(kind: &'static str, x: usize, y: usize) -> Step {
    // 回廊(x=8)の(8,y)に立って左右に置く
    Step { front: (x, y), room: Some(kind), stand: (8, y), face: if x < 8 { Dir::Left } else { Dir::Right } }
}

fn cap(kind: &'static str, x: usize, y: usize) -> Step {
    Step { front: (x, y), room: Some(kind), stand: (x, y + 1), face: Dir::Up }
}

fn plan(floor: usize) -> Vec<Step> {
    let mut v = vec![];
    if floor >= 3 {
        return crate::layout::plan_steps(floor);
    }
    if floor == 1 {
        let kinds = ["わらのベッド", "ゴミすてば", "エサおけ", "わらのベッド", "ゴミすてば", "エサおけ", "てつのたからばこ"];
        let mut k = 0;
        for y in (5..=11).rev() {
            v.push(corridor(8, y));
            if y == 9 || y == 7 || y == 5 {
                v.push(side(kinds[k % 7], 7, y));
                k += 1;
                v.push(side(kinds[k % 7], 9, y));
                k += 1;
            }
        }
        v.push(cap("わらのベッド", 8, 4));
    } else {
        for y in (6..=11).rev() {
            v.push(corridor(8, y));
            match y {
                10 => v.push(side("さかば", 9, 10)),
                9 => v.push(side("はたけ", 7, 9)),
                8 => v.push(side("さかば", 9, 8)),
                7 => {
                    v.push(side("さかば", 9, 7));
                    v.push(side("エサおけ", 7, 7));
                }
                6 => {
                    v.push(side("さかば", 9, 6));
                    v.push(side("わらのベッド", 7, 6));
                }
                _ => {}
            }
        }
        v.push(corridor(8, 5));
        v.push(cap("さいだん", 8, 4));
    }
    v
}

fn bfs(g: &Game, from: (usize, usize), goal: (usize, usize)) -> Option<Vec<Dir>> {
    let mut prev: BTreeMap<(usize, usize), ((usize, usize), Dir)> = BTreeMap::new();
    let mut q = VecDeque::new();
    q.push_back(from);
    let mut seen = BTreeSet::new();
    seen.insert(from);
    while let Some(p) = q.pop_front() {
        if p == goal {
            let mut path = vec![];
            let mut c = p;
            while c != from {
                let (pp, d) = prev[&c];
                path.push(d);
                c = pp;
            }
            path.reverse();
            return Some(path);
        }
        for d in Dir::ALL {
            if let Some(n) = pos_add(p, d) {
                if g.dungeon.walkable(n.0, n.1) && !seen.contains(&n) {
                    seen.insert(n);
                    prev.insert(n, (p, d));
                    q.push_back(n);
                }
            }
        }
    }
    None
}

pub(super) fn auto_battle(g: &mut Game, srcs: &[Source]) -> Outcome {
    let mut b = Battle::new(g, srcs);
    let mut log: Vec<String> = vec![];
    for _ in 0..800 {
        match b.advance(g) {
            Advance::End(o) => {
                if o == Outcome::Lost && std::env::var("BOTDBG").is_ok() && g.floor_no() == 20 {
                    println!("  LOSS log (last 30): {:?}", &log[log.len().saturating_sub(30)..]);
                }
                return o;
            }
            Advance::Msgs(v) => log.extend(v),
            Advance::NeedCmd(i) => {
                // ゲームの「おまかせ」と同じ考え方(回復・つよくなれ・なおす・全体魔法・単体魔法・こうげき)
                let cmd = b.auto_cmd(g, i);
                let m = b.command(g, i, cmd);
                log.extend(m);
            }
        }
    }
    panic!("battle did not end");
}

fn walk_to(g: &mut Game, goal: (usize, usize)) -> bool {
    let Some(path) = bfs(g, g.pos, goal) else { return false };
    for d in path {
        match g.step(d) {
            StepResult::Battle(srcs) => {
                let o = auto_battle(g, &srcs);
                if o != Outcome::Won {
                    if o == Outcome::Lost {
                        g.wipe();
                    }
                    return false;
                }
            }
            StepResult::Descend => {
                g.descend();
                return true;
            }
            _ => {}
        }
    }
    true
}

/// 隣の床から (x,y) の壁を1マス掘る
fn dig_tile(g: &mut Game, t: (usize, usize)) -> bool {
    if g.dungeon.tile(t.0, t.1) != Tile::Rock {
        return true;
    }
    for d in Dir::ALL {
        let Some(n) = pos_add(t, d) else { continue };
        if g.dungeon.tile(n.0, n.1) == Tile::Floor && g.dungeon.room_at(n.0, n.1).is_none() && g.dungeon.walkable(n.0, n.1) && bfs(g, g.pos, n).is_some() {
            if g.mp < 1 || !walk_to(g, n) {
                return false;
            }
            g.facing = d.opposite();
            return g.dig().is_ok();
        }
    }
    false
}

fn build(g: &mut Game, plan: &[Step]) {
    for s in plan {
        let exists = match s.room {
            None => g.dungeon.tile(s.front.0, s.front.1) == Tile::Floor,
            Some(_) => g.dungeon.room_at(s.front.0, s.front.1).is_some(),
        };
        if exists {
            continue;
        }
        if let Some(kind) = s.room {
            let def = room_def(kind);
            if def.w > 1 {
                // 大きな部屋は 3x3 ぜんぶ掘ってからでないと置けない
                let c = pos_add(s.front, s.face).unwrap();
                let mut tiles: Vec<(usize, usize)> = (0..def.h).flat_map(|dy| (0..def.w).map(move |dx| (c.0 - 1 + dx, c.1 - 1 + dy))).collect();
                tiles.sort_by_key(|t| (t.0 as i32 - s.stand.0 as i32).abs() + (t.1 as i32 - s.stand.1 as i32).abs());
                for t in tiles {
                    if !dig_tile(g, t) {
                        return;
                    }
                }
            }
        }
        if g.mp < 1 || !walk_to(g, s.stand) {
            return;
        }
        g.facing = s.face;
        if let Some(kind) = s.room {
            let def = room_def(kind);
            if def.w == 1 && g.dungeon.tile(s.front.0, s.front.1) == Tile::Rock && g.dig().is_err() {
                return;
            }
            if g.count(kind) < 1 && g.buy(kind).is_err() {
                return;
            }
            if g.place_room(kind).is_err() {
                return;
            }
        } else if g.dig().is_err() {
            return;
        }
    }
}

fn hurt_frac(g: &Game) -> i32 {
    g.members().iter().map(|&i| g.party[i].hp * 100 / g.party[i].hp_max).min().unwrap()
}

/// 武具屋でいちばん強い物を、お金の余裕があれば買って装備する
fn buy_gear(g: &mut Game) {
    for who in [0usize, 1, 2] {
        for slot in 0..6 {
            if !g.party[who].can_wear(slot) {
                continue;
            }
            let score = |d: &ItemDef| d.power * 10 + if d.ring == 4 { 5 } else { 0 };
            let cur = g.party[who].equip[slot].as_deref().and_then(item).map(score).unwrap_or(-1);
            let best = ITEMS
                .iter()
                .filter(|d| d.kind.slot() == Some(slot) && g.armor_stock.contains(d.name) && d.ring != 4)
                .filter(|d| score(d) > cur && g.gold >= d.buy + 2500)
                .max_by_key(|d| score(d));
            if let Some(d) = best {
                if g.buy(d.name).is_ok() {
                    let _ = g.equip(who, d.name);
                }
            }
        }
    }
}

/// かいふくやくで、ボスに挑む前に体力を戻す
fn heal_party(g: &mut Game) {
    for _ in 0..80 {
        let low = g.members().into_iter().filter(|&i| g.party[i].alive()).min_by_key(|&i| g.party[i].hp * 100 / g.party[i].hp_max);
        match low {
            Some(i) if g.party[i].hp * 100 / g.party[i].hp_max < 90 && g.take_item("かいふくやく", 1) => {
                g.party[i].hp = (g.party[i].hp + 30).min(g.party[i].hp_max);
            }
            _ => break,
        }
    }
}

fn kill_roamers(g: &mut Game) {
    let roam: Vec<(usize, usize)> = g.dungeon.roamers.iter().map(|r| (r.x, r.y)).collect();
    for p in roam {
        if hurt_frac(g) < 40 {
            break;
        }
        if g.dungeon.roamer_at(p.0, p.1) && !walk_to(g, p) {
            break;
        }
    }
    g.return_to_entrance();
}

fn hunt(g: &mut Game) {
    let mut spots: Vec<(usize, usize)> = vec![];
    for r in &g.dungeon.rooms {
        for (k, m) in r.mons.iter().enumerate() {
            if m.is_none() {
                continue;
            }
            if r.open4 {
                spots.push((r.x + k % r.w, r.y + k / r.w));
            } else if let Some(s) = r.stand() {
                spots.push(s);
            }
        }
    }
    for s in spots {
        if hurt_frac(g) < 40 {
            break;
        }
        if !walk_to(g, s) {
            break;
        }
    }
    let roam: Vec<(usize, usize)> = g.dungeon.roamers.iter().map(|r| (r.x, r.y)).collect();
    for p in roam {
        if hurt_frac(g) < 40 {
            break;
        }
        if g.dungeon.roamer_at(p.0, p.1) && !walk_to(g, p) {
            break;
        }
    }
    g.return_to_entrance();
}

/// 魔穴までの掘り進み: MPの許す限り掘り、届けば挑戦する
fn boss_attempt(g: &mut Game) -> Option<Outcome> {
    let goal = g.dungeon.portal?;
    if g.dungeon.boss_defeated {
        return None;
    }
    let start = g.dungeon.entrance();
    let mut prev: BTreeMap<(usize, usize), (usize, usize)> = BTreeMap::new();
    let mut q = VecDeque::new();
    q.push_back(start);
    let mut seen = BTreeSet::new();
    seen.insert(start);
    while let Some(p) = q.pop_front() {
        if p == goal {
            break;
        }
        for d in Dir::ALL {
            if let Some(n) = pos_add(p, d) {
                if seen.contains(&n) {
                    continue;
                }
                let t = g.dungeon.tile(n.0, n.1);
                let free = match g.dungeon.room_at(n.0, n.1) {
                    None => true,
                    Some(i) => g.dungeon.rooms[i].open4,
                };
                let ok = free && (t == Tile::Floor || t == Tile::Portal || (t == Tile::Rock && g.diggable(n.0, n.1)));
                if ok {
                    seen.insert(n);
                    prev.insert(n, p);
                    q.push_back(n);
                }
            }
        }
    }
    let mut path = vec![goal];
    let mut c = goal;
    while c != start {
        c = *prev.get(&c)?;
        path.push(c);
    }
    path.reverse();
    for w in path.windows(2) {
        let (a, b) = (w[0], w[1]);
        if g.dungeon.tile(b.0, b.1) == Tile::Rock {
            if g.mp < 1 || !walk_to(g, a) {
                return None;
            }
            let d = Dir::ALL.into_iter().find(|d| pos_add(a, *d) == Some(b)).unwrap();
            g.facing = d;
            g.dig().ok()?;
        }
    }
    let last = path[path.len() - 2];
    if hurt_frac(g) < 90 {
        return None;
    }
    if !walk_to(g, last) {
        return None;
    }
    let d = Dir::ALL.into_iter().find(|d| pos_add(last, *d) == Some(goal)).unwrap();
    match g.step(d) {
        StepResult::Battle(srcs) => {
            let o = auto_battle(g, &srcs);
            if o == Outcome::Lost {
                g.wipe();
            }
            Some(o)
        }
        _ => None,
    }
}

fn town(g: &mut Game) {
    // 換金アイテム(毛皮・コインなど)は、町の「まとめて売る」と同じ売り方で全部売る
    g.sell_loot();
    g.auto_mimic(1);
    for who in [0usize, 1, 2] {
        for (n, _) in g.inv.clone() {
            if let Some(d) = item(&n) {
                if d.kind.is_equip() {
                    let slot = d.kind.slot().unwrap();
                    let cur = g.party[who].equip[slot].as_deref().and_then(item).map(|c| c.power).unwrap_or(-1);
                    if d.power > cur || (d.ring == 4 && who == 0) {
                        let _ = g.equip(who, &n);
                    }
                }
            }
        }
    }
    for q in QUESTS {
        if g.quest_state(q.id) == 0 {
            g.accept_quest(q.id);
        }
        if g.quest_state(q.id) == 1 && g.count(q.need) >= q.count {
            g.deliver_quest(q.id).ok();
        }
    }
    g.visited_magic = true;
    g.veg_stock.insert("カツオブシ".to_string());
    for n in g.teachable() {
        if g.count(n) > 1 || n != "ホウレンソウ" {
            let _ = g.show_ingredient(n);
        }
    }
    // 使わない装備は売る(売ると武具屋に並ぶ)。そのあと 足りないところを買い足す
    for (n, c) in g.inv.clone() {
        if item(&n).map(|d| d.kind.is_equip()).unwrap_or(false) {
            for _ in 0..c {
                let _ = g.sell(&n);
            }
        }
    }
    // はじめから: 1日目は 武器なし。まず ナイフを 買って そうびする
    if g.party[0].equip[0].is_none() && g.gold >= 150 {
        if g.buy("ナイフ").is_ok() {
            let _ = g.equip(0, "ナイフ");
        }
    }
    buy_gear(g);
    let want = if g.floor_no() >= 6 { 24 } else if g.floor_no() >= 3 { 6 } else { 3 };
    while g.count("かいふくやく") < want && g.gold > 400 {
        g.buy("かいふくやく").unwrap();
    }
    for sp in SPELLS {
        if sp.price > 0 && g.spell_for_sale(sp) && g.gold > sp.price * 2 + 500 {
            g.buy_spell(sp.id).unwrap();
        }
    }
    // ジェリーが あたまをマネした魔法・メイミのつよさに合わせて ぜんいんかいふくも ほしい

    for who in g.members() {
        if g.count("ホウレンソウ") < 2 && g.gold > 20 {
            g.buy("ホウレンソウ").ok();
            g.buy("ホウレンソウ").ok();
        }
        let mut best: Option<(&Recipe, i32)> = None;
        for r in RECIPES {
            if g.can_cook(r) {
                let score: i32 = r.gain.iter().sum();
                if best.map(|b| score > b.1).unwrap_or(true) {
                    best = Some((r, score));
                }
            }
        }
        if let Some((r, _)) = best {
            g.cook(r.name, who).ok();
        }
    }
    if g.count("魔法のオリーブオイル") > 0 {
        g.shovel_polish().ok();
    }
}

/// 1Fのボスを倒していたら階段で2Fへ降りる
fn go_down_if_ready(g: &mut Game) {
    if g.dungeon.floor < MAX_FLOOR && g.dungeon.boss_defeated {
        if let Some(p) = g.dungeon.portal {
            walk_to(g, p);
        }
    }
}

// ---------------- ルールのテスト ----------------

#[test]
fn dig_and_place_rules() {
    let mut g = Game::new(1);
    g.facing = Dir::Up;
    assert!(g.dig().is_ok());
    assert_eq!(g.mp, START_MP - 1);
    let t = g.front().unwrap();
    assert_eq!(g.dungeon.tile(t.0, t.1), Tile::Floor);
    assert!(g.dig().is_ok(), "A で埋まる");
    assert_eq!(g.dungeon.tile(t.0, t.1), Tile::Rock);
    assert!(g.undo().is_ok());
    assert_eq!(g.dungeon.tile(t.0, t.1), Tile::Floor);
    g.pos = (4, 4);
    g.facing = Dir::Left;
    assert!(g.dig().is_err(), "外周は掘れない");
    g.pos = ENTRANCE;
    g.facing = Dir::Up;
    assert!(g.place_room("わらのベッド").is_ok());
    assert_eq!(g.dungeon.rooms[0].door, Dir::Down);
    assert_eq!(g.count("わらのベッド"), 0);
    g.mp = 0;
    assert!(g.remove_room().is_err());
    g.pos = ENTRANCE;
    g.facing = Dir::Down;
    assert!(g.dig().is_err(), "入口の真下は掘れない");
}

#[test]
fn stairs_no_spawn() {
    let mut g = Game::new(2);
    let mk = |x, y, kind: &str, door| Room { x, y, w: 1, h: 1, front: (x, y), kind: kind.into(), door, open4: false, mons: vec![None], chest_open: false, refill: 0 };
    g.dungeon.rooms.push(mk(8, 11, "わらのベッド", Dir::Down));
    g.dungeon.set(8, 11, Tile::Floor);
    g.dungeon.rooms.push(mk(8, 9, "エサおけ", Dir::Down));
    g.dungeon.set(8, 9, Tile::Floor);
    assert!(g.no_spawn_room(&g.dungeon.rooms[0].clone()));
    g.sleep();
    assert!(g.dungeon.rooms[0].mons[0].is_none(), "階段のすぐそばには住みつかない");
    assert!(g.dungeon.rooms[1].mons[0].is_some());
}

#[test]
fn big_room_and_open_rooms() {
    let mut g = Game::new(3);
    g.dungeon.hidden.clear();
    g.mp = 50;
    g.add_item("はたけ", 1);
    g.add_item("さかば", 1);
    for y in (8..=11).rev() {
        g.dungeon.set(8, y, Tile::Floor);
    }
    // 8,10 に立って 左を向く: はたけは (7,10) が入口側の中央、(5..7, 9..11) の3x3
    g.pos = (8, 10);
    g.facing = Dir::Left;
    // 3x3 がまだ掘られていなければ置けない(MPも持ち物も減らない)
    assert!(!g.can_place("はたけ"));
    let why = g.place_room("はたけ").unwrap_err();
    assert!(why.contains("掘ってから"), "{}", why);
    assert_eq!((g.mp, g.count("はたけ")), (50, 1));
    // 1マスでも掘っていない所があると置けない
    for y in 9..=11 {
        for x in 5..=7 {
            g.dungeon.set(x, y, Tile::Floor);
        }
    }
    g.dungeon.set(5, 11, Tile::Rock);
    assert!(!g.can_place("はたけ"));
    g.dungeon.set(5, 11, Tile::Floor);
    // ぜんぶ掘りずみなら置ける
    assert!(g.can_place("はたけ"));
    g.place_room("はたけ").unwrap();
    assert_eq!(g.dungeon.rooms[0].tiles().len(), 9);
    assert_eq!(g.mp, 47);
    assert!(g.dungeon.walkable(6, 10), "はたけは通り抜けられる");
    // すでに部屋(物)があるマスをふくむ場所には置けない(となりの列も掘りずみにして、原因を「部屋がある」だけにする)
    for x in 5..=7 {
        g.dungeon.set(x, 8, Tile::Floor);
    }
    g.add_item("はたけ", 1);
    g.pos = (8, 9);
    g.facing = Dir::Left;
    let why = g.place_room("はたけ").unwrap_err();
    assert!(why.contains("置いてあって") || why.contains("置けない"), "{}", why);
    assert_eq!(g.dungeon.rooms.len(), 1);
    // ひとつ戻すと部屋だけ消えて、掘った床は残る
    assert!(g.undo().is_ok());
    assert!(g.dungeon.rooms.is_empty());
    assert_eq!(g.dungeon.tile(6, 10), Tile::Floor, "掘った床は残る");
    assert_eq!(g.count("はたけ"), 2);
    // さかばは 隣に立つと魔物が出てくる
    g.pos = (8, 9);
    g.facing = Dir::Left;
    g.dungeon.set(7, 9, Tile::Floor);
    g.dungeon.rooms.clear();
    g.place_room("さかば").unwrap();
    g.dungeon.rooms[0].mons[0] = Some("gobl".into());
    g.pos = (8, 10);
    match g.step(Dir::Up) {
        StepResult::Battle(s) => assert_eq!(s, vec![Source::Room(0, 0)]),
        o => panic!("{:?}", o),
    }
}

#[test]
fn undo_only_reverts_its_own_action() {
    let mut g = Game::new(4);
    g.add_item("ちからのゆびわ", 1);
    g.add_item("かいふくやく", 2);
    g.facing = Dir::Up;
    g.dig().unwrap();
    // 指輪を装備してから undo しても、指輪は増えない
    g.equip(0, "ちからのゆびわ").unwrap();
    g.undo().unwrap();
    assert_eq!(g.count("ちからのゆびわ"), 0);
    assert_eq!(g.party[0].equip[5].as_deref(), Some("ちからのゆびわ"));
    // 薬を使ってから undo しても、薬は戻らない
    g.dig().unwrap();
    g.take_item("かいふくやく", 1);
    g.undo().unwrap();
    assert_eq!(g.count("かいふくやく"), 1);
    // 宝箱を開けると undo は消える
    g.add_item("てつのカギ", 1);
    g.dig().unwrap();
    g.dungeon.rooms.push(Room { x: 8, y: 9, w: 1, h: 1, front: (8, 9), kind: "てつのたからばこ".into(), door: Dir::Down, open4: false, mons: vec![None], chest_open: false, refill: 0 });
    g.dungeon.set(8, 9, Tile::Floor);
    g.pos = (8, 11);
    g.dungeon.set(8, 10, Tile::Floor);
    let before = g.count("てつのカギ");
    if let StepResult::Chest(_) = g.step(Dir::Up) {
        assert_eq!(g.count("てつのカギ"), before - 1);
        assert!(g.undo().is_err(), "宝箱のあとは もどせない");
    } else {
        panic!("chest");
    }
}

#[test]
fn undo_cleared_after_battle() {
    let mut g = Game::new(6);
    g.facing = Dir::Up;
    g.dig().unwrap();
    assert!(!g.undo.is_empty());
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    assert_eq!(auto_battle(&mut g, &[Source::Roamer(0)]), Outcome::Won);
    assert!(g.undo().is_err(), "戦闘のあとは もどせない");
}

#[test]
fn poison_ticks_once_per_turn() {
    let mut g = Game::new(5);
    // いちばん素早いメイミが先頭
    g.party[2].poison = true;
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    let hp0 = g.party[2].hp;
    let mut got_cmd = false;
    for _ in 0..6 {
        if let Advance::NeedCmd(i) = b.advance(&mut g) {
            assert_eq!(i, 2);
            got_cmd = true;
            break;
        }
    }
    assert!(got_cmd, "毒でも行動できる");
    assert!(g.party[2].hp < hp0 && hp0 - g.party[2].hp <= 3, "毒は1ターンに1回だけ");
}

#[test]
fn bed_monsters_start_asleep_and_tavern_drunk() {
    let mut g = Game::new(7);
    let mk = |kind: &str, open4| Room { x: 8, y: 9, w: 1, h: 1, front: (8, 9), kind: kind.into(), door: Dir::Down, open4, mons: vec![Some("cat".into())], chest_open: false, refill: 0 };
    g.dungeon.rooms.push(mk("わらのベッド", false));
    let b = Battle::new(&g, &[Source::Room(0, 0)]);
    assert_eq!(b.enemies[0].sleep, 2);
    g.dungeon.rooms[0] = mk("さかば", true);
    let b = Battle::new(&g, &[Source::Room(0, 0)]);
    assert!(b.enemies[0].drunk);
}

#[test]
fn holy_hurts_ghosts() {
    let mut g = Game::new(8);
    g.party[0].spells.push("holy".into());
    g.dungeon.roamers.push(Roamer { x: 8, y: 10, mon: "ghost".into() });
    g.dungeon.roamers.push(Roamer { x: 8, y: 9, mon: "slime".into() });
    let mut b = Battle::new(&g, &[Source::Roamer(0), Source::Roamer(1)]);
    let (h0, h1) = (b.enemies[0].hp, b.enemies[1].hp);
    b.command(&mut g, 0, Cmd::Spell("holy", 0));
    b.command(&mut g, 0, Cmd::Spell("holy", 1));
    assert!(h0 - b.enemies[0].hp > h1 - b.enemies[1].hp, "アンデットに大ダメージ");
}

#[test]
fn floors_keep_state_and_spawn_at_night() {
    let mut g = Game::new(9);
    g.dungeon.boss_defeated = true;
    g.dungeon.set(15, 8, Tile::StairsDown);
    g.dungeon.portal = Some((15, 8));
    g.add_item("エサおけ", 1);
    g.dungeon.set(8, 10, Tile::Floor);
    g.dungeon.set(8, 9, Tile::Floor);
    g.pos = (8, 10);
    g.facing = Dir::Up;
    g.place_room("エサおけ").unwrap();
    g.pos = (15, 8);
    assert!(matches!(g.check_encounter(), StepResult::Descend));
    g.descend();
    assert_eq!(g.floor_no(), 2);
    assert_eq!(g.reached, 2);
    g.add_item("エサおけ", 1);
    g.dungeon.set(8, 11, Tile::Floor);
    g.dungeon.set(8, 10, Tile::Floor);
    g.dungeon.set(8, 9, Tile::Floor);
    g.pos = (8, 10);
    g.facing = Dir::Up;
    g.place_room("エサおけ").unwrap();
    assert!(g.ascend());
    assert_eq!(g.floor_no(), 1);
    assert_eq!(g.pos, (15, 8));
    assert_eq!(g.dungeon.rooms.len(), 1);
    g.sleep();
    assert_eq!(g.floor_no(), 1, "寝ても居る階は変わらない");
    // 1Fにいても 2Fの部屋に魔物がすみつく
    let f2 = g.floors[1].as_ref().unwrap();
    assert!(f2.rooms[0].mons[0].is_some());
    assert!(g.dungeon.rooms[0].mons[0].is_some());
}

#[test]
fn everyone_eats_once_a_day() {
    let mut g = Game::new(8);
    g.add_item("ホウレンソウ", 5);
    assert!(g.cook("ホウレンソウサラダ", 0).is_ok());
    assert!(g.cook("ホウレンソウサラダ", 0).is_err(), "同じ人は1日1回");
    assert!(g.cook("ホウレンソウサラダ", 1).is_ok());
    assert!(g.cook("ホウレンソウサラダ", 2).is_ok());
    g.sleep();
    assert!(g.cook("ホウレンソウサラダ", 0).is_ok(), "翌日は また食べられる");
}

#[test]
fn save_roundtrip_and_version() {
    let mut g = Game::new(3);
    g.descend_for_test();
    let s = g.save_text();
    let g2 = Game::load_text(&s).expect("load");
    assert_eq!(g2.gold, g.gold);
    assert_eq!(g2.floor_no(), 2);
    assert_eq!(g2.floors.len(), MAX_FLOOR);
    let old = s.replace(&format!("\"version\":{}", SAVE_VERSION), "\"version\":1");
    assert!(Game::load_text(&old).is_none(), "古いセーブは読まない");
}

impl Game {
    fn descend_for_test(&mut self) {
        self.dungeon.boss_defeated = true;
        self.change_floor(2);
    }
}

// ---------------- bot: 1F → 2F ----------------

#[test]
fn many_days_loop() {
    let mut all_days: Vec<Vec<u32>> = vec![];
    for seed in 1..=3u64 {
        // 物語どおりの はじまり: アレンひとり → 3日めにメイミ → 1Fのボスのあとにジェリー
        let mut g = Game::start_story(seed);
        let mut boss_day: Vec<Option<u32>> = vec![None; MAX_FLOOR + 1];
        let mut step = 0;
        let mut done = false;
        // 負けたら しばらく鍛えてから(食事でステータスを上げ、装備を整えて)また挑む
        let mut next_try = 0u32;
        for _ in 0..700 {
            town(&mut g);
            // 全滅したら町に戻る。ワープで いちばん深い階へ
            let r = g.reached;
            g.warp(r);
            go_down_if_ready(&mut g);
            let f = g.floor_no();
            // 通路をうろつく魔物が、これから置く部屋の場所に居ると置けないので、先に倒す
            kill_roamers(&mut g);
            build(&mut g, &plan(f));
            if g.dungeon.portal.is_some() && !g.dungeon.boss_defeated && g.day >= next_try {
                if std::env::var("BOTDBG").is_ok() && g.day % 20 == 0 {
                    println!("  dbg day {} floor {} hurt {} mp {}/{} party {:?}", g.day, f, hurt_frac(&g), g.mp, g.mp_max, g.party.iter().map(|m| (m.hp, m.hp_max, m.atk(), m.def())).collect::<Vec<_>>());
                }
                heal_party(&mut g);
                if let Some(o) = boss_attempt(&mut g) {
                    println!("    boss fight floor {} -> {:?}, lowest member hp after fight {}% day {} {:?}", f, o, hurt_frac(&g), g.day, g.boss_progress());
                    if o == Outcome::Won {
                        boss_day[f] = Some(g.day);
                    } else {
                        next_try = g.day + 8;
                    }
                }
            }
            hunt(&mut g);
            g.sleep();
            for fl in 1..=MAX_FLOOR {
                if boss_day[fl].is_none() && g.floor_dungeon(fl).map(|d| d.boss_defeated).unwrap_or(false) {
                    boss_day[fl] = Some(g.day);
                }
            }
            let s = crate::quest::main_quest(&g).idx;
            assert!(s >= step, "メインクエストが後戻りした: {} -> {}", step, s);
            step = s;
            if boss_day[MAX_FLOOR].is_some() {
                town(&mut g);
                let s = crate::quest::main_quest(&g).idx;
                assert!(s >= step);
                assert_eq!(s, crate::quest::MAIN_TOTAL - 1, "最後の段階まで進む");
                done = true;
                break;
            }
        }
        println!("seed {} boss days {:?} wipes {} gold {} mp {} party {:?} floor {} day {}", seed, &boss_day[1..], g.total_wipes, g.gold, g.mp_max,
            g.party.iter().map(|m| (m.hp_max, m.atk(), m.def())).collect::<Vec<_>>(), g.floor_no(), g.day);
        if !done {
            println!("  stuck: progress {:?} rooms {:?} portal {:?} mp {}/{}", g.boss_progress(), g.dungeon.rooms.iter().map(|r| r.kind.as_str()).collect::<Vec<_>>(), g.dungeon.portal, g.mp, g.mp_max);
        }
        assert!(done, "最上階のボスまで倒せるはず: seed {} (floor {} day {})", seed, g.floor_no(), g.day);
        all_days.push(boss_day[1..].iter().map(|d| d.unwrap()).collect());
    }
    println!("boss days {:?}", all_days);
    for d in &all_days {
        assert!((6..=40).contains(&d[0]), "1Fボスの日数が範囲外: {}", d[0]);
        assert!(d.windows(2).all(|w| w[0] < w[1]), "階が進むほど日数が増える: {:?}", d);
    }
}

// ---------------- 1F/2F の通し(画面で見にくい流れ) ----------------

#[test]
fn hidden_items_and_quests() {
    let mut g = Game::new(11);
    g.mp = 50;
    // 隠しアイテム(内周左上)
    g.dungeon.set(5, 6, Tile::Floor);
    g.pos = (5, 6);
    g.facing = Dir::Up;
    g.dig().unwrap();
    assert_eq!(g.count("どうのゆびわ"), 1);
    assert!(g.undo.is_empty(), "見つけたら もどせない");
    // 1Fのクエスト4つ
    for (id, need) in [("snake", "ヘビのはっぱ"), ("crow", "カラスのももにく"), ("slime", "スライムのひもの"), ("halter", "ワーネコホルター")] {
        let gold0 = g.gold;
        g.accept_quest(id);
        assert_eq!(g.quest_state(id), 1);
        assert!(g.deliver_quest(id).is_err(), "持っていないと渡せない");
        g.add_item(need, 1);
        g.deliver_quest(id).unwrap();
        assert_eq!(g.quest_state(id), 2);
        assert!(g.gold != gold0 || id == "slime" || id == "halter");
    }
    assert_eq!(g.count("魔法のオリーブオイル"), 1);
    assert_eq!(g.count("ひかりのゆびわ"), 1);
    // すいみんは翌日から
    let s = spell("sleep");
    assert!(!g.spell_for_sale(s));
    g.sleep();
    assert!(g.spell_for_sale(s));
    // オイルでMPが増える
    g.shovel_polish().unwrap();
    assert_eq!(g.mp_max, START_MP + OIL_MP_GAIN);
    // 2Fのクエストは 降りるまで受けられない
    g.accept_quest("tart");
    assert_eq!(g.quest_state("tart"), 0);
    g.reached = 2;
    g.accept_quest("tart");
    assert_eq!(g.quest_state("tart"), 1);
}

#[test]
fn wipe_and_flee() {
    let mut g = Game::new(12);
    g.gold = 1000;
    g.change_floor(2);
    g.wipe();
    assert_eq!(g.floor_no(), 1, "全滅したら1階の入口から");
    assert_eq!(g.gold, 900);
    assert!(g.party.iter().all(|m| m.hp >= 1));
    // 逃げる
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    let mut fled = false;
    for _ in 0..40 {
        match b.advance(&mut g) {
            Advance::NeedCmd(i) => {
                b.command(&mut g, i, Cmd::Flee);
            }
            Advance::End(o) => {
                fled = o == Outcome::Fled;
                break;
            }
            _ => {}
        }
    }
    assert!(fled);
    assert_eq!(g.dungeon.roamers.len(), 1, "逃げたら魔物は残る");
}

#[test]
fn boss_gates_per_floor() {
    let mut g = Game::new(13);
    g.preset(1); // 1F 今夜開く直前
    assert!(g.dungeon.portal.is_none());
    assert!(g.boss_ready());
    let rep = g.sleep();
    assert_eq!(rep.portals, vec![1]);
    // 2Fは さかば4・評価40・30体が必要
    g.preset(3);
    assert_eq!(g.floor_no(), 2);
    assert!(!g.boss_ready());
    let prog = g.boss_progress();
    assert_eq!(prog.len(), 3);
}

#[test]
fn big_room_does_not_swallow_hidden_items() {
    let mut g = Game::new(21);
    g.change_floor(2);
    g.mp = 20;
    g.add_item("はたけ", 1);
    // (4,4) の隠しアイテムを3x3がふくむ位置。掘っていないので置けず、隠しアイテムも残る
    g.dungeon.set(5, 7, Tile::Floor);
    g.pos = (5, 7);
    g.facing = Dir::Up;
    assert!(g.place_room("はたけ").is_err());
    assert_eq!(g.dungeon.hidden.len(), 1);
    // 掘れば(隠しアイテムは掘ったときに手に入る)置ける
    let gold0 = g.gold;
    for y in 4..=6 {
        for x in 4..=6 {
            if (x, y) == (4, 4) {
                g.pos = (4, 5);
                g.dungeon.set(4, 5, Tile::Floor);
                g.facing = Dir::Up;
                g.dig().unwrap();
            } else {
                g.dungeon.set(x, y, Tile::Floor);
            }
        }
    }
    assert_eq!(g.gold, gold0 + 4000, "掘ると 隠しアイテム(4000G)が手に入る");
    g.pos = (5, 7);
    g.facing = Dir::Up;
    assert!(g.place_room("はたけ").is_ok());
}

#[test]
fn lion_claw_hits_everyone_and_staggers() {
    let mut g = Game::new(31);
    g.change_floor(2);
    g.dungeon.portal = Some((15, 8));
    g.dungeon.set(15, 8, Tile::Portal);
    for m in g.party.iter_mut() {
        m.hp_max = 999;
        m.hp = 999;
        m.con = 20;
    }
    let mut b = Battle::new(&g, &[Source::Boss]);
    let mut saw_claw = false;
    let mut saw_stagger = false;
    for _ in 0..400 {
        match b.advance(&mut g) {
            Advance::Msgs(v) => {
                saw_claw |= v.iter().any(|m| m.contains("フライングクロウ"));
                saw_stagger |= v.iter().any(|m| m.contains("ふらついて うごけない"));
            }
            Advance::NeedCmd(i) => {
                b.command(&mut g, i, Cmd::Defend);
            }
            Advance::End(_) => break,
        }
        if saw_claw && saw_stagger {
            break;
        }
    }
    assert!(saw_claw && saw_stagger);
}

// ---------------- メインクエスト・おまかせ ----------------

#[test]
fn main_quest_follows_state() {
    use crate::quest::main_quest;
    let g = Game::new(40);
    assert_eq!(main_quest(&g).idx, 0, "最初は部屋を置く");
    let mut g = Game::new(41);
    g.preset(1);
    assert!(main_quest(&g).idx >= 3, "魔穴の直前プリセット");
    assert_eq!(main_quest(&g).idx, 3, "魔穴はまだ");
    g.sleep();
    assert_eq!(main_quest(&g).idx, 4, "魔穴が開いたらボスへ");
    g.preset(2);
    assert_eq!(main_quest(&g).idx, 4);
    g.preset(3);
    assert_eq!(main_quest(&g).idx, 8, "2F開始は 2Fを育てる段階");
    g.preset(4);
    assert_eq!(main_quest(&g).idx, 9, "2Fボス直前");
    // ボスを倒して たてがみを届ける
    g.dungeon.boss_defeated = true;
    assert_eq!(main_quest(&g).idx, 10);
    g.add_item("リオンのたてがみ", 1);
    g.deliver_quest("gobl").unwrap();
    assert_eq!(main_quest(&g).idx, 11);
}

#[test]
fn main_quests_are_auto_accepted() {
    let mut g = Game::new(42);
    // ヘビリンゴたいじは受けなくても届けられる(はじめから受けた状態)
    assert_eq!(g.quest_state("snake"), 1);
    g.add_item("ヘビのはっぱ", 1);
    g.deliver_quest("snake").unwrap();
    assert_eq!(g.count("魔法のオリーブオイル"), 1);
    assert_eq!(g.quest_state("gobl"), 0);
    g.change_floor(2);
    assert_eq!(g.quest_state("gobl"), 1, "2Fに降りたらゴブリンたいじも受けた状態");
}

#[test]
fn auto_battle_command_heals_and_attacks() {
    let mut g = Game::new(43);
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    g.dungeon.roamers.push(Roamer { x: 8, y: 10, mon: "ghost".into() });
    let b = Battle::new(&g, &[Source::Roamer(0), Source::Roamer(1)]);
    // 元気なら メイミは炎、アレンは弱い方(HPの低い敵)を殴る
    assert!(matches!(b.auto_cmd(&g, 2), Cmd::Spell("fire", _)));
    assert!(matches!(b.auto_cmd(&g, 0), Cmd::Attack(_)));
    // 瀕死なら かいふくやく
    g.add_item("かいふくやく", 1);
    g.party[1].hp = 3;
    assert!(matches!(b.auto_cmd(&g, 0), Cmd::Item(_, 1)));
    // 食事のおまかせ
    let mut g = Game::new(44);
    g.add_item("ホウレンソウ", 5);
    let msgs = g.auto_meals();
    assert_eq!(msgs.len(), 3);
    assert!(g.ate.iter().all(|a| *a));
    assert_eq!(g.auto_meals(), vec!["みんな もう食べた".to_string()]);
}

#[test]
fn best_equip_and_bulk_sell() {
    let mut g = Game::new(45);
    g.add_item("ワーワンソード", 1);
    g.add_item("かわよろい", 1);
    g.add_item("バックラー", 1);
    let n = g.best_equip(0);
    assert!(n >= 3);
    assert_eq!(g.party[0].equip[0].as_deref(), Some("ワーワンソード"));
    assert_eq!(g.best_equip(0), 0);
    assert_eq!(g.best_equip(1), 0, "ジェリーは装備できない");
    g.add_item("イノシシのけがわ", 2);
    g.add_item("シルバーコイン", 1);
    let gold0 = g.gold;
    let (n, gold) = g.sell_loot();
    assert_eq!(n, 3);
    assert_eq!(gold, 160 * 2 + 500);
    assert_eq!(g.gold, gold0 + gold);
}

#[test]
fn news_marks_for_stock_and_ingredients() {
    let mut g = Game::new(50);
    for k in ["armor", "magic", "furn", "plaza"] {
        assert!(!g.has_news(k), "はじめは新入荷なし: {}", k);
    }
    assert_eq!(g.reportable_quests(), 0);
    // 2Fに降りると家具屋と魔法屋に新入荷
    g.change_floor(2);
    assert!(g.has_news("furn"));
    assert!(g.shop_news("magic").iter().any(|n| n == "ふぶきバクダン"));
    assert!(g.shop_news("magic").iter().any(|n| n == "魔法:せいなる光"));
    g.mark_seen("magic", false);
    assert!(g.has_news("magic"), "魔法はまだ見ていない");
    g.mark_seen("magic", true);
    assert!(!g.has_news("magic"));
    g.mark_seen("furn", false);
    assert!(!g.has_news("furn"));
    // 新しい野菜は広場、新しい食材は魔法屋で赤い!
    g.add_item("ジャガイモ", 1);
    assert!(g.has_news("plaza"));
    assert!(g.has_news("magic"), "ジャガイモは新しい料理を教えてもらえる");
    g.show_ingredient("ジャガイモ").unwrap();
    assert!(!g.has_news("magic"));
    // 装備を売ると武具屋に並ぶ
    g.add_item("バックラー", 1);
    g.sell("バックラー").unwrap();
    assert!(g.has_news("armor"));
    // 報告できるクエスト
    g.add_item("カラスのももにく", 1);
    g.accept_quest("crow");
    assert_eq!(g.reportable_quests(), 1);
}

// ---------------- 3F〜5F ----------------

#[test]
fn data_tables_are_consistent() {
    // すべての魔物・ドロップ・料理・クエスト・宝箱・隠しアイテムの名前が、ちゃんと定義されている
    for m in MONS {
        for (name, _) in m.drops {
            assert!(item(name).is_some(), "ドロップ {} が ITEMS に無い({})", name, m.name);
        }
    }
    for r in RECIPES {
        for n in r.needs {
            assert!(item(n).is_some(), "料理 {} の材料 {} が無い", r.name, n);
        }
    }
    for q in QUESTS {
        assert!(item(q.need).is_some(), "クエスト {} の品 {} が無い", q.id, q.need);
        if let Some(it) = q.reward_item {
            assert!(item(it).is_some(), "報酬 {} が無い", it);
        }
        if let Some(sp) = q.reward_spell {
            assert!(SPELLS.iter().any(|s| s.id == sp));
        }
    }
    for f in 1..=MAX_FLOOR {
        for n in iron_chest(f) {
            assert!(item(n).is_some(), "{}F の宝箱 {} が無い", f, n);
        }
        for (_, _, name) in hidden_items(f) {
            assert!(name.starts_with('G') || item(&name).is_some(), "{}F の隠し {} が無い", f, name);
        }
        let _ = mon(boss_id(f));
        for id in corridor_spawn(f) {
            let _ = mon(id);
        }
        // その階で置ける部屋には、その階の出現表がある
        let mut g = Game::new(1);
        g.reached = f;
        for kind in g.furniture_stock() {
            if chest_tier(kind).is_some() || kind == MIASMA_ROOM {
                continue;
            }
            assert!(!room_spawns(f, kind).is_empty(), "{}F の {} に出現表が無い", f, kind);
            for id in room_spawns(f, kind) {
                let _ = mon(id);
            }
            let _ = room_def(kind);
        }
    }
    for m in FLOOR_MAIN_QUEST {
        assert!(quest(m).main);
    }
}

#[test]
fn dojo_one_door_big_room() {
    let mut g = Game::new(60);
    g.change_floor(5);
    g.mp = 50;
    g.add_item("どうじょう", 1);
    for y in 8..=11 {
        g.dungeon.set(8, y, Tile::Floor);
    }
    for y in 8..=10 {
        for x in 5..=7 {
            g.dungeon.set(x, y, Tile::Floor);
        }
    }
    g.pos = (8, 9);
    g.facing = Dir::Left;
    g.place_room("どうじょう").unwrap();
    let r = g.dungeon.rooms[0].clone();
    assert!(!r.open4 && r.w == 3 && r.h == 3);
    assert_eq!(r.stand(), Some((8, 9)), "入口の前のマスは 回廊側");
    assert!(!g.dungeon.walkable(6, 9), "1方向の部屋の中は歩けない");
    // 夜: 4匹まですみつく(階段から遠いので)
    g.sleep();
    let n = g.dungeon.rooms[0].alive();
    assert!((1..=4).contains(&n), "どうじょうの魔物の数 {}", n);
    // 入口の前に立つと、中の魔物が最大3匹出てくる
    g.pos = (8, 10);
    match g.step(Dir::Up) {
        StepResult::Battle(s) => {
            assert!(!s.is_empty() && s.len() <= 3);
            assert!(s.iter().all(|x| matches!(x, Source::Room(0, _))));
        }
        o => panic!("{:?}", o),
    }
}

fn boss_skill_seen(floor: usize, needle: &str) -> bool {
    let mut g = Game::new(70 + floor as u64);
    g.change_floor(floor);
    g.dungeon.portal = Some((15, 8));
    g.dungeon.set(15, 8, Tile::Portal);
    for m in g.party.iter_mut() {
        m.hp_max = 9999;
        m.hp = 9999;
        m.con = 30;
    }
    let mut b = Battle::new(&g, &[Source::Boss]);
    for _ in 0..800 {
        match b.advance(&mut g) {
            Advance::Msgs(v) => {
                if v.iter().any(|m| m.contains(needle)) {
                    return true;
                }
            }
            Advance::NeedCmd(i) => {
                b.command(&mut g, i, Cmd::Defend);
            }
            Advance::End(_) => return false,
        }
    }
    false
}

#[test]
fn new_bosses_use_their_skills() {
    assert!(boss_skill_seen(3, "ようがんこうげき"));
    assert!(boss_skill_seen(3, "のしかかり"));
    assert!(boss_skill_seen(4, "こおりのつえ"));
    assert!(boss_skill_seen(5, "きょうれつなひかり"));
    assert!(boss_skill_seen(5, "きょうれつないちげき"));
    // 6F〜10F
    assert!(boss_skill_seen(6, "ほのおのつぼ"));
    assert!(boss_skill_seen(7, "うずしお"));
    assert!(boss_skill_seen(7, "しめつけ"));
    assert!(boss_skill_seen(8, "フライングクロウ"));
    assert!(boss_skill_seen(9, "ドラゴンブレス"));
    assert!(boss_skill_seen(9, "しっぽなぎはらい"));
    for skill in ["ヘルファイア", "ブリザード", "ダークホーリー", "あくまのたたき"] {
        assert!(boss_skill_seen(10, skill), "{}", skill);
    }
    // 11F〜20F
    for (floor, skill) in [
        (11, "ドロのなみ"), (11, "ぬまにひきずりこむ"), (12, "ほえたけり"), (13, "どくのきり"), (13, "いのちいただき"), (14, "ぶんまわし"), (14, "ギリギリこうせん"),
        (15, "ほのおのはなびら"), (15, "つるのなぎはらい"), (16, "ひかりのかげ"), (16, "かげのいちげき"), (17, "ふういんビーム"), (17, "ひっさつのいちげき"),
        (18, "じごくの炎"), (18, "のしかかり"), (19, "じごくのひかり"), (19, "めいおうひっさつけん"), (19, "いのちいただき"), (20, "どくリンゴのしる"), (20, "ほのおのいき"), (20, "とぐろのしめつけ"),
    ] {
        assert!(boss_skill_seen(floor, skill), "{}F {}", floor, skill);
    }
}

#[test]
fn layouts_are_valid_on_every_floor() {
    for f in 3..=MAX_FLOOR {
        let d = Dungeon::new(f);
        let (lo, hi) = d.inner();
        let ent = d.entrance();
        let steps = crate::layout::plan_steps(f);
        let mut road: BTreeSet<(usize, usize)> = BTreeSet::new();
        road.insert(ent);
        let mut used: BTreeMap<(usize, usize), &str> = BTreeMap::new();
        let mut have: BTreeMap<&str, usize> = BTreeMap::new();
        for st in &steps {
            match st.room {
                None => {
                    assert!(!used.contains_key(&st.front), "{}F: 通路が部屋と重なる {:?}", f, st.front);
                    road.insert(st.front);
                }
                Some(kind) => {
                    let def = room_def(kind);
                    let tiles: Vec<(usize, usize)> = if def.w == 1 {
                        vec![st.front]
                    } else {
                        let c = pos_add(st.front, st.face).unwrap();
                        (0..def.h).flat_map(|dy| (0..def.w).map(move |dx| (c.0 - 1 + dx, c.1 - 1 + dy))).collect()
                    };
                    for t in tiles {
                        assert!((lo..=hi).contains(&t.0) && (lo..=hi).contains(&t.1), "{}F: {} が内周の外 {:?}", f, kind, t);
                        assert!(used.insert(t, kind).is_none(), "{}F: {} が重なる {:?}", f, kind, t);
                    }
                    *have.entry(kind).or_default() += 1;
                }
            }
        }
        for t in used.keys() {
            assert!(!road.contains(t), "{}F: 部屋の場所が通路になっている {:?}", f, t);
        }
        // 入口から通路をたどって 全部の部屋の入口に着ける(1つの扉の大部屋は 扉が通路に向く)
        let mut seen: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut q = vec![ent];
        while let Some(p) = q.pop() {
            if !seen.insert(p) {
                continue;
            }
            for dir in Dir::ALL {
                if let Some(n) = pos_add(p, dir) {
                    if road.contains(&n) && !seen.contains(&n) {
                        q.push(n);
                    }
                }
            }
        }
        for st in &steps {
            assert!(seen.contains(&st.stand), "{}F: {:?} に通路から着けない", f, st.stand);
            assert_eq!(pos_add(st.stand, st.face), Some(st.front), "{}F: 立つ場所と向きが合わない", f);
        }
        for (kind, n) in boss_rule(f).rooms {
            assert!(have.get(kind).copied().unwrap_or(0) >= *n, "{}F: {} が {} 個ない", f, kind, n);
            assert!(!room_spawns(f, kind).is_empty(), "{}F の {} に出現表が無い", f, kind);
        }
    }
}

#[test]
fn a_floor_without_roamers_still_sleeps_fine() {
    // 15F の通路には何も出ない(wiki)。夜の処理で落ちない
    assert!(corridor_spawn(15).is_empty());
    let mut g = Game::new(140);
    g.reached = 15;
    g.change_floor(15);
    let e = g.dungeon.entrance();
    g.dungeon.set(e.0, e.1 - 1, Tile::Floor);
    g.dungeon.set(e.0, e.1 - 2, Tile::Floor);
    for _ in 0..3 {
        g.sleep();
    }
    assert!(g.dungeon.roamers.is_empty());
}

/// 戦闘で アレン(0)が item を使う。使ったときのメッセージと、その直後の全員のHPを返す
fn first_round_with_item(g: &mut Game, srcs: &[Source], item: &str) -> (Vec<String>, Vec<i32>) {
    let mut b = Battle::new(g, srcs);
    for _ in 0..200 {
        match b.advance(g) {
            Advance::Msgs(_) => {}
            Advance::NeedCmd(i) => {
                if i == 0 {
                    let msgs = b.command(g, i, Cmd::Item(item.to_string(), 0));
                    return (msgs, g.party.iter().map(|m| m.hp).collect());
                }
                b.command(g, i, Cmd::Defend);
            }
            Advance::End(_) => break,
        }
    }
    panic!("アレンの番が来なかった");
}

#[test]
fn banno_potion_heals_everyone_and_itadaki_bomb_hits_all_enemies() {
    let mut g = Game::new(130);
    for m in g.party.iter_mut() {
        m.hp_max = 300;
        m.hp = 10;
    }
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    g.add_item("ばんのうやく", 1);
    let (_, hp) = first_round_with_item(&mut g, &[Source::Roamer(0)], "ばんのうやく");
    assert_eq!(g.count("ばんのうやく"), 0);
    assert_eq!(hp.len(), 3);
    assert!(hp.iter().all(|h| *h >= 60), "全員のHPが戻る: {:?}", hp);

    let mut g = Game::new(131);
    for k in 0..3 {
        g.dungeon.roamers.push(Roamer { x: 8, y: 11 - k, mon: "slime".into() });
    }
    g.add_item("いただきバクダン", 1);
    let (msgs, _) = first_round_with_item(&mut g, &[Source::Roamer(0), Source::Roamer(1), Source::Roamer(2)], "いただきバクダン");
    assert_eq!(g.count("いただきバクダン"), 0);
    let hits = msgs.iter().filter(|m| m.starts_with("スライム に") && m.contains("のダメージ")).count();
    assert_eq!(hits, 3, "敵3体ぜんいんに当たる: {:?}", msgs);
}

#[test]
fn elemental_resist_reduces_damage() {
    let mut g = Game::new(80);
    assert_eq!(g.party[0].resist(0), 0);
    g.add_item("アツイローブ", 1);
    g.add_item("炎のティアラ", 1);
    g.add_item("ルビーのゆびわ", 1);
    for n in ["アツイローブ", "炎のティアラ", "ルビーのゆびわ"] {
        g.equip(0, n).unwrap();
    }
    assert_eq!(g.party[0].resist(0), 70);
    assert_eq!(g.party[0].resist(1), 0);
    assert_eq!(g.party[0].resist(3), 0, "物理は耐性なし");
    g.add_item("ひかりのゆびわ", 1);
    g.equip(0, "ひかりのゆびわ").unwrap();
    assert_eq!(g.party[0].resist(2), 50);
}

#[test]
fn quest_counts_and_spell_rewards() {
    let mut g = Game::new(81);
    g.reached = 5;
    g.accept_quest("crowtail");
    assert_eq!(g.gold, START_GOLD + 700, "前払い");
    g.add_item("カラスネコのしっぽ", 1);
    assert!(g.deliver_quest("crowtail").is_err(), "2本いる");
    assert_eq!(g.reportable_quests(), 0);
    g.add_item("カラスネコのしっぽ", 1);
    assert_eq!(g.reportable_quests(), 1);
    let m = g.deliver_quest("crowtail").unwrap();
    assert!(m.contains("ぜんいんかいふく"));
    assert!(g.party[0].spells.iter().any(|s| s == "healall"));
    assert_eq!(g.count("カラスネコのしっぽ"), 0);
    // ぜんいんかいふくは戦闘で全員を回復する
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    for m in g.party.iter_mut() {
        m.hp = 1;
    }
    g.party[0].mp = 20;
    let msgs = b.command(&mut g, 0, Cmd::Spell("healall", 0));
    assert!(msgs.len() >= 4);
    assert!(g.party.iter().all(|m| m.hp > 10));
}

#[test]
fn old_save_with_two_floors_still_loads() {
    let mut g = Game::new(82);
    g.floors.truncate(2); // 2Fまでだった頃のセーブ
    let s = g.save_text();
    let g2 = Game::load_text(&s).expect("load");
    assert_eq!(g2.floors.len(), MAX_FLOOR);
    let mut g2 = g2;
    g2.change_floor(4); // 3F以降へも行ける
    assert_eq!(g2.floor_no(), 4);
}

#[test]
fn main_quest_presets_3f_to_top() {
    use crate::quest::main_quest;
    // 階 f の「開始」は 12 + 4*(f-3) 段階め、「ボス直前」はその次
    for f in 3..=MAX_FLOOR {
        for ready in [false, true] {
            let preset = 5 + (f - 3) * 2 + ready as usize;
            let mut g = Game::new(90);
            g.preset(preset);
            assert_eq!(g.floor_no(), f, "preset {}", preset);
            let idx = 12 + 4 * (f - 3) + ready as usize;
            assert_eq!(main_quest(&g).idx, idx, "preset {} → {}", preset, main_quest(&g).title);
        }
    }
    // 最上階のボスを倒して届けると最後の段階
    let mut g = Game::new(91);
    g.preset(5 + (MAX_FLOOR - 3) * 2 + 1);
    g.dungeon.boss_defeated = true;
    assert_eq!(main_quest(&g).idx, crate::quest::MAIN_TOTAL - 2);
    let q = quest(FLOOR_MAIN_QUEST[MAX_FLOOR - 1]);
    g.add_item(q.need, q.count);
    g.deliver_quest(q.id).unwrap();
    assert_eq!(main_quest(&g).idx, crate::quest::MAIN_TOTAL - 1);
}

// ---------------- 宝箱・鍵 ----------------

fn chest_room(kind: &str, open: bool) -> Room {
    Room { x: 8, y: 9, w: 1, h: 1, front: (8, 9), kind: kind.into(), door: Dir::Down, open4: false, mons: vec![None], chest_open: open, refill: if open { CHESTS[chest_tier(kind).unwrap()].refill_days } else { 0 } }
}

fn put_roamers(g: &mut Game, n: usize) {
    g.dungeon.roamers.clear();
    // 魔物は 入口から歩ける床にだけ いられる。入口に近い順に床にして、n 匹ならべる
    let (lo, hi) = g.dungeon.inner();
    let e = g.dungeon.entrance();
    let mut tiles: Vec<(usize, usize)> = (lo..=hi).flat_map(|y| (lo..=hi).map(move |x| (x, y))).filter(|&p| p != e && g.dungeon.room_at(p.0, p.1).is_none()).collect();
    tiles.sort_by_key(|&(x, y)| ((x as i32 - e.0 as i32).abs() + (y as i32 - e.1 as i32).abs(), x, y));
    for &(x, y) in tiles.iter().take(n + 8) {
        g.dungeon.set(x, y, Tile::Floor);
    }
    for &(x, y) in tiles.iter().take(n) {
        g.dungeon.roamers.push(Roamer { x, y, mon: "slime".into() });
    }
}

/// 何泊で宝箱が補充されるか(魔物が alive 匹いる状態で、最大 limit 泊まで)
fn nights_until_refill(kind: &str, alive: usize, limit: u32) -> Option<u32> {
    let mut g = Game::new(100);
    g.dungeon.set(8, 9, Tile::Floor);
    g.dungeon.rooms.push(chest_room(kind, true));
    for n in 1..=limit {
        put_roamers(&mut g, alive);
        g.sleep();
        if !g.dungeon.rooms[0].chest_open {
            return Some(n);
        }
    }
    None
}

#[test]
fn chests_refill_after_2_4_7_days_only_with_enough_monsters() {
    // 鉄: 2日(魔物3匹以上)。足りない夜は 日数が進まない
    assert_eq!(nights_until_refill("てつのたからばこ", 3, 10), Some(2));
    assert_eq!(nights_until_refill("てつのたからばこ", 2, 10), None);
    // 銀: 4日(6匹以上)
    assert_eq!(nights_until_refill("ぎんのたからばこ", 6, 10), Some(4));
    assert_eq!(nights_until_refill("ぎんのたからばこ", 5, 10), None);
    // 金: 最大7日(9匹以上)。魔物が18匹以上だと 2倍の早さ
    assert_eq!(nights_until_refill("おうごんチェスト", 9, 10), Some(7));
    assert_eq!(nights_until_refill("おうごんチェスト", 18, 10), Some(4));
    assert_eq!(nights_until_refill("おうごんチェスト", 8, 10), None);
    // 魔物が足りない夜をはさんでも、たまった日数は消えない
    let mut g = Game::new(101);
    g.dungeon.set(8, 9, Tile::Floor);
    g.dungeon.rooms.push(chest_room("てつのたからばこ", true));
    put_roamers(&mut g, 3);
    g.sleep();
    put_roamers(&mut g, 0);
    g.sleep();
    assert!(g.dungeon.rooms[0].chest_open, "足りない夜は進まない");
    put_roamers(&mut g, 3);
    g.sleep();
    assert!(!g.dungeon.rooms[0].chest_open);
}

fn battle_got(g: &mut Game, srcs: &[Source]) -> Vec<String> {
    let mut b = Battle::new(g, srcs);
    for _ in 0..800 {
        match b.advance(g) {
            Advance::End(_) => return b.got.clone(),
            Advance::Msgs(_) => {}
            Advance::NeedCmd(i) => {
                let t = b.alive_enemies()[0];
                b.command(g, i, Cmd::Attack(t));
            }
        }
    }
    panic!("battle did not end");
}

fn cleared_marker(got: &[String]) -> bool {
    got.iter().any(|s| s.contains("かりつくした"))
}

#[test]
fn clearing_the_floor_guarantees_a_key_when_a_chest_is_filled() {
    for seed in 1..=12u64 {
        // 宝箱に中身がある + 最後の1匹を倒した → 鍵が出る
        let mut g = Game::new(seed);
        g.dungeon.set(8, 9, Tile::Floor);
        g.dungeon.rooms.push(chest_room("てつのたからばこ", false));
        g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
        let got = battle_got(&mut g, &[Source::Roamer(0)]);
        // (5%の通常ドロップで先に鍵が出ていた場合は、重ねて出さない)
        assert!(cleared_marker(&got) || got.iter().any(|s| s == "てつのカギ"), "seed {}: {:?}", seed, got);
        assert_eq!(g.count("てつのカギ"), 1, "seed {}: {:?}", seed, got);
        // もう鍵を持っていれば(箱は1つ)、重ねては出ない
        let mut g = Game::new(seed);
        g.dungeon.set(8, 9, Tile::Floor);
        g.dungeon.rooms.push(chest_room("てつのたからばこ", false));
        g.add_item("てつのカギ", 1);
        g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
        assert!(!cleared_marker(&battle_got(&mut g, &[Source::Roamer(0)])));
        // 宝箱が空(開けたあと・補充待ち)なら出ない
        let mut g = Game::new(seed);
        g.dungeon.set(8, 9, Tile::Floor);
        g.dungeon.rooms.push(chest_room("てつのたからばこ", true));
        g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
        assert!(!cleared_marker(&battle_got(&mut g, &[Source::Roamer(0)])));
        // 宝箱が無ければ出ない
        let mut g = Game::new(seed);
        g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
        assert!(!cleared_marker(&battle_got(&mut g, &[Source::Roamer(0)])));
        // まだ魔物が残っていれば出ない
        let mut g = Game::new(seed);
        g.dungeon.set(8, 9, Tile::Floor);
        g.dungeon.rooms.push(chest_room("てつのたからばこ", false));
        g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
        g.dungeon.roamers.push(Roamer { x: 8, y: 10, mon: "slime".into() });
        assert!(!cleared_marker(&battle_got(&mut g, &[Source::Roamer(0)])));
    }
}

#[test]
fn silver_and_gold_chests_use_deeper_tables_and_their_own_keys() {
    let mut g = Game::new(110);
    g.dungeon.set(8, 9, Tile::Floor);
    g.dungeon.set(8, 10, Tile::Floor);
    g.dungeon.rooms.push(chest_room("ぎんのたからばこ", false));
    g.pos = (8, 11);
    g.dungeon.set(8, 11, Tile::Floor);
    // 鉄のカギでは銀の宝箱は開かない
    g.add_item("てつのカギ", 1);
    assert!(matches!(g.step(Dir::Up), StepResult::Moved));
    assert!(!g.dungeon.rooms[0].chest_open);
    // 銀のカギで開く。中身は 1階ぶん深い(2F)の表から
    g.add_item("ぎんのカギ", 1);
    g.pos = (8, 11);
    match g.step(Dir::Up) {
        StepResult::Chest(m) => assert!(m.contains("ぎんのカギ"), "{}", m),
        o => panic!("{:?}", o),
    }
    assert_eq!(g.count("ぎんのカギ"), 0);
    assert!(iron_chest(2).iter().any(|n| g.count(n) > 0), "2Fの表のアイテムが出る");
    assert_eq!(g.dungeon.rooms[0].refill, 4);
    // 金は2階ぶん深い
    let mut g = Game::new(111);
    g.dungeon.set(8, 9, Tile::Floor);
    g.dungeon.set(8, 10, Tile::Floor);
    g.dungeon.set(8, 11, Tile::Floor);
    g.dungeon.rooms.push(chest_room("おうごんチェスト", false));
    g.add_item("おうごんのカギ", 1);
    g.pos = (8, 11);
    assert!(matches!(g.step(Dir::Up), StepResult::Chest(_)));
    assert!(iron_chest(3).iter().any(|n| g.count(n) > 0));
    assert_eq!(g.dungeon.rooms[0].refill, 7);
    // 銀・金の宝箱がある階を狩りつくすと、それぞれの鍵が出る
    let mut g = Game::new(112);
    g.dungeon.set(8, 9, Tile::Floor);
    g.dungeon.rooms.push(chest_room("ぎんのたからばこ", false));
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    let got = battle_got(&mut g, &[Source::Roamer(0)]);
    assert!(got.iter().any(|s| s.contains("ぎんのカギ") && s.contains("かりつくした")), "{:?}", got);
}


// ---------------- 階ごとの表がそろっているか ----------------

#[test]
fn floor_tables_are_complete() {
    assert_eq!(FLOOR_MAIN_QUEST.len(), MAX_FLOOR);
    assert_eq!(crate::quest::MAIN_TOTAL, 4 * MAX_FLOOR + 4);
    let mut bosses = BTreeSet::new();
    let mut hidden = BTreeSet::new();
    for f in 1..=MAX_FLOOR {
        assert!(bosses.insert(boss_id(f)), "{}F のボスが他の階と同じ", f);
        assert!(mon(boss_id(f)).boss);
        let q = quest(FLOOR_MAIN_QUEST[f - 1]);
        assert!(q.main && q.min_floor == f, "{}F のメインクエスト", f);
        assert_eq!(q.need, mon(boss_id(f)).drops[0].0, "{}F のクエストの品はボスが必ず落とす物", f);
        for (_, _, name) in hidden_items(f) {
            // オイルとお金は 何階にあってもよい(wiki どおり)
            if name != "魔法のオリーブオイル" && !name.starts_with('G') {
                assert!(hidden.insert(name.clone()), "隠しアイテム {} が別の階と重なっている", name);
            }
        }
        let rule = boss_rule(f);
        for (kind, n) in rule.rooms {
            let _ = room_def(kind);
            assert!(item(kind).is_some());
            assert!(!room_spawns(f, kind).is_empty(), "{}F の {} に出現表が無い", f, kind);
        }
        assert!(!iron_chest(f).is_empty());
    }
    // 家具屋の品はぜんぶ部屋の形を持つ
    for (fl, kind) in FURNITURE_UNLOCK {
        assert!(*fl >= 2);
        let _ = room_def(kind);
        assert_eq!(item(kind).map(|d| d.kind), Some(Kind::Room), "{}", kind);
    }
    // 宝箱の鍵は 銀=6F 金=11F から
    assert_eq!(CHESTS[1].drop_from_floor, 6);
    assert_eq!(CHESTS[2].drop_from_floor, 11);
    assert!(FURNITURE_UNLOCK.contains(&(6, "ぎんのたからばこ")) && FURNITURE_UNLOCK.contains(&(11, "おうごんチェスト")));
}

#[test]
fn every_sprite_has_art() {
    let mut names: Vec<String> = MONS.iter().map(|m| m.sprite.to_string()).collect();
    for r in ROOM_DEFS {
        names.push(format!("room_{}", r.name));
        if chest_tier(r.name).is_some() {
            names.push(format!("room_{}_open", r.name));
        }
    }
    for n in &names {
        assert!(crate::art::declared(n), "{} が art.rs に登録されていない", n);
    }
    for n in crate::art::png_only() {
        let f = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/img").join(format!("{}.png", n));
        assert!(f.exists(), "{} の PNG が無い", n);
    }
}

#[test]
fn deep_chests_skip_food_only_tables_and_use_top_tables_past_the_top_floor() {
    let only_food = |t: &[&str]| t.iter().all(|n| item(n).map(|d| d.kind == Kind::Food).unwrap_or(false));
    // 10F の鉄の宝箱は食べ物だけ。銀・金はその次の階の表を使う
    assert!(only_food(iron_chest(10)) && !only_food(iron_chest(9)));
    for (floor, deeper) in [(8usize, 2usize), (9, 1), (9, 2), (10, 1), (10, 2)] {
        assert!(!only_food(chest_items(floor, deeper)), "{}F deeper {}", floor, deeper);
    }
    assert_eq!(chest_items(9, 1), iron_chest(11));
    // 食べ物の表でも、鉄の宝箱(deeper=0)はそのまま
    assert_eq!(chest_items(10, 0), iron_chest(10));
    assert_eq!(chest_items(5, 1), iron_chest(6));
    // 10F〜の銀・16F〜の金は wiki の表そのまま
    assert_eq!(chest_items(20, 1), SILVER_CHEST[10]);
    assert_eq!(chest_items(19, 2), GOLD_CHEST[3]);
    assert!(chest_items(10, 1).contains(&"かがみのころも"), "10Fの銀の宝箱から かがみのころも");
    assert_eq!(chest_items(20, 0), iron_chest(20));
    // 実際に開けて確かめる(20F の銀)
    let mut g = Game::new(120);
    g.dungeon.floor = 20;
    g.dungeon.set(8, 9, Tile::Floor);
    g.dungeon.set(8, 10, Tile::Floor);
    g.dungeon.set(8, 11, Tile::Floor);
    g.dungeon.rooms.push(chest_room("ぎんのたからばこ", false));
    g.add_item("ぎんのカギ", 1);
    g.pos = (8, 11);
    assert!(matches!(g.step(Dir::Up), StepResult::Chest(_)));
    assert!(SILVER_CHEST[10].iter().any(|n| g.count(n) > 0));
}

#[test]
fn stairs_on_the_top_floor_do_not_descend() {
    let mut g = Game::new(150);
    g.preset(5 + (MAX_FLOOR - 3) * 2 + 1);
    assert_eq!(g.floor_no(), MAX_FLOOR);
    g.dungeon.boss_defeated = true;
    let e = g.dungeon.entrance();
    g.dungeon.set(e.0, e.1 - 1, Tile::StairsDown);
    g.pos = e;
    assert!(matches!(g.step(Dir::Up), StepResult::Stairs));
    assert_eq!(g.floor_no(), MAX_FLOOR);
    g.descend();
    assert_eq!(g.floor_no(), MAX_FLOOR, "最上階の下には降りない");
}

#[test]
fn dungeons_grow_gradually_with_the_floor() {
    assert!((1..=4).all(|f| floor_size(f) == 9), "1F〜4F は 9x9");
    let mut prev = 9;
    for f in 1..=MAX_FLOOR {
        let sz = floor_size(f);
        assert!(sz >= prev && sz - prev <= 2 && sz % 2 == 1, "{}F: {}", f, sz);
        prev = sz;
        let d = Dungeon::new(f);
        assert_eq!(d.w(), sz + 8);
        let (lo, hi) = d.inner();
        let e = d.entrance();
        assert_eq!(e.1, hi);
        assert!((lo..=hi).contains(&e.0));
        assert_eq!(d.tile(e.0, e.1), Tile::Floor);
        // バリアは内周の1つ外、外周は岩盤
        assert_eq!(d.tile(lo - 1, lo), Tile::Barrier);
        assert_eq!(d.tile(hi + 1, hi), Tile::Barrier);
        assert_eq!(d.tile(0, 0), Tile::Bedrock);
        assert_eq!(d.tile(d.w() - 1, d.h() - 1), Tile::Bedrock);
        // 地図の外は 岩盤あつかい
        assert_eq!(d.tile(d.w() + 3, 2), Tile::Bedrock);
        // 魔穴の候補は 地図の縁
        for p in d.portal_spots() {
            assert!(d.in_map(p.0, p.1));
        }
        // 隠しアイテムは 外周(岩盤)の内側。右・下に寄った物は 広がった分だけずれている
        for (x, y, _) in &d.hidden {
            assert!((1..d.w() - 1).contains(x) && (1..d.h() - 1).contains(y), "{}F の隠し ({}, {})", f, x, y);
            assert_eq!(d.tile(*x, *y), Tile::Rock);
        }
    }
    assert_eq!(floor_size(MAX_FLOOR), 21);
}

#[test]
fn old_save_with_a_smaller_floor_is_resized_on_load() {
    // 9x9 で作った 12F を、いまの大きさで読み込む(部屋の位置・魔物が 入口に合わせてずれる)
    let mut g = Game::new(160);
    g.reached = 12;
    g.change_floor(12);
    g.dungeon = Dungeon::with_size(12, 9);
    let e = g.dungeon.entrance();
    g.dungeon.set(e.0, e.1 - 1, Tile::Floor);
    g.dungeon.set(e.0, e.1 - 2, Tile::Floor);
    g.dungeon.rooms.push(Room { x: e.0, y: e.1 - 2, w: 1, h: 1, front: (e.0, e.1 - 2), kind: "ごうかなベッド".into(), door: Dir::Down, open4: false, mons: vec![Some("catqueen".into())], chest_open: false, refill: 0 });
    g.pos = e;
    let text = g.save_text();
    let g2 = Game::load_text(&text).expect("読み込める");
    assert_eq!(g2.dungeon.size, floor_size(12));
    let e2 = g2.dungeon.entrance();
    assert_eq!(g2.pos, e2);
    assert_eq!(g2.dungeon.tile(e2.0, e2.1 - 2), Tile::Floor);
    let r = &g2.dungeon.rooms[0];
    assert_eq!((r.x, r.y), (e2.0, e2.1 - 2));
    assert_eq!(r.mons[0].as_deref(), Some("catqueen"));
    // 1F は 大きさが同じなので そのまま
    let g3 = Game::load_text(&Game::new(1).save_text()).unwrap();
    assert_eq!(g3.dungeon.size, 9);
}

#[test]
fn adjacent_roamers_attack_and_walled_off_holes_stay_empty() {
    let mut g = Game::new(170);
    let e = g.dungeon.entrance();
    for k in 1..=4 {
        g.dungeon.set(e.0, e.1 - k, Tile::Floor);
    }
    // となりに来ると 向かってくる(戦闘になる)
    g.dungeon.roamers.push(Roamer { x: e.0, y: e.1 - 3, mon: "slime".into() });
    assert!(matches!(g.step(Dir::Up), StepResult::Moved), "まだ2マス先");
    assert!(matches!(g.step(Dir::Up), StepResult::Battle(_)), "となりに来たら戦闘");
    // 部屋でふさがれた行き止まりの穴には 魔物が湧かない(いた魔物も消える)
    let mut g = Game::new(171);
    let (lo, _) = g.dungeon.inner();
    g.dungeon.set(lo, lo, Tile::Floor);
    g.dungeon.roamers.push(Roamer { x: lo, y: lo, mon: "bat".into() });
    assert!(!g.dungeon.reachable().contains(&(lo, lo)));
    for _ in 0..5 {
        g.sleep();
        assert!(!g.dungeon.roamer_at(lo, lo));
    }
}

#[test]
fn every_sound_has_a_valid_file_and_a_credit() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let credits = std::fs::read_to_string(base.join("sfx/CREDITS.md")).expect("CREDITS.md");
    for name in crate::audio::SFX {
        let b = std::fs::read(base.join(format!("sfx/{}.ogg", name))).unwrap_or_else(|_| panic!("{} の効果音が無い", name));
        assert!(b.len() > 1000 && &b[..4] == b"OggS", "{} が ogg でない", name);
        let row = credits.lines().find(|l| l.starts_with(&format!("| {} |", name))).unwrap_or_else(|| panic!("{} が CREDITS に無い", name));
        assert!(row.contains("CC0") || row.contains("PDM") || row.contains("BY 4.0") || row.contains("BY 3.0") || row.contains("BY 2.0") || row.contains("BY 1.0") || row.contains("BY "), "{}: {}", name, row);
        assert!(!row.contains("NC") && !row.contains("SAMPLING"), "{}: 使えないライセンス", name);
    }
    for (name, vol) in crate::audio::BGM {
        let b = std::fs::read(base.join(format!("bgm/{}.ogg", name))).unwrap_or_else(|_| panic!("{} の曲が無い", name));
        assert!(b.len() > 20_000 && &b[..4] == b"OggS", "{}", name);
        assert!(*vol > 0.0 && *vol <= 0.9, "{}", name);
    }
}

#[test]
fn bulk_sell_only_sells_loot_never_quest_items_or_gear() {
    // クエストの品は 換金アイテム(Loot)ではない
    for q in QUESTS {
        let d = item(q.need).unwrap();
        assert_ne!(d.kind, Kind::Loot, "クエストの品 {} が換金アイテムになっている", q.need);
    }
    let mut g = Game::new(180);
    for name in ["イノシシのけがわ", "ヘビのはっぱ", "ひかりのゆびわ", "カラスのももにく", "かいふくやく", "魔法のオリーブオイル", "リオンのたてがみ"] {
        g.add_item(name, 2);
    }
    let (n, _) = g.sell_loot();
    assert_eq!(n, 2, "毛皮だけ売れる");
    for name in ["ヘビのはっぱ", "ひかりのゆびわ", "カラスのももにく", "かいふくやく", "魔法のオリーブオイル", "リオンのたてがみ"] {
        assert_eq!(g.count(name), 2, "{} は売らない", name);
    }
}

#[test]
fn quest_board_hides_far_floors_and_splits_done_from_open() {
    // いま進んでいる階までのクエストだけが一覧に出る(それより先は 出さない)
    let g = Game::new(181);
    let visible: Vec<&Quest> = QUESTS.iter().filter(|q| q.min_floor <= g.reached).collect();
    assert!(visible.iter().all(|q| q.min_floor <= 1));
    assert!(visible.len() < QUESTS.len());
}

#[test]
fn town_people_talk_and_give_quests() {
    use crate::script::*;
    use crate::shop::ShopKind;
    let mut g = Game::new(190);
    // 武具屋(おねえさん)の頼みごと: 1Fで ワーネコのホルター。話を聞くと受けた状態になる
    assert!(has_talk(&g, ShopKind::Armor));
    assert_eq!(g.quest_state("halter"), 0);
    let lines = on_enter(&mut g, ShopKind::Armor);
    assert!(lines.len() >= 3 && lines.iter().any(|l| l.contains("ワーネコのホルター")), "{:?}", lines);
    assert_eq!(g.quest_state("halter"), 1);
    assert!(!has_talk(&g, ShopKind::Armor), "いちど聞いたら もう出ない");
    assert!(on_enter(&mut g, ShopKind::Armor).is_empty());
    // 魔法屋の前金つきの頼み(スライムのひもの 200G)
    let gold = g.gold;
    let l = on_enter(&mut g, ShopKind::Magic);
    assert!(l.iter().any(|x| x.contains("前金")));
    assert_eq!(g.gold, gold + 200);
    // 先の階のクエストは 頼まれない
    assert_eq!(g.quest_state("tart"), 0);
    // 村長: メインクエストは 話すだけ(状態はそのまま)
    let l = on_enter(&mut g, ShopKind::Mayor);
    assert!(l.iter().any(|x| x.contains("ヘビリンゴ")));
    // 家具屋: はじめて行くと基本の部屋を説明。新しい部屋が入ったときだけ また話す
    let l = on_enter(&mut g, ShopKind::Furniture);
    assert!(l.iter().any(|x| x.contains("ここは家具屋")) && l.len() >= 9, "{:?}", l);
    assert!(on_enter(&mut g, ShopKind::Furniture).is_empty());
    g.reached = 2;
    let l = on_enter(&mut g, ShopKind::Furniture);
    assert!(l.iter().any(|x| x.contains("新しい部屋が入った")) && l.iter().any(|x| x.contains("さかば")) && l.iter().any(|x| x.contains("はたけ")), "{:?}", l);
    // 部屋ごとの説明がそろっている
    for (_, n) in FURNITURE_UNLOCK {
        assert!(crate::script::furniture_text(n).is_some(), "{} の家具屋の説明が無い", n);
    }
    for n in FURNITURE_STOCK {
        assert!(crate::script::furniture_text(n).is_some(), "{} の家具屋の説明が無い", n);
    }
}

#[test]
fn mayor_tells_the_next_goal_for_every_step() {
    use crate::quest::StepKind;
    // 村長の台詞は 階ごとに1つずつ(1F〜最上階)
    for f in 1..=MAX_FLOOR {
        let mut g = Game::new(191);
        g.preset(if f == 1 { 1 } else if f == 2 { 3 } else { 5 + (f - 3) * 2 });
        let lines = crate::script::mayor_goal(&g);
        assert!(lines.len() >= 2 && lines[0].contains("次の目標"), "{}F: {:?}", f, lines);
        let _ = StepKind::Done;
    }
    // どの段階でも 1行が長すぎない(メッセージ欄は3行まで)。数を言わない(階の数字を言う行を除く)
    for preset in 0..(5 + 2 * (MAX_FLOOR - 2)) {
        let mut g = Game::new(193);
        g.preset(preset);
        let lines = crate::script::mayor_goal(&g);
        assert!(lines.len() <= 24, "preset {}: 行が多すぎる {}", preset, lines.len());
        for l in &lines {
            assert!(l.chars().count() <= 62, "preset {}: 長すぎる行 {}", preset, l);
        }
    }
    // 段階の種類ごとに それらしいことを言う
    let g = Game::new(192);
    let l = crate::script::mayor_goal(&g);
    assert!(l.iter().any(|x| x.contains("ダンジョンづくり")), "{:?}", l);
}

#[test]
fn mayor_talk_stays_vague_and_fits_the_message_box() {
    // 村長の話は 数を言わない・1行が長すぎない(メッセージ欄は 3行まで)
    for f in 1..=MAX_FLOOR {
        let (grow, boss) = crate::script::floor_talks(f);
        for l in grow.iter().chain(boss.iter()) {
            assert!(l.chars().count() <= 62, "{}F: 長すぎる行 {}", f, l);
            assert!(!l.chars().any(|c| c.is_ascii_digit() || "０１２３４５６７８９".contains(c)), "{}F: 数が入っている {}", f, l);
        }
        assert!(grow.iter().any(|l| l.starts_with("アレン「")), "{}F: アレンとのやりとりが無い", f);
        assert!(boss.iter().any(|l| l.starts_with("アレン「")));
    }
    // メイミも ときどき話に入る。アレンの台詞は 単文ばかりでない
    let mei = (1..=MAX_FLOOR).filter(|f| { let (g, b) = crate::script::floor_talks(*f); g.iter().chain(b.iter()).any(|l| l.starts_with("メイミ「")) }).count();
    assert!(mei >= 12, "メイミの出番が少ない: {}", mei);
    let long_allen = (1..=MAX_FLOOR).filter(|f| { let (g, b) = crate::script::floor_talks(*f); g.iter().chain(b.iter()).any(|l| l.starts_with("アレン「") && l.chars().count() >= 28) }).count();
    assert!(long_allen >= 10, "アレンの言い返しが少ない: {}", long_allen);
}


#[test]
fn party_grows_as_the_story_goes() {
    let mut g = Game::start_story(200);
    assert_eq!(g.members(), vec![0], "はじめは アレンひとり");
    assert!(!g.party[1].alive() && !g.party[2].alive());
    assert_eq!(g.party[1].status_text(), "");
    // 食事の数には いない人を入れない
    assert!(g.ate[1] && g.ate[2] && !g.ate[0]);
    // 2日めの朝までは ひとり。3日めの朝に メイミが来る
    let r = g.sleep();
    assert_eq!(g.day, 2);
    assert!(r.joined.is_empty());
    let r = g.sleep();
    assert_eq!(g.day, 3);
    assert_eq!(r.joined, vec![2]);
    assert_eq!(g.members(), vec![0, 2]);
    assert!(!g.ate[2], "メイミは その日から ごはんを食べられる");
    // 戦闘にも ふたりだけ出る
    let b = Battle::new(&g, &[Source::Boss]);
    assert!(b.order.iter().all(|a| !matches!(a, Actor::Party(1))));
    // 1Fのボスをたおすと、ジェリーが仲間になる
    g.dungeon.portal = Some((15, 8));
    g.dungeon.set(15, 8, Tile::Portal);
    for m in g.party.iter_mut() {
        m.hp_max = 9999;
        m.hp = 9999;
    }
    g.dbg.onehit = true;
    let mut b = Battle::new(&g, &[Source::Boss]);
    let mut won = false;
    for _ in 0..200 {
        match b.advance(&mut g) {
            Advance::NeedCmd(i) => {
                let t = b.alive_enemies()[0];
                b.command(&mut g, i, Cmd::Attack(t));
            }
            Advance::End(o) => {
                won = o == Outcome::Won;
                break;
            }
            Advance::Msgs(_) => {}
        }
    }
    assert!(won);
    assert_eq!(b.joined, vec![1]);
    assert_eq!(g.members(), vec![0, 1, 2]);
}

#[test]
fn nobody_mentions_mei_before_she_joins() {
    use crate::script::*;
    use crate::shop::ShopKind;
    let mut g = Game::start_story(201);
    let mut all: Vec<String> = vec![];
    for kind in [ShopKind::Mayor, ShopKind::Armor, ShopKind::Magic, ShopKind::Furniture, ShopKind::Plaza] {
        all.extend(on_enter(&mut g, kind));
    }
    all.extend(tour_town_lines());
    for q in QUESTS {
        all.extend(quest_done_lines(q, false, "ok"));
    }
    all.extend(mayor_goal(&g));
    for preset in 0..5 {
        let mut h = Game::start_story(202);
        h.preset(preset);
        h.party[2].joined = false;
        all.extend(mayor_goal(&h));
    }
    assert!(all.len() > 20);
    for l in &all {
        assert!(!l.contains("メイミ"), "メイミがいないのに: {}", l);
        assert!(!l.starts_with('※'));
    }
    // 仲間になったあとは ふつうに出る
    let mut g = Game::start_story(203);
    g.party[2].joined = true;
    g.talks_seen.remove("tour");
    let l = on_enter(&mut g, ShopKind::Furniture);
    assert!(l.iter().any(|x| x.starts_with("メイミ「")), "{:?}", l);
    // 参加の場面
    assert!(mei_join_scene().iter().any(|l| l.contains("心配だから、私も一緒に行くわ")));
    assert!(jerry_join_scene(false).iter().all(|l| !l.contains("メイミ")));
    assert!(jerry_join_scene(true).iter().any(|l| l.contains("メイミ")));
}

// ---------------- 魔法(wiki p22) ----------------

/// 戦闘で 魔法を1回つかう(アレン who の番で)。出たメッセージを返す
fn cast(g: &mut Game, b: &mut Battle, who: usize, id: &'static str, t: usize) -> Vec<String> {
    b.command(g, who, Cmd::Spell(id, t))
}

#[test]
fn every_spell_has_a_user_a_target_kind_and_a_price_rule() {
    for s in SPELLS {
        assert!(s.user == 0 || s.user == 2, "{}", s.id);
        assert!(s.mp > 0 && !s.desc.is_empty());
    }
    // wiki p22 の MP
    for (id, mp) in [("heal", 3), ("healall", 10), ("cure", 1), ("sleep", 2), ("strong", 20), ("holy", 2), ("reflect", 10), ("fire", 3), ("firestorm", 8), ("frost", 4), ("blizzard", 10), ("seal", 2), ("onehp", 3), ("drain", 7)] {
        assert_eq!(spell(id).mp, mp, "{}", id);
    }
    // 食材を見せると 入荷する魔法
    let mut g = Game::new(210);
    assert!(!g.spell_for_sale(spell("frost")));
    g.add_item("水ライムのひもの", 1);
    assert!(g.teachable().contains(&"水ライムのひもの"));
    let m = g.show_ingredient("水ライムのひもの").unwrap();
    assert!(m.contains("氷のつぶて"), "{}", m);
    assert!(g.spell_for_sale(spell("frost")));
    g.gold = 5000;
    g.buy_spell("frost").unwrap();
    assert!(g.party[2].spells.contains(&"frost".to_string()));
    for (sp, ing) in SPELL_TAUGHT_BY {
        assert!(item(ing).is_some(), "{} の食材 {}", sp, ing);
    }
}

#[test]
fn strong_raises_attack_and_ends_with_the_battle() {
    let mut g = Game::new(211);
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    let base = g.party[0].atk();
    g.party[0].mp = 99;
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    let m = cast(&mut g, &mut b, 0, "strong", 0);
    assert!(m.iter().any(|l| l.contains("あがった")), "{:?}", m);
    assert_eq!(g.party[0].atk(), base * 3 / 2);
    assert_eq!(g.party[0].mp, 99 - 20);
}

#[test]
fn reflect_blocks_one_magic_attack_then_wears_off() {
    let mut g = Game::new(212);
    g.party[0].mp = 99;
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    cast(&mut g, &mut b, 0, "reflect", 0);
    assert!(g.party[0].reflect);
    let hp = g.party[0].hp;
    let mut v = vec![];
    for _ in 0..2 {
        v.extend(b.test_elem_all(&mut g, &[0], "テスト", "ほのお", 50, 0));
    }
    // 1回目は はねかえす、2回目は あたる
    assert!(v.iter().any(|l| l.contains("はねかえした")), "{:?}", v);
    assert!(g.party[0].hp < hp);
    assert!(!g.party[0].reflect);
}

#[test]
fn sealed_enemies_use_no_special_moves_and_sealed_party_cannot_cast() {
    // ふういんされた ボスは 技を使わない
    let mut g = Game::new(213);
    g.change_floor(3);
    g.dungeon.portal = Some((15, 8));
    g.dungeon.set(15, 8, Tile::Portal);
    for m in g.party.iter_mut() {
        m.hp_max = 99999;
        m.hp = 99999;
        m.con = 30;
    }
    let mut b = Battle::new(&g, &[Source::Boss]);
    b.enemies[0].sealed = true;
    let mut seen_special = false;
    for _ in 0..600 {
        match b.advance(&mut g) {
            Advance::Msgs(v) => seen_special |= v.iter().any(|m| m.contains("ようがんこうげき") || m.contains("のしかかり")),
            Advance::NeedCmd(i) => {
                b.command(&mut g, i, Cmd::Defend);
            }
            Advance::End(_) => break,
        }
    }
    assert!(!seen_special, "ふういん中なのに技が出た");
    // ふういんされた味方は 魔法が使えない(MPも へらない)
    let mut g = Game::new(214);
    g.party[2].seal = 2;
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    let mp = g.party[2].mp;
    let m = cast(&mut g, &mut b, 2, "fire", 0);
    assert!(m.iter().any(|l| l.contains("ふういん")), "{:?}", m);
    assert_eq!(g.party[2].mp, mp);
    assert_eq!(g.party[2].seal, 1, "行動するたびに へる");
    // ふういんの魔法で 敵の技を とめる
    let mut g = Game::new(215);
    g.party[2].mp = 99;
    g.party[2].wis = 50;
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    for _ in 0..5 {
        cast(&mut g, &mut b, 2, "seal", 0);
        if b.enemies[0].sealed {
            break;
        }
    }
    assert!(b.enemies[0].sealed);
}

#[test]
fn onehp_and_drain_work_and_the_elephantom_reverses_drain() {
    let mut g = Game::new(216);
    g.party[2].mp = 99;
    g.party[2].wis = 40;
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "ningen".into() });
    g.dungeon.roamers.push(Roamer { x: 8, y: 10, mon: "ningen".into() });
    let mut b = Battle::new(&g, &[Source::Roamer(0), Source::Roamer(1)]);
    cast(&mut g, &mut b, 2, "onehp", 0);
    assert!(b.enemies[0].hp == 1 || b.enemies[0].hp == b.enemies[0].def.hp, "ききめは ふたとおり");
    // いのちいただき: 敵のHPが へり、仲間のHPが ふえる
    g.party[0].hp = 5;
    let before: i32 = b.enemies.iter().map(|e| e.hp).sum();
    cast(&mut g, &mut b, 2, "drain", 0);
    let after: i32 = b.enemies.iter().map(|e| e.hp).sum();
    assert!(after < before);
    assert!(g.party[0].hp > 5);
    // エレファントムは はねかえす(ボスのHPがふえる)
    let mut g = Game::new(217);
    g.change_floor(18);
    g.dungeon.portal = Some((15, 8));
    g.dungeon.set(15, 8, Tile::Portal);
    g.party[2].mp = 99;
    let mut b = Battle::new(&g, &[Source::Boss]);
    b.enemies[0].hp -= 500;
    let hp = b.enemies[0].hp;
    let m = cast(&mut g, &mut b, 2, "drain", 0);
    assert!(b.enemies[0].hp > hp && m.iter().any(|l| l.contains("はねかえした")), "{:?}", m);
}

#[test]
fn mitsuoni_beam_really_seals_and_reflect_stops_it() {
    let mut g = Game::new(218);
    g.change_floor(17);
    g.dungeon.portal = Some((15, 8));
    g.dungeon.set(15, 8, Tile::Portal);
    for m in g.party.iter_mut() {
        m.hp_max = 99999;
        m.hp = 99999;
        m.con = 30;
    }
    let mut b = Battle::new(&g, &[Source::Boss]);
    let mut sealed = false;
    for _ in 0..800 {
        match b.advance(&mut g) {
            Advance::NeedCmd(i) => {
                b.command(&mut g, i, Cmd::Defend);
            }
            Advance::Msgs(_) => sealed |= g.party.iter().any(|m| m.seal > 0),
            Advance::End(_) => break,
        }
        if sealed {
            break;
        }
    }
    assert!(sealed, "ミツメオニの封印ビームで ふういんされる");
}

#[test]
fn mirror_and_life_stone_quests_reward_reflect_and_a_staff() {
    let mut g = Game::new(219);
    g.reached = 10;
    assert!(!g.party[0].spells.contains(&"reflect".to_string()));
    g.accept_quest("mirror");
    g.add_item("かがみのころも", 1);
    g.deliver_quest("mirror").unwrap();
    assert!(g.party[0].spells.contains(&"reflect".to_string()));
    g.accept_quest("lifestone");
    g.add_item("いのちの石", 1);
    g.deliver_quest("lifestone").unwrap();
    assert_eq!(g.count("かいふくのつえ"), 1);
    // 売らない
    g.add_item("かがみのころも", 1);
    assert_eq!(g.sell_loot().0, 0);
    // アクマノヨロイが いのちの石を落とす
    assert!(mon("demonarmor").drops.iter().any(|(n, _)| *n == "いのちの石"));
}


// ---------------- モノマネスライム ----------------

#[test]
fn defeated_monsters_become_mimicable_including_bosses() {
    let mut g = Game::new(230);
    assert!(g.mimic_choices().is_empty());
    assert!(g.set_mimic(1, 1, Some("wan")).is_err(), "まだたおしていない");
    g.mimic_known.insert("wan".into());
    assert_eq!(g.mimic_choices(), vec!["wan"]);
    // ボスも たおせば マネできる(能力の増え方は 1/3)
    assert!(g.set_mimic(1, 1, Some("lion")).is_err(), "まだたおしていない");
    g.mimic_known.insert("lion".into());
    assert!(g.mimic_choices().contains(&"lion"));
    assert!(g.set_mimic(1, 1, Some("lion")).is_ok());
    let lion = mon("lion");
    assert_eq!(crate::mimic::bonus(1, lion).str_, lion.atk * 2 / 5 / 3);
    for id in ["needleworm", "hinezumi", "shieldmaiden", "detaramedice", "kingsnake", "devillord"] {
        assert!(crate::mimic::can_mimic(id), "{}", id);
    }
    assert!(g.set_mimic(0, 1, Some("wan")).is_err(), "ジェリーだけ");
    // 戦闘でたおすと 覚える(にげた魔物は 覚えない)
    let mut g = Game::new(231);
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "bat".into() });
    g.dbg.onehit = true;
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    for _ in 0..50 {
        match b.advance(&mut g) {
            Advance::NeedCmd(i) => {
                let t = b.alive_enemies()[0];
                b.command(&mut g, i, Cmd::Attack(t));
            }
            Advance::End(_) => break,
            Advance::Msgs(_) => {}
        }
    }
    assert!(g.mimic_known.contains("bat"));
}

#[test]
fn mimicking_changes_stats_equipment_and_magic() {
    let mut g = Game::new(232);
    for id in ["gobl", "ghost", "nekomata", "pig", "powerwan", "cat"] {
        g.mimic_known.insert(id.into());
    }
    let j = 1;
    let (hp0, str0, spd0, wis0) = (g.party[j].hp_max, g.party[j].str_total(), g.party[j].spd_total(), g.party[j].wis_total());
    // はじめは そうびできない
    assert!(!g.party[j].can_wear(0) && !g.party[j].can_wear(3));
    assert!(g.equip(j, "ナイフ").is_err());
    // うで=ゴブリン: つよさ+、武器・盾・指輪○
    g.set_mimic(j, 1, Some("gobl")).unwrap();
    assert!(g.party[j].str_total() > str0);
    assert!(g.party[j].can_wear(0) && g.party[j].can_wear(1) && g.party[j].can_wear(5));
    g.add_item("ナイフ", 1);
    assert!(g.equip(j, "ナイフ").is_ok());
    // からだ=ブタイノシシ(鎧×)・HPは増える。あし=ワーネコ(靴○)
    g.set_mimic(j, 2, Some("pig")).unwrap();
    assert!(g.party[j].hp_max > hp0);
    assert!(!g.party[j].can_wear(3));
    g.set_mimic(j, 3, Some("cat")).unwrap();
    assert!(g.party[j].spd_total() > spd0 && g.party[j].can_wear(4));
    // あたま=ワライオバケ: 魔法「ふういん」を おぼえる(兜○)
    assert!(!g.party[j].spells.contains(&"seal".to_string()));
    g.set_mimic(j, 0, Some("ghost")).unwrap();
    assert!(g.party[j].spells.contains(&"seal".to_string()));
    assert!(g.party[j].wis_total() > wis0 && g.party[j].mp_max > 3);
    // あたまを かえても 魔法は わすれない
    g.set_mimic(j, 0, None).unwrap();
    assert!(g.party[j].spells.contains(&"seal".to_string()));
    // ネコマタ: ひとりかいふく。ぜんいんかいふくは 7F以降
    g.set_mimic(j, 0, Some("nekomata")).unwrap();
    assert!(g.party[j].spells.contains(&"heal".to_string()) && !g.party[j].spells.contains(&"healall".to_string()));
    g.reached = 7;
    g.refresh_mimic_spells(j);
    assert!(g.party[j].spells.contains(&"healall".to_string()));
    // うでをかえて 武器×になると そうびは はずれて 持ち物にもどる
    g.set_mimic(j, 1, Some("pig")).unwrap();
    assert!(g.party[j].equip[0].is_none());
    assert_eq!(g.count("ナイフ"), 1);
    // HPは ものまねをやめると もとにもどる
    g.set_mimic(j, 2, None).unwrap();
    assert_eq!(g.party[j].hp_max, hp0);
}

#[test]
fn slime_uses_the_skill_of_the_monster_it_mimics() {
    let mut g = Game::new(233);
    g.mimic_known.insert("powerwan".into());
    g.set_mimic(1, 1, Some("powerwan")).unwrap();
    g.party[1].str_ = 60;
    let mut seen = false;
    for _ in 0..40 {
        g.dungeon.roamers.clear();
        g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "ningen".into() });
        let mut b = Battle::new(&g, &[Source::Roamer(0)]);
        let m = b.command(&mut g, 1, Cmd::Attack(0));
        seen |= m.iter().any(|l| l.contains("パワーアタック"));
        if seen {
            break;
        }
    }
    assert!(seen, "ときどき特技がでる");
    // かいてんぎり(ニンゲンザウルス)は 全体攻撃
    g.mimic_known.insert("ningen".into());
    g.set_mimic(1, 1, Some("ningen")).unwrap();
    let mut all_hit = false;
    for _ in 0..60 {
        g.dungeon.roamers.clear();
        g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "slime".into() });
        g.dungeon.roamers.push(Roamer { x: 8, y: 10, mon: "slime".into() });
        let mut b = Battle::new(&g, &[Source::Roamer(0), Source::Roamer(1)]);
        let m = b.command(&mut g, 1, Cmd::Attack(0));
        if m.iter().any(|l| l.contains("かいてんぎり")) {
            all_hit = m.iter().filter(|l| l.contains("のダメージ")).count() >= 2;
            break;
        }
    }
    assert!(all_hit);
    // おまかせで いちばん強い組み合わせ
    let mut h = Game::new(234);
    for id in ["bat", "pig", "powerwan", "icewar", "wizdrill"] {
        h.mimic_known.insert(id.into());
    }
    assert!(h.auto_mimic(1) >= 3);
    assert!(h.party[1].mimic.iter().all(|m| m.is_some()));
}

#[test]
fn room_effects_seal_boost_and_sleep_enemies() {
    let mut g = Game::new(235);
    let mk = |kind: &str| Room { x: 8, y: 9, w: 1, h: 1, front: (8, 9), kind: kind.into(), door: Dir::Down, open4: false, mons: vec![Some("ningen".into())], chest_open: false, refill: 0 };
    g.dungeon.rooms.push(mk("けっかいのへや"));
    g.dungeon.rooms.push(mk("まほうじん"));
    g.dungeon.rooms.push(mk("りっぱなベッド"));
    let b = Battle::new(&g, &[Source::Room(0, 0)]);
    assert!(b.enemies[0].sealed, "けっかいのへや: ふういん状態ではじまる");
    let b = Battle::new(&g, &[Source::Room(1, 0)]);
    assert!(b.enemies[0].hp > mon("ningen").hp && b.enemies[0].defense() > mon("ningen").def, "まほうじん: 強い");
    let b = Battle::new(&g, &[Source::Room(2, 0)]);
    assert!(b.enemies[0].sleep > 0, "りっぱなベッド: 寝ている");
}

#[test]
fn detarame_sword_hits_for_random_damage() {
    let mut g = Game::new(236);
    g.add_item("でたらめなけん", 1);
    g.equip(0, "でたらめなけん").unwrap();
    g.party[0].str_ = 30;
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..60 {
        g.dungeon.roamers.clear();
        g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "ningen".into() });
        let mut b = Battle::new(&g, &[Source::Roamer(0)]);
        let hp = b.enemies[0].hp;
        b.command(&mut g, 0, Cmd::Attack(0));
        seen.insert(hp - b.enemies[0].hp);
    }
    assert!(seen.len() >= 5, "ダメージが ばらばら: {:?}", seen);
}


// ---------------- 手に入るか(入手経路のない品が無い) ----------------

#[test]
fn every_quest_item_recipe_ingredient_and_spell_item_can_be_obtained() {
    let mut sources: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for m in MONS {
        for (n, _) in m.drops {
            sources.insert(n);
        }
    }
    for f in 1..=MAX_FLOOR {
        for d in 0..=2 {
            for n in chest_items(f, d) {
                sources.insert(n);
            }
        }
        for (_, _, n) in hidden_items(f) {
            sources.insert(Box::leak(n.into_boxed_str()));
        }
    }
    // 魔法屋で買える・クエストの報酬・最初から持っている物
    for q in QUESTS {
        if let Some(r) = q.reward_item {
            sources.insert(r);
        }
    }
    for n in ["ホウレンソウ", "ジャガイモ", "ニンジン", "カツオブシ", "ピーマン"] {
        sources.insert(n); // 広場で買える野菜
    }
    // クエストの品
    for q in QUESTS {
        assert!(sources.contains(q.need), "クエスト「{}」の品 {} の入手経路が無い", q.title, q.need);
    }
    // 料理の材料(グルメのカンヅメは wiki に入手方法が無いので 料理にも入れていない)
    for r in RECIPES {
        for n in r.needs {
            assert!(sources.contains(n), "料理「{}」の材料 {} の入手経路が無い", r.name, n);
        }
    }
    // 魔法屋に見せる食材
    for (sp, ing) in SPELL_TAUGHT_BY {
        assert!(sources.contains(ing), "魔法「{}」の食材 {} の入手経路が無い", sp, ing);
    }
}

#[test]
fn fridge_monsters_drop_more_food() {
    let mut fridge = 0;
    let mut plain = 0;
    for seed in 0..400u64 {
        for (kind, count) in [("れいぞうこ", &mut fridge), ("エサおけ", &mut plain)] {
            let mut g = Game::new(300 + seed);
            g.dungeon.rooms.push(Room { x: 8, y: 9, w: 1, h: 1, front: (8, 9), kind: kind.into(), door: Dir::Down, open4: false, mons: vec![Some("mushroomlizard".into())], chest_open: false, refill: 0 });
            g.dbg.onehit = true;
            let mut b = Battle::new(&g, &[Source::Room(0, 0)]);
            for _ in 0..30 {
                match b.advance(&mut g) {
                    Advance::NeedCmd(i) => {
                        b.command(&mut g, i, Cmd::Attack(0));
                    }
                    Advance::End(_) => break,
                    Advance::Msgs(_) => {}
                }
            }
            *count += g.count("キノコ") + g.count("トカゲのももにく");
        }
    }
    assert!(fridge > plain * 3 / 2, "れいぞうこは 食べ物が落ちやすい: {} vs {}", fridge, plain);
}

// ---------------- 立ち絵・オープニング・ボスの知らせ ----------------

#[test]
fn next_boss_news_names_the_next_boss_and_has_no_mei_before_she_joins() {
    for f in 1..MAX_FLOOR {
        let v = crate::script::next_boss_scene(f, true);
        let all = v.join("\n");
        assert!(all.contains(mon(boss_id(f + 1)).name), "{}F のボスを倒したあとの知らせに {} が出てこない", f, mon(boss_id(f + 1)).name);
        let no = crate::script::next_boss_scene(f, false);
        assert!(no.iter().all(|l| !l.starts_with("メイミ「") && !l.starts_with('※')), "{}F: メイミ未加入なのに メイミの行が残っている", f);
        assert!(no.len() >= 5);
    }
    // 最後のボスを倒したあとは おしまいの手紙
    assert!(crate::script::next_boss_scene(MAX_FLOOR, false).iter().any(|l| l.contains("ご苦労")));
}

#[test]
fn boss_news_arrives_once_on_the_morning_after_the_boss_falls() {
    let mut g = Game::start_story(5);
    assert!(g.sleep().news.is_empty());
    g.warp(1);
    g.dungeon.boss_defeated = true;
    let r = g.sleep();
    assert_eq!(r.news, vec![1]);
    assert!(g.sleep().news.is_empty(), "知らせは 一度だけ");
    // セーブを読み込んでも 同じ知らせは 来ない
    let g2 = Game::load_text(&g.save_text()).unwrap();
    assert!(g2.talks_seen.contains("news:1"));
}

#[test]
fn stage_parses_speakers() {
    use crate::stage::parse;
    assert_eq!(parse("アレン「どうして?」", Some("村長")), (Some("アレン"), "「どうして?」".to_string()));
    assert_eq!(parse("ふむ、そうじゃな。", Some("村長")), (Some("村長"), "「ふむ、そうじゃな。」".to_string()));
    assert_eq!(parse("＊朝になった。", Some("村長")), (None, "朝になった。".to_string()));
    assert_eq!(parse("朝になった。", None), (None, "朝になった。".to_string()));
    for n in ["stand_a", "stand_m", "stand_j"] {
        assert!(crate::art::declared(n));
    }
}

#[test]
fn story_start_is_unarmed_and_the_tour_greets_each_place_once() {
    use crate::script::*;
    use crate::shop::ShopKind;
    let mut g = Game::start_story(210);
    assert!(g.party[0].equip[0].is_none(), "はじめからは 武器なし");
    assert_eq!(g.count("ナイフ"), 0);
    assert!(g.armor_stock.contains("ナイフ"));
    assert_eq!(tour_left(&g), 5);
    for kind in [ShopKind::Armor, ShopKind::Magic, ShopKind::Furniture, ShopKind::Plaza, ShopKind::Mayor] {
        assert!(has_talk(&g, kind), "あいさつに行く家には ! が出る");
        let l = on_enter(&mut g, kind);
        assert!(l.iter().any(|x| x.contains("ダンジョンマスター")), "{:?}", l);
        assert!(l.len() <= 14, "あいさつが長すぎる: {}", l.len());
        assert!(on_enter(&mut g, kind).iter().all(|x| !x.contains("ダンジョンマスター")), "あいさつは一度だけ");
    }
    assert_eq!(tour_left(&g), 0);
    // 最初の依頼も あいさつのときに 出る(武具屋は ホルター、魔法屋は スライムのひもの)
    assert_eq!(g.quest_state("halter"), 1);
    assert_eq!(g.quest_state("slime"), 1);
    assert_eq!(g.quest_state("crow"), 1, "村長の家でも ついでに 頼まれる");
    // ナイフを買えば そうびできる
    assert!(g.buy("ナイフ").is_ok());
    assert!(g.equip(0, "ナイフ").is_ok());
    // 古い物語でない ゲームは あいさつをしない
    let h = Game::new(211);
    assert_eq!(tour_left(&h), 0);
    assert!(h.party[0].equip[0].is_some());
}

#[test]
fn every_quest_has_a_completion_talk_and_sub_quests_have_a_reminder() {
    use crate::script::*;
    for q in QUESTS {
        let l = quest_done_lines(q, true, "達成");
        assert!(l.len() >= 3, "{} の達成の会話が無い", q.id);
        assert!(l.iter().all(|x| !x.contains('{')), "{}: 置きかえ忘れ {:?}", q.id, l);
        let no = quest_done_lines(q, false, "達成");
        assert!(no.iter().all(|x| !x.starts_with("メイミ「") && !x.starts_with('※')));
        if q.reward_item.is_some() {
            // ほうびの品を 話の中で 言っている(システム文だけに しない)
            assert!(l.iter().any(|x| x.contains(q.reward_item.unwrap())) || q.id == "crow", "{}: ほうびの品を言っていない", q.id);
        }
    }
    // 受けてから日がたつと 一度だけ 催促される
    let mut g = Game::new(212);
    g.accept_quest("crow");
    assert!(remind_pending(&g, crate::shop::ShopKind::Mayor).is_empty(), "その日のうちは 言わない");
    g.day += 1;
    let r = remind_pending(&g, crate::shop::ShopKind::Mayor);
    assert_eq!(r.len(), 1);
    let _ = on_enter(&mut g, crate::shop::ShopKind::Mayor);
    assert!(remind_pending(&g, crate::shop::ShopKind::Mayor).is_empty());
}

#[test]
fn magic_shop_talks_about_new_stock_once() {
    use crate::script::*;
    use crate::shop::ShopKind;
    let mut g = Game::start_story(213);
    let _ = on_enter(&mut g, ShopKind::Magic); // あいさつ
    assert!(!has_talk(&g, ShopKind::Magic) || g.quest_state("slime") == 0);
    // 2階に降りたら せいなる光・ふぶきバクダンの話
    g.reached = 2;
    let l = on_enter(&mut g, ShopKind::Magic);
    assert!(l.iter().any(|x| x.contains("せいなる光")), "{:?}", l);
    assert!(l.iter().any(|x| x.contains("ふぶきバクダン")), "{:?}", l);
    assert!(on_enter(&mut g, ShopKind::Magic).iter().all(|x| !x.contains("せいなる光")), "一度だけ");
    // スライムのひものを届けた翌日に すいみん
    g.add_item("スライムのひもの", 1);
    g.deliver_quest("slime").unwrap();
    assert!(on_enter(&mut g, ShopKind::Magic).iter().all(|x| !x.contains("すいみん")));
    g.day += 1;
    let l = on_enter(&mut g, ShopKind::Magic);
    assert!(l.iter().any(|x| x.contains("すいみん")), "{:?}", l);
    // 食材を見せると その魔法の話が出る(店の中で)
    for (id, _) in SPELL_TAUGHT_BY {
        assert!(!spell_talk(id).is_empty(), "{} の入荷の話が無い", id);
        assert!(spell_talk(id).iter().all(|x| !x.contains('{')));
    }
}

#[test]
fn furniture_shop_talks_about_new_rooms_after_descending() {
    use crate::script::*;
    use crate::shop::ShopKind;
    let mut g = Game::start_story(214);
    let first = on_enter(&mut g, ShopKind::Furniture);
    assert!(first.iter().any(|x| x.contains("ダンジョンマスター")));
    assert!(first.iter().any(|x| x.contains("エサおけ")) && first.iter().any(|x| x.contains("わらのベッド")));
    assert!(on_enter(&mut g, ShopKind::Furniture).is_empty());
    g.reached = 2;
    assert!(has_talk(&g, ShopKind::Furniture));
    let l = on_enter(&mut g, ShopKind::Furniture);
    assert_eq!(l.iter().filter(|x| x.starts_with("さかばは")).count(), 1, "{:?}", l);
    assert!(on_enter(&mut g, ShopKind::Furniture).is_empty());
}


#[test]
fn a_broke_allen_still_gets_the_knife_on_credit() {
    use crate::script::*;
    use crate::shop::ShopKind;
    let mut g = Game::start_story(215);
    g.gold = 10;
    let l = on_enter(&mut g, ShopKind::Armor);
    assert!(l.iter().any(|x| x.contains("出世払い")), "{:?}", l);
    assert!(g.party[0].equip[0].as_deref() == Some("ナイフ"));
    // お金があるときは 買ってもらう
    let mut h = Game::start_story(216);
    let l = on_enter(&mut h, ShopKind::Armor);
    assert!(l.iter().all(|x| !x.contains("出世払い")));
    assert!(h.party[0].equip[0].is_none());
}

#[test]
fn roamers_cut_off_by_a_room_are_removed_at_once() {
    let mut g = Game::new(300);
    g.dungeon.set(8, 11, Tile::Floor);
    g.dungeon.set(8, 10, Tile::Floor);
    g.dungeon.set(8, 9, Tile::Floor);
    g.dungeon.roamers.push(Roamer { x: 8, y: 9, mon: "bat".into() });
    g.prune_roamers();
    assert_eq!(g.dungeon.roamers.len(), 1);
    // 通路が 埋まって 魔物の場所が 入口から つながらなくなる
    g.dungeon.set(8, 10, Tile::Rock);
    g.prune_roamers();
    assert!(g.dungeon.roamers.is_empty());
}

#[test]
fn every_facility_has_a_hand_made_model() {
    for r in ROOM_DEFS {
        let art = format!("room_{}", r.name);
        assert!(crate::modeling::has_hand(&art), "{} の手組みモデルが無い", r.name);
        let m = crate::modeling::hand_model(&art).unwrap_or_else(|| panic!("{} のモデル", r.name));
        assert!(m.quads.len() > 200, "{} の面が少なすぎる", r.name);
        // 1マスの施設は 1マスに おさまる(はみ出さない)
        let cells = crate::modeling::hand_cells(&art) as f32;
        let lim = 0.62 * cells.max(1.0) + if cells > 1.0 { 0.9 } else { 0.0 };
        for q in &m.quads {
            for c in &q.c {
                assert!(c[0].abs() < lim && c[2].abs() < lim && c[1] > -0.05 && c[1] < 1.5, "{} がはみ出している: {:?}", r.name, c);
            }
        }
        if chest_tier(r.name).is_some() {
            assert!(crate::modeling::has_hand(&format!("{}_open", art)));
        }
    }
}

#[test]
fn every_monster_has_a_hand_made_model() {
    let mut seen = std::collections::BTreeSet::new();
    for mon in MONS {
        if !seen.insert(mon.sprite) {
            continue;
        }
        let m = crate::modeling::hand_model(mon.sprite).unwrap_or_else(|| panic!("{} の手組みモデルが無い", mon.sprite));
        assert!(m.quads.len() > 150, "{} の面が少なすぎる: {}", mon.sprite, m.quads.len());
        assert!(m.quads.len() < if crate::bosses::is_boss_art(mon.sprite) { 16000 } else { 9000 }, "{} の面が多すぎる: {}", mon.sprite, m.quads.len());
        for q in &m.quads {
            for c in &q.c {
                assert!(c[0].abs() < 0.9 && c[2].abs() < 0.9 && c[1] > -0.1 && c[1] < 1.9, "{} がはみ出している: {:?}", mon.sprite, c);
            }
        }
    }
}

#[test]
fn every_room_spawns_monsters_on_every_floor() {
    for f in 1..=MAX_FLOOR {
        for r in ROOM_DEFS {
            if chest_tier(r.name).is_some() || r.name == MIASMA_ROOM {
                continue;
            }
            assert!(!room_spawns(f, r.name).is_empty(), "{}F の {} に魔物の表が無い", f, r.name);
        }
    }
    // 1F にいろりを置いても 魔物が湧く(3Fの表から)
    assert_eq!(room_spawns(8, "れいはいどう"), room_spawns(9, "れいはいどう"));
}

#[test]
fn worm_shield_hurts_the_wearer_each_action_and_thorns_back() {
    let mut g = Game::new(320);
    g.add_item("ワームシールド", 1);
    g.equip(0, "ワームシールド").unwrap();
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "needleworm".into() });
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    b.enemies[0].hp = 900;
    let (mut self_hits, mut thorns) = (0, 0);
    for _ in 0..60 {
        match b.advance(&mut g) {
            Advance::NeedCmd(i) => {
                let hp = g.party[i].hp;
                let m = b.command(&mut g, i, Cmd::Defend);
                if i == 0 {
                    self_hits += m.iter().filter(|l| l.contains("1ダメージ")).count();
                    assert_eq!(hp - g.party[0].hp, 1, "{:?}", m);
                }
            }
            Advance::Msgs(m) => thorns += m.iter().filter(|l| l.contains("はんげき")).count(),
            Advance::End(_) => break,
        }
        g.party[0].hp = g.party[0].hp_max;
    }
    assert!(self_hits > 3);
    assert!(thorns > 0, "なぐられたら はね返す");
    let lost = 900 - b.enemies[0].hp;
    assert!(lost >= 3 * thorns as i32 && lost <= 5 * thorns as i32, "{}", lost);
}

#[test]
fn new_early_monsters_are_assigned_and_drop_things() {
    assert_eq!(room_spawns(1, "さかば"), &["needleworm"]);
    assert_eq!(room_spawns(1, "さいだん"), &["needleworm"]);
    assert_eq!(room_spawns(1, "はたけ"), &["needleworm"]);
    assert_eq!(room_spawns(1, "いろり"), &["hinezumi"]);
    assert_eq!(room_spawns(2, "いろり"), &["hinezumi"]);
    assert_eq!(room_spawns(3, "どうじょう"), &["gob", "blackcat"]);
    // ヒネズミのけがわは 炎に強い 防具(防御は低め)
    let d = item("ヒネズミのけがわ").unwrap();
    assert!(d.kind == Kind::Body && d.power <= 4);
    assert_eq!(resist("ヒネズミのけがわ").0, 20);
    // ヒネズミのにく の料理の上がり幅は 他とかぶらない
    let r = RECIPES.iter().find(|r| r.name == "ヒネズミのあぶりやき").unwrap();
    for o in RECIPES.iter().filter(|o| o.name != r.name) {
        assert_ne!(o.gain, r.gain, "{} と同じ上がり幅", o.name);
    }
}


#[test]
fn shield_maiden_dice_and_magic_circle_bosses() {
    for f in 1..=5 {
        assert_eq!(room_spawns(f, "れいはいどう"), &["shieldmaiden"]);
    }
    // メイデンシールドは ワルキリーの装備より 少し弱い(ヘルム17・ブーツ22・ホルター43)
    let sh = item("メイデンシールド").unwrap();
    assert!(sh.kind == Kind::Shield && sh.power < item("ワルキリーヘルム").unwrap().power && sh.power > item("メイデンヘルム").unwrap().power);
    assert!(mon("shieldmaiden").drops.iter().any(|(n, _)| *n == "メイデンシールド"));
    for f in 1..=9 {
        let t = room_spawns(f, "じっけんしつ");
        assert!(t.contains(&"detaramedice"), "{}F", f);
        assert!(t.iter().any(|m| *m != "detaramedice"), "その階の魔物もいる");
    }
    // 10F以降の じっけんしつに サイコロは 出ない
    assert!(!room_spawns(10, "じっけんしつ").contains(&"detaramedice"));
    // でたらめなにくへん は でたらめなにく の弱い版(8F相当の合計8)
    let r = RECIPES.iter().find(|r| r.name == "でたらめソテー").unwrap();
    assert_eq!(r.gain.iter().sum::<i32>(), 8);
    assert!(r.gain.iter().sum::<i32>() < RECIPES.iter().find(|r| r.name == "でたらめシチュー").unwrap().gain.iter().sum::<i32>());
    for o in RECIPES.iter().filter(|o| o.name != r.name) {
        assert_ne!(o.gain, r.gain, "{} と同じ上がり幅", o.name);
    }
    // まほうじんには その階のボスが出る(1〜19F)
    for f in 1..=19 {
        assert_eq!(room_spawns(f, "まほうじん"), &[boss_id(f)]);
        assert!(mon(boss_id(f)).boss);
        assert!(crate::mimic::can_mimic(boss_id(f)), "{}F のボスは まねできる", f);
    }
}

#[test]
fn peddler_comes_every_fifth_day_and_sells_each_obtained_food_once() {
    let mut g = Game::new(400);
    g.day = 4;
    assert!(!g.peddler_here());
    g.day = 5;
    assert!(g.peddler_here());
    assert!(g.peddler_wares().is_empty(), "手に入れた食材が まだ ない");
    g.add_item("カラスのももにく", 1);
    g.add_item("ホウレンソウ", 1); // 野菜は 広場で売っている
    let w = g.peddler_wares();
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].0, "カラスのももにく");
    assert_eq!(w[0].1, floor_loot_value(1) * 3);
    g.gold = 100000;
    assert_eq!(g.peddler_buy("カラスのももにく"), Ok(w[0].1));
    assert!(g.peddler_buy("カラスのももにく").is_err(), "その日は 1個だけ");
    g.day = 10;
    assert!(g.peddler_buy("カラスのももにく").is_ok(), "次の行商人の日に また 買える");
    g.day = 6;
    assert!(g.peddler_buy("カラスのももにく").is_err());
    // ボスの食材は さらに 高い
    let boss_food = mon("lion").drops.iter().map(|d| d.0).find(|n| item(n).map(|i| i.kind == Kind::Food).unwrap_or(false));
    if let Some(n) = boss_food {
        let (f, boss) = food_origin(n).unwrap();
        assert!(boss);
        assert_eq!(peddler_price(n), Some(floor_loot_value(f) * 9));
    }
}

// ---------------- 瘴気発生装置 ----------------

fn put_miasma(g: &mut Game, n: usize) {
    for i in 0..n {
        g.dungeon.rooms.push(Room { x: 2 + i, y: 2, w: 1, h: 1, front: (2 + i, 2), kind: MIASMA_ROOM.into(), door: Dir::Down, open4: false, mons: vec![None], chest_open: false, refill: 0 });
    }
}

#[test]
fn plus_equipment_is_six_percent_stronger_per_step_rounded_up() {
    let base = item("ナイフ").unwrap();
    for k in 1..=5 {
        let d = item(&format!("ナイフ+{}", k)).unwrap();
        assert_eq!(d.power, (base.power * (100 + 6 * k) + 99) / 100, "+{}", k);
        assert_eq!(d.kind, base.kind);
    }
    // +5 は 30% 増(切り上げ)
    let big = item("ワルキリースピア").unwrap();
    assert_eq!(item("ワルキリースピア+5").unwrap().power, (big.power * 130 + 99) / 100);
    assert_eq!(item("ワルキリースピア+5").unwrap().power, 91);
    // 特殊な効果(耐性・名前での判定)は そのまま
    assert_eq!(resist("炎のたて+5"), resist("炎のたて"));
    assert_eq!(base_name("ワームシールド+2"), "ワームシールド");
    assert!(item("ナイフ+6").is_none() && item("ホウレンソウ+1").is_none() && item("ナイフ+0").is_none());
}

#[test]
fn miasma_generators_cap_at_five_make_monsters_stronger_and_tag_drops() {
    let mut g = Game::new(500);
    put_miasma(&mut g, 5);
    assert_eq!(g.miasma_count(), 5);
    g.dungeon.set(8, 11, Tile::Floor);
    g.dungeon.set(8, 10, Tile::Floor);
    g.pos = (8, 11);
    g.facing = Dir::Up;
    g.mp = 99;
    g.add_item(MIASMA_ROOM, 1);
    assert!(g.place_room(MIASMA_ROOM).is_err(), "6個めは 置けない");
    // 魔物は 1.2倍ずつ(切り上げ)
    for (n, boost) in [(0usize, 100), (1, 120), (2, 144), (3, 173), (4, 208), (5, 250)] {
        let mut h = Game::new(501);
        put_miasma(&mut h, n);
        h.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "wan".into() });
        let b = Battle::new(&h, &[Source::Roamer(0)]);
        assert_eq!(b.enemies[0].boost, boost, "{}個", n);
    }
    // 落とす装備に +1〜+n が つく(n は 置いた数)
    let mut seen_plus = std::collections::BTreeSet::new();
    for seed in 0..400u64 {
        let mut h = Game::new(600 + seed);
        put_miasma(&mut h, 3);
        h.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "wan".into() });
        h.dbg.onehit = true;
        let mut b = Battle::new(&h, &[Source::Roamer(0)]);
        for _ in 0..20 {
            match b.advance(&mut h) {
                Advance::NeedCmd(i) => {
                    b.command(&mut h, i, Cmd::Attack(0));
                }
                Advance::End(_) => break,
                Advance::Msgs(_) => {}
            }
        }
        for n in &b.got {
            let (base, k) = plus_split(n);
            if item(base).map(|d| d.kind.is_equip()).unwrap_or(false) {
                assert!((1..=3).contains(&k), "装備は +1〜+3: {}", n);
                seen_plus.insert(k);
            }
        }
    }
    assert_eq!(seen_plus.len(), 3, "+1〜+3 が 出る: {:?}", seen_plus);
    // 置いていない階では +は つかない
    let mut h = Game::new(700);
    h.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "wan".into() });
    h.dbg.onehit = true;
    let mut b = Battle::new(&h, &[Source::Roamer(0)]);
    for _ in 0..20 {
        match b.advance(&mut h) {
            Advance::NeedCmd(i) => {
                b.command(&mut h, i, Cmd::Attack(0));
            }
            Advance::End(_) => break,
            Advance::Msgs(_) => {}
        }
    }
    assert!(b.got.iter().all(|n| plus_split(n).1 == 0));
}

#[test]
fn defeating_miasma_monsters_overwrites_the_mimic_with_the_strongest() {
    let mut g = Game::new(510);
    g.mimic_known.insert("wan".into());
    g.party[1].mimic[1] = None;
    let plain = crate::mimic::bonus(1, mon("wan")).str_;
    g.set_mimic(1, 1, Some("wan")).unwrap();
    let s0 = g.party[1].str_total();
    put_miasma(&mut g, 3);
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "wan".into() });
    g.dbg.onehit = true;
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    for _ in 0..20 {
        match b.advance(&mut g) {
            Advance::NeedCmd(i) => {
                b.command(&mut g, i, Cmd::Attack(0));
            }
            Advance::End(_) => break,
            Advance::Msgs(_) => {}
        }
    }
    assert_eq!(g.party[1].mimic_plus.get("wan"), Some(&3));
    assert!(g.party[1].str_total() > s0, "+3 の ぶん つよい");
    assert_eq!(crate::mimic::bonus_plus(1, mon("wan"), 3).str_, (((plain * 12 + 9) / 10) * 12 + 9) / 10 * 12 / 10 + if (((plain * 12 + 9) / 10) * 12 + 9) / 10 * 12 % 10 > 0 { 1 } else { 0 });
    // 弱い +では 上書きされない(最大を ほぞん)
    g.dungeon.rooms.pop();
    g.dungeon.rooms.pop();
    g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "wan".into() });
    let mut b = Battle::new(&g, &[Source::Roamer(0)]);
    for _ in 0..20 {
        match b.advance(&mut g) {
            Advance::NeedCmd(i) => {
                b.command(&mut g, i, Cmd::Attack(0));
            }
            Advance::End(_) => break,
            Advance::Msgs(_) => {}
        }
    }
    assert_eq!(g.party[1].mimic_plus.get("wan"), Some(&3));
}

#[test]
fn miasma_generator_is_sold_after_5f_and_spawns_nothing() {
    let mut g = Game::new(520);
    g.reached = 5;
    assert!(!g.furniture_stock().contains(&MIASMA_ROOM));
    g.reached = 6;
    assert!(g.furniture_stock().contains(&MIASMA_ROOM));
    assert!(room_spawns(6, MIASMA_ROOM).is_empty());
    assert!(crate::script::furniture_text(MIASMA_ROOM).is_some());
}

#[test]
fn plus_items_can_be_equipped_sold_and_listed() {
    let mut g = Game::new(530);
    g.add_item("ナイフ+3", 1);
    assert!(g.owned_defs().iter().any(|d| d.name == "ナイフ+3"));
    g.equip(0, "ナイフ+3").unwrap();
    let atk_plus = g.party[0].atk();
    g.unequip(0, 0);
    g.equip(0, "ナイフ").ok();
    assert!(atk_plus >= g.party[0].atk());
    let gold = g.gold;
    g.add_item("ナイフ+3", 1);
    assert_eq!(g.sell("ナイフ+3"), Ok(item("ナイフ+3").unwrap().sell));
    assert_eq!(g.gold, gold + item("ナイフ+3").unwrap().sell);
    assert!(g.armor_stock.contains("ナイフ") && !g.armor_stock.contains("ナイフ+3"));
}

#[test]
fn jerry_chimes_in_only_after_he_joins() {
    use crate::script::*;
    let lines = |g: &Game| -> Vec<String> {
        sync_party(g);
        let mut all = vec![];
        for f in 1..=MAX_FLOOR {
            let (a, b) = floor_talks(f);
            all.extend(clean(a.iter().chain(b.iter()).map(|l| l.to_string()).collect(), true));
        }
        for q in QUESTS {
            all.extend(quest_done_lines(q, true, "ok"));
        }
        for f in 1..MAX_FLOOR {
            all.extend(next_boss_scene(f, true));
        }
        all
    };
    let mut g = Game::start_story(900);
    assert!(lines(&g).iter().all(|l| !l.starts_with("ジェリー「")), "仲間になる前は ジェリーは しゃべらない");
    g.party[1].joined = true;
    let with = lines(&g);
    assert!(with.iter().filter(|l| l.starts_with("ジェリー「")).count() >= 15, "あいづちが ところどころ ある");
    sync_party(&Game::start_story(901)); // もとに もどす
}

#[test]
fn mayor_tells_the_multi_battle_tip_once_from_day_three() {
    use crate::script::*;
    use crate::shop::ShopKind;
    let mut g = Game::start_story(950);
    for k in [ShopKind::Armor, ShopKind::Magic, ShopKind::Furniture, ShopKind::Plaza, ShopKind::Mayor] {
        let _ = on_enter(&mut g, k);
    }
    assert!(!battle_tip_pending(&g), "1日めは 言わない");
    g.day = 2;
    assert!(!battle_tip_pending(&g), "2日めも 言わない");
    g.day = 3;
    assert!(has_talk(&g, ShopKind::Mayor));
    let l = on_enter(&mut g, ShopKind::Mayor);
    assert!(l.iter().any(|x| x.contains("3びき いっしょに")));
    assert!(l.iter().all(|x| !x.starts_with("メイミ「")), "メイミが 仲間になる前は しゃべらない");
    assert!(on_enter(&mut g, ShopKind::Mayor).iter().all(|x| !x.contains("3びき いっしょに")), "一度だけ");
    let mut h = Game::start_story(951);
    h.party[2].joined = true;
    h.day = 3;
    for k in [ShopKind::Armor, ShopKind::Magic, ShopKind::Furniture, ShopKind::Plaza] {
        let _ = on_enter(&mut h, k);
    }
    h.talks_seen.insert(format!("hello:{}", "mayor"));
    assert!(on_enter(&mut h, ShopKind::Mayor).iter().any(|x| x.starts_with("メイミ「")), "メイミも 話す");
    // 戦闘の ドロップ倍率: 1匹100%・2匹150%・3匹200%(sim.rs finish_win)
    assert!(mon("wan").drops.len() > 0);
}

#[test]
fn chest_keys_drop_only_when_that_chest_is_on_the_floor() {
    let kill = |g: &mut Game| {
        g.dungeon.roamers.push(Roamer { x: 8, y: 11, mon: "bat".into() });
        g.dbg.onehit = true;
        let mut b = Battle::new(g, &[Source::Roamer(0)]);
        for _ in 0..20 {
            match b.advance(g) {
                Advance::NeedCmd(i) => {
                    b.command(g, i, Cmd::Attack(0));
                }
                Advance::End(_) => break,
                Advance::Msgs(_) => {}
            }
        }
        b.got.iter().filter(|n| n.contains("カギ")).count()
    };
    let mut none = 0;
    let mut some = 0;
    for seed in 0..600u64 {
        let mut g = Game::new(2000 + seed);
        none += kill(&mut g);
        let mut h = Game::new(2000 + seed);
        h.dungeon.rooms.push(Room { x: 2, y: 2, w: 1, h: 1, front: (2, 2), kind: "てつのたからばこ".into(), door: Dir::Down, open4: false, mons: vec![None], chest_open: false, refill: 0 });
        some += kill(&mut h);
    }
    assert_eq!(none, 0, "たからばこが ない階では カギは 落ちない");
    assert!(some > 0, "ある階では 落ちる");
}

#[test]
fn mei_spells_are_sold_only_after_she_joins_and_are_hers() {
    let mut g = Game::start_story(960);
    g.reached = 5;
    g.spells_open.insert("frost".into());
    let frost = spell("frost");
    assert!(!g.spell_for_sale(frost), "メイミが まだ いない");
    g.party[2].joined = true;
    assert!(g.spell_for_sale(frost));
    g.gold = 100000;
    g.buy_spell("frost").unwrap();
    assert!(g.party[2].spells.iter().any(|s| s == "frost") && !g.party[0].spells.iter().any(|s| s == "frost"));
}
