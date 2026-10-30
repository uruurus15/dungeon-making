//! メインクエスト: セーブには持たず、いまの状態から「最初に満たされていない段階」を計算する。
//! 段階: 部屋を置く → 寝る → 戦う → (階ごとに) 魔穴を開く → ボスを倒す → 届ける →(1Fは磨く)→ 降りる … → つづく

use crate::data::*;
use crate::sim::*;

/// 段階の種類(村長の台詞を選ぶのに使う)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepKind {
    PlaceRooms,
    Sleep,
    Fight,
    Grow(usize),
    Boss(usize),
    Deliver(usize),
    Polish,
    Descend(usize),
    Done,
}

pub struct MainStep {
    pub kind: StepKind,
    /// 0 始まり
    pub idx: usize,
    pub total: usize,
    pub title: String,
    pub hint: String,
    /// (ラベル, いま, 必要)
    pub progress: Vec<(String, i32, i32)>,
}

/// 3(最初の3段階) + 1F:5 + 中間の階:各4 + 最上階:3 + 最後の1(= 4*MAX_FLOOR+4)
pub const MAIN_TOTAL: usize = 4 * MAX_FLOOR + 4;

fn any_dungeon<F: Fn(&Dungeon) -> bool>(g: &Game, f: F) -> bool {
    f(&g.dungeon) || g.floors.iter().flatten().any(|d| f(d))
}

fn grow_hint(floor: usize) -> &'static str {
    match floor {
        1 => "評価と倒した数をそろえて寝ると、夜に魔穴が開く。部屋を増やして種類もそろえよう。",
        2 => "家具屋に さかば・さいだん・はたけ が並んだ。さかばを4つ以上置こう。はたけは3x3マスを先に掘ってから。",
        3 => "いろり・はなぞのが並んだ。いろりを6つ以上置こう。ボスは炎が得意なので、炎に強い装備(アツイローブ・ルビーのゆびわなど)を。",
        4 => "水のみばが並んだ。水のみばを6つ以上置こう。ボスは氷が得意なので、氷に強い装備(サムイローブ・かいのよろいなど)を。",
        5 => "りっぱなベッド・どうじょうが並んだ。りっぱなベッドを5つ、どうじょう(3x3・入口は1つ)を1つ置こう。",
        6 => "だんろ・けっかいのへやが並んだ。だんろを6つ以上置こう。けっかいのへやは4方向から入れる。",
        7 => "ふんすいが並んだ。ふんすいを6つ以上置こう。ボスは水が得意なので、氷・水に強い装備を。",
        8 => "ごうかなベッドが並んだ。ごうかなベッドを5つ、どうじょうとはなぞのを1つずつ置こう。",
        9 => "れいはいどうが並んだ。れいはいどうを5つ、どうじょうを1つ置こう。",
        10 => "じっけんしつ(3x3・入口は1つ)が並んだ。じっけんしつを1つ、れいはいどうを4つ、ごうかなベッドを2つ置こう。",
        11 => "ごうかなベッド・コロシアムまでの部屋が使える。はたけを2つ置いて、魔物をたくさん集めよう。",
        12 => "はなぞの6・れいはいどう2・ごうかなベッド18・はたけ3を置こう。ダンジョンは下の階ほど広い。ウルフはごうかなベッドにすみつく。",
        13 => "はなぞのを2つ、ごうかなベッドを6つ置こう。ワーネコクイーンなどのドロップがよくなる階。",
        14 => "コロシアム(3x3・4方向から入れる)が使える。コロシアムを1つ、ごうかなベッドを4つ置こう。",
        15 => "だんろを20個置こう。炎の魔物が集まる階。ボスは炎が得意。",
        16 => "れいぞうこ6・ふんすい8を置こう。氷の魔物が集まる階。れいぞうこが使える。",
        17 => "はたけ4・はなぞの6・ごうかなベッド12を置こう。",
        18 => "はたけ4・はなぞの6・じっけんしつ2を置こう。魔法使いがふえる階。",
        19 => "ごうかなベッド23・じっけんしつ2・れいはいどう4・はたけ1を置こう。あくまの大王のさいごのフロア。",
        20 => "まほうじん4・コロシアム2・じっけんしつ2・ごうかなベッド10・れいはいどう4を置こう。まほうじんが使える。",
        _ => panic!("grow_hint: {}F", floor),
    }
}

fn boss_hint(floor: usize) -> &'static str {
    match floor {
        1 => "外周の壁が掘れるようになった。紫の渦(魔穴)まで通路をつなげて踏み込もう。",
        2 => "リオンは全体攻撃(フライングクロウ)で仲間をふらつかせる。回復の用意を。",
        3 => "ホノオザウルスは炎の全体攻撃とのしかかりを使う。炎に強い装備と、回復薬・ぜんいんかいふくを多めに。",
        4 => "カワウソーサラーは氷の全体攻撃を使う。氷に強い装備を。物理は効きにくいので魔法(もえあがれ・せいなる光)も使おう。",
        5 => "テツジンはDEFがとても高い。魔法や必殺の一撃(くろのロングソード)がたより。光の全体攻撃に備えて、光に強い装備を。",
        6 => "デブリンは炎の全体攻撃(ほのおのつぼ)を使う。炎に強い装備と回復を用意しよう。",
        7 => "アクトパスは水の全体攻撃と、ふらつかせるしめつけを使う。氷・水に強い装備を。",
        8 => "キングリオンは光に弱い。せいなる光や光バクダンが効く。全体のひきさき攻撃に備えて回復を。",
        9 => "アンクルドラゴンは炎のブレスと、ふらつかせるしっぽ攻撃を使う。炎に強い装備を。",
        10 => "デビルプリンスは炎・氷・光の全体攻撃を使い分ける。耐性装備と回復をたっぷり。",
        11 => "ドロヌマの攻撃はふらつきやすい。ちからのヘルムなど ふらつきに強い装備と、ばんのうやくを。",
        12 => "ショーグンウルフは単体攻撃と全体攻撃だけ。DEFを上げておけば苦戦しない。",
        13 => "デビルプリンセスは毒の全体攻撃といのちいただきを使う。毒けしと回復を多めに。",
        14 => "ヒトツメオニは通常攻撃でもふらつく。ぶんまわし(全体)とギリギリこうせん(HPが1に)に注意。",
        15 => "フレイムフラワーは炎の全体攻撃を使う。炎に強い装備(炎のローブなど)と回復を。",
        16 => "カゲは単体の強い物理攻撃と光の全体攻撃を使う。HPの低い仲間が即死しないように。",
        17 => "ミツメオニは全体ふらつきのビームと強い物理攻撃を使う。ばんのうやくを多めに。",
        18 => "エレファントムはじごくの炎(全体)とのしかかりを使う。炎に強い装備を。",
        19 => "デビルロードはいちばんHPの高い仲間を狙う強烈な一撃と、光の全体攻撃、いのちいただきを使う。",
        20 => "キングヘビリンゴは毒・炎・いのちいただき・全体しめつけを使う。耐性装備と回復をたっぷり。",
        _ => panic!("boss_hint: {}F", floor),
    }
}

pub fn main_quest(g: &Game) -> MainStep {
    let step = |idx: usize, kind: StepKind, title: String, hint: String, progress: Vec<(String, i32, i32)>| MainStep { kind, idx, total: MAIN_TOTAL, title, hint, progress };

    if !any_dungeon(g, |d| !d.rooms.is_empty()) {
        return step(0, StepKind::PlaceRooms, "ダンジョンに部屋を置こう".into(), "街の家具屋で部屋を買って、ダンジョンへ。壁を掘って(Z)、通路のつきあたりや横に部屋(X)を置こう。".into(), vec![]);
    }
    if !any_dungeon(g, |d| d.kills > 0 || d.rooms.iter().any(|r| r.alive() > 0)) {
        return step(1, StepKind::Sleep, "家で寝て、魔物をすみつかせよう".into(), "魔物は夜のあいだに部屋へすみつく。町に戻って、自宅で「ねる」。".into(), vec![]);
    }
    if !any_dungeon(g, |d| d.kills > 0) {
        return step(2, StepKind::Fight, "部屋の入口の前に立って、魔物を倒そう".into(), "部屋の入口(黄色い三角のある側)の前のマスに立つと戦闘になる。".into(), vec![]);
    }
    let mut idx = 3;
    for f in 1..=MAX_FLOOR {
        let d = g.floor_dungeon(f);
        let boss = mon(boss_id(f)).name;
        let q = quest(FLOOR_MAIN_QUEST[f - 1]);
        // 魔穴を開く
        if !d.map(|d| d.portal.is_some() || d.boss_defeated).unwrap_or(false) {
            let p = d.map(Game::progress_of).unwrap_or_default();
            return step(idx, StepKind::Grow(f), format!("{}Fを育てて「魔穴」を開こう", f), grow_hint(f).into(), p);
        }
        idx += 1;
        // ボスを倒す
        if !d.map(|d| d.boss_defeated).unwrap_or(false) {
            return step(idx, StepKind::Boss(f), format!("魔穴まで掘って、{}を倒そう", boss), boss_hint(f).into(), vec![]);
        }
        idx += 1;
        // 持ち帰る
        if g.quest_state(q.id) != 2 {
            return step(idx, StepKind::Deliver(f), format!("{}を クエストボードに届けよう", q.need), format!("町の「クエスト」で 「{}」を達成しよう。", q.title), vec![(q.need.to_string(), g.count(q.need), q.count)]);
        }
        idx += 1;
        if f == 1 {
            if g.mp_max < START_MP + OIL_MP_GAIN {
                return step(idx, StepKind::Polish, "家でシャベルを磨こう".into(), "自宅の「シャベルを みがく」で 魔法のオリーブオイルを使うと、シャベルMPが増える。".into(), vec![("魔法のオリーブオイル".to_string(), g.count("魔法のオリーブオイル"), 1)]);
            }
            idx += 1;
        }
        if f < MAX_FLOOR {
            if g.reached < f + 1 {
                return step(idx, StepKind::Descend(f), format!("階段で地下{}階へ降りよう", f + 1), "ダンジョンの階段(魔穴があった場所)に乗ると、次の階へ降りられる。".into(), vec![]);
            }
            idx += 1;
        }
    }
    step(MAIN_TOTAL - 1, StepKind::Done, format!("おめでとう! 地下{}階へは「つづく」", MAX_FLOOR + 1), format!("いまは {}F までの体験版。部屋を増やしたり、装備を強くしたりして遊ぼう。", MAX_FLOOR), vec![])
}
