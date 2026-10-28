//! ゲームの中身(描画に依存しない)。ダンジョン編集・夜の湧き・戦闘・食事・店・クエスト・階の移動。

use crate::data::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SAVE_VERSION: u32 = 2;

// ---------------- 乱数 ----------------

#[derive(Clone, Serialize, Deserialize)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1)
    }
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
    /// lo..=hi
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % ((hi - lo + 1) as u32)) as i32
    }
    pub fn chance(&mut self, pct: u32) -> bool {
        self.next() % 100 < pct
    }
    pub fn pick<T: Copy>(&mut self, v: &[T]) -> T {
        v[(self.next() as usize) % v.len()]
    }
}

// ---------------- 方向・マップ ----------------

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];
    pub fn d(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }
    pub fn opposite(self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Tile {
    Rock,
    Bedrock,
    /// ボスが出るまで越えられない内周と外周の境目
    Barrier,
    Floor,
    Portal,
    StairsDown,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Room {
    /// 左上
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    /// 入口側の中央の1マス(1x1なら自分)
    pub front: (usize, usize),
    pub kind: String,
    /// 部屋から見て入口がある向き(1方向の部屋)
    pub door: Dir,
    /// 4方向から入れて、通り抜けられる部屋
    pub open4: bool,
    /// マスごとの魔物(行優先)
    pub mons: Vec<Option<String>>,
    pub chest_open: bool,
    /// 宝箱: 補充までの残り日数
    #[serde(default)]
    pub refill: i32,
}

impl Room {
    pub fn tiles(&self) -> Vec<(usize, usize)> {
        let mut v = vec![];
        for dy in 0..self.h {
            for dx in 0..self.w {
                v.push((self.x + dx, self.y + dy));
            }
        }
        v
    }
    pub fn slot_at(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h {
            Some((y - self.y) * self.w + (x - self.x))
        } else {
            None
        }
    }
    /// 1方向の部屋の戦闘位置(入口の前のマス)
    pub fn stand(&self) -> Option<(usize, usize)> {
        pos_add(self.front, self.door)
    }
    pub fn alive(&self) -> usize {
        self.mons.iter().filter(|m| m.is_some()).count()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Roamer {
    pub x: usize,
    pub y: usize,
    pub mon: String,
}

pub fn pos_add(p: (usize, usize), d: Dir) -> Option<(usize, usize)> {
    let (dx, dy) = d.d();
    let nx = p.0 as i32 + dx;
    let ny = p.1 as i32 + dy;
    if nx < 0 || ny < 0 || nx >= MAP_MAX as i32 || ny >= MAP_MAX as i32 {
        None
    } else {
        Some((nx as usize, ny as usize))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Dungeon {
    pub floor: usize,
    /// 内周の一辺(階ごとに広がる)。古いセーブは 9
    #[serde(default = "default_size")]
    pub size: usize,
    pub tiles: Vec<Tile>,
    pub rooms: Vec<Room>,
    pub roamers: Vec<Roamer>,
    /// まだ見つかっていない隠しアイテム
    pub hidden: Vec<(usize, usize, String)>,
    pub boss_defeated: bool,
    pub barrier_open: bool,
    pub portal: Option<(usize, usize)>,
    /// この階で倒した魔物の数
    pub kills: u32,
}

fn default_size() -> usize {
    9
}

impl Dungeon {
    pub fn new(floor: usize) -> Dungeon {
        Dungeon::with_size(floor, floor_size(floor))
    }

    pub fn with_size(floor: usize, size: usize) -> Dungeon {
        let w = size + 8;
        let (lo, hi) = (3, 4 + size); // バリアの輪(内周の1つ外)
        let mut tiles = vec![Tile::Rock; w * w];
        for y in 0..w {
            for x in 0..w {
                let edge = x == 0 || y == 0 || x == w - 1 || y == w - 1;
                let barrier = ((x == lo || x == hi) && (lo..=hi).contains(&y)) || ((y == lo || y == hi) && (lo..=hi).contains(&x));
                if edge {
                    tiles[y * w + x] = Tile::Bedrock;
                } else if barrier {
                    tiles[y * w + x] = Tile::Barrier;
                }
            }
        }
        let mut d = Dungeon { floor, size, tiles, rooms: vec![], roamers: vec![], hidden: vec![], boss_defeated: false, barrier_open: false, portal: None, kills: 0 };
        let e = d.entrance();
        d.set(e.0, e.1, Tile::Floor);
        // 隠しアイテムの位置は 9x9 のときの座標。右・下に寄った物は 広がった分だけ ずらす(「左下」は左下のまま)
        let grow = size.saturating_sub(9);
        d.hidden = hidden_items(floor).into_iter().map(|(x, y, n)| (if x > 8 { x + grow } else { x }, if y > 8 { y + grow } else { y }, n)).collect();
        d
    }
    /// 地図の幅・高さ(外周ふくむ)
    pub fn w(&self) -> usize {
        self.size + 8
    }
    pub fn h(&self) -> usize {
        self.size + 8
    }
    /// 内周(掘れる四角)の端
    pub fn inner(&self) -> (usize, usize) {
        (INNER_MIN, INNER_MIN + self.size - 1)
    }
    /// 入口(上り階段): 内周の下の端の中央
    pub fn entrance(&self) -> (usize, usize) {
        (INNER_MIN + self.size / 2, INNER_MIN + self.size - 1)
    }
    /// 魔穴の候補(右・左・上の端の中央)
    pub fn portal_spots(&self) -> [(usize, usize); 3] {
        [(self.w() - 2, self.h() / 2), (1, self.h() / 2), (self.w() / 2, 1)]
    }
    /// 別の大きさに作り直す。床・部屋・魔物・魔穴を 入口が重なるように ずらして写す
    pub fn resized(&self, size: usize) -> Dungeon {
        let mut n = Dungeon::with_size(self.floor, size);
        let (oe, ne) = (self.entrance(), n.entrance());
        let (dx, dy) = (ne.0 as i32 - oe.0 as i32, ne.1 as i32 - oe.1 as i32);
        let (nw, nh) = (n.w(), n.h());
        let map = |x: usize, y: usize| -> Option<(usize, usize)> {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if nx >= 0 && ny >= 0 && (nx as usize) < nw && (ny as usize) < nh {
                Some((nx as usize, ny as usize))
            } else {
                None
            }
        };
        for y in 0..self.h() {
            for x in 0..self.w() {
                let t = self.tile(x, y);
                // 内周の中身(床・魔穴・階段)だけ写す。壁と外周は 新しい地図のまま
                if matches!(t, Tile::Floor | Tile::Portal | Tile::StairsDown) {
                    if let Some((nx, ny)) = map(x, y) {
                        n.set(nx, ny, t);
                    }
                }
            }
        }
        n.rooms = self.rooms.clone();
        for r in n.rooms.iter_mut() {
            if let (Some(p), Some(f)) = (map(r.x, r.y), map(r.front.0, r.front.1)) {
                r.x = p.0;
                r.y = p.1;
                r.front = f;
            }
        }
        n.roamers = self.roamers.iter().filter_map(|r| map(r.x, r.y).map(|p| Roamer { x: p.0, y: p.1, mon: r.mon.clone() })).collect();
        n.hidden = self.hidden.iter().filter_map(|(x, y, s)| map(*x, *y).map(|p| (p.0, p.1, s.clone()))).collect();
        n.boss_defeated = self.boss_defeated;
        n.barrier_open = self.barrier_open;
        if n.barrier_open {
            for t in n.tiles.iter_mut() {
                if *t == Tile::Barrier {
                    *t = Tile::Rock;
                }
            }
        }
        // 魔穴は 縁にあるので、縁の同じ場所(右・左・上)へ
        n.portal = self.portal.and_then(|p| self.portal_spots().iter().position(|s| *s == p).map(|i| n.portal_spots()[i]).or_else(|| map(p.0, p.1)));
        if let Some(p) = n.portal {
            n.set(p.0, p.1, self.tile(self.portal.unwrap().0, self.portal.unwrap().1));
        }
        n.kills = self.kills;
        n
    }

    pub fn in_map(&self, x: usize, y: usize) -> bool {
        x < self.w() && y < self.h()
    }
    pub fn tile(&self, x: usize, y: usize) -> Tile {
        if !self.in_map(x, y) {
            return Tile::Bedrock;
        }
        self.tiles[y * self.w() + x]
    }
    pub fn set(&mut self, x: usize, y: usize, t: Tile) {
        if self.in_map(x, y) {
            let w = self.w();
            self.tiles[y * w + x] = t;
        }
    }
    pub fn room_at(&self, x: usize, y: usize) -> Option<usize> {
        self.rooms.iter().position(|r| r.slot_at(x, y).is_some())
    }
    pub fn walkable(&self, x: usize, y: usize) -> bool {
        if !matches!(self.tile(x, y), Tile::Floor | Tile::Portal | Tile::StairsDown) {
            return false;
        }
        match self.room_at(x, y) {
            None => true,
            Some(i) => self.rooms[i].open4,
        }
    }
    pub fn floor_count(&self) -> usize {
        self.tiles.iter().filter(|t| matches!(t, Tile::Floor)).count()
    }
    /// 入口から 歩いて行ける床(部屋でふさがれた行き止まりの穴は 含まない)
    pub fn reachable(&self) -> BTreeSet<(usize, usize)> {
        let mut seen: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut q = vec![self.entrance()];
        while let Some(p) = q.pop() {
            if !seen.insert(p) {
                continue;
            }
            for d in Dir::ALL {
                if let Some(n) = pos_add(p, d) {
                    if self.walkable(n.0, n.1) && !seen.contains(&n) {
                        q.push(n);
                    }
                }
            }
        }
        seen
    }
    pub fn roamer_at(&self, x: usize, y: usize) -> bool {
        self.roamers.iter().any(|r| r.x == x && r.y == y)
    }
}

// ---------------- パーティ ----------------

#[derive(Clone, Serialize, Deserialize)]
pub struct Member {
    pub name: String,
    pub hp: i32,
    pub hp_max: i32,
    pub mp: i32,
    pub mp_max: i32,
    pub str_: i32,
    pub spd: i32,
    pub con: i32,
    pub wis: i32,
    pub equip: [Option<String>; 6],
    pub spells: Vec<String>,
    pub poison: bool,
    #[serde(default)]
    pub stun: bool,
    pub can_equip: bool,
    /// 仲間になっているか(なっていない人は 戦わず、画面にも出ない)。古いセーブは 全員いる
    #[serde(default = "yes_joined")]
    pub joined: bool,
    /// 戦闘中だけの状態(セーブしない): つよくなれ / はねかえせ(1回) / ふういん(残りの行動回数)
    #[serde(skip)]
    pub strong: bool,
    #[serde(skip)]
    pub reflect: bool,
    #[serde(skip)]
    pub seal: u8,
    /// モノマネスライムか(ジェリー)。そうびは ものまねした部位でしか できない
    #[serde(default)]
    pub is_slime: bool,
    /// ものまね中の魔物(あたま・うで・からだ・あし)
    #[serde(default)]
    pub mimic: [Option<String>; 4],
    /// ものまねで 足した HP・MP(はずすとき ひく)
    #[serde(default)]
    pub mimic_hp: i32,
    #[serde(default)]
    pub mimic_mp: i32,
    /// 瘴気で 強くなった魔物を 倒したときの 最大の +(魔物の id → +n)。ものまねの 能力に 反映される
    #[serde(default)]
    pub mimic_plus: BTreeMap<String, u8>,
}

fn yes_joined() -> bool {
    true
}

impl Member {
    fn new(name: &str, hp: i32, mp: i32, s: i32, sp: i32, c: i32, w: i32, can_equip: bool, spells: &[&str]) -> Member {
        Member {
            name: name.to_string(),
            hp,
            hp_max: hp,
            mp,
            mp_max: mp,
            str_: s,
            spd: sp,
            con: c,
            wis: w,
            equip: Default::default(),
            spells: spells.iter().map(|s| s.to_string()).collect(),
            poison: false,
            stun: false,
            can_equip,
            joined: true,
            strong: false,
            reflect: false,
            seal: 0,
            is_slime: false,
            mimic: Default::default(),
            mimic_hp: 0,
            mimic_mp: 0,
            mimic_plus: Default::default(),
        }
    }
    /// 戦える状態か(仲間になっていない人は 戦えない)
    pub fn alive(&self) -> bool {
        self.joined && self.hp > 0
    }
    pub fn status_text(&self) -> &'static str {
        if !self.joined {
            ""
        } else if self.hp <= 0 {
            "うごけない"
        } else if self.seal > 0 {
            "ふういん"
        } else if self.poison {
            "どく"
        } else if self.stun {
            "ふらつき"
        } else if self.hp * 3 < self.hp_max {
            "ひんし"
        } else {
            "げんき"
        }
    }
    fn equipped(&self) -> impl Iterator<Item = &'static ItemDef> + '_ {
        self.equip.iter().flatten().filter_map(|n| item(n))
    }
    fn ring_bonus(&self, kind: u8) -> i32 {
        self.equipped().filter(|i| i.ring == kind).map(|i| i.power).sum()
    }
    /// ものまねで 増えた能力
    pub fn mimic_bonus(&self) -> crate::mimic::Bonus {
        let mut t = crate::mimic::Bonus::default();
        for (part, id) in self.mimic.iter().enumerate() {
            if let Some(mon) = id.as_deref().and_then(|id| MONS.iter().find(|m| m.id == id)) {
                let plus = self.mimic_plus.get(mon.id).copied().unwrap_or(0);
                let b = crate::mimic::bonus_plus(part, mon, plus);
                t.str_ += b.str_;
                t.wis += b.wis;
                t.con += b.con;
                t.spd += b.spd;
                t.hp += b.hp;
                t.mp += b.mp;
            }
        }
        t
    }
    pub fn str_total(&self) -> i32 {
        self.str_ + self.ring_bonus(1) + self.mimic_bonus().str_
    }
    pub fn con_total(&self) -> i32 {
        self.con + self.ring_bonus(2) + self.mimic_bonus().con
    }
    pub fn wis_total(&self) -> i32 {
        self.wis + self.ring_bonus(3) + self.mimic_bonus().wis
    }
    pub fn spd_total(&self) -> i32 {
        self.spd + self.mimic_bonus().spd
    }
    /// その部位(0=ぶき 1=たて 2=あたま 3=からだ 4=あし 5=ゆびわ)に そうびできるか
    pub fn can_wear(&self, slot: usize) -> bool {
        if !self.is_slime {
            return self.can_equip;
        }
        let m = |part: usize| self.mimic[part].as_deref().and_then(crate::mimic::mimic_of);
        match slot {
            0 | 1 => m(1).map(|x| x.arm_wear).unwrap_or(false),
            2 => m(0).map(|x| x.head_wear).unwrap_or(false),
            3 => m(2).map(|x| x.body_wear).unwrap_or(false),
            4 => m(3).map(|x| x.feet_wear).unwrap_or(false),
            _ => m(1).map(|x| x.arm_ring).unwrap_or(false),
        }
    }
    pub fn atk(&self) -> i32 {
        let base = self.str_total() + self.equipped().filter(|i| i.kind == Kind::Weapon).map(|i| i.power).sum::<i32>();
        if self.strong {
            base * 3 / 2
        } else {
            base
        }
    }
    /// 戦闘中の状態を もとにもどす
    pub fn clear_battle_status(&mut self) {
        self.strong = false;
        self.reflect = false;
        self.seal = 0;
    }
    pub fn def(&self) -> i32 {
        self.con_total()
            + self
                .equipped()
                .filter(|i| matches!(i.kind, Kind::Shield | Kind::Head | Kind::Body | Kind::Feet))
                .map(|i| i.power)
                .sum::<i32>()
    }
    /// 属性の耐性(%): 0=炎 1=氷 2=光
    pub fn resist(&self, elem: usize) -> i32 {
        if elem > 2 {
            return 0; // 物理攻撃(のしかかりなど)は耐性なし
        }
        let sum: i32 = self
            .equip
            .iter()
            .flatten()
            .map(|n| {
                let r = resist(n);
                [r.0, r.1, r.2][elem]
            })
            .sum();
        sum.min(85)
    }
}

// ---------------- 戦闘の型 ----------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    Roamer(usize),
    /// (部屋, マス)
    Room(usize, usize),
    Boss,
}

#[derive(Clone)]
pub struct Enemy {
    pub def: &'static MonDef,
    pub hp: i32,
    pub sleep: u32,
    pub stun: bool,
    pub drunk: bool,
    pub fled: bool,
    /// ふういんされている(魔法・特殊攻撃が つかえない)
    pub sealed: bool,
    /// 強さ(%)。まほうじんから出た魔物は 強い
    pub boost: i32,
    pub src: Source,
}

impl Enemy {
    pub fn max_hp(&self) -> i32 {
        self.def.hp * self.boost / 100
    }
    pub fn defense(&self) -> i32 {
        self.def.def * self.boost / 100
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Cmd {
    /// (特技で 行動ずみ。何もしない)
    Nothing,
    Attack(usize),
    Spell(&'static str, usize),
    Item(String, usize),
    Defend,
    Flee,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Actor {
    Party(usize),
    Foe(usize),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Won,
    Lost,
    Fled,
}

fn default_bgm_vol() -> u8 {
    6
}

fn default_sfx_vol() -> u8 {
    8
}

fn default_manual_log() -> bool {
    true
}

pub enum Advance {
    NeedCmd(usize),
    Msgs(Vec<String>),
    End(Outcome),
}

#[derive(Clone)]
pub struct Battle {
    pub enemies: Vec<Enemy>,
    pub order: Vec<Actor>,
    idx: usize,
    pub round: u32,
    pub defending: [bool; 3],
    pub outcome: Option<Outcome>,
    pub gold: i32,
    pub got: Vec<String>,
    /// この戦いのあとで 仲間になった人(番号)
    pub joined: Vec<usize>,
    pub boss: bool,
    pending_end: bool,
    ticked: [bool; 3],
    god: bool,
    onehit: bool,
}

#[derive(Clone, Debug)]
pub enum StepResult {
    Blocked,
    Moved,
    Battle(Vec<Source>),
    Chest(String),
    Descend,
    Stairs,
}

// ---------------- ゲーム全体 ----------------

/// 「ひとつもどす」用の操作記録。その操作だけを取り消す(持ち物全体は戻さない)
#[derive(Clone, Debug)]
pub enum Undo {
    Dig { x: usize, y: usize },
    Fill { x: usize, y: usize },
    Place { x: usize, y: usize, kind: String, converted: Vec<(usize, usize)>, mp: i32 },
    Remove { room: Room, mp: i32 },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    #[serde(default)]
    pub version: u32,
    pub day: u32,
    pub gold: i32,
    pub party: Vec<Member>,
    pub inv: BTreeMap<String, i32>,
    pub recipes: BTreeSet<String>,
    pub armor_stock: BTreeSet<String>,
    pub veg_stock: BTreeSet<String>,
    pub quests: BTreeMap<String, (u8, u32)>,
    pub visited_magic: bool,
    /// 戦闘のメッセージを、決定キー/クリックで進める(false なら時間で自動で流れる)
    #[serde(default = "default_manual_log")]
    pub manual_log: bool,
    /// 画面の並べ方。0=オート(横長の窓ならPC) 1=スマホ縦 2=PC横
    #[serde(default)]
    pub view_mode: u8,
    /// ダンジョンのマップを 3D(一人称)で見せる
    #[serde(default = "yes")]
    pub map3d: bool,
    /// 音。0=ぜんぶ 1=効果音だけ 2=なし
    #[serde(default)]
    pub sound_mode: u8,
    /// 町の人の話をもう聞いたか("order:クエストid" / "furn:部屋の名前")
    #[serde(default)]
    pub talks_seen: BTreeSet<String>,
    /// 魔法屋に食材を見せて 入荷した魔法
    #[serde(default)]
    pub spells_open: BTreeSet<String>,
    /// たおしたことのある魔物(ジェリーが ものまねできる)
    #[serde(default)]
    pub mimic_known: BTreeSet<String>,
    /// 音楽・効果音の音量(0〜10)
    #[serde(default = "default_bgm_vol")]
    pub bgm_vol: u8,
    #[serde(default = "default_sfx_vol")]
    pub sfx_vol: u8,
    pub mp: i32,
    pub mp_max: i32,
    /// いま居る階
    pub dungeon: Dungeon,
    /// 他の階(居る階は None)
    pub floors: Vec<Option<Dungeon>>,
    /// 降りたことのある一番深い階(1始まり)
    pub reached: usize,
    pub pos: (usize, usize),
    pub facing: Dir,
    #[serde(skip)]
    pub undo: Vec<Undo>,
    /// その日に食事をしたか(メンバーごと)
    #[serde(default)]
    pub ate: [bool; 3],
    pub rng: Rng,
    pub total_wipes: u32,
    #[serde(skip)]
    pub dbg: DbgFlags,
    /// 店で見たことのある品物(まだ見ていない新入荷に赤い!を出す)
    #[serde(default)]
    pub seen: BTreeSet<String>,
    /// これまでに 手に入れたことのある 魔物の食材(行商人が 売ってくれる)
    #[serde(default)]
    pub obtained_foods: BTreeSet<String>,
    /// 行商人から 買った もの("日付:品物")。1日1個まで
    #[serde(default)]
    pub peddler_bought: BTreeSet<String>,
}

/// デバッグ用の戦闘フラグ(セーブされない)
#[derive(Debug, Default, Clone, Copy)]
pub struct DbgFlags {
    pub god: bool,
    pub onehit: bool,
}

#[derive(Debug, Default, Clone)]
pub struct NightReport {
    pub spawned: usize,
    pub roamers: usize,
    /// 魔穴が開いた階
    pub portals: Vec<usize>,
    /// 夜のあいだに 仲間になった人(番号)
    pub joined: Vec<usize>,
    /// 村長から次のボスの知らせが届いた階(その階のボスを倒した知らせ)
    pub news: Vec<usize>,
    pub rating: i32,
}

fn yes() -> bool {
    true
}

/// ふつうの魔物の 強さの 倍率(%)
pub const NORMAL_BOOST: i32 = 130;

impl Game {
    /// はじめから(物語のはじまり): アレンひとり。メイミは3日めに、ジェリーは1Fのボスのあとに 仲間になる
    pub fn start_story(seed: u64) -> Game {
        let mut g = Game::new(seed);
        g.party[1].joined = false;
        g.party[2].joined = false;
        g.ate[1] = true;
        g.ate[2] = true;
        // ヘビリンゴの話は オープニングで済んでいる(村長の家で もう一度は 話さない)
        g.talks_seen.insert("order:snake".to_string());
        // 1日目は 武器なし。町のみんなに あいさつしながら 武具屋で ナイフを買う(script.rs の あいさつまわり)
        g.party[0].equip[0] = None;
        g.talks_seen.insert("tour".to_string());
        g
    }

    /// ボスを倒した階のうち、村長の知らせ(次のボスのこと)が まだ届いていない階を 届いたことにして返す
    pub fn take_news(&mut self) -> Vec<usize> {
        let mut v = vec![];
        for f in 1..=MAX_FLOOR {
            let beaten = self.floor_dungeon(f).map(|d| d.boss_defeated).unwrap_or(false);
            if beaten && self.talks_seen.insert(format!("news:{}", f)) {
                v.push(f);
            }
        }
        v
    }

    /// 仲間になっている人の番号(0=アレン 1=ジェリー 2=メイミ)
    pub fn members(&self) -> Vec<usize> {
        (0..self.party.len()).filter(|&i| self.party[i].joined).collect()
    }

    pub fn new(seed: u64) -> Game {
        let mut inv = BTreeMap::new();
        inv.insert("わらのベッド".to_string(), 1);
        inv.insert("エサおけ".to_string(), 1);
        inv.insert("ホウレンソウ".to_string(), 3);
        let mut g = Game {
            version: SAVE_VERSION,
            day: 1,
            gold: START_GOLD,
            party: vec![
                Member::new("アレン", 30, 8, 3, 4, 3, 5, true, &[]),
                Member::new("ジェリー", 24, 4, 3, 3, 3, 2, false, &[]),
                Member::new("メイミ", 26, 12, 2, 5, 2, 8, true, &["fire"]),
            ],
            inv,
            recipes: RECIPES.iter().filter(|r| r.taught_by.is_none()).map(|r| r.name.to_string()).collect(),
            armor_stock: ARMOR_STOCK0.iter().map(|s| s.to_string()).collect(),
            veg_stock: ["ホウレンソウ".to_string()].into_iter().collect(),
            quests: BTreeMap::new(),
            visited_magic: false,
            manual_log: true,
            view_mode: 0,
            map3d: true,
            sound_mode: 0,
            talks_seen: BTreeSet::new(),
            spells_open: BTreeSet::new(),
            mimic_known: BTreeSet::new(),
            bgm_vol: 6,
            sfx_vol: 8,
            mp: START_MP,
            mp_max: START_MP,
            dungeon: Dungeon::new(1),
            floors: (0..MAX_FLOOR).map(|_| None).collect(),
            reached: 1,
            pos: ENTRANCE,
            facing: Dir::Up,
            undo: vec![],
            ate: [false; 3],
            rng: Rng::new(seed),
            total_wipes: 0,
            dbg: DbgFlags::default(),
            seen: BTreeSet::new(),
            obtained_foods: BTreeSet::new(),
            peddler_bought: BTreeSet::new(),
        };
        g.party[1].is_slime = true;
        g.party[0].equip[0] = Some("ナイフ".to_string());
        g.quests.insert("snake".to_string(), (1, 1));
        g.mark_all_seen();
        g
    }

    pub fn floor_no(&self) -> usize {
        self.dungeon.floor
    }

    // ----- 持ち物 -----

    pub fn count(&self, name: &str) -> i32 {
        *self.inv.get(name).unwrap_or(&0)
    }

    pub fn add_item(&mut self, name: &str, n: i32) {
        *self.inv.entry(name.to_string()).or_insert(0) += n;
        if let Some(d) = item(name) {
            if d.kind == Kind::Veg {
                self.veg_stock.insert(name.to_string());
            }
            if d.kind == Kind::Food && n > 0 {
                self.obtained_foods.insert(name.to_string());
            }
        }
    }

    /// 今日 広場に 行商人が いるか(5の倍数の日)
    pub fn peddler_here(&self) -> bool {
        self.day % 5 == 0
    }

    /// 行商人の 品物(これまでに 手に入れた 食材。今日 まだ 買っていない もの。ねだん順)
    pub fn peddler_wares(&self) -> Vec<(String, i32)> {
        let mut v: Vec<(String, i32)> = self
            .obtained_foods
            .iter()
            .filter(|n| !self.peddler_bought.contains(&format!("{}:{}", self.day, n)))
            .filter_map(|n| peddler_price(n).map(|p| (n.clone(), p)))
            .collect();
        v.sort_by_key(|(_, p)| *p);
        v
    }

    pub fn peddler_buy(&mut self, name: &str) -> Result<i32, String> {
        if !self.peddler_here() {
            return Err("行商人は 今日は 来ていない".to_string());
        }
        let key = format!("{}:{}", self.day, name);
        if self.peddler_bought.contains(&key) {
            return Err("それは 1つだけ。もう 買ったよ".to_string());
        }
        let price = self.peddler_wares().into_iter().find(|(n, _)| n == name).map(|(_, p)| p).ok_or("そんな品は ない")?;
        if self.gold < price {
            return Err("おかねが足りない".to_string());
        }
        self.gold -= price;
        self.peddler_bought.insert(key);
        self.add_item(name, 1);
        Ok(price)
    }

    pub fn take_item(&mut self, name: &str, n: i32) -> bool {
        let c = self.count(name);
        if c < n {
            return false;
        }
        if c == n {
            self.inv.remove(name);
        } else {
            self.inv.insert(name.to_string(), c - n);
        }
        true
    }

    // ----- 店の品揃え(階の進み具合で増える) -----

    pub fn furniture_stock(&self) -> Vec<&'static str> {
        let mut v: Vec<&'static str> = FURNITURE_STOCK.to_vec();
        v.extend(FURNITURE_UNLOCK.iter().filter(|(f, _)| self.reached >= *f).map(|(_, n)| *n));
        v
    }

    pub fn magic_stock(&self) -> Vec<&'static str> {
        let mut v: Vec<&'static str> = MAGIC_SHOP_ITEMS.to_vec();
        v.extend(MAGIC_SHOP_UNLOCK.iter().filter(|(f, _)| self.reached >= *f).map(|(_, n)| *n));
        v
    }

    // ----- 新入荷・新しい食材(赤い!マーク) -----

    /// 店の品揃え(kind: armor / magic / furn / plaza)。魔法は「魔法:名前」
    pub fn stock_names(&self, kind: &str) -> Vec<String> {
        match kind {
            "armor" => ITEMS.iter().filter(|i| self.armor_stock.contains(i.name)).map(|i| i.name.to_string()).collect(),
            "magic" => {
                let mut v: Vec<String> = self.magic_stock().iter().map(|s| s.to_string()).collect();
                v.extend(SPELLS.iter().filter(|s| self.spell_for_sale(s)).map(|s| format!("魔法:{}", s.name)));
                v
            }
            "furn" => self.furniture_stock().iter().map(|s| s.to_string()).collect(),
            "plaza" => ITEMS.iter().filter(|i| self.veg_stock.contains(i.name)).map(|i| i.name.to_string()).collect(),
            _ => vec![],
        }
    }

    /// まだ見ていない新入荷
    pub fn shop_news(&self, kind: &str) -> Vec<String> {
        self.stock_names(kind).into_iter().filter(|n| !self.seen.contains(n)).collect()
    }

    /// 魔法屋に見せると新しい料理を教えてもらえる食材
    pub fn teachable(&self) -> Vec<&'static str> {
        ITEMS
            .iter()
            .filter(|i| matches!(i.kind, Kind::Food | Kind::Veg) && self.count(i.name) > 0)
            .filter(|i| RECIPES.iter().any(|r| r.taught_by == Some(i.name) && !self.recipes.contains(r.name)) || SPELL_TAUGHT_BY.iter().any(|(sp, ing)| *ing == i.name && !self.spells_open.contains(*sp)))
            .map(|i| i.name)
            .collect()
    }

    pub fn has_news(&self, kind: &str) -> bool {
        !self.shop_news(kind).is_empty() || (kind == "magic" && !self.teachable().is_empty())
    }

    /// 見たことにする(spells=true なら魔法だけ、false なら品物だけ)
    pub fn mark_seen(&mut self, kind: &str, spells: bool) {
        for n in self.stock_names(kind) {
            if n.starts_with("魔法:") == spells {
                self.seen.insert(n);
            }
        }
    }

    pub fn mark_all_seen(&mut self) {
        for k in ["armor", "magic", "furn", "plaza"] {
            for n in self.stock_names(k) {
                self.seen.insert(n);
            }
        }
    }

    /// 報告できるクエストの数
    pub fn reportable_quests(&self) -> usize {
        QUESTS.iter().filter(|q| self.quest_available(q) && (self.quest_state(q.id) == 1 || (q.main && self.quest_state(q.id) == 0)) && self.count(q.need) >= q.count).count()
    }

    // ----- 階の移動 -----

    pub fn change_floor(&mut self, to: usize) {
        let idx = to.wrapping_sub(1);
        if to == self.dungeon.floor || to < 1 || idx >= self.floors.len() {
            return;
        }
        let cur = self.dungeon.floor - 1;
        let next = self.floors[idx].take().unwrap_or_else(|| Dungeon::new(to));
        let live = std::mem::replace(&mut self.dungeon, next);
        self.floors[cur] = Some(live);
        self.reached = self.reached.max(to);
        if self.party[1].is_slime {
            self.refresh_mimic_spells(1);
        }
        // その階のメインクエストは、降りたら受けた状態になる
        for q in QUESTS.iter().filter(|q| q.main && q.min_floor <= to) {
            if self.quest_state(q.id) == 0 {
                self.quests.insert(q.id.to_string(), (1, self.day));
            }
        }
        self.undo.clear();
    }

    /// 下り階段から次の階へ
    pub fn descend(&mut self) {
        let to = self.dungeon.floor + 1;
        if to <= MAX_FLOOR {
            self.change_floor(to);
            self.pos = self.dungeon.entrance();
            self.facing = Dir::Up;
        }
    }

    /// 上り階段(入口)から1つ上の階へ。1階なら何もしない
    pub fn ascend(&mut self) -> bool {
        let f = self.dungeon.floor;
        if f <= 1 {
            return false;
        }
        self.change_floor(f - 1);
        self.pos = self.dungeon.portal.unwrap_or(self.dungeon.entrance());
        self.facing = Dir::Up;
        true
    }

    /// ワープ/リターン: 指定した階の入口へ
    pub fn warp(&mut self, floor: usize) {
        if floor >= 1 && floor <= self.reached {
            self.change_floor(floor);
            self.pos = self.dungeon.entrance();
            self.facing = Dir::Up;
        }
    }

    pub fn return_to_entrance(&mut self) {
        self.pos = self.dungeon.entrance();
        self.facing = Dir::Up;
    }

    // ----- ダンジョン編集 -----

    /// undo(成功したら、行けなくなった場所の 通路の魔物を けす)
    pub fn undo(&mut self) -> Result<String, String> {
        let r = self.undo_inner();
        if r.is_ok() {
            self.prune_roamers();
        }
        r
    }

    fn undo_inner(&mut self) -> Result<String, String> {
        let u = self.undo.pop().ok_or("もどせる操作がない")?;
        let occupied = |g: &Game, x: usize, y: usize| g.pos == (x, y) || g.dungeon.roamer_at(x, y) || g.dungeon.room_at(x, y).is_some();
        match u {
            Undo::Dig { x, y } => {
                if occupied(self, x, y) {
                    return Err("そこに何かいて もどせない".to_string());
                }
                self.dungeon.set(x, y, Tile::Rock);
                self.mp += DIG_COST;
            }
            Undo::Fill { x, y } => {
                self.dungeon.set(x, y, Tile::Floor);
                self.mp += FILL_COST;
            }
            Undo::Place { x, y, kind, converted, mp } => {
                let Some(i) = self.dungeon.rooms.iter().position(|r| r.x == x && r.y == y && r.kind == kind) else {
                    return Err("その部屋はもうない".to_string());
                };
                if self.dungeon.rooms[i].alive() > 0 {
                    return Err("魔物がいて もどせない".to_string());
                }
                self.dungeon.rooms.remove(i);
                for (cx, cy) in converted {
                    if self.pos != (cx, cy) && !self.dungeon.roamer_at(cx, cy) && self.dungeon.tile(cx, cy) == Tile::Floor {
                        self.dungeon.set(cx, cy, Tile::Rock);
                    }
                }
                self.add_item(&kind, 1);
                self.mp += mp;
            }
            Undo::Remove { room, mp } => {
                if room.tiles().iter().any(|&(x, y)| occupied(self, x, y)) {
                    return Err("そこに何かいて もどせない".to_string());
                }
                if !self.take_item(&room.kind, 1) {
                    return Err("部屋を持っていない".to_string());
                }
                self.dungeon.rooms.push(room);
                self.mp += mp;
            }
        }
        Ok("ひとつ前にもどした".to_string())
    }

    pub fn front(&self) -> Option<(usize, usize)> {
        pos_add(self.pos, self.facing)
    }

    /// 掘れる場所か(内周か、ボス出現後の外周)
    pub fn diggable(&self, x: usize, y: usize) -> bool {
        // 入口の真下は掘れない(町への出口を守る)
        let e = self.dungeon.entrance();
        if (x, y) == (e.0, e.1 + 1) {
            return false;
        }
        let (lo, hi) = self.dungeon.inner();
        if (lo..=hi).contains(&x) && (lo..=hi).contains(&y) {
            return true;
        }
        self.dungeon.barrier_open && x >= 1 && y >= 1 && x + 2 <= self.dungeon.w() && y + 2 <= self.dungeon.h()
    }

    /// Aボタン: 壁なら掘る、何もない床なら埋める
    /// 入口から歩けなくなった場所(部屋でふさいだ・埋めた通路)の 通路の魔物を 消す
    pub fn prune_roamers(&mut self) {
        if self.dungeon.roamers.is_empty() {
            return;
        }
        let reach = self.dungeon.reachable();
        self.dungeon.roamers.retain(|r| reach.contains(&(r.x, r.y)));
    }

    /// dig(成功したら、行けなくなった場所の 通路の魔物を けす)
    pub fn dig(&mut self) -> Result<String, String> {
        let r = self.dig_inner();
        if r.is_ok() {
            self.prune_roamers();
        }
        r
    }

    fn dig_inner(&mut self) -> Result<String, String> {
        let (x, y) = self.front().ok_or("掘れない壁だ")?;
        match self.dungeon.tile(x, y) {
            Tile::Rock => {
                if !self.diggable(x, y) {
                    return Err("掘れない壁だ".to_string());
                }
                if self.mp < DIG_COST {
                    return Err("MPが足りない".to_string());
                }
                self.mp -= DIG_COST;
                self.dungeon.set(x, y, Tile::Floor);
                if let Some(i) = self.dungeon.hidden.iter().position(|h| h.0 == x && h.1 == y) {
                    let (_, _, name) = self.dungeon.hidden.remove(i);
                    // 見つけた物があるので、この掘りは戻せない
                    self.undo.clear();
                    if let Some(g) = name.strip_prefix('G') {
                        let v: i32 = g.parse().unwrap_or(0);
                        self.gold += v;
                        return Ok(format!("壁の中から {}G を見つけた!", v));
                    }
                    self.add_item(&name, 1);
                    return Ok(format!("壁の中から {} を見つけた!", name));
                }
                self.undo.push(Undo::Dig { x, y });
                Ok("壁を掘った".to_string())
            }
            Tile::Floor => {
                if (x, y) == self.dungeon.entrance() {
                    return Err("入口は埋められない".to_string());
                }
                if self.dungeon.room_at(x, y).is_some() {
                    return Err("部屋がある。へや(B)でこわそう".to_string());
                }
                if self.dungeon.roamer_at(x, y) {
                    return Err("モンスターがいる".to_string());
                }
                if self.mp < FILL_COST {
                    return Err("MPが足りない".to_string());
                }
                self.mp -= FILL_COST;
                self.dungeon.set(x, y, Tile::Rock);
                self.undo.push(Undo::Fill { x, y });
                Ok("穴を埋めた".to_string())
            }
            _ => Err("ここは掘れない".to_string()),
        }
    }

    /// 部屋を置く場所(左上, 入口側の中央, 掘って広げる壁)。置けなければ理由
    #[allow(clippy::type_complexity)]
    /// この階に 置いてある 瘴気発生装置の 数(最大5)
    pub fn miasma_count(&self) -> usize {
        self.dungeon.rooms.iter().filter(|r| r.kind == MIASMA_ROOM).count().min(MAX_MIASMA)
    }

    fn room_block(&self, kind: &str) -> Result<((usize, usize), (usize, usize), Vec<(usize, usize)>), String> {
        if kind == MIASMA_ROOM && self.miasma_count() >= MAX_MIASMA {
            return Err(format!("瘴気発生装置は 1つの階に {}個までしか 置けない", MAX_MIASMA));
        }
        let def = room_def(kind);
        let f = self.front().ok_or("ここには置けない")?;
        let (x0, y0) = if def.w == 1 && def.h == 1 {
            f
        } else {
            let c = pos_add(f, self.facing).ok_or("ここには置けない")?;
            if c.0 < 1 || c.1 < 1 {
                return Err("ここには置けない".to_string());
            }
            (c.0 - 1, c.1 - 1)
        };
        let conv: Vec<(usize, usize)> = vec![];
        for dy in 0..def.h {
            for dx in 0..def.w {
                let (x, y) = (x0 + dx, y0 + dy);
                if !self.dungeon.in_map(x, y) {
                    return Err("ここには置けない".to_string());
                }
                if self.dungeon.room_at(x, y).is_some() {
                    return Err("もう部屋(物)が置いてあって、置けない".to_string());
                }
                if self.dungeon.roamer_at(x, y) {
                    return Err("魔物がいて、置けない".to_string());
                }
                if (x, y) == self.pos || (x, y) == self.dungeon.entrance() {
                    return Err("ここには置けない".to_string());
                }
                match self.dungeon.tile(x, y) {
                    Tile::Floor => {}
                    Tile::Rock if def.w > 1 => {
                        return Err(format!("{} は {}x{}マスぜんぶ 掘ってから置こう", kind, def.w, def.h));
                    }
                    _ => return Err("ここには置けない".to_string()),
                }
            }
        }
        Ok(((x0, y0), f, conv))
    }

    /// 置けない理由(置けるなら None)
    pub fn place_reason(&self, kind: &str) -> Option<String> {
        if let Err(e) = self.room_block(kind) {
            return Some(e);
        }
        let cost = room_def(kind).cost;
        if self.mp < cost {
            return Some(format!("MPが足りない(MP{}が必要)", cost));
        }
        None
    }

    pub fn can_place(&self, kind: &str) -> bool {
        self.room_block(kind).is_ok()
    }

    /// 前のマスに部屋を置けそうか(掘りずみの床で、物が置かれていない)
    pub fn can_place_here(&self) -> bool {
        match self.front() {
            Some((x, y)) => {
                self.dungeon.room_at(x, y).is_none()
                    && !self.dungeon.roamer_at(x, y)
                    && (x, y) != self.dungeon.entrance()
                    && self.dungeon.tile(x, y) == Tile::Floor
            }
            None => false,
        }
    }

    pub fn front_room(&self) -> Option<usize> {
        let (x, y) = self.front()?;
        self.dungeon.room_at(x, y)
    }

    /// 階段の近くで、魔物がすみつけない部屋か
    pub fn no_spawn_room(&self, r: &Room) -> bool {
        let e = (self.dungeon.entrance().0 as i32, self.dungeon.entrance().1 as i32);
        !r.tiles().iter().any(|&(x, y)| !self.no_spawn_tile(r, x, y, e))
    }

    fn no_spawn_tile(&self, r: &Room, x: usize, y: usize, e: (i32, i32)) -> bool {
        let cheb = (x as i32 - e.0).abs().max((y as i32 - e.1).abs());
        if cheb <= 1 {
            return true;
        }
        if !r.open4 {
            if let Some(s) = r.stand() {
                let manh = (s.0 as i32 - e.0).abs() + (s.1 as i32 - e.1).abs();
                if manh <= 1 {
                    return true;
                }
            }
        }
        false
    }

    /// place_room(成功したら、行けなくなった場所の 通路の魔物を けす)
    pub fn place_room(&mut self, kind: &str) -> Result<String, String> {
        let r = self.place_room_inner(kind);
        if r.is_ok() {
            self.prune_roamers();
        }
        r
    }

    fn place_room_inner(&mut self, kind: &str) -> Result<String, String> {
        if self.count(kind) < 1 {
            return Err("その部屋を持っていない".to_string());
        }
        let def = room_def(kind);
        let ((x0, y0), front, conv) = self.room_block(kind)?;
        if self.mp < def.cost {
            return Err("MPが足りない".to_string());
        }
        self.mp -= def.cost;
        self.take_item(kind, 1);
        let mut found: Vec<String> = vec![];
        for &(x, y) in &conv {
            self.dungeon.set(x, y, Tile::Floor);
            if let Some(i) = self.dungeon.hidden.iter().position(|h| h.0 == x && h.1 == y) {
                let (_, _, name) = self.dungeon.hidden.remove(i);
                if let Some(gil) = name.strip_prefix('G') {
                    let v: i32 = gil.parse().unwrap_or(0);
                    self.gold += v;
                    found.push(format!("{}G", v));
                } else {
                    self.add_item(&name, 1);
                    found.push(name);
                }
            }
        }
        let room = Room {
            x: x0,
            y: y0,
            w: def.w,
            h: def.h,
            front,
            kind: kind.to_string(),
            door: self.facing.opposite(),
            open4: def.open4,
            mons: vec![None; def.w * def.h],
            chest_open: false,
            refill: 0,
        };
        let warn = self.no_spawn_room(&room);
        self.dungeon.rooms.push(room);
        if found.is_empty() {
            self.undo.push(Undo::Place { x: x0, y: y0, kind: kind.to_string(), converted: conv, mp: def.cost });
        } else {
            // 見つけた物があるので、この操作は戻せない
            self.undo.clear();
            return Ok(format!("{} を置いた。壁の中から {} を見つけた!", kind, found.join("・")));
        }
        if warn {
            Ok(format!("{} を置いた(階段の近くには魔物がすみつかない)", kind))
        } else {
            Ok(format!("{} を置いた", kind))
        }
    }

    /// remove_room(成功したら、行けなくなった場所の 通路の魔物を けす)
    pub fn remove_room(&mut self) -> Result<String, String> {
        let r = self.remove_room_inner();
        if r.is_ok() {
            self.prune_roamers();
        }
        r
    }

    fn remove_room_inner(&mut self) -> Result<String, String> {
        let i = self.front_room().ok_or("こわす部屋がない")?;
        let cost = room_def(&self.dungeon.rooms[i].kind).cost;
        if self.mp < cost {
            return Err("MPが足りない".to_string());
        }
        self.mp -= cost;
        let r = self.dungeon.rooms.remove(i);
        self.add_item(&r.kind, 1);
        let kind = r.kind.clone();
        self.undo.push(Undo::Remove { room: r, mp: cost });
        Ok(format!("{} をこわした", kind))
    }

    // ----- 評価 -----

    /// (部屋, 種類ボーナス, 分かれ道, 曲がり角, 掘りすぎの減点)
    pub fn rating_parts(&self) -> (i32, i32, i32, i32, i32) {
        Game::parts_of(&self.dungeon)
    }

    pub fn parts_of(d: &Dungeon) -> (i32, i32, i32, i32, i32) {
        let rooms: i32 = d.rooms.iter().map(|r| if r.w * r.h > 1 { 5 } else { 3 }).sum();
        let kinds = d.rooms.iter().map(|r| r.kind.as_str()).collect::<BTreeSet<_>>().len() as i32;
        let mut junction = 0;
        let mut corner = 0;
        for y in 1..d.h() - 1 {
            for x in 1..d.w() - 1 {
                if d.tile(x, y) != Tile::Floor {
                    continue;
                }
                let open = |dx: i32, dy: i32| {
                    let (nx, ny) = ((x as i32 + dx) as usize, (y as i32 + dy) as usize);
                    d.tile(nx, ny) == Tile::Floor
                };
                let n = [open(0, -1), open(0, 1), open(-1, 0), open(1, 0)];
                let cnt = n.iter().filter(|b| **b).count();
                if cnt >= 3 {
                    junction += 1;
                } else if cnt == 2 && ((n[0] || n[1]) && (n[2] || n[3])) {
                    corner += 1;
                }
            }
        }
        let room_tiles: usize = d.rooms.iter().map(|r| r.w * r.h).sum();
        let dug = d.floor_count().saturating_sub(room_tiles) as i32;
        // 掘ってよい数は 内周の広さに比例(9x9 なら30)
        let free = (30 * (d.size * d.size) / 81) as i32;
        let pen = ((dug - free) / 2).max(0);
        (rooms, 3 * kinds, junction, corner / 2, pen)
    }

    pub fn rating(&self) -> i32 {
        let (r, k, j, c, p) = self.rating_parts();
        (r + k + j + c - p).max(0)
    }

    /// ボス出現条件の進み具合(名前, いま, 必要)
    pub fn boss_progress(&self) -> Vec<(String, i32, i32)> {
        Game::progress_of(&self.dungeon)
    }

    pub fn progress_of(d: &Dungeon) -> Vec<(String, i32, i32)> {
        let rule = boss_rule(d.floor);
        let (r, k, j, c, p) = Game::parts_of(d);
        let rating = (r + k + j + c - p).max(0);
        let mut v = vec![("評価".to_string(), rating, rule.rating), ("倒した数".to_string(), d.kills as i32, rule.kills as i32)];
        for (kind, n) in rule.rooms {
            let have = d.rooms.iter().filter(|r| r.kind == *kind).count() as i32;
            v.push((format!("{}の数", kind), have, *n as i32));
        }
        v
    }

    /// 指定した階のダンジョン(居る階でも、他の階でも)
    pub fn floor_dungeon(&self, f: usize) -> Option<&Dungeon> {
        if self.dungeon.floor == f {
            Some(&self.dungeon)
        } else {
            self.floors.get(f.wrapping_sub(1)).and_then(|d| d.as_ref())
        }
    }

    pub fn boss_ready(&self) -> bool {
        self.boss_progress().iter().all(|(_, a, b)| a >= b)
    }

    // ----- 移動 -----

    pub fn step(&mut self, dir: Dir) -> StepResult {
        self.facing = dir;
        let Some((x, y)) = pos_add(self.pos, dir) else { return StepResult::Blocked };
        if !self.dungeon.walkable(x, y) {
            return StepResult::Blocked;
        }
        self.pos = (x, y);
        self.check_encounter()
    }

    pub fn check_encounter(&mut self) -> StepResult {
        let (x, y) = self.pos;
        match self.dungeon.tile(x, y) {
            Tile::Portal if !self.dungeon.boss_defeated => return StepResult::Battle(vec![Source::Boss]),
            Tile::StairsDown => {
                return if self.dungeon.floor < MAX_FLOOR { StepResult::Descend } else { StepResult::Stairs };
            }
            _ => {}
        }
        let mut srcs: Vec<Source> = vec![];
        if let Some(i) = self.dungeon.roamers.iter().position(|r| r.x == x && r.y == y) {
            srcs.push(Source::Roamer(i));
        } else {
            // となりのマスの通路の魔物は 向かってくる
            for (i, r) in self.dungeon.roamers.iter().enumerate() {
                if (r.x as i32 - x as i32).abs() + (r.y as i32 - y as i32).abs() == 1 && srcs.len() < 3 {
                    srcs.push(Source::Roamer(i));
                }
            }
        }
        // 1方向の部屋: 入口の前に立つと魔物が出てくる
        for (i, r) in self.dungeon.rooms.iter().enumerate() {
            if r.open4 || r.stand() != Some((x, y)) {
                continue;
            }
            for (k, m) in r.mons.iter().enumerate() {
                if m.is_some() && srcs.len() < 3 {
                    srcs.push(Source::Room(i, k));
                }
            }
        }
        // 4方向の部屋: 上に乗るか、となりに立つと魔物が出てくる
        let mut near: Vec<(usize, usize)> = vec![];
        for (i, r) in self.dungeon.rooms.iter().enumerate() {
            if !r.open4 {
                continue;
            }
            if let Some(k) = r.slot_at(x, y) {
                if r.mons[k].is_some() {
                    near.insert(0, (i, k));
                }
            }
            for d in Dir::ALL {
                if let Some(p) = pos_add((x, y), d) {
                    if let Some(k) = r.slot_at(p.0, p.1) {
                        if r.mons[k].is_some() {
                            near.push((i, k));
                        }
                    }
                }
            }
        }
        for (i, k) in near {
            if srcs.len() < 3 && !srcs.contains(&Source::Room(i, k)) {
                srcs.push(Source::Room(i, k));
            }
        }
        if !srcs.is_empty() {
            return StepResult::Battle(srcs);
        }
        // 宝箱
        for i in 0..self.dungeon.rooms.len() {
            let r = &self.dungeon.rooms[i];
            let Some(t) = chest_tier(&r.kind) else { continue };
            let tier = &CHESTS[t];
            if !r.chest_open && r.stand() == Some((x, y)) && self.count(tier.key) > 0 {
                self.take_item(tier.key, 1);
                self.dungeon.rooms[i].chest_open = true;
                self.dungeon.rooms[i].refill = tier.refill_days;
                let name = self.rng.pick(chest_items(self.dungeon.floor, tier.deeper));
                self.add_item(name, 1);
                self.undo.clear();
                return StepResult::Chest(format!("{}で宝箱を開けた! {} を手に入れた", tier.key, name));
            }
        }
        StepResult::Moved
    }

    // ----- 夜 -----

    pub fn sleep(&mut self) -> NightReport {
        let mut rep = NightReport::default();
        for m in self.party.iter_mut() {
            m.hp = m.hp_max;
            m.mp = m.mp_max;
            m.poison = false;
            m.stun = false;
        }
        self.day += 1;
        self.ate = [false; 3];
        // メイミは 3日めの朝に アレンの家へ来て 仲間になる
        if self.day >= 3 && !self.party[2].joined {
            self.party[2].joined = true;
            rep.joined.push(2);
        }
        rep.news = self.take_news();
        for i in 0..self.party.len() {
            if !self.party[i].joined {
                self.ate[i] = true; // まだ いない人は ごはんの数に入れない
            }
        }
        self.mp = self.mp_max;
        self.undo.clear();
        let here = self.dungeon.floor;
        for f in 1..=MAX_FLOOR {
            if f != self.dungeon.floor {
                if self.floors[f - 1].is_none() {
                    continue;
                }
                self.change_floor(f);
            }
            self.night_here(&mut rep);
        }
        if self.dungeon.floor != here {
            self.change_floor(here);
        }
        rep.rating = self.rating();
        self.pos = self.dungeon.entrance();
        self.facing = Dir::Up;
        rep
    }

    pub fn night_here(&mut self, rep: &mut NightReport) {
        let floor = self.dungeon.floor;
        // 部屋に魔物がすみつく
        for i in 0..self.dungeon.rooms.len() {
            let room = self.dungeon.rooms[i].clone();
            if chest_tier(&room.kind).is_some() {
                continue;
            }
            let table = room_spawns(floor, &room.kind);
            if table.is_empty() {
                continue;
            }
            let e = (self.dungeon.entrance().0 as i32, self.dungeon.entrance().1 as i32);
            let cap: usize = if room.w * room.h > 1 { 4 } else { 1 };
            let mut free: Vec<usize> = (0..room.mons.len())
                .filter(|&k| room.mons[k].is_none())
                .filter(|&k| {
                    let t = (room.x + k % room.w, room.y + k / room.w);
                    !self.no_spawn_tile(&room, t.0, t.1, e)
                })
                .collect();
            let mut need = cap.saturating_sub(room.alive());
            while need > 0 && !free.is_empty() {
                let j = (self.rng.next() as usize) % free.len();
                let k = free.remove(j);
                let m = self.rng.pick(table);
                self.dungeon.rooms[i].mons[k] = Some(m.to_string());
                rep.spawned += 1;
                need -= 1;
            }
        }
        // 通路の魔物
        let rating = self.rating();
        let ent = self.dungeon.entrance();
        // 通路にいない魔物(部屋でふさがれた穴など)は 消す。新しく湧くのも 入口から歩ける通路だけ
        let reach = self.dungeon.reachable();
        self.dungeon.roamers.retain(|r| reach.contains(&(r.x, r.y)));
        let corridor: Vec<(usize, usize)> = (1..self.dungeon.h() - 1)
            .flat_map(|y| (1..self.dungeon.w() - 1).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                reach.contains(&(x, y))
                    && self.dungeon.tile(x, y) == Tile::Floor
                    && self.dungeon.room_at(x, y).is_none()
                    && (x as i32 - ent.0 as i32).abs() + (y as i32 - ent.1 as i32).abs() >= 3
            })
            .collect();
        let target = ((1 + rating / 10) as usize).min(4).min(corridor.len() / 5);
        let mut tries = 0;
        while self.dungeon.roamers.len() < target && tries < 50 {
            tries += 1;
            let p = self.rng.pick(&corridor);
            if self.dungeon.roamer_at(p.0, p.1) {
                continue;
            }
            let table = corridor_spawn(floor);
            if table.is_empty() {
                break;
            }
            let m = self.rng.pick(table);
            self.dungeon.roamers.push(Roamer { x: p.0, y: p.1, mon: m.to_string() });
            rep.roamers += 1;
        }
        // 開けた宝箱の補充: その階に魔物が一定数いる夜だけ、日数が進む
        let alive: usize = self.dungeon.rooms.iter().map(|r| r.alive()).sum::<usize>() + self.dungeon.roamers.len();
        for i in 0..self.dungeon.rooms.len() {
            let Some(t) = chest_tier(&self.dungeon.rooms[i].kind) else { continue };
            if !self.dungeon.rooms[i].chest_open {
                continue;
            }
            let tier = &CHESTS[t];
            if alive >= tier.need_monsters {
                // 金だけ: 魔物がとても多いと 2倍の早さ(最短でも4日)
                let step = if t == 2 && alive >= tier.need_monsters * 2 { 2 } else { 1 };
                self.dungeon.rooms[i].refill -= step;
                if self.dungeon.rooms[i].refill <= 0 {
                    self.dungeon.rooms[i].chest_open = false;
                }
            }
        }
        // 魔穴
        if !self.dungeon.boss_defeated && self.dungeon.portal.is_none() && self.boss_ready() {
            for t in self.dungeon.tiles.iter_mut() {
                if *t == Tile::Barrier {
                    *t = Tile::Rock;
                }
            }
            self.dungeon.barrier_open = true;
            let spots = self.dungeon.portal_spots();
            let p = self.rng.pick(&spots);
            self.dungeon.set(p.0, p.1, Tile::Portal);
            self.dungeon.portal = Some(p);
            rep.portals.push(floor);
        }
    }

    // ----- 店・食事 -----

    pub fn buy(&mut self, name: &str) -> Result<(), String> {
        let d = item(name).ok_or("そんな品はない")?;
        if self.gold < d.buy {
            return Err("おかねが足りない".to_string());
        }
        self.gold -= d.buy;
        self.add_item(name, 1);
        Ok(())
    }

    pub fn sell(&mut self, name: &str) -> Result<i32, String> {
        let d = item(name).ok_or("そんな品はない")?;
        if d.sell <= 0 {
            return Err("それは売れない".to_string());
        }
        if !self.take_item(name, 1) {
            return Err("持っていない".to_string());
        }
        self.gold += d.sell;
        if d.kind.is_equip() {
            self.armor_stock.insert(base_name(name).to_string());
        }
        Ok(d.sell)
    }

    pub fn can_cook(&self, r: &Recipe) -> bool {
        self.recipes.contains(r.name) && r.needs.iter().all(|n| self.count(n) >= needs_count(r, n))
    }

    pub fn cook(&mut self, recipe_name: &str, who: usize) -> Result<String, String> {
        if self.ate[who] {
            return Err(format!("{} は今日もう食べた", self.party[who].name));
        }
        let r = recipe(recipe_name).ok_or("そんな料理はない")?;
        if !self.can_cook(r) {
            return Err("材料が足りない".to_string());
        }
        for n in r.needs {
            self.take_item(n, 1);
        }
        self.ate[who] = true;
        let m = &mut self.party[who];
        let labels = ["HP", "MP", "つよさ", "すばやさ", "じょうぶさ", "かしこさ"];
        let mut parts = vec![];
        for (i, v) in r.gain.iter().enumerate() {
            if *v == 0 {
                continue;
            }
            match i {
                0 => {
                    m.hp_max += v;
                    m.hp += v;
                }
                1 => {
                    m.mp_max += v;
                    m.mp += v;
                }
                2 => m.str_ += v,
                3 => m.spd += v,
                4 => m.con += v,
                _ => m.wis += v,
            }
            parts.push(format!("{} +{}", labels[i], v));
        }
        Ok(format!("{} は {} を食べた! {}", m.name, r.name, parts.join(" ")))
    }

    /// 魔法屋に食材を見せる: 新しい料理を覚える
    pub fn show_ingredient(&mut self, name: &str) -> Result<String, String> {
        if !self.take_item(name, 1) {
            return Err("持っていない".to_string());
        }
        let mut learned = vec![];
        for r in RECIPES {
            if r.taught_by == Some(name) && self.recipes.insert(r.name.to_string()) {
                learned.push(r.name);
            }
        }
        let mut opened = vec![];
        for (sp, ing) in SPELL_TAUGHT_BY {
            if *ing == name && self.spells_open.insert(sp.to_string()) {
                opened.push(spell(sp).name);
            }
        }
        if learned.is_empty() && opened.is_empty() {
            self.add_item(name, 1);
            return Err("それはもう知っている料理じゃな".to_string());
        }
        let mut msg = format!("{} を渡した。", name);
        if !learned.is_empty() {
            msg += &format!("「{}」の作り方を教えてもらった!", learned.join("」「"));
        }
        if !opened.is_empty() {
            msg += &format!("魔法「{}」が 入荷した!", opened.join("」「"));
        }
        Ok(msg)
    }

    /// 食事のおまかせ: まだ食べていない人それぞれに、作れる中でいちばん伸びる料理を食べさせる
    pub fn auto_meals(&mut self) -> Vec<String> {
        let mut out = vec![];
        for who in 0..self.party.len() {
            if self.ate[who] || !self.party[who].joined {
                continue;
            }
            let mut best: Option<(&Recipe, i32)> = None;
            for r in RECIPES {
                if self.can_cook(r) {
                    let score: i32 = r.gain.iter().sum();
                    if best.map(|b| score > b.1).unwrap_or(true) {
                        best = Some((r, score));
                    }
                }
            }
            match best {
                Some((r, _)) => {
                    if let Ok(m) = self.cook(r.name, who) {
                        out.push(m);
                    }
                }
                None => {
                    out.push(format!("{} に 食べさせられる料理がない", self.party[who].name));
                    break;
                }
            }
        }
        if out.is_empty() {
            out.push("みんな もう食べた".to_string());
        }
        out
    }

    /// 持っている物から、その人のいちばん強い装備を選ぶ。入れ替えた数を返す
    pub fn best_equip(&mut self, who: usize) -> usize {
        let score = |d: &ItemDef| d.power * 10 + if d.ring == 4 { 5 } else { 0 };
        let mut changed = 0;
        for slot in 0..6 {
            if !self.party[who].can_wear(slot) {
                continue;
            }
            let cur = self.party[who].equip[slot].as_deref().and_then(item).map(score).unwrap_or(-1);
            let best = self.owned_defs().into_iter().filter(|d| d.kind.slot() == Some(slot)).max_by_key(|d| score(d));
            if let Some(d) = best {
                if score(d) > cur && self.equip(who, d.name).is_ok() {
                    changed += 1;
                }
            }
        }
        changed
    }

    /// 換金アイテム(毛皮・コインなど。分類が Loot の物だけ。装備・食材・クエストの品は売らない)をぜんぶ売る。(個数, 合計ギル)
    pub fn sell_loot(&mut self) -> (i32, i32) {
        let mut n = 0;
        let mut gold = 0;
        for d in ITEMS.iter().filter(|d| d.kind == Kind::Loot && d.sell > 0) {
            let c = self.count(d.name);
            if c > 0 {
                self.take_item(d.name, c);
                self.gold += d.sell * c;
                n += c;
                gold += d.sell * c;
            }
        }
        (n, gold)
    }

    pub fn shovel_polish(&mut self) -> Result<String, String> {
        if !self.take_item("魔法のオリーブオイル", 1) {
            return Err("オイルを持っていない".to_string());
        }
        self.mp_max += OIL_MP_GAIN;
        self.mp += OIL_MP_GAIN;
        Ok(format!("シャベルを磨いた! MPが {} になった", self.mp_max))
    }

    // ----- 装備 -----

    /// 持っている 品物の 定義(「ナイフ+3」のような +つきも ふくむ)
    pub fn owned_defs(&self) -> Vec<&'static ItemDef> {
        let mut v: Vec<&'static ItemDef> = ITEMS.iter().filter(|d| self.count(d.name) > 0).collect();
        let mut plus: Vec<&'static ItemDef> = self.inv.iter().filter(|(n, c)| **c > 0 && plus_split(n).1 > 0).filter_map(|(n, _)| item(n)).collect();
        plus.sort_by_key(|d| (plus_split(d.name).0.to_string(), plus_split(d.name).1));
        v.extend(plus);
        v
    }

    /// 装備したときの (今のATK, 後のATK, 今のDEF, 後のDEF)
    pub fn preview(&self, who: usize, name: &str) -> Option<(i32, i32, i32, i32)> {
        let d = item(name)?;
        let slot = d.kind.slot()?;
        let m = &self.party[who];
        if !m.can_wear(slot) {
            return None;
        }
        let mut n = m.clone();
        n.equip[slot] = Some(name.to_string());
        Some((m.atk(), n.atk(), m.def(), n.def()))
    }

    pub fn equip(&mut self, who: usize, name: &str) -> Result<(), String> {
        let d = item(name).ok_or("そんな装備はない")?;
        let slot = d.kind.slot().ok_or("装備できない")?;
        if !self.party[who].can_wear(slot) {
            return Err(if self.party[who].is_slime { "ものまねした部分に 合わないから そうびできない".to_string() } else { "そうびできない".to_string() });
        }
        if !self.take_item(name, 1) {
            return Err("持っていない".to_string());
        }
        if let Some(old) = self.party[who].equip[slot].take() {
            self.add_item(&old, 1);
        }
        self.party[who].equip[slot] = Some(name.to_string());
        Ok(())
    }

    pub fn unequip(&mut self, who: usize, slot: usize) {
        if let Some(old) = self.party[who].equip[slot].take() {
            self.add_item(&old, 1);
        }
    }

    // ----- ものまね(ジェリー) -----

    /// マネできる(たおしたことがあって、表にある)魔物の id
    pub fn mimic_choices(&self) -> Vec<&'static str> {
        MONS.iter().filter(|m| self.mimic_known.contains(m.id) && crate::mimic::can_mimic(m.id)).map(|m| m.id).collect()
    }

    /// ものまねの部位 part を 魔物 id(None なら もとのスライム)に かえる
    pub fn set_mimic(&mut self, who: usize, part: usize, id: Option<&str>) -> Result<String, String> {
        if !self.party[who].is_slime {
            return Err("ものまねできるのは ジェリーだけ".to_string());
        }
        if let Some(id) = id {
            if !self.mimic_known.contains(id) || !crate::mimic::can_mimic(id) {
                return Err("まだ その魔物を たおしていない".to_string());
            }
        }
        self.party[who].mimic[part] = id.map(|s| s.to_string());
        // HP・MP は 数をたして/ひいて あわせる
        let b = self.party[who].mimic_bonus();
        let m = &mut self.party[who];
        m.hp_max += b.hp - m.mimic_hp;
        m.mp_max += b.mp - m.mimic_mp;
        m.mimic_hp = b.hp;
        m.mimic_mp = b.mp;
        m.hp = m.hp.min(m.hp_max);
        m.mp = m.mp.min(m.mp_max);
        // 合わなくなった そうびは はずす
        for slot in 0..6 {
            if !self.party[who].can_wear(slot) {
                self.unequip(who, slot);
            }
        }
        self.refresh_mimic_spells(who);
        Ok(match id {
            Some(id) => format!("ジェリーは {} の {} を マネした!", mon(id).name, crate::mimic::PARTS[part]),
            None => format!("ジェリーの {} が もとのスライムに もどった", crate::mimic::PARTS[part]),
        })
    }

    /// あたまをマネしている魔物の魔法を おぼえる(一度おぼえたら わすれない)
    pub fn refresh_mimic_spells(&mut self, who: usize) {
        let Some(id) = self.party[who].mimic[0].clone() else { return };
        let Some(m) = crate::mimic::mimic_of(&id) else { return };
        for (sp, floor) in m.head_magic {
            if self.reached >= *floor && !self.party[who].spells.iter().any(|x| x == sp) {
                self.party[who].spells.push(sp.to_string());
            }
        }
    }

    /// おまかせ: 4か所とも、マネできる中でいちばん能力が増える魔物に かえる
    pub fn auto_mimic(&mut self, who: usize) -> usize {
        let mut changed = 0;
        for part in 0..4 {
            let score = |id: &str| {
                let b = crate::mimic::bonus_plus(part, mon(id), self.party[who].mimic_plus.get(id).copied().unwrap_or(0));
                let m = crate::mimic::mimic_of(id).unwrap();
                let magic = if part == 0 { m.head_magic.len() as i32 * 40 } else { 0 };
                let skill = if part == 1 && m.arm_skill.is_some() { 10 } else { 0 };
                b.str_ * 3 + b.wis * 2 + b.hp + b.con * 3 + b.spd * 3 + b.mp / 2 + magic + skill
            };
            let best = self.mimic_choices().into_iter().max_by_key(|id| score(id));
            if let Some(id) = best {
                if self.party[who].mimic[part].as_deref() != Some(id) && self.set_mimic(who, part, Some(id)).is_ok() {
                    changed += 1;
                }
            }
        }
        changed
    }

    // ----- クエスト -----

    pub fn quest_state(&self, id: &str) -> u8 {
        self.quests.get(id).map(|q| q.0).unwrap_or(0)
    }

    pub fn quest_available(&self, q: &Quest) -> bool {
        self.reached >= q.min_floor
    }

    pub fn accept_quest(&mut self, id: &str) {
        if self.quest_state(id) == 0 && self.quest_available(quest(id)) {
            let q = quest(id);
            self.gold += q.prepay;
            self.quests.insert(id.to_string(), (1, self.day));
        }
    }

    pub fn deliver_quest(&mut self, id: &str) -> Result<String, String> {
        let q = quest(id);
        if self.quest_state(id) != 1 && !(q.main && self.quest_state(id) == 0) {
            return Err("依頼を受けていない".to_string());
        }
        if self.count(q.need) < q.count || !self.take_item(q.need, q.count) {
            return Err(format!("{} が足りない({}個ひつよう)", q.need, q.count));
        }
        self.gold += q.reward_gold;
        let mut msg = format!("「{}」を達成!", q.title);
        if q.reward_gold > 0 {
            msg += &format!(" {}Gもらった。", q.reward_gold);
        }
        if let Some(it) = q.reward_item {
            self.add_item(it, 1);
            msg += &format!(" {} をもらった。", it);
        }
        if let Some(sp) = q.reward_spell {
            let owner = spell(sp).user;
            if !self.party[owner].spells.iter().any(|x| x == sp) {
                self.party[owner].spells.push(sp.to_string());
                msg += &format!(" 魔法「{}」をおぼえた!", spell(sp).name);
            }
        }
        self.quests.insert(id.to_string(), (2, self.day));
        Ok(msg)
    }

    pub fn spell_for_sale(&self, s: &Spell) -> bool {
        if s.price == 0 || !self.party[s.user].joined || self.party[s.user].spells.iter().any(|x| x == s.id) {
            return false;
        }
        match s.id {
            "sleep" => self.quests.get("slime").map(|q| q.0 == 2 && self.day > q.1).unwrap_or(false),
            "holy" => self.reached >= 2,
            "strong" => self.reached >= 6,
            id if SPELL_TAUGHT_BY.iter().any(|(sp, _)| *sp == id) => self.spells_open.contains(id),
            _ => true,
        }
    }

    pub fn buy_spell(&mut self, id: &str) -> Result<(), String> {
        let s = spell(id);
        if self.gold < s.price {
            return Err("おかねが足りない".to_string());
        }
        self.gold -= s.price;
        self.party[s.user].spells.push(id.to_string());
        Ok(())
    }

    /// 全滅したときの処理
    pub fn wipe(&mut self) {
        self.total_wipes += 1;
        self.undo.clear();
        self.gold -= self.gold / 10;
        for m in self.party.iter_mut() {
            m.hp = m.hp.max(1);
            m.poison = false;
            m.stun = false;
            m.clear_battle_status();
        }
        self.warp(1);
        self.return_to_entrance();
    }

    // ----- セーブ -----

    pub fn save_text(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// 階の大きさの予定(floor_size)が変わっていたら、その階を新しい大きさに写す
    fn migrate_floor_sizes(&mut self) {
        let cur = self.dungeon.floor;
        let want = floor_size(cur);
        if self.dungeon.size != want {
            self.dungeon = self.dungeon.resized(want);
            self.pos = self.dungeon.entrance();
        }
        for d in self.floors.iter_mut().flatten() {
            let want = floor_size(d.floor);
            if d.size != want {
                *d = d.resized(want);
            }
        }
    }

    /// 端末ごとの好みの設定(音量・画面・戦闘ログ)。ほかのセーブに 引きつぐ
    pub fn settings(&self) -> (u8, u8, u8, u8, bool, bool) {
        (self.bgm_vol, self.sfx_vol, self.sound_mode, self.view_mode, self.manual_log, self.map3d)
    }
    pub fn set_settings(&mut self, s: (u8, u8, u8, u8, bool, bool)) {
        self.map3d = s.5;
        self.bgm_vol = s.0;
        self.sfx_vol = s.1;
        self.sound_mode = s.2;
        self.view_mode = s.3;
        self.manual_log = s.4;
    }

    pub fn load_text(s: &str) -> Option<Game> {
        let g: Game = serde_json::from_str(s).ok()?;
        if g.version != SAVE_VERSION {
            return None;
        }
        let mut g = g;
        while g.floors.len() < MAX_FLOOR {
            g.floors.push(None);
        }
        g.migrate_floor_sizes();
        if g.party.len() > 1 && g.party[1].name == "ジェリー" {
            g.party[1].is_slime = true; // 古いセーブ
        }
        // 話を聞く前のセーブ: 受けているクエストと、見たことのある部屋は 聞いたことにする
        if g.talks_seen.is_empty() {
            let mut heard: Vec<String> = QUESTS.iter().filter(|q| g.quest_state(q.id) >= 1).map(|q| format!("order:{}", q.id)).collect();
            heard.extend(g.furniture_stock().into_iter().filter(|n| g.seen.contains(*n)).map(|n| format!("furn:{}", n)));
            g.talks_seen.extend(heard);
        }
        if g.seen.is_empty() {
            g.mark_all_seen();
        }
        // 行商人の仕組みより前の セーブ: いま 持っている 食材は 手に入れたことにする
        let owned: Vec<String> = g.inv.keys().filter(|n| item(n).map(|d| d.kind == Kind::Food).unwrap_or(false)).cloned().collect();
        g.obtained_foods.extend(owned);
        // 知らせの仕組みより前のセーブ: すでにボスを倒した階の知らせは 届いたことにする
        if !g.talks_seen.iter().any(|s| s.starts_with("news:")) {
            let _ = g.take_news();
        }
        Some(g)
    }
}

pub fn needs_count(r: &Recipe, name: &str) -> i32 {
    r.needs.iter().filter(|n| **n == name).count() as i32
}

// ---------------- 戦闘ロジック ----------------

impl Battle {
    pub fn new(g: &Game, srcs: &[Source]) -> Battle {
        let mut enemies = vec![];
        let mut boss = false;
        for s in srcs {
            let (id, kind) = match *s {
                Source::Boss => {
                    boss = true;
                    (boss_id(g.dungeon.floor).to_string(), String::new())
                }
                Source::Roamer(i) => (g.dungeon.roamers[i].mon.clone(), String::new()),
                Source::Room(i, k) => (g.dungeon.rooms[i].mons[k].clone().unwrap_or_default(), g.dungeon.rooms[i].kind.clone()),
            };
            let def = mon(&id);
            // 部屋ごとの開始状態: わらのベッド=寝ている / さかば=よっぱらい
            // 部屋のとくせい: ベッド=寝ている / さかば=よっぱらい / けっかいのへや=ふういん / まほうじん=魔物が最高に強い
            let sleep = if kind == "わらのベッド" || kind == "りっぱなベッド" { 2 } else { 0 };
            let mut boost = if kind == "まほうじん" { 150 } else { 100 };
            // ふつうの 魔物は ボスに くらべて 弱すぎたので、全体に 少し 強くする(HP・ATK・DEF が 1.3倍。ボスは そのまま)
            if !def.boss {
                boost = boost * NORMAL_BOOST / 100;
            }
            // 瘴気発生装置: 1個ごとに 魔物が 1.2倍(切り上げ)
            for _ in 0..g.miasma_count() {
                boost = (boost * 12 + 9) / 10;
            }
            enemies.push(Enemy { def, hp: def.hp * boost / 100, sleep, stun: false, drunk: kind == "さかば", fled: false, sealed: kind == "けっかいのへや", boost, src: *s });
        }
        // 行動順は戦闘ごとに固定(すばやさ順)
        let mut order: Vec<(i32, Actor)> = vec![];
        for (i, m) in g.party.iter().enumerate() {
            if m.joined {
                order.push((m.spd_total(), Actor::Party(i)));
            }
        }
        for (j, e) in enemies.iter().enumerate() {
            order.push((e.def.spd, Actor::Foe(j)));
        }
        order.sort_by(|a, b| b.0.cmp(&a.0));
        Battle {
            enemies,
            order: order.into_iter().map(|o| o.1).collect(),
            idx: 0,
            round: 1,
            defending: [false; 3],
            outcome: None,
            gold: 0,
            got: vec![],
            joined: vec![],
            boss,
            pending_end: false,
            ticked: [false; 3],
            god: g.dbg.god,
            onehit: g.dbg.onehit,
        }
    }

    /// 「おまかせ」の行動: 瀕死の仲間を回復 → アンデットに光 → 魔法 → いちばん弱い敵を攻撃
    pub fn auto_cmd(&self, g: &Game, who: usize) -> Cmd {
        let alive: Vec<usize> = (0..g.party.len()).filter(|&k| g.party[k].alive()).collect();
        let hurt = alive.iter().copied().min_by_key(|&k| g.party[k].hp * 100 / g.party[k].hp_max).unwrap_or(who);
        let frac = g.party[hurt].hp * 100 / g.party[hurt].hp_max;
        let m = &g.party[who];
        // ふういんされていると 魔法は使えない
        let knows = |id: &str| m.seal == 0 && m.spells.iter().any(|s| s == id) && m.mp >= spell(id).mp;
        let weak = alive.iter().filter(|&&k| g.party[k].hp * 100 / g.party[k].hp_max < 60).count();
        if weak >= 2 && knows("healall") {
            return Cmd::Spell("healall", who);
        }
        if frac < 40 {
            if knows("heal") {
                return Cmd::Spell("heal", hurt);
            }
            if g.count("かいふくやく") > 0 {
                return Cmd::Item("かいふくやく".to_string(), hurt);
            }
        }
        // どく・ふらつき・ふういんを なおす
        if knows("cure") {
            if let Some(&k) = alive.iter().find(|&&k| g.party[k].poison || g.party[k].stun || g.party[k].seal > 0) {
                return Cmd::Spell("cure", k);
            }
        }
        // ボス戦では いちばん強い攻撃役に つよくなれ
        if self.boss && knows("strong") {
            if let Some(&k) = alive.iter().filter(|&&k| !g.party[k].strong).max_by_key(|&&k| g.party[k].atk()) {
                return Cmd::Spell("strong", k);
            }
        }
        let targets = self.alive_enemies();
        let tgt = targets.iter().copied().min_by_key(|&j| self.enemies[j].hp).unwrap_or(0);
        if self.enemies[tgt].def.undead && knows("holy") {
            return Cmd::Spell("holy", tgt);
        }
        if targets.len() >= 2 {
            for id in ["blizzard", "firestorm"] {
                if knows(id) {
                    return Cmd::Spell(id, tgt);
                }
            }
        }
        for id in ["frost", "fire"] {
            if knows(id) && (id == "fire" || !knows("fire")) {
                return Cmd::Spell(id, tgt);
            }
        }
        Cmd::Attack(tgt)
    }

    pub fn alive_enemies(&self) -> Vec<usize> {
        (0..self.enemies.len()).filter(|&i| self.enemies[i].hp > 0 && !self.enemies[i].fled).collect()
    }

    fn check_end(&mut self, g: &mut Game) -> bool {
        if self.alive_enemies().is_empty() {
            for m in g.party.iter_mut() {
                m.clear_battle_status();
            }
            self.finish_win(g);
            self.outcome = Some(Outcome::Won);
            return true;
        }
        if g.party.iter().all(|m| !m.alive()) {
            for m in g.party.iter_mut() {
                m.clear_battle_status();
            }
            self.outcome = Some(Outcome::Lost);
            return true;
        }
        false
    }

    fn finish_win(&mut self, g: &mut Game) {
        let beaten: Vec<&Enemy> = self.enemies.iter().filter(|e| !e.fled).collect();
        let miasma = g.miasma_count();
        for e in &beaten {
            // ボスは まほうじんで 出た 個体を 倒したときだけ ものまねできる(本物の ボス戦では 覚えない)
            let from_circle = matches!(e.src, Source::Room(i, _) if g.dungeon.rooms[i].kind == "まほうじん");
            if !e.def.boss || from_circle {
                g.mimic_known.insert(e.def.id.to_string());
            }
            // 瘴気で 強くなった魔物を 倒すと、その 最大の 強さで ものまねの 能力が 上書きされる
            if miasma > 0 && g.party.len() > 1 && (!e.def.boss || from_circle) {
                let ent = g.party[1].mimic_plus.entry(e.def.id.to_string()).or_insert(0);
                *ent = (*ent).max(miasma as u8);
            }
        }
        let n = beaten.len().max(1) as u32;
        let mut gold = 0;
        let mut got: Vec<String> = vec![];
        for e in &beaten {
            gold += e.def.gold;
            let mult = 100 + 50 * (n - 1);
            // れいぞうこから出た魔物は 食べ物を落としやすい(2倍)
            let fridge = matches!(e.src, Source::Room(i, _) if g.dungeon.rooms[i].kind == "れいぞうこ");
            for (name, pct) in e.def.drops {
                let food = item(name).map(|d| matches!(d.kind, Kind::Food | Kind::Veg)).unwrap_or(false);
                let p = (pct * mult / 100 * if fridge && food { 2 } else { 1 }).min(100);
                // クエスト用の品は、必要な 数だけ(持っている・この戦闘で もう 出た ぶんを ふくむ)。達成ずみの クエストの 品は 出ない
                if let Some(d) = item(name) {
                    if d.kind == Kind::Quest {
                        let have = g.count(name) + got.iter().filter(|x| x.as_str() == *name).count() as i32;
                        let wanted = QUESTS.iter().any(|q| q.need == *name && g.quest_state(q.id) != 2 && have < q.count);
                        if !wanted {
                            continue;
                        }
                    }
                }
                if g.rng.chance(p) {
                    // 瘴気発生装置: 装備には +1〜+(置いた数)が つく
                    let is_equip = item(name).map(|d| d.kind.is_equip()).unwrap_or(false);
                    if is_equip && miasma > 0 {
                        let k = g.rng.range(1, miasma as i32);
                        got.push(format!("{}+{}", name, k));
                    } else {
                        got.push(name.to_string());
                    }
                }
            }
            if !e.def.boss {
                for tier in CHESTS.iter() {
                    // その階に 同じ種類の たからばこが 置いてあるときだけ カギを 落とす
                    let has_chest = g.dungeon.rooms.iter().any(|r| r.kind == tier.room);
                    if has_chest && g.dungeon.floor >= tier.drop_from_floor && g.rng.chance(tier.drop_pct) {
                        got.push(tier.key.to_string());
                    }
                }
            }
        }
        g.undo.clear();
        g.gold += gold;
        self.gold = gold;
        for name in &got {
            g.add_item(name, 1);
        }
        self.got = got;
        // 倒した魔物をダンジョンから消す(インデックスずれを避けて後ろから)
        let mut roam_clear = vec![];
        for e in &self.enemies {
            match e.src {
                Source::Room(i, k) => g.dungeon.rooms[i].mons[k] = None,
                Source::Roamer(i) => roam_clear.push(i),
                Source::Boss => {
                    // 1Fのボスのあと、なついたスライム(ジェリー)が 仲間になる
                    if g.dungeon.floor == 1 && !g.party[1].joined {
                        g.party[1].joined = true;
                        g.ate[1] = false;
                        self.joined.push(1);
                    }
                    g.dungeon.boss_defeated = true;
                    if let Some(p) = g.dungeon.portal {
                        g.dungeon.set(p.0, p.1, Tile::StairsDown);
                    }
                }
            }
            if !e.def.boss && !e.fled {
                g.dungeon.kills += 1;
            }
        }
        roam_clear.sort_unstable_by(|a, b| b.cmp(a));
        for i in roam_clear {
            g.dungeon.roamers.remove(i);
        }
        // フロアの魔物を狩りつくしたら、中身のある宝箱の鍵がかならず出る(持っている鍵が足りないときだけ)
        let left: usize = g.dungeon.rooms.iter().map(|r| r.alive()).sum::<usize>() + g.dungeon.roamers.len();
        if left == 0 && !self.boss {
            for tier in CHESTS.iter() {
                let filled = g.dungeon.rooms.iter().filter(|r| r.kind == tier.room && !r.chest_open).count() as i32;
                if filled > 0 && g.count(tier.key) < filled {
                    g.add_item(tier.key, 1);
                    self.got.push(format!("{}(フロアの魔物をかりつくした!)", tier.key));
                }
            }
        }
        for m in g.party.iter_mut() {
            if !m.alive() {
                m.hp = 1;
            }
            m.stun = false;
        }
    }

    /// 次に進める。コマンド待ちか、メッセージか、決着を返す
    pub fn advance(&mut self, g: &mut Game) -> Advance {
        if let Some(o) = self.outcome.clone() {
            return Advance::End(o);
        }
        if self.pending_end {
            for m in g.party.iter_mut() {
                m.clear_battle_status();
            }
            return Advance::End(Outcome::Fled);
        }
        loop {
            if self.idx >= self.order.len() {
                self.idx = 0;
                self.round += 1;
                self.defending = [false; 3];
            }
            let actor = self.order[self.idx];
            match actor {
                Actor::Party(i) => {
                    if !g.party[i].alive() {
                        self.ticked[i] = false;
                        self.idx += 1;
                        continue;
                    }
                    self.defending[i] = false;
                    if g.party[i].poison && !self.ticked[i] {
                        self.ticked[i] = true;
                        let d = g.rng.range(1, 3);
                        g.party[i].hp = (g.party[i].hp - d).max(0);
                        let msg = format!("{} は どくで {} のダメージ!", g.party[i].name, d);
                        if !g.party[i].alive() {
                            self.ticked[i] = false;
                            self.idx += 1;
                            if self.check_end(g) {
                                return Advance::Msgs(vec![msg]);
                            }
                            return Advance::Msgs(vec![msg, format!("{} はうごけなくなった", g.party[i].name)]);
                        }
                        return Advance::Msgs(vec![msg]);
                    }
                    if g.party[i].stun {
                        g.party[i].stun = false;
                        self.ticked[i] = false;
                        self.idx += 1;
                        return Advance::Msgs(vec![format!("{} は ふらついて うごけない!", g.party[i].name)]);
                    }
                    return Advance::NeedCmd(i);
                }
                Actor::Foe(j) => {
                    self.idx += 1;
                    if self.enemies[j].hp <= 0 || self.enemies[j].fled {
                        continue;
                    }
                    if self.enemies[j].sleep > 0 {
                        self.enemies[j].sleep -= 1;
                        return Advance::Msgs(vec![format!("{} は ねむっている", self.enemies[j].def.name)]);
                    }
                    if self.enemies[j].stun {
                        self.enemies[j].stun = false;
                        return Advance::Msgs(vec![format!("{} は ふらついている", self.enemies[j].def.name)]);
                    }
                    if self.enemies[j].drunk && g.rng.chance(40) {
                        self.enemies[j].sleep = 1;
                        return Advance::Msgs(vec![format!("{} は よっぱらって ねてしまった", self.enemies[j].def.name)]);
                    }
                    let hps: Vec<i32> = g.party.iter().map(|m| m.hp).collect();
                    let msgs = self.foe_act(g, j);
                    if self.god {
                        for (m, h) in g.party.iter_mut().zip(hps) {
                            m.hp = h;
                            m.poison = false;
                            m.stun = false;
                        }
                    }
                    self.check_end(g);
                    return Advance::Msgs(msgs);
                }
            }
        }
    }

    /// 全員への属性攻撃(0=炎 1=氷 2=光)。耐性で減る。stun なら ふらつく
    #[allow(clippy::too_many_arguments)]
    fn elem_all(&mut self, g: &mut Game, alive: &[usize], who: &str, skill: &str, base: i32, elem: usize, stun: bool) -> Vec<String> {
        let mut msgs = vec![format!("{} の {}!", who, skill)];
        for &i in alive {
            if elem < 3 && g.party[i].reflect {
                g.party[i].reflect = false;
                msgs.push(format!("{} は {} を はねかえした!", g.party[i].name, skill));
                continue;
            }
            let pct = g.party[i].resist(elem);
            let mut dmg = ((base - g.party[i].def() / 2) * (100 - pct) / 100).max(1);
            if self.defending[i] {
                dmg = (dmg + 1) / 2;
            }
            g.party[i].hp = (g.party[i].hp - dmg).max(0);
            msgs.push(format!("{} に {} のダメージ!", g.party[i].name, dmg));
            if g.party[i].alive() {
                if stun {
                    g.party[i].stun = true;
                }
            } else {
                msgs.push(format!("{} は うごけなくなった!", g.party[i].name));
            }
        }
        msgs
    }

    #[cfg(test)]
    pub fn test_elem_all(&mut self, g: &mut Game, alive: &[usize], who: &str, skill: &str, base: i32, elem: usize) -> Vec<String> {
        self.elem_all(g, alive, who, skill, base, elem, false)
    }

    /// 1人への強い一撃(攻撃力の mul/den 倍)。stun ならふらつく
    fn heavy_hit(&mut self, g: &mut Game, t: usize, who: &str, skill: &str, atk: i32, stun: bool) -> Vec<String> {
        let mut dmg = (atk - g.party[t].def()).max(1);
        if self.defending[t] {
            dmg = (dmg + 1) / 2;
        }
        g.party[t].hp = (g.party[t].hp - dmg).max(0);
        let mut m = vec![format!("{} の {}! {} に {} のダメージ!", who, skill, g.party[t].name, dmg)];
        if g.party[t].alive() {
            if stun {
                g.party[t].stun = true;
            }
        } else {
            m.push(format!("{} は うごけなくなった!", g.party[t].name));
        }
        m
    }

    /// 全員への物理攻撃(攻撃力 atk)。ふらつき付き
    fn all_hit(&mut self, g: &mut Game, alive: &[usize], who: &str, skill: &str, atk: i32, stun: bool) -> Vec<String> {
        self.elem_all(g, alive, who, skill, atk, 3, stun)
    }

    /// 毒の全体攻撃(ダメージは小さい。毒状態にする)
    fn poison_all(&mut self, g: &mut Game, alive: &[usize], who: &str, skill: &str, base: i32) -> Vec<String> {
        let mut msgs = self.elem_all(g, alive, who, skill, base, 3, false);
        for &i in alive {
            if g.party[i].alive() && !g.party[i].poison && !g.party[i].reflect {
                g.party[i].poison = true;
                msgs.push(format!("{} は どくをうけた!", g.party[i].name));
            }
        }
        msgs
    }

    /// いのちいただき: 1人に ダメージ。与えたぶんの 半分だけ 自分のHPが回復する
    fn drain(&mut self, g: &mut Game, j: usize, t: usize, who: &str, atk: i32) -> Vec<String> {
        if g.party[t].reflect {
            g.party[t].reflect = false;
            return vec![format!("{} の いのちいただき! {} は はねかえした!", who, g.party[t].name)];
        }
        let mut m = self.heavy_hit(g, t, who, "いのちいただき", atk, false);
        let dmg = (atk - g.party[t].def()).max(1);
        let e = &mut self.enemies[j];
        e.hp = (e.hp + dmg / 2).min(e.max_hp());
        m.push(format!("{} は HPを すいとった!", who));
        m
    }

    /// ふういんの術: 味方の魔法を ふうじる(rounds 回の行動のあいだ)。全員か、1人
    fn seal_party(&mut self, g: &mut Game, targets: &[usize], who: &str, skill: &str, rounds: u8) -> Vec<String> {
        let mut msgs = vec![format!("{} の {}!", who, skill)];
        for &i in targets {
            if g.party[i].reflect {
                g.party[i].reflect = false;
                msgs.push(format!("{} は {} を はねかえした!", g.party[i].name, skill));
            } else if g.party[i].alive() {
                g.party[i].seal = g.party[i].seal.max(rounds);
                msgs.push(format!("{} は ふういんされた! 魔法が つかえない!", g.party[i].name));
            }
        }
        msgs
    }

    /// ギリギリこうせん: 1人のHPが 1になる
    fn hp_one(&mut self, g: &mut Game, t: usize, who: &str) -> Vec<String> {
        if g.party[t].reflect {
            g.party[t].reflect = false;
            return vec![format!("{} の ギリギリこうせん! {} は はねかえした!", who, g.party[t].name)];
        }
        g.party[t].hp = 1;
        vec![format!("{} の ギリギリこうせん! {} の HPが 1に なった!", who, g.party[t].name)]
    }

    fn foe_act(&mut self, g: &mut Game, j: usize) -> Vec<String> {
        let alive: Vec<usize> = (0..g.party.len()).filter(|&i| g.party[i].alive()).collect();
        if alive.is_empty() {
            return vec![];
        }
        let t = g.rng.pick(&alive);
        let (id, name, atk, hp, maxhp) = {
            let e = &self.enemies[j];
            (e.def.id, e.def.name, e.def.atk * e.boost / 100, e.hp, e.max_hp())
        };
        let mut msgs = vec![];
        // 逃げることがある
        if matches!(id, "ghost" | "hotghost" | "coldghost" | "crowcat" | "wizdrill" | "platypus" | "pepper") && g.rng.chance(12) {
            self.enemies[j].fled = true;
            return vec![format!("{} は にげだした!", name)];
        }
        let sealed = self.enemies[j].sealed;
        if sealed {
            msgs.push(format!("{} は ふういんされていて 特殊な技が つかえない!", name));
        }
        // 新しいボスの技
        match id {
            _ if sealed => {}
            "dino" => {
                if g.rng.chance(30) {
                    return self.elem_all(g, &alive, name, "ようがんこうげき", 40, 0, false);
                }
                if g.rng.chance(25) {
                    return self.elem_all(g, &alive, name, "のしかかり", atk * 3 / 5, 3, true);
                }
            }
            "sorcerer" => {
                if g.rng.chance(40) {
                    return self.elem_all(g, &alive, name, "こおりのつえ", 42, 1, false);
                }
            }
            "tetsu" => {
                if g.rng.chance(30) {
                    return self.elem_all(g, &alive, name, "きょうれつなひかり", 45, 2, false);
                }
                if g.rng.chance(20) {
                    let mut dmg = (atk * 3 / 2 - g.party[t].def()).max(1);
                    if self.defending[t] {
                        dmg = (dmg + 1) / 2;
                    }
                    g.party[t].hp = (g.party[t].hp - dmg).max(0);
                    let mut m = vec![format!("{} の きょうれつないちげき! {} に {} のダメージ!", name, g.party[t].name, dmg)];
                    if g.party[t].alive() {
                        g.party[t].stun = true;
                    } else {
                        m.push(format!("{} は うごけなくなった!", g.party[t].name));
                    }
                    return m;
                }
            }
            // 6F〜10F のボス(技の強さは攻撃力に比例)
            "debrin" => {
                if g.rng.chance(30) {
                    return self.elem_all(g, &alive, name, "ほのおのつぼ", atk * 11 / 10, 0, false);
                }
            }
            "octopus" => {
                if g.rng.chance(30) {
                    return self.elem_all(g, &alive, name, "うずしお", atk * 11 / 10, 1, false);
                }
                if g.rng.chance(25) {
                    return self.elem_all(g, &alive, name, "しめつけ", atk * 3 / 5, 3, true);
                }
            }
            "kinglion" => {
                if g.rng.chance(35) {
                    return self.elem_all(g, &alive, name, "フライングクロウ", atk * 3 / 5, 3, true);
                }
            }
            "dragon" => {
                if g.rng.chance(30) {
                    return self.elem_all(g, &alive, name, "ドラゴンブレス", atk * 11 / 10, 0, false);
                }
                if g.rng.chance(20) {
                    let mut dmg = (atk * 3 / 2 - g.party[t].def()).max(1);
                    if self.defending[t] {
                        dmg = (dmg + 1) / 2;
                    }
                    g.party[t].hp = (g.party[t].hp - dmg).max(0);
                    let mut m = vec![format!("{} の しっぽなぎはらい! {} に {} のダメージ!", name, g.party[t].name, dmg)];
                    if g.party[t].alive() {
                        g.party[t].stun = true;
                    } else {
                        m.push(format!("{} は うごけなくなった!", g.party[t].name));
                    }
                    return m;
                }
            }
            "devilprince" => {
                let r = g.rng.range(0, 100);
                if r < 22 {
                    return self.elem_all(g, &alive, name, "ヘルファイア", atk * 11 / 10, 0, false);
                } else if r < 40 {
                    return self.elem_all(g, &alive, name, "ブリザード", atk * 11 / 10, 1, false);
                } else if r < 52 {
                    return self.elem_all(g, &alive, name, "ダークホーリー", atk, 2, false);
                } else if r < 64 {
                    return self.elem_all(g, &alive, name, "あくまのたたき", atk * 3 / 5, 3, true);
                }
            }
            // 11F〜20F のボス
            "dorunuma" => {
                if g.rng.chance(35) {
                    return self.all_hit(g, &alive, name, "ドロのなみ", atk * 3 / 5, true);
                }
                if g.rng.chance(35) {
                    return self.heavy_hit(g, t, name, "ぬまにひきずりこむ", atk * 4 / 3, true);
                }
            }
            "shogunboss" => {
                if g.rng.chance(35) {
                    return self.all_hit(g, &alive, name, "ほえたけり", atk * 3 / 5, false);
                }
            }
            "devilprincess" => {
                let r = g.rng.range(0, 100);
                if r < 30 {
                    return self.poison_all(g, &alive, name, "どくのきり", atk / 2);
                } else if r < 50 {
                    return self.drain(g, j, t, name, atk * 2 / 3);
                }
            }
            "hitotsuoni" => {
                let r = g.rng.range(0, 100);
                if r < 25 {
                    return self.all_hit(g, &alive, name, "ぶんまわし", atk * 3 / 5, true);
                } else if r < 35 {
                    return self.hp_one(g, t, name);
                } else {
                    let stun = g.rng.chance(50);
                    return self.heavy_hit(g, t, name, "こうげき", atk, stun);
                }
            }
            "flameflower" => {
                let r = g.rng.range(0, 100);
                if r < 28 {
                    return self.elem_all(g, &alive, name, "ほのおのはなびら", atk * 4 / 5, 0, false);
                } else if r < 55 {
                    return self.all_hit(g, &alive, name, "つるのなぎはらい", atk * 3 / 4, false);
                }
            }
            "kage" => {
                let r = g.rng.range(0, 100);
                if r < 25 {
                    return self.elem_all(g, &alive, name, "ひかりのかげ", atk * 4 / 5, 2, false);
                } else if r < 50 {
                    return self.heavy_hit(g, t, name, "かげのいちげき", atk * 7 / 5, false);
                }
            }
            "mitsuoni" => {
                let r = g.rng.range(0, 100);
                if r < 28 {
                    let mut m = self.all_hit(g, &alive, name, "ふういんビーム", atk / 3, false);
                    m.extend(self.seal_party(g, &alive, name, "ふういん", 3));
                    return m;
                } else if r < 50 {
                    return self.all_hit(g, &alive, name, "ぶんまわし", atk * 4 / 5, false);
                } else if r < 70 {
                    return self.heavy_hit(g, t, name, "ひっさつのいちげき", atk * 5 / 4, false);
                }
            }
            "elephantom" => {
                let r = g.rng.range(0, 100);
                if r < 30 {
                    return self.elem_all(g, &alive, name, "じごくの炎", atk, 0, false);
                } else if r < 50 {
                    return self.heavy_hit(g, t, name, "のしかかり", atk * 7 / 5, true);
                } else if r < 65 {
                    return self.all_hit(g, &alive, name, "あしぶみ", atk * 3 / 4, false);
                }
            }
            "devillord" => {
                let r = g.rng.range(0, 100);
                if r < 22 {
                    return self.elem_all(g, &alive, name, "じごくのひかり", atk * 4 / 5, 2, false);
                } else if r < 42 {
                    // めいおうひっさつけん: いちばんHPが高い仲間を狙う
                    let top = *alive.iter().max_by_key(|&&i| g.party[i].hp).unwrap();
                    return self.heavy_hit(g, top, name, "めいおうひっさつけん", atk * 3 / 2, true);
                } else if r < 57 {
                    return self.drain(g, j, t, name, atk * 2 / 3);
                } else if r < 69 {
                    return self.seal_party(g, &[t], name, "ふういんのじゅつ", 6);
                }
            }
            "kingsnake" => {
                let r = g.rng.range(0, 100);
                if r < 20 {
                    return self.poison_all(g, &alive, name, "どくリンゴのしる", atk / 2);
                } else if r < 38 {
                    return self.elem_all(g, &alive, name, "ほのおのいき", atk, 0, false);
                } else if r < 52 {
                    return self.drain(g, j, t, name, atk * 2 / 3);
                } else if r < 66 {
                    return self.all_hit(g, &alive, name, "とぐろのしめつけ", atk * 3 / 5, true);
                }
            }
            _ => {}
        }
        if id == "boss" && !sealed && hp * 2 < maxhp && g.rng.chance(40) {
            let pct = g.party[t].resist(2);
            let dmg = (11 * (100 - pct) / 100).max(1);
            g.party[t].hp = (g.party[t].hp - dmg).max(0);
            msgs.push(format!("{} は ひかりのまほうを となえた! {} に {} のダメージ!", name, g.party[t].name, dmg));
            if pct >= 40 {
                msgs.push("光をやわらげた".to_string());
            }
        } else if id == "lion" && !sealed && g.rng.chance(35) {
            msgs.push(format!("{} の フライングクロウ! みんなが ひきさかれた!", name));
            for i in alive.iter().copied() {
                let mut dmg = (atk * 3 / 5 - g.party[i].def()).max(1);
                if self.defending[i] {
                    dmg = (dmg + 1) / 2;
                }
                g.party[i].hp = (g.party[i].hp - dmg).max(0);
                msgs.push(format!("{} に {} のダメージ!", g.party[i].name, dmg));
                if g.party[i].alive() {
                    g.party[i].stun = true;
                } else {
                    msgs.push(format!("{} は うごけなくなった!", g.party[i].name));
                }
            }
            return msgs;
        } else if id == "potato" && !sealed && g.rng.chance(25) {
            msgs.push(format!("{} は 炎バクダンを なげた!", name));
            for i in alive.iter().copied() {
                let dmg = (8 - g.party[i].def() / 2).max(1);
                g.party[i].hp = (g.party[i].hp - dmg).max(0);
                msgs.push(format!("{} に {} のダメージ!", g.party[i].name, dmg));
                if !g.party[i].alive() {
                    msgs.push(format!("{} は うごけなくなった!", g.party[i].name));
                }
            }
            return msgs;
        } else {
            let mut dmg = (atk - g.party[t].def() + g.rng.range(-1, 1)).max(1);
            if self.defending[t] {
                dmg = (dmg + 1) / 2;
            }
            g.party[t].hp = (g.party[t].hp - dmg).max(0);
            msgs.push(format!("{} の こうげき! {} に {} のダメージ!", name, g.party[t].name, dmg));
            // ワームシールド: なぐられると とげで 3〜5ダメージを はね返す
            if g.party[t].equip[1].as_deref().map(base_name) == Some("ワームシールド") && self.enemies[j].hp > 0 {
                let back = g.rng.range(3, 5);
                self.hit_enemy(j, back);
                msgs.push(format!("{} の ワームシールドが はんげき! {} に {} のダメージ!", g.party[t].name, name, back));
                if self.enemies[j].hp <= 0 {
                    msgs.push(format!("{} を たおした!", name));
                }
            }
            if id == "boss" && g.rng.chance(30) && !g.party[t].poison && g.party[t].alive() {
                g.party[t].poison = true;
                msgs.push(format!("{} は どくをうけた!", g.party[t].name));
            }
        }
        if !g.party[t].alive() {
            msgs.push(format!("{} は うごけなくなった!", g.party[t].name));
        }
        msgs
    }

    fn hit_enemy(&mut self, t: usize, dmg: i32) {
        let dmg = if self.onehit { 9999 } else { dmg };
        let e = &mut self.enemies[t];
        e.hp = (e.hp - dmg).max(0);
        e.sleep = 0;
    }

    fn first_target(&self, t: usize) -> usize {
        if self.enemies[t].hp > 0 && !self.enemies[t].fled {
            t
        } else {
            self.alive_enemies()[0]
        }
    }

    /// ジェリーの特技(うで 35% / あたま 25%)。使ったら メッセージを返す
    fn try_slime_skill(&mut self, g: &mut Game, who: usize, t: usize) -> Option<Vec<String>> {
        use crate::mimic::*;
        let arm = g.party[who].mimic[1].as_deref().and_then(mimic_of).and_then(|m| m.arm_skill);
        let head = g.party[who].mimic[0].as_deref().and_then(mimic_of).and_then(|m| m.head_skill);
        let sk = if arm.is_some() && g.rng.chance(35) {
            arm?
        } else if head.is_some() && g.rng.chance(25) {
            head?
        } else {
            return None;
        };
        let name = g.party[who].name.clone();
        let atk = g.party[who].atk();
        let wis = g.party[who].wis_total();
        let mut msgs = vec![];
        let t = self.first_target(t);
        let rnd = g.rng.range(-1, 2);
        match sk.kind {
            SkillKind::Heal => {
                let hurt = (0..g.party.len()).filter(|&k| g.party[k].alive()).min_by_key(|&k| g.party[k].hp * 100 / g.party[k].hp_max)?;
                if g.party[hurt].hp * 100 / g.party[hurt].hp_max >= 70 {
                    return None;
                }
                let amount = 15 + wis;
                let before = g.party[hurt].hp;
                g.party[hurt].hp = (before + amount).min(g.party[hurt].hp_max);
                msgs.push(format!("{} の {}! {} のHPが {} かいふく!", name, sk.name, g.party[hurt].name, g.party[hurt].hp - before));
            }
            SkillKind::All(_) | SkillKind::FireAll => {
                msgs.push(format!("{} の {}!", name, sk.name));
                for k in self.alive_enemies() {
                    let dmg = match sk.kind {
                        SkillKind::FireAll => (3 + wis * 3 / 5 + rnd - self.enemies[k].defense() / 2).max(1),
                        SkillKind::All(p) => (atk * p / 100 - self.enemies[k].defense() + rnd).max(1),
                        _ => 1,
                    };
                    msgs.push(format!("{} に {} のダメージ!", self.enemies[k].def.name, dmg));
                    self.hit_enemy(k, dmg);
                    if self.enemies[k].hp <= 0 {
                        msgs.push(format!("{} を たおした!", self.enemies[k].def.name));
                    }
                }
            }
            kind => {
                let en = self.enemies[t].def.name;
                let def = self.enemies[t].defense();
                let (dmg, hits, extra) = match kind {
                    SkillKind::Pierce => ((atk - def / 2 + rnd).max(1), 1, ""),
                    SkillKind::Blunt => (((atk - def) * 13 / 10 + rnd).max(1), 1, "stun"),
                    SkillKind::Big(p) => ((atk * p / 100 - def + rnd).max(1), 1, ""),
                    SkillKind::Double => ((atk - def + rnd).max(1), 2, ""),
                    SkillKind::Fire(p) | SkillKind::Ice(p) => (((wis + atk / 3) * p / 100 + rnd - def / 2).max(1), 1, ""),
                    SkillKind::Sleep => ((atk - def + rnd).max(1), 1, "sleep"),
                    SkillKind::OneHp => (0, 1, "onehp"),
                    _ => (1, 1, ""),
                };
                if extra == "onehp" {
                    let ok = g.rng.chance(if self.enemies[t].def.boss { 10 } else { 30 });
                    msgs.push(format!("{} の {}!", name, sk.name));
                    if ok {
                        self.enemies[t].hp = 1;
                        msgs.push(format!("{} の HPが 1に なった!", en));
                    } else {
                        msgs.push(format!("{} には きかなかった", en));
                    }
                } else {
                    for _ in 0..hits {
                        if self.enemies[t].hp <= 0 {
                            break;
                        }
                        msgs.push(format!("{} の {}! {} に {} のダメージ!", name, sk.name, en, dmg));
                        self.hit_enemy(t, dmg);
                    }
                    if self.enemies[t].hp <= 0 {
                        msgs.push(format!("{} を たおした!", en));
                    } else if extra == "stun" && g.rng.chance(25) {
                        self.enemies[t].stun = true;
                        msgs.push(format!("{} は ふらついた!", en));
                    } else if extra == "sleep" && !self.enemies[t].def.boss && g.rng.chance(40) {
                        self.enemies[t].sleep = 2;
                        msgs.push(format!("{} は ねむってしまった!", en));
                    }
                }
            }
        }
        Some(msgs)
    }

    /// 1人の行動。ワームシールドを 持っていると 行動のたびに 1ダメージ くらう
    pub fn command(&mut self, g: &mut Game, who: usize, cmd: Cmd) -> Vec<String> {
        let mut msgs = self.command_core(g, who, cmd);
        if g.party[who].alive() && g.party[who].equip[1].as_deref().map(base_name) == Some("ワームシールド") {
            g.party[who].hp = (g.party[who].hp - 1).max(0);
            msgs.push(format!("{} は ワームシールドの とげで 1ダメージ!", g.party[who].name));
            if !g.party[who].alive() {
                msgs.push(format!("{} は うごけなくなった!", g.party[who].name));
            }
        }
        msgs
    }

    fn command_core(&mut self, g: &mut Game, who: usize, cmd: Cmd) -> Vec<String> {
        self.idx += 1;
        self.ticked[who] = false;
        let mut msgs = vec![];
        let name = g.party[who].name.clone();
        // ジェリーは ものまねの特技を ときどき使う
        let cmd = match cmd {
            Cmd::Attack(t) if g.party[who].is_slime => match self.try_slime_skill(g, who, t) {
                Some(m) => {
                    msgs.extend(m);
                    Cmd::Nothing
                }
                None => Cmd::Attack(t),
            },
            c => c,
        };
        match cmd {
            Cmd::Nothing => {}
            Cmd::Attack(t) => {
                let t = self.first_target(t);
                let mut atk = g.party[who].atk();
                if g.party[who].equip[0].as_deref().map(base_name) == Some("でたらめなけん") {
                    // でたらめなけん: ダメージが 0〜3倍で ばらばら
                    atk = atk * g.rng.range(0, 31) / 10;
                }
                let base = (atk - self.enemies[t].defense() + g.rng.range(-1, 1)).max(1);
                let crit = g.rng.chance(3);
                let dmg = if crit { base * 10 } else { base };
                let en = self.enemies[t].def.name;
                if crit {
                    msgs.push(format!("{} の こうげき! ひっさつの一撃! {} に {} のダメージ!", name, en, dmg));
                } else {
                    msgs.push(format!("{} の こうげき! {} に {} のダメージ!", name, en, dmg));
                }
                self.hit_enemy(t, dmg);
                let wp = g.party[who].equip[0].as_deref().map(|n| base_name(n).to_string());
                if self.enemies[t].hp <= 0 {
                    msgs.push(format!("{} を たおした!", en));
                } else if matches!(wp.as_deref(), Some("デカすぎこんぼう") | Some("ゴブリンクラブ")) && g.rng.chance(15) {
                    self.enemies[t].stun = true;
                    msgs.push(format!("{} は ふらついた!", en));
                } else if wp.as_deref() == Some("ナイフ") && g.rng.chance(15) {
                    let d2 = (g.party[who].atk() - self.enemies[t].defense()).max(1);
                    self.hit_enemy(t, d2);
                    msgs.push(format!("れんぞくこうげき! {} に {} のダメージ!", en, d2));
                    if self.enemies[t].hp <= 0 {
                        msgs.push(format!("{} を たおした!", en));
                    }
                } else if wp.as_deref() == Some("ワーワンソード") && g.rng.chance(12) {
                    let d2 = base * 3;
                    self.hit_enemy(t, d2);
                    msgs.push(format!("ひっさつの一撃! {} に {} のダメージ!", en, d2));
                    if self.enemies[t].hp <= 0 {
                        msgs.push(format!("{} を たおした!", en));
                    }
                }
            }
            Cmd::Spell(id, t) => {
                let s = spell(id);
                if g.party[who].seal > 0 {
                    msgs.push(format!("{} は ふういんされていて 魔法が つかえない!", name));
                } else if g.party[who].mp < s.mp {
                    msgs.push("MPが たりない!".to_string());
                } else {
                    g.party[who].mp -= s.mp;
                    match id {
                        "fire" | "holy" => {
                            let t = self.first_target(t);
                            let wis = g.party[who].wis_total();
                            let en = self.enemies[t].def.name;
                            let dmg = if id == "fire" {
                                (4 + wis + g.rng.range(0, 3) - self.enemies[t].defense() / 2).max(1)
                            } else if self.enemies[t].def.undead {
                                (4 + wis) * 3 + g.rng.range(0, 3)
                            } else {
                                ((4 + wis) * 2 / 5).max(1)
                            };
                            msgs.push(format!("{} は {} を となえた! {} に {} のダメージ!", name, s.name, en, dmg));
                            self.hit_enemy(t, dmg);
                            if self.enemies[t].hp <= 0 {
                                msgs.push(format!("{} を たおした!", en));
                            }
                        }
                        "healall" => {
                            let amount = 20 + g.party[who].wis_total();
                            msgs.push(format!("{} は ぜんいんかいふく を となえた!", name));
                            for m in g.party.iter_mut().filter(|m| m.alive()) {
                                let before = m.hp;
                                m.hp = (m.hp + amount).min(m.hp_max);
                                msgs.push(format!("{} のHPが {} かいふく!", m.name, m.hp - before));
                            }
                        }
                        "heal" => {
                            let amount = 15 + g.party[who].wis_total();
                            let before = g.party[t].hp;
                            if g.party[t].alive() {
                                g.party[t].hp = (g.party[t].hp + amount).min(g.party[t].hp_max);
                            }
                            msgs.push(format!("{} は ひとりかいふく を となえた! {} のHPが {} かいふく!", name, g.party[t].name, g.party[t].hp - before));
                        }
                        "cure" => {
                            let m = &mut g.party[t];
                            m.poison = false;
                            m.stun = false;
                            m.seal = 0;
                            msgs.push(format!("{} は ひとりなおす を となえた! {} は げんきになった!", name, g.party[t].name));
                        }
                        "strong" => {
                            g.party[t].strong = true;
                            msgs.push(format!("{} は つよくなれ を となえた! {} の こうげきりょくが あがった!", name, g.party[t].name));
                        }
                        "reflect" => {
                            g.party[t].reflect = true;
                            msgs.push(format!("{} は はねかえせ を となえた! {} は 魔法をはねかえすバリアに つつまれた!", name, g.party[t].name));
                        }
                        "frost" => {
                            let t = self.first_target(t);
                            let wis = g.party[who].wis_total();
                            let en = self.enemies[t].def.name;
                            let dmg = (4 + wis * 9 / 10 + g.rng.range(0, 3) - self.enemies[t].defense() / 2).max(1);
                            msgs.push(format!("{} は 氷のつぶて を となえた! {} に {} のダメージ!", name, en, dmg));
                            self.hit_enemy(t, dmg);
                            if self.enemies[t].hp <= 0 {
                                msgs.push(format!("{} を たおした!", en));
                            }
                        }
                        "firestorm" | "blizzard" => {
                            let wis = g.party[who].wis_total();
                            msgs.push(format!("{} は {} を となえた!", name, s.name));
                            for k in self.alive_enemies() {
                                let base = if id == "firestorm" { 2 + wis * 3 / 5 } else { 6 + wis };
                                let dmg = (base + g.rng.range(0, 3) - self.enemies[k].defense() / 2).max(1);
                                msgs.push(format!("{} に {} のダメージ!", self.enemies[k].def.name, dmg));
                                self.hit_enemy(k, dmg);
                                if self.enemies[k].hp <= 0 {
                                    msgs.push(format!("{} を たおした!", self.enemies[k].def.name));
                                }
                            }
                        }
                        "seal" => {
                            let t = self.first_target(t);
                            let en = self.enemies[t].def.name;
                            let ok = g.rng.chance(if self.enemies[t].def.boss { 50 } else { 90 });
                            msgs.push(format!("{} は ふういん を となえた!", name));
                            if ok {
                                self.enemies[t].sealed = true;
                                msgs.push(format!("{} は ふういんされた! 魔法や特殊攻撃を つかえない!", en));
                            } else {
                                msgs.push(format!("{} には きかなかった", en));
                            }
                        }
                        "onehp" => {
                            let t = self.first_target(t);
                            let en = self.enemies[t].def.name;
                            let ok = g.rng.chance(if self.enemies[t].def.boss { 20 } else { 90 });
                            msgs.push(format!("{} は いのちぎりぎり を となえた!", name));
                            if ok {
                                self.enemies[t].hp = 1;
                                msgs.push(format!("{} の HPが 1に なった!", en));
                            } else {
                                msgs.push(format!("{} には きかなかった", en));
                            }
                        }
                        "drain" => {
                            let wis = g.party[who].wis_total();
                            msgs.push(format!("{} は いのちいただき を となえた!", name));
                            let mut total = 0;
                            for k in self.alive_enemies() {
                                let dmg = (3 + wis / 3 + g.rng.range(0, 3)).max(1);
                                if self.enemies[k].def.id == "elephantom" {
                                    // エレファントムには 反転する(体力を吸わせてしまう)
                                    let e = &mut self.enemies[k];
                                    e.hp = (e.hp + dmg).min(e.max_hp());
                                    msgs.push(format!("{} は いのちいただきを はねかえした! HPが {} かいふくした!", e.def.name, dmg));
                                    continue;
                                }
                                total += dmg;
                                msgs.push(format!("{} から HPを {} すいとった!", self.enemies[k].def.name, dmg));
                                self.hit_enemy(k, dmg);
                                if self.enemies[k].hp <= 0 {
                                    msgs.push(format!("{} を たおした!", self.enemies[k].def.name));
                                }
                            }
                            let each = total * 2 / 3;
                            if each > 0 {
                                for m in g.party.iter_mut().filter(|m| m.alive()) {
                                    m.hp = (m.hp + each).min(m.hp_max);
                                }
                                msgs.push(format!("みんなのHPが {} かいふく!", each));
                            }
                        }
                        "sleep" => {
                            msgs.push(format!("{} は すいみん を となえた!", name));
                            for e in self.enemies.iter_mut().filter(|e| e.hp > 0 && !e.fled) {
                                if !e.def.boss && g.rng.chance(80) {
                                    e.sleep = 2;
                                    msgs.push(format!("{} は ねむってしまった!", e.def.name));
                                } else {
                                    msgs.push(format!("{} には きかなかった", e.def.name));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Cmd::Item(it, t) => {
                let d = item(&it).unwrap();
                if !g.take_item(&it, 1) {
                    msgs.push("もっていない!".to_string());
                } else {
                    match d.kind {
                        Kind::Potion if it == "かいふくやく" => {
                            let before = g.party[t].hp;
                            if g.party[t].alive() {
                                g.party[t].hp = (g.party[t].hp + d.power).min(g.party[t].hp_max);
                            }
                            msgs.push(format!("{} は {} をつかった! {} のHPが {} かいふく!", name, it, g.party[t].name, g.party[t].hp - before));
                        }
                        Kind::Potion if it == "ばんのうやく" => {
                            msgs.push(format!("{} は {} をつかった! みんなのHPが かいふく!", name, it));
                            for m in g.party.iter_mut() {
                                if m.alive() {
                                    m.hp = (m.hp + d.power).min(m.hp_max);
                                }
                            }
                        }
                        Kind::Potion => {
                            let before = g.party[t].mp;
                            g.party[t].mp = (g.party[t].mp + d.power).min(g.party[t].mp_max);
                            msgs.push(format!("{} は {} をつかった! {} のMPが {} かいふく!", name, it, g.party[t].name, g.party[t].mp - before));
                        }
                        Kind::Bomb if it == "炎バクダン" || it == "ふぶきバクダン" || it == "いただきバクダン" => {
                            msgs.push(format!("{} は {} をなげた!", name, it));
                            for k in self.alive_enemies() {
                                let dmg = (d.power + g.rng.range(-1, 2)).max(1);
                                msgs.push(format!("{} に {} のダメージ!", self.enemies[k].def.name, dmg));
                                self.hit_enemy(k, dmg);
                                if self.enemies[k].hp <= 0 {
                                    msgs.push(format!("{} を たおした!", self.enemies[k].def.name));
                                }
                            }
                        }
                        _ => {
                            let t = self.first_target(t);
                            let mut dmg = (d.power + g.rng.range(-1, 2)).max(1);
                            if self.enemies[t].def.undead {
                                dmg *= 2;
                            }
                            msgs.push(format!("{} は {} をなげた! {} に {} のダメージ!", name, it, self.enemies[t].def.name, dmg));
                            self.hit_enemy(t, dmg);
                            if self.enemies[t].hp <= 0 {
                                msgs.push(format!("{} を たおした!", self.enemies[t].def.name));
                            }
                        }
                    }
                }
            }
            Cmd::Defend => {
                self.defending[who] = true;
                msgs.push(format!("{} は みをまもっている", name));
            }
            Cmd::Flee => {
                let chance = if self.boss { 40 } else { 65 };
                if g.rng.chance(chance) {
                    msgs.push("うまく にげきれた!".to_string());
                    self.pending_end = true;
                } else {
                    msgs.push("にげられない!".to_string());
                }
            }
        }
        if g.party[who].seal > 0 {
            g.party[who].seal -= 1;
            if g.party[who].seal == 0 {
                msgs.push(format!("{} の ふういんが とけた!", name));
            }
        }
        if !self.pending_end {
            self.check_end(g);
        }
        msgs
    }
}
