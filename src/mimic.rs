//! モノマネスライム(ジェリー)。倒した魔物の「あたま・うで・からだ・あし」をマネして、
//! つよさ・HP・かしこさ・すばやさを得たり、装備や特技を使えるようになる。出典: wiki「モノマネスライム」
//!   あたま: MP・かしこさ / 兜 / 特技 / 魔法をおぼえる
//!   うで:   つよさ / 武器・盾・指輪 / 特技(攻撃のとき ときどき出る)
//!   からだ: HP・じょうぶさ / 鎧
//!   あし:   すばやさ / 靴
//! ボス(1〜19Fのまほうじんに出る個体も 本物の ボスも)は 倒すと マネできる(能力の増え方は 1/3 の設計値)。wiki の表に無い魔物(11F以降など)の行は 設計値。数値の増え方も設計値(wiki は階ごとの実測値)。

use crate::data::*;

pub const PARTS: [&str; 4] = ["あたま", "うで", "からだ", "あし"];

/// 特技の効き方
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SkillKind {
    /// 守りを半分むしして つらぬく
    Pierce,
    /// 鈍器: 1.3倍、ときどき ふらつかせる
    Blunt,
    /// 大ダメージ(攻撃力の 何%)
    Big(i32),
    /// 2回こうげき
    Double,
    /// 全体攻撃(攻撃力の 何%)
    All(i32),
    /// 炎ダメージ(かしこさ依存。%)
    Fire(i32),
    /// 氷ダメージ(かしこさ依存。%)
    Ice(i32),
    /// 敵全体に炎
    FireAll,
    /// 眠らせる
    Sleep,
    /// いのちぎりぎり(HPを1に)
    OneHp,
    /// 攻撃のかわりに 仲間の誰かを回復
    Heal,
}

#[derive(Clone, Copy, Debug)]
pub struct Skill {
    pub name: &'static str,
    pub kind: SkillKind,
}

/// 魔法をおぼえる あたま(魔法, その階以降)
pub type MagicRule = (&'static str, usize);

pub struct Mimic {
    pub id: &'static str,
    pub head_wear: bool,
    pub head_skill: Option<Skill>,
    pub head_magic: &'static [MagicRule],
    /// 武器・盾を持てるか / 指輪
    pub arm_wear: bool,
    pub arm_ring: bool,
    pub arm_skill: Option<Skill>,
    pub body_wear: bool,
    pub feet_wear: bool,
}

const fn sk(name: &'static str, kind: SkillKind) -> Option<Skill> {
    Some(Skill { name, kind })
}

#[allow(clippy::too_many_arguments)]
const fn mm(id: &'static str, head_wear: bool, head_skill: Option<Skill>, head_magic: &'static [MagicRule], arm_wear: bool, arm_ring: bool, arm_skill: Option<Skill>, body_wear: bool, feet_wear: bool) -> Mimic {
    Mimic { id, head_wear, head_skill, head_magic, arm_wear, arm_ring, arm_skill, body_wear, feet_wear }
}

const NO: &[MagicRule] = &[];

/// wiki の表(頭・腕・体・足)。載っていない部分は 設計値
pub static MIMICS: &[Mimic] = &[
    mm("bat", true, sk("つっつき攻撃", SkillKind::Pierce), NO, false, false, None, false, false),
    mm("pig", true, sk("とつげき", SkillKind::Blunt), NO, false, false, None, false, false),
    mm("cat", true, None, NO, true, true, sk("とびつきこうげき", SkillKind::Double), true, true),
    mm("gob", true, None, NO, true, true, sk("きょうれつないちげき", SkillKind::Big(150)), true, true),
    mm("slime", true, None, NO, false, false, sk("のしかかり", SkillKind::Blunt), false, false),
    mm("potato", true, sk("炎バクダン攻撃", SkillKind::FireAll), NO, false, false, None, false, false),
    mm("wan", true, None, NO, true, true, sk("ちからまかせ", SkillKind::Big(200)), true, true),
    mm("gobl", true, None, NO, true, true, sk("ダブルクラッシュ", SkillKind::Double), true, true),
    mm("ghost", true, None, &[("seal", 1)], true, true, sk("スリープチョップ", SkillKind::Sleep), false, false),
    mm("hibuta", true, sk("炎のとつげき", SkillKind::Fire(130)), NO, false, false, None, false, false),
    mm("nekomata", true, None, &[("heal", 1), ("healall", 7)], true, true, None, true, true),
    mm("ningen", true, None, NO, true, true, sk("かいてんぎり", SkillKind::All(70)), true, true),
    mm("firelime", false, None, NO, false, false, sk("炎のたいあたり", SkillKind::Fire(130)), false, false),
    mm("hotghost", true, None, &[("fire", 1), ("firestorm", 11)], true, true, sk("フレイムチョップ", SkillKind::Fire(110)), false, false),
    mm("waterlime", false, None, NO, false, false, None, false, false),
    mm("coldghost", true, None, &[("frost", 1), ("blizzard", 11)], true, true, sk("コールドチョップ", SkillKind::Ice(110)), false, false),
    mm("otter", true, None, NO, true, true, sk("たつまきぎり", SkillKind::All(70)), true, true),
    mm("carrot", true, None, NO, true, true, None, false, false),
    mm("crowcat", true, None, &[("sleep", 1)], false, false, None, false, false),
    mm("blackcat", true, None, &[("cure", 1)], true, true, None, true, true),
    mm("wizdrill", true, None, &[("holy", 1)], true, true, None, true, true),
    mm("blackwar", true, None, NO, true, true, sk("ひっさつけん", SkillKind::Big(220)), true, true),
    mm("bone", true, None, NO, true, true, None, true, true),
    mm("chikaraushi", true, None, NO, true, true, None, true, true),
    mm("powerwan", true, None, NO, true, true, sk("パワーアタック", SkillKind::Big(200)), true, true),
    mm("flamewar", true, None, NO, true, true, sk("炎のひっさつけん", SkillKind::Fire(200)), true, true),
    mm("hotjigoku", false, None, NO, false, false, None, false, false),
    mm("buffalo", true, None, NO, true, true, None, true, true),
    mm("penguin", true, None, &[("frost", 1)], true, true, None, true, true),
    mm("icewar", true, None, NO, true, true, sk("あらしのひっさつけん", SkillKind::Ice(200)), true, true),
    mm("icejigoku", false, None, NO, false, false, None, false, false),
    mm("demonarmor", true, None, &[("strong", 1)], true, true, sk("きゅうしょちょくげき", SkillKind::OneHp), true, true),
    mm("platypus", true, None, NO, true, true, sk("のうてんちょくげき", SkillKind::Big(180)), true, true),
    mm("detarame", true, sk("でたらめかいふく", SkillKind::Heal), NO, true, true, None, true, true),
    mm("daruma", true, None, NO, true, true, sk("かいてんぎり", SkillKind::All(70)), true, true),
    mm("samuraiwolf", true, None, NO, true, true, sk("ひっさつけん", SkillKind::Big(220)), true, true),
    // ---- wiki の魔法の表から ----
    mm("nekomata2", true, None, &[("onehp", 1)], true, true, None, true, true),
    mm("nekomata3", true, None, &[("drain", 1)], true, true, None, true, true),
    mm("magicshield", true, None, &[("reflect", 1)], false, false, None, false, false),
    // ---- wiki の表に無い魔物(設計値: 体のつくりで 装備できるかを決めた) ----
    mm("masterwar", true, None, NO, true, true, sk("ひっさつけん", SkillKind::Big(240)), true, true),
    mm("pepper", true, None, NO, false, false, None, false, false),
    mm("zombie", true, None, NO, true, true, None, true, true),
    mm("valkyrie", true, None, NO, true, true, sk("ワルキリースピア", SkillKind::Pierce), true, true),
    mm("valkyrie2", true, None, NO, true, true, sk("ワルキリースピア", SkillKind::Pierce), true, true),
    mm("catqueen", true, None, NO, true, true, sk("とびつきこうげき", SkillKind::Double), true, true),
    mm("emperorwan", true, None, NO, true, true, sk("ひっさつけん", SkillKind::Big(240)), true, true),
    mm("poisonlime", false, None, NO, false, false, sk("どくのたいあたり", SkillKind::Blunt), false, false),
    mm("anagerilla", true, None, NO, true, true, None, true, true),
    mm("lumberduck", true, None, NO, true, true, sk("きこりのいちげき", SkillKind::Big(180)), true, true),
    mm("shogun", true, None, NO, true, true, sk("ひっさつけん", SkillKind::Big(240)), true, true),
    mm("killercock", true, None, NO, true, true, sk("くちばしつつき", SkillKind::Pierce), true, true),
    mm("hidaruma", true, None, NO, false, false, sk("炎のたいあたり", SkillKind::Fire(130)), false, false),
    mm("maidoll", true, None, NO, true, true, None, true, true),
    mm("yukidaruma", true, None, NO, false, false, sk("こおりのたいあたり", SkillKind::Ice(130)), false, false),
    mm("mushroomlizard", true, None, NO, true, true, None, false, false),
    mm("warbeetle", true, None, NO, true, true, sk("つのでつきあげ", SkillKind::Pierce), true, true),
    mm("flamemaster", true, None, NO, true, true, sk("炎のひっさつけん", SkillKind::Fire(220)), true, true),
    mm("highmaster", true, None, NO, true, true, sk("ひっさつけん", SkillKind::Big(260)), true, true),
    mm("caterpillar", false, None, NO, false, false, None, false, false),
    mm("killerflower", true, None, NO, false, false, sk("ばくだんタネ", SkillKind::FireAll), false, false),
    mm("bonewizard", true, None, NO, true, true, None, true, true),
    mm("magicdonkey", true, None, NO, true, true, None, true, true),
    mm("devildice", false, None, NO, false, false, sk("ダイスアタック", SkillKind::Blunt), false, false),
    mm("hellscream", true, None, NO, false, false, None, false, false),
    mm("magicbow", false, None, NO, true, true, None, false, false),
    mm("magicsword", false, None, NO, true, true, sk("まけんのつるぎ", SkillKind::Big(250)), false, false),
    mm("sorcerer_n", true, None, NO, true, true, sk("ふぶきのつえ", SkillKind::Ice(150)), true, true),
    // ---- 序盤に足した魔物(設計値) ----
    mm("needleworm", false, None, NO, false, false, sk("とげのたいあたり", SkillKind::Pierce), false, false),
    mm("hinezumi", true, None, NO, true, true, sk("ひのこ", SkillKind::Fire(110)), false, false),
    mm("shieldmaiden", true, None, NO, true, true, None, true, true),
    mm("detaramedice", false, None, NO, false, false, sk("でたらめアタック", SkillKind::Blunt), false, false),
    // ---- ボス(まほうじんに出る個体を倒すと マネできる。設計値) ----
    mm("boss", false, None, NO, false, false, sk("どくのしめつけ", SkillKind::Blunt), false, false),
    mm("lion", true, None, NO, true, true, sk("フライングクロウ", SkillKind::All(60)), true, true),
    mm("dino", true, sk("ようがんこうげき", SkillKind::FireAll), NO, false, false, sk("のしかかり", SkillKind::Blunt), false, false),
    mm("sorcerer", true, sk("こおりのつえ", SkillKind::Ice(160)), NO, true, true, None, true, true),
    mm("tetsu", true, None, NO, true, true, sk("きょうれつないちげき", SkillKind::Big(240)), true, true),
    mm("debrin", true, sk("ほのおのつぼ", SkillKind::FireAll), NO, true, true, None, false, false),
    mm("octopus", false, sk("うずしお", SkillKind::Ice(150)), NO, false, false, sk("しめつけ", SkillKind::Blunt), false, false),
    mm("kinglion", true, None, NO, true, true, sk("フライングクロウ", SkillKind::All(60)), true, true),
    mm("dragon", true, sk("ドラゴンブレス", SkillKind::Fire(160)), NO, true, true, sk("しっぽなぎはらい", SkillKind::Blunt), true, true),
    mm("devilprince", true, None, NO, true, true, sk("あくまのつるぎ", SkillKind::Big(260)), true, true),
    mm("dorunuma", false, None, NO, false, false, sk("どろのたたきつけ", SkillKind::Blunt), false, false),
    mm("shogunboss", true, None, NO, true, true, sk("ひっさつけん", SkillKind::Big(260)), true, true),
    mm("devilprincess", true, sk("いのちいただき", SkillKind::Heal), NO, true, true, None, true, true),
    mm("hitotsuoni", true, None, NO, true, true, sk("ぶんまわし", SkillKind::All(60)), true, true),
    mm("flameflower", true, sk("ばくえん", SkillKind::FireAll), NO, false, false, None, false, false),
    mm("kage", true, None, NO, true, true, sk("かげのいちげき", SkillKind::Big(240)), true, true),
    mm("mitsuoni", true, sk("ねむりビーム", SkillKind::Sleep), NO, true, true, sk("ぶんまわし", SkillKind::All(60)), true, true),
    mm("elephantom", true, None, NO, true, true, sk("のしかかり", SkillKind::Blunt), true, true),
    mm("devillord", true, None, NO, true, true, sk("きょうれつなひっさつけん", SkillKind::Big(300)), true, true),
    mm("kingsnake", false, None, NO, false, false, sk("どくのしめつけ", SkillKind::Blunt), false, false),
];


/// マネできる魔物(wiki の表にある物。ボスは 載っていない)。ものまね用の 規定値 を返す
pub fn mimic_of(id: &str) -> Option<&'static Mimic> {
    MIMICS.iter().find(|m| m.id == id)
}

/// 部位ごとの 増える能力(設計値)
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bonus {
    pub str_: i32,
    pub wis: i32,
    pub con: i32,
    pub spd: i32,
    pub hp: i32,
    pub mp: i32,
}

/// 魔物 mon の 部位 part(0=あたま 1=うで 2=からだ 3=あし)をマネしたときの 増え方(ボスは 数値が大きいので 1/3 に した設計値)
/// +n(瘴気で 強くなった魔物を 倒した)ぶん、増え方を 1.2倍ずつ(切り上げ)
pub fn bonus_plus(part: usize, mon: &MonDef, plus: u8) -> Bonus {
    let mut b = bonus(part, mon);
    for _ in 0..plus {
        let up = |v: i32| (v * 12 + 9) / 10;
        b = Bonus { str_: up(b.str_), wis: up(b.wis), con: up(b.con), spd: up(b.spd), hp: up(b.hp), mp: up(b.mp) };
    }
    b
}

pub fn bonus(part: usize, mon: &MonDef) -> Bonus {
    let mut b = bonus_raw(part, mon);
    if mon.boss {
        b = Bonus { str_: b.str_ / 3, wis: b.wis / 3, con: b.con / 3, spd: b.spd / 3, hp: b.hp / 3, mp: b.mp / 3 };
    }
    b
}

fn bonus_raw(part: usize, mon: &MonDef) -> Bonus {
    let mut b = Bonus::default();
    match part {
        0 => {
            b.wis = mon.atk * 3 / 8;
            let m = mimic_of(mon.id);
            if m.map(|m| !m.head_magic.is_empty() || m.head_skill.is_some()).unwrap_or(false) {
                b.mp = mon.hp * 3 / 5;
            }
        }
        1 => b.str_ = mon.atk * 2 / 5,
        2 => {
            b.hp = mon.hp * 3 / 5;
            b.con = mon.def * 3 / 5;
        }
        _ => b.spd = mon.spd,
    }
    b
}

/// マネできる魔物か(wiki に載っていて ボスでない)
pub fn can_mimic(id: &str) -> bool {
    mimic_of(id).is_some() && MONS.iter().any(|m| m.id == id)
}

/// 部位ごとの説明(「つよさ+8 / ちからまかせ / 武器・盾○」)
pub fn describe(part: usize, id: &str, plus: u8) -> String {
    let mon = mon(id);
    let b = bonus_plus(part, mon, plus);
    let Some(m) = mimic_of(id) else { return String::new() };
    let mut v: Vec<String> = vec![];
    match part {
        0 => {
            v.push(format!("かしこさ+{}", b.wis));
            if b.mp > 0 {
                v.push(format!("MP+{}", b.mp));
            }
            v.push(if m.head_wear { "兜○".to_string() } else { "兜×".to_string() });
            if let Some(s) = m.head_skill {
                v.push(s.name.to_string());
            }
            for (sp, fl) in m.head_magic {
                v.push(format!("魔法「{}」{}", spell(sp).name, if *fl > 1 { format!("({}F以降)", fl) } else { String::new() }));
            }
        }
        1 => {
            v.push(format!("つよさ+{}", b.str_));
            v.push(if m.arm_wear { "武器・盾○".to_string() } else { "武器×".to_string() });
            if m.arm_ring {
                v.push("指輪○".to_string());
            }
            if let Some(s) = m.arm_skill {
                v.push(s.name.to_string());
            }
        }
        2 => {
            v.push(format!("HP+{}", b.hp));
            v.push(format!("じょうぶさ+{}", b.con));
            v.push(if m.body_wear { "鎧○".to_string() } else { "鎧×".to_string() });
        }
        _ => {
            v.push(format!("すばやさ+{}", b.spd));
            v.push(if m.feet_wear { "靴○".to_string() } else { "靴×".to_string() });
        }
    }
    v.join(" ")
}
