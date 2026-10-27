//! 1F分のデータ表。出典は research/dungeon-maker-shovel の atwiki 各ページ。
//! 「設計値」と書いたものは原作に数値の記載がなく、こちらで決めた値(後で調整してよい)。

/// 地図の大きさは階ごと(floor_size)。MAP_MAX は どの階よりも大きい上限
pub const MAP_MAX: usize = 40;
/// 内周(自由に掘れる)の範囲
pub const INNER_MIN: usize = 4;
/// 入口(階段)の位置
pub const ENTRANCE: (usize, usize) = (8, 12);

/// 内周(掘れる四角)の一辺。外周とバリアで 左右上下に4マスずつ足したものが地図の大きさ。9のとき入口は (8, 12)
/// 1F〜4F は 9。そこから 3階ごとに 2ずつ広がる(5F〜7F は 11、8F〜10F は 13、…、20F は 21)
pub fn floor_size(floor: usize) -> usize {
    9 + 2 * ((floor.max(2) - 2) / 3)
}

/// 設計値: 原作wikiは初期MP5・PocketGamerは「約10マス」。10を採用
pub const START_MP: i32 = 10;
pub const OIL_MP_GAIN: i32 = 5;
pub const DIG_COST: i32 = 1;
pub const FILL_COST: i32 = 1;
pub const ROOM_COST: i32 = 1;
/// ボス(魔穴)出現条件。wiki「評価30程度」+ クエスト「魔物の数20匹ほど」
pub const BOSS_RATING: i32 = 30;
pub const BOSS_KILLS: u32 = 20;
pub const START_GOLD: i32 = 600;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Food,
    Veg,
    Loot,
    Potion,
    Bomb,
    Key,
    Quest,
    Oil,
    Weapon,
    Shield,
    Head,
    Body,
    Feet,
    Ring,
    Room,
}

impl Kind {
    pub fn is_equip(self) -> bool {
        matches!(self, Kind::Weapon | Kind::Shield | Kind::Head | Kind::Body | Kind::Feet | Kind::Ring)
    }
    /// 装備スロット番号(0..6)
    pub fn slot(self) -> Option<usize> {
        match self {
            Kind::Weapon => Some(0),
            Kind::Shield => Some(1),
            Kind::Head => Some(2),
            Kind::Body => Some(3),
            Kind::Feet => Some(4),
            Kind::Ring => Some(5),
            _ => None,
        }
    }
}

pub const SLOT_NAMES: [&str; 6] = ["ぶき", "たて", "あたま", "からだ", "あし", "ゆびわ"];

pub struct ItemDef {
    pub name: &'static str,
    pub kind: Kind,
    pub buy: i32,
    pub sell: i32,
    /// 武器=攻撃 / 防具=防御
    pub power: i32,
    /// リング効果: 0=なし 1=つよさ 2=じょうぶさ 3=かしこさ 4=光耐性
    pub ring: u8,
    pub desc: &'static str,
}

const fn it(name: &'static str, kind: Kind, buy: i32, sell: i32, power: i32, ring: u8, desc: &'static str) -> ItemDef {
    ItemDef { name, kind, buy, sell, power, ring, desc }
}

pub static ITEMS: &[ItemDef] = &[
    // 食材(野菜)
    it("ホウレンソウ", Kind::Veg, 20, 10, 0, 0, "広場で買える野菜。"),
    it("カツオブシ", Kind::Veg, 40, 20, 0, 0, "ワーネコが落とす。広場で買えるようになる。"),
    it("ジャガイモ", Kind::Veg, 60, 30, 0, 0, "バクダンイモが落とす。広場で買えるようになる。"),
    // 食材(肉など)
    it("カラスのももにく", Kind::Food, 0, 20, 0, 0, "カラスコウモリのもも肉。"),
    it("ブタイノシシのバラ", Kind::Food, 0, 30, 0, 0, "ブタイノシシのバラ肉。"),
    it("スライムのひもの", Kind::Food, 0, 60, 0, 0, "スライムを干したもの。"),
    it("ヒネズミのにく", Kind::Food, 0, 50, 0, 0, "ヒネズミの香ばしい肉。いろりの魔物が落とす。"),
    it("ヒネズミのけがわ", Kind::Body, 250, 200, 3, 0, "ほのおに強い毛皮。防御は低め(炎耐性20)"),
    it("オバケタルト", Kind::Food, 0, 90, 0, 0, "ワライオバケが落とす不思議なタルト。"),
    // 換金
    it("イノシシのけがわ", Kind::Loot, 0, 160, 0, 0, "高く売れる毛皮。"),
    it("シルバーコイン", Kind::Loot, 0, 500, 0, 0, "銀貨。"),
    // 薬・バクダン
    it("かいふくやく", Kind::Potion, 50, 30, 30, 0, "HPを30回復する。"),
    it("まほうのくすり", Kind::Potion, 300, 180, 10, 0, "MPを10回復する。"),
    it("光バクダン", Kind::Bomb, 20, 10, 16, 0, "光で敵1体にダメージ。"),
    it("炎バクダン", Kind::Bomb, 50, 30, 11, 0, "炎で敵全体にダメージ。"),
    it("ふぶきバクダン", Kind::Bomb, 50, 30, 11, 0, "吹雪で敵全体にダメージ。"),
    // 鍵・特別
    it("てつのカギ", Kind::Key, 100, 0, 0, 0, "鉄の宝箱のカギ。"),
    it("ぎんのカギ", Kind::Key, 400, 0, 0, 0, "銀の宝箱のカギ。"),
    it("おうごんのカギ", Kind::Key, 1200, 0, 0, 0, "金の宝箱のカギ。"),
    it("ヘビのはっぱ", Kind::Quest, 0, 0, 0, 0, "ヘビリンゴの落とした葉。"),
    it("リオンのたてがみ", Kind::Quest, 0, 0, 0, 0, "ボス・リオンのたてがみ。村長に見せよう。"),
    it("ワーネコホルター", Kind::Quest, 0, 0, 0, 0, "ワーネコのホルター。武具屋のおねえさんがほしがっている。"),
    it("魔法のオリーブオイル", Kind::Oil, 0, 0, 0, 0, "家でシャベルを磨くとMPが増える。"),
    // 武器
    it("ナイフ", Kind::Weapon, 150, 120, 7, 0, "連続攻撃が出ることがある。"),
    it("ワーネコダッガー", Kind::Weapon, 250, 200, 10, 0, "ワーネコの短剣。"),
    it("デカすぎこんぼう", Kind::Weapon, 250, 200, 10, 0, "相手をふらつかせることがある。"),
    it("ワーワンソード", Kind::Weapon, 300, 240, 14, 0, "必殺の一撃が出ることがある。"),
    it("ゴブリンクラブ", Kind::Weapon, 350, 280, 13, 0, "相手をふらつかせることがある。"),
    it("ショートスピア", Kind::Weapon, 400, 320, 12, 0, "短めの槍。"),
    // 盾
    it("バックラー", Kind::Shield, 150, 120, 1, 0, "小さな盾。"),
    it("ワームシールド", Kind::Shield, 300, 240, 3, 0, "とげの盾。自分が動くたびに 1ダメージ くらうが、なぐられると 3〜5ダメージを はね返す。"),
    it("ウッドシールド", Kind::Shield, 200, 160, 2, 0, "木の盾。"),
    // 頭
    it("こぶりなかぶと", Kind::Head, 150, 120, 2, 0, "ふらつきにくくなる。"),
    it("ネコみみぼうし", Kind::Head, 150, 120, 2, 0, "ネコの耳つきぼうし。"),
    it("かわのぼうし", Kind::Head, 120, 96, 1, 0, "革の帽子。"),
    it("キルトのぼうし", Kind::Head, 150, 120, 1, 0, "キルト地の帽子。"),
    it("ゴブリメット", Kind::Head, 200, 150, 4, 0, "ふらつきにくくなる。"),
    it("オバケぼうし", Kind::Head, 300, 240, 3, 0, "オバケの帽子。"),
    it("ガラスのティアラ", Kind::Head, 250, 200, 2, 0, "ガラスのティアラ。"),
    // 体
    it("かわのベスト", Kind::Body, 200, 160, 2, 0, "革のベスト。"),
    it("セーラーふく", Kind::Body, 300, 240, 3, 0, "なぜかここにあった服。"),
    it("かわよろい", Kind::Body, 300, 240, 6, 0, "革のよろい。"),
    it("オバケローブ", Kind::Body, 200, 160, 4, 0, "オバケのローブ。"),
    it("デニムジャケット", Kind::Body, 400, 320, 5, 0, "丈夫なジャケット。"),
    // 足
    it("かわのくつ", Kind::Feet, 100, 80, 1, 0, "革のくつ。"),
    it("けがわのくつ", Kind::Feet, 120, 96, 1, 0, "毛皮のくつ。"),
    it("かわのブーツ", Kind::Feet, 250, 200, 3, 0, "革のブーツ。"),
    // 指輪
    it("ちからのゆびわ", Kind::Ring, 400, 320, 2, 1, "つよさ+2。"),
    it("どうのゆびわ", Kind::Ring, 250, 200, 2, 2, "じょうぶさ+2。"),
    it("ガラスのゆびわ", Kind::Ring, 250, 200, 2, 3, "かしこさ+2。"),
    it("ひかりのゆびわ", Kind::Ring, 600, 480, 0, 4, "光の攻撃に強くなる。"),
    // ---- 3F〜5F ----
    it("ヒブタのロース", Kind::Food, 0, 110, 0, 0, "ヒブタのロース肉。"),
    it("ヒブタのけがわ", Kind::Loot, 0, 280, 0, 0, "高く売れる毛皮。"),
    it("火ライムのひもの", Kind::Food, 0, 120, 0, 0, "火ライムを干したもの。"),
    it("水ライムのひもの", Kind::Food, 0, 150, 0, 0, "水ライムを干したもの。"),
    it("ニンジン", Kind::Veg, 100, 50, 0, 0, "ミサイルニンジンが落とす。広場で買えるようになる。"),
    it("ハマグリ", Kind::Food, 0, 170, 0, 0, "カワウソードマンが落とすハマグリ。"),
    it("カラスネコのけがわ", Kind::Loot, 0, 400, 0, 0, "高く売れる毛皮。"),
    it("カラスネコのしっぽ", Kind::Food, 0, 0, 0, 0, "カラスネコのしっぽ。魔法屋のおじさんがほしがっている。"),
    it("ゴールドコイン", Kind::Loot, 0, 1000, 0, 0, "金貨。"),
    it("ザウルスはくせい", Kind::Quest, 0, 0, 0, 0, "ホノオザウルスのはくせい。村長に見せよう。"),
    it("カワウソのはた", Kind::Quest, 0, 0, 0, 0, "カワウソぐんだんの旗。村長に見せよう。"),
    it("テツジンのかけら", Kind::Quest, 0, 0, 0, 0, "テツジンのかけら。村長に見せよう。"),
    it("炎のたんけん", Kind::Weapon, 600, 480, 17, 0, "炎の追加攻撃を出すことがある短剣。"),
    it("ハンドアクス", Kind::Weapon, 500, 400, 18, 0, "手斧。"),
    it("ネコマタボウ", Kind::Weapon, 450, 360, 16, 0, "ネコマタの杖。"),
    it("カワウソード", Kind::Weapon, 500, 400, 21, 0, "カワウソの剣。"),
    it("ニンジンミサイル", Kind::Weapon, 500, 400, 19, 0, "連続で突く。"),
    it("ブラックダッガー", Kind::Weapon, 500, 400, 22, 0, "黒い短剣。"),
    it("光のゆみ", Kind::Weapon, 600, 480, 22, 0, "光の矢を放つ弓。"),
    it("くろのロングソード", Kind::Weapon, 750, 600, 24, 0, "必殺の一撃が出やすい。"),
    it("ハデスソード", Kind::Weapon, 450, 360, 22, 0, "冥界の剣。"),
    it("てつのつえ", Kind::Weapon, 350, 280, 15, 0, "鉄の杖。"),
    it("ロングスピア", Kind::Weapon, 500, 400, 20, 0, "長い槍。"),
    it("ヌンチャク", Kind::Weapon, 400, 320, 17, 0, "二本の棒をつないだ武器。"),
    it("ラウンドシールド", Kind::Shield, 250, 200, 4, 0, "丸い盾。"),
    it("かいのシールド", Kind::Shield, 300, 240, 5, 0, "貝の盾。水・氷に少し強い。"),
    it("アイアンシールド", Kind::Shield, 600, 480, 8, 0, "鉄の盾。"),
    it("かいのヘルメット", Kind::Head, 250, 200, 5, 0, "貝のかぶと。水・氷に少し強い。"),
    it("くろネコぼうし", Kind::Head, 450, 360, 6, 0, "黒ネコの帽子。"),
    it("アツイぼうし", Kind::Head, 350, 280, 4, 0, "炎に少し強い。"),
    it("サムイぼうし", Kind::Head, 400, 320, 4, 0, "氷に少し強い。"),
    it("ネコのティアラ", Kind::Head, 250, 200, 4, 0, "ネコのティアラ。"),
    it("アイアンヘルム", Kind::Head, 350, 280, 6, 0, "鉄のかぶと。"),
    it("光のティアラ", Kind::Head, 600, 480, 6, 0, "光に強い。"),
    it("炎のティアラ", Kind::Head, 500, 400, 5, 0, "炎に強い。"),
    it("氷のティアラ", Kind::Head, 500, 400, 5, 0, "氷に強い。"),
    it("むねあて", Kind::Body, 400, 320, 7, 0, "胸当て。"),
    it("アツイローブ", Kind::Body, 350, 280, 6, 0, "炎に強い。"),
    it("サムイローブ", Kind::Body, 500, 400, 7, 0, "氷に強い。"),
    it("かいのよろい", Kind::Body, 500, 400, 8, 0, "貝のよろい。水・氷に強い。"),
    it("ネコのドレス", Kind::Body, 500, 400, 8, 0, "ネコのドレス。"),
    it("くろねこホルター", Kind::Body, 750, 600, 12, 0, "黒ネコのホルター。"),
    it("ドリルローブ", Kind::Body, 750, 600, 11, 0, "魔法のダメージを少し減らす。"),
    it("スケールメイル", Kind::Body, 750, 600, 10, 0, "うろこのよろい。"),
    it("ブロンズブーツ", Kind::Feet, 300, 240, 6, 0, "青銅のブーツ。"),
    it("ルビーのゆびわ", Kind::Ring, 350, 280, 1, 2, "じょうぶさ+1。炎に少し強い。"),
    it("サファイアのゆびわ", Kind::Ring, 350, 280, 1, 2, "じょうぶさ+1。氷に少し強い。"),
    it("かいのゆびわ", Kind::Ring, 250, 200, 1, 2, "じょうぶさ+1。水・氷に強い。"),
    it("ぎんのゆびわ", Kind::Ring, 400, 320, 2, 2, "じょうぶさ+2。"),
    it("いちげきのゆびわ", Kind::Ring, 350, 280, 1, 1, "つよさ+1。"),
    it("だげきのゆびわ", Kind::Ring, 400, 320, 1, 1, "つよさ+1。"),
    it("シニスターリング", Kind::Ring, 800, 640, 3, 1, "つよさ+3。"),
    it("いろり", Kind::Room, 700, 560, 0, 0, "人型か炎系の魔物が集まる。"),
    it("水のみば", Kind::Room, 1500, 1200, 0, 0, "水系の魔物が集まる。"),
    it("はなぞの", Kind::Room, 2000, 1600, 0, 0, "3x3の花園。ネコ系の魔物が集まる。4方向から入れる。"),
    it("りっぱなベッド", Kind::Room, 1500, 1200, 0, 0, "高級な人型の魔物が集まる。戦闘開始時に寝ている。"),
    it("どうじょう", Kind::Room, 6000, 4800, 0, 0, "3x3の道場。戦士系の魔物が集まる。入口は1つ。"),
    // 部屋(家具)
    it("エサおけ", Kind::Room, 150, 120, 0, 0, "獣系の魔物が集まる。"),
    it("ゴミすてば", Kind::Room, 100, 80, 0, 0, "スライム系の魔物が集まる。"),
    it("わらのベッド", Kind::Room, 200, 160, 0, 0, "人型の魔物が集まる。戦闘開始時に寝ている。"),
    it("てつのたからばこ", Kind::Room, 700, 560, 0, 0, "鉄のカギで開く。その階のアイテムが出る。開けたあと2日で補充される。"),
    it("ぎんのたからばこ", Kind::Room, 4000, 3200, 0, 0, "銀のカギで開く。1階ぶん深い階のアイテムが出る。開けたあと4日で補充される。"),
    it("おうごんチェスト", Kind::Room, 9000, 7200, 0, 0, "金のカギで開く。2階ぶん深い階のアイテムが出る。開けたあと最大7日で補充される。"),
    it("さかば", Kind::Room, 600, 480, 0, 0, "人型の魔物が集まる。4方向から入れる。戦闘開始時によっぱらい状態。"),
    it("さいだん", Kind::Room, 800, 640, 0, 0, "アンデット系の魔物が集まる。"),
    it("はたけ", Kind::Room, 800, 640, 0, 0, "3x3の大きな部屋。獣・植物系の魔物が集まる。4方向から入れる。"),
    // ---- 6F〜10F ----
    it("パワーソード", Kind::Weapon, 800, 480, 26, 0, "ふらつかせることがある"),
    it("フランベルジュ", Kind::Weapon, 850, 680, 26, 0, "炎の力を持っている"),
    it("アイスソード", Kind::Weapon, 1150, 920, 28, 0, "氷の力を持っている"),
    it("ブラックブレード", Kind::Weapon, 1000, 800, 31, 0, "必殺の一撃が出やすい"),
    it("ペンギンスピア", Kind::Weapon, 750, 600, 27, 0, "水の力を持っている"),
    it("ネコマタミサイル", Kind::Weapon, 1000, 800, 29, 0, "必殺の一撃が出ることがある"),
    it("とうぞくのオノ", Kind::Weapon, 1500, 1200, 37, 0, "敵全員を攻撃することがある"),
    it("メイデンスピア", Kind::Weapon, 1000, 800, 30, 0, "敵全員を攻撃できる"),
    it("クイーンダッガー", Kind::Weapon, 1750, 1400, 39, 0, "連続攻撃が出やすい"),
    it("カイザーブレード", Kind::Weapon, 1750, 1400, 41, 0, "テイオウワーワン"),
    it("シニスタースピア", Kind::Weapon, 1750, 1400, 40, 0, "封印することがある"),
    it("氷のつえ", Kind::Weapon, 750, 600, 24, 0, "ふぶきをはき出すことがある"),
    it("カタナ", Kind::Weapon, 1250, 1000, 32, 0, "連続攻撃が出ることがある"),
    it("かいふくのつえ", Kind::Weapon, 1250, 1000, 28, 0, "味方のHPを回復"),
    it("バトルアクス", Kind::Weapon, 1000, 800, 29, 0, "敵全員を攻撃することがある"),
    it("炎のつえ", Kind::Weapon, 500, 400, 21, 0, "炎をはき出す"),
    it("モーニングスター", Kind::Weapon, 750, 600, 27, 0, "追加攻撃が出ることがある"),
    it("シルフのゆみ", Kind::Weapon, 1500, 1200, 38, 0, "連続攻撃が出ることがある"),
    it("パワーシールド", Kind::Shield, 500, 400, 9, 0, "パワーワン"),
    it("炎のたて", Kind::Shield, 450, 360, 7, 0, "炎耐性30"),
    it("氷のたて", Kind::Shield, 550, 440, 8, 0, "水・氷ダメージ軽減(水耐性30)"),
    it("氷のバックラー", Kind::Shield, 350, 280, 7, 0, "水・氷ダメージ軽減(水耐性10)"),
    it("ブラックシールド", Kind::Shield, 650, 520, 11, 0, "マスターウオリア"),
    it("テイオウのたて", Kind::Shield, 1000, 800, 14, 0, "テイオウワーワン"),
    it("カイトシールド", Kind::Shield, 800, 640, 13, 0, "B09F 鉄の宝箱、パワーワン（[B14F](h"),
    it("ちからのヘルム", Kind::Head, 450, 360, 5, 0, "攻撃力+2、ふらつき軽減大"),
    it("ブラックヘルム", Kind::Head, 600, 480, 8, 0, "ふらつき軽減"),
    it("氷のかざり", Kind::Head, 500, 400, 7, 0, "氷のダメージ軽減(水耐性20)"),
    it("ぎんのティアラ", Kind::Head, 750, 600, 9, 0, "ふらつき軽減"),
    it("おうごんのティアラ", Kind::Head, 850, 680, 10, 0, "ふらつき軽減"),
    it("キノコのハット", Kind::Head, 500, 400, 6, 0, "魔法のダメージ軽減(炎水光耐性各10)"),
    it("とうぞくのバンダナ", Kind::Head, 750, 600, 10, 0, "カモノハシーフ"),
    it("メイデンヘルム", Kind::Head, 750, 600, 9, 0, "光の魔法ダメージ軽減(光耐性10)"),
    it("シルバーヘルム", Kind::Head, 650, 520, 9, 0, "ふらつき軽減"),
    it("ゴールドヘルム", Kind::Head, 1150, 920, 10, 0, "ふらつき軽減"),
    it("プレートメイル", Kind::Body, 1100, 880, 14, 0, "パワーワン"),
    it("ブラックメイル", Kind::Body, 1500, 1200, 17, 0, "マスターウオリア"),
    it("のろいのローブ", Kind::Body, 250, 200, 1, 0, "ノロイピーマン"),
    it("ぎんいろドレス", Kind::Body, 1750, 1400, 19, 0, "ぎりぎりネコマタ"),
    it("くさったタキシード", Kind::Body, 250, 200, 2, 0, "マダシンデネー"),
    it("シルバンレザー", Kind::Body, 2000, 1600, 19, 0, "カモノハシーフ"),
    it("メイデンホルター", Kind::Body, 2500, 2000, 22, 0, "光耐性10"),
    it("クイーンホルター", Kind::Body, 2750, 2200, 23, 0, "ワーネコクイーン"),
    it("くさりのローブ", Kind::Body, 1000, 800, 14, 0, "B06F 鉄の宝箱"),
    it("ルビーホルター", Kind::Body, 1250, 1000, 12, 0, "炎のダメージ軽減(炎耐性30)"),
    it("サファイアホルター", Kind::Body, 1500, 1200, 14, 0, "氷のダメージ軽減(水耐性30)"),
    it("ブラックブーツ", Kind::Feet, 700, 560, 10, 0, "マスターウオリア"),
    it("くさったかわぐつ", Kind::Feet, 250, 200, 1, 0, "マダシンデネー"),
    it("とうぞくのくつ", Kind::Feet, 750, 600, 11, 0, "カモノハシーフ"),
    it("メイデンブーツ", Kind::Feet, 850, 680, 12, 0, "光耐性10"),
    it("ロイヤルブーツ", Kind::Feet, 1000, 800, 13, 0, "テイオウワーワン"),
    it("炎のゆびわ", Kind::Ring, 400, 320, 2, 1, "つよさ+2。武器に炎の力。"),
    it("氷のゆびわ", Kind::Ring, 400, 320, 2, 1, "つよさ+2。武器に氷の力。"),
    it("きんのゆびわ", Kind::Ring, 700, 560, 3, 2, "じょうぶさ+3。"),
    it("みこのゆびわ", Kind::Ring, 750, 600, 3, 3, "かしこさ+3。武器に光の力。"),
    it("かいりきのゆびわ", Kind::Ring, 500, 400, 2, 1, "つよさ+2。"),
    it("きりさきのゆびわ", Kind::Ring, 800, 640, 3, 1, "つよさ+3。"),
    it("大だげきのゆびわ", Kind::Ring, 650, 520, 2, 1, "つよさ+2。"),
    it("かがみのころも", Kind::Quest, 0, 0, 0, 0, "魔法をはねかえす不思議な衣。魔法屋のおじさんがほしがっている。"),
    it("いのちの石", Kind::Quest, 0, 0, 0, 0, "アクマノヨロイが持っていた石。武具屋のおねえさんがほしがっている。"),
    it("じごくのゆびわ", Kind::Quest, 0, 0, 0, 0, "ただの宝物。武具屋のおねえさんがほしがっている。"),
    it("げいにんのゆびわ", Kind::Quest, 0, 0, 0, 0, "人を笑わせる指輪。武具屋のおねえさんがほしがっている。"),
    it("チカラウシのロース", Kind::Food, 0, 200, 0, 0, "チカラウシのロース。"),
    it("ブルのサーロイン", Kind::Food, 0, 400, 0, 0, "ブルのサーロイン。"),
    it("イワシ", Kind::Food, 0, 230, 0, 0, "イワシ。"),
    it("タコのあし", Kind::Food, 0, 260, 0, 0, "タコのあし。"),
    it("かわざかな", Kind::Food, 0, 270, 0, 0, "かわざかな。"),
    it("ドクライムのひもの", Kind::Food, 0, 280, 0, 0, "ドクライムのひもの。"),
    it("ピーマン", Kind::Veg, 300, 150, 0, 0, "ノロイピーマンが落とす。広場で買えるようになる。"),
    it("チカラウシのけがわ", Kind::Loot, 0, 480, 0, 0, "高く売れる。"),
    it("ブルのけがわ", Kind::Loot, 0, 480, 0, 0, "高く売れる。"),
    it("プラチナコイン", Kind::Loot, 0, 2000, 0, 0, "高く売れる。"),
    it("ばんのうやく", Kind::Potion, 200, 120, 80, 0, "味方全員のHPを80回復する。"),
    it("いただきバクダン", Kind::Bomb, 100, 60, 18, 0, "敵全体にダメージ。"),
    it("炎のつぼ", Kind::Quest, 0, 0, 0, 0, "デブリンが持っていたつぼ。村長に見せよう。"),
    it("うみのおまもり", Kind::Quest, 0, 0, 0, 0, "アクトパスのおまもり。村長に見せよう。"),
    it("リオンのもんしょう", Kind::Quest, 0, 0, 0, 0, "キングリオンの紋章。村長に見せよう。"),
    it("ドラゴンのはくせい", Kind::Quest, 0, 0, 0, 0, "アンクルドラゴンのはくせい。村長に見せよう。"),
    it("あくまのもんしょう", Kind::Quest, 0, 0, 0, 0, "デビルプリンスの紋章。村長に見せよう。"),
    it("でたらめなけん", Kind::Weapon, 1000, 800, 1, 0, "ダメージがでたらめな変わった剣。"),
    it("だんろ", Kind::Room, 2000, 1600, 0, 0, "人型・獣系の魔物が集まる。"),
    it("けっかいのへや", Kind::Room, 2500, 2000, 0, 0, "ジゴク系の魔物が集まる。4方向から入れる。戦闘開始時にふういん状態。"),
    it("ふんすい", Kind::Room, 3000, 2400, 0, 0, "水辺の魔物が集まる。"),
    it("ごうかなベッド", Kind::Room, 3000, 2400, 0, 0, "ぜいたくな魔物が集まる。"),
    it("れいはいどう", Kind::Room, 4000, 3200, 0, 0, "気高い魔物が集まる。"),
    it("じっけんしつ", Kind::Room, 8000, 6400, 0, 0, "3x3の大きな部屋。あやしい魔物が集まる。"),
    // ---- 11F〜20F ----
    it("モグラのけがわ", Kind::Loot, 0, 800, 0, 0, "モグラのけがわ。"),
    it("モグラのももにく", Kind::Food, 0, 300, 0, 0, "モグラのももにく。"),
    it("ランバーアックス", Kind::Weapon, 2250, 1800, 45, 0, "敵全員を攻撃することがある"),
    it("ランバーキャップ", Kind::Head, 1000, 800, 11, 0, "ふらつき軽減"),
    it("ランバーベスト", Kind::Body, 2500, 2000, 22, 0, "ランバーベスト。"),
    it("みつまたやり", Kind::Weapon, 2500, 2000, 48, 0, "みつまたやり。"),
    it("ダルマのゲタ", Kind::Feet, 1250, 1000, 15, 0, "ダルマのゲタ。"),
    it("サムライえぼし", Kind::Head, 1250, 1000, 12, 0, "サムライえぼし。"),
    it("サムライのヨロイ", Kind::Body, 3250, 2600, 25, 0, "サムライのヨロイ。"),
    it("サムライシールド", Kind::Shield, 1500, 1200, 17, 0, "サムライシールド。"),
    it("きくいちもんじ", Kind::Weapon, 4500, 3600, 67, 0, "連続攻撃が出やすい"),
    it("ショーグンのカブト", Kind::Head, 3000, 2400, 16, 0, "必殺の一撃が出やすくなる"),
    it("ショーグンのヨロイ", Kind::Body, 9000, 7200, 43, 0, "ショーグンのヨロイ。"),
    it("ショーグンシールド", Kind::Shield, 3250, 2600, 25, 0, "ショーグンシールド。"),
    it("羽のかみかざり", Kind::Head, 1250, 1000, 13, 0, "炎水光耐性各10"),
    it("てばさき", Kind::Food, 0, 310, 0, 0, "てばさき。"),
    it("でたらめなたて", Kind::Shield, 2250, 1800, 0, 0, "必殺の一撃が出やすい"),
    it("でたらめなぼうし", Kind::Head, 1500, 1200, 16, 0, "でたらめなぼうし。"),
    it("でたらめなよろい", Kind::Body, 5000, 4000, 1, 0, "攻撃力+10"),
    it("でたらめなくつ", Kind::Feet, 1750, 1400, 18, 0, "でたらめなくつ。"),
    it("でたらめなゆびわ", Kind::Ring, 1249, 999, 1, 1, "でたらめなゆびわ。"),
    it("でたらめなにく", Kind::Food, 0, 330, 0, 0, "でたらめなにく。"),
    it("でたらめなにくへん", Kind::Food, 0, 160, 0, 0, "でたらめサイコロが落とす 小ぶりなにく(でたらめなにく の弱い版)。"),
    it("ファイアースピア", Kind::Weapon, 3750, 3000, 61, 0, "炎の力を持っている"),
    it("炎のローブ", Kind::Body, 5500, 4400, 24, 0, "炎耐性70"),
    it("炎のゲタ", Kind::Feet, 2000, 1600, 16, 0, "炎耐性20"),
    it("メイドのエプロン", Kind::Body, 6000, 4800, 28, 0, "攻撃力+5"),
    it("メイドのパンプス", Kind::Feet, 1850, 1480, 18, 0, "メイドのパンプス。"),
    it("まもりのゆびわ", Kind::Ring, 1200, 1000, 0, 0, "炎水光耐性各10"),
    it("キノコ", Kind::Food, 0, 360, 0, 0, "キノコ。"),
    it("トカゲのももにく", Kind::Food, 0, 370, 0, 0, "トカゲのももにく。"),
    it("ブリザードスピア", Kind::Weapon, 4250, 3400, 65, 0, "氷の力を持っている"),
    it("氷のローブ", Kind::Body, 5500, 4400, 24, 0, "水耐性70"),
    it("氷のゲタ", Kind::Feet, 2200, 1760, 17, 0, "水耐性20"),
    it("カブトムシのカブト", Kind::Head, 2000, 1600, 15, 0, "ふらつき軽減"),
    it("ビートルアーマー", Kind::Body, 6750, 5400, 36, 0, "ビートルアーマー。"),
    it("ビートルブーツ", Kind::Feet, 2250, 1800, 20, 0, "ビートルブーツ。"),
    it("せんしのゆびわ", Kind::Ring, 1000, 800, 5, 1, "攻撃力+5"),
    it("サツマイモ", Kind::Food, 0, 390, 0, 0, "サツマイモ。"),
    it("花のかみかざり", Kind::Head, 2250, 1800, 13, 0, "花のかみかざり。"),
    it("キラーリング", Kind::Ring, 1300, 1120, 5, 1, "攻撃力+5"),
    it("タネあぶら", Kind::Food, 0, 400, 0, 0, "タネあぶら。"),
    it("マスターブレード", Kind::Weapon, 5500, 4400, 73, 0, "マスターブレード。"),
    it("まてつのかぶと", Kind::Head, 2500, 2000, 15, 0, "まてつのかぶと。"),
    it("まてつのたて", Kind::Shield, 3000, 2400, 24, 0, "まてつのたて。"),
    it("まてつのよろい", Kind::Body, 7500, 6000, 38, 0, "まてつのよろい。"),
    it("まてつのブーツ", Kind::Feet, 2500, 2000, 21, 0, "まてつのブーツ。"),
    it("ソードオブファイア", Kind::Weapon, 4250, 3400, 65, 0, "炎の追加攻撃が出ることがある"),
    it("しゃくねつのつえ", Kind::Weapon, 6000, 4800, 76, 0, "炎のちからをもつ"),
    it("まどうしのかめん", Kind::Head, 2750, 2200, 13, 0, "炎水光耐性各20"),
    it("かぜのローブ", Kind::Body, 4000, 3200, 28, 0, "かぜのローブ。"),
    it("とくじょうニンジン", Kind::Veg, 800, 600, 0, 0, "マジックドンキーが落とす。"),
    it("かぜのつえ", Kind::Weapon, 3250, 2600, 57, 0, "連続攻撃が出ることがある"),
    it("ウイズコーン", Kind::Head, 2250, 1800, 14, 0, "炎水光耐性各20"),
    it("まどうのローブ", Kind::Body, 6000, 4800, 33, 0, "炎水耐性各30"),
    it("ふぶきのつえ", Kind::Weapon, 6250, 5000, 75, 0, "ふぶきをはき出す"),
    it("氷のブーツ", Kind::Feet, 600, 480, 8, 0, "水耐性30"),
    it("ホタテ", Kind::Food, 0, 500, 0, 0, "ホタテ。"),
    it("サイコロにく", Kind::Food, 0, 550, 0, 0, "サイコロにく。"),
    it("ダイヤモンド", Kind::Loot, 0, 5000, 0, 0, "ダイヤモンド。"),
    it("メイデンシールド", Kind::Shield, 3000, 2400, 16, 0, "シールドメイデンの盾。光耐性20(ワルキリーの装備より少し弱い)"),
    it("ワルキリースピア", Kind::Weapon, 5000, 4000, 70, 0, "敵全員を攻撃できる"),
    it("ワルキリーヘルム", Kind::Head, 3500, 2800, 17, 0, "光耐性20"),
    it("ワルキリーホルター", Kind::Body, 9250, 7400, 43, 0, "ワルキリーホルター。"),
    it("ワルキリーブーツ", Kind::Feet, 2500, 2000, 22, 0, "ワルキリーブーツ。"),
    it("よみのゆびわ", Kind::Ring, 1600, 1280, 0, 0, "オバケや骨の魔物に大ダメージ"),
    it("まほうのたて", Kind::Shield, 4000, 3200, 22, 0, "炎水光耐性各20"),
    it("かがみのたて", Kind::Shield, 2750, 2200, 22, 0, "光耐性30"),
    it("まけんのたて", Kind::Shield, 2000, 1600, 20, 0, "まけんのたて。"),
    it("まほうのけん", Kind::Weapon, 8250, 6600, 90, 0, "相手を確実に封印する"),
    it("カオスバスター", Kind::Weapon, 7500, 6000, 86, 0, "光の追加攻撃が出る"),
    it("まけんのつるぎ", Kind::Weapon, 3500, 2800, 58, 0, "まけんのつるぎ。"),
    it("まほうのゆみ", Kind::Weapon, 8750, 7000, 93, 0, "仲間のMPが回復することがある"),
    it("エルフのゆみ", Kind::Weapon, 6250, 5000, 78, 0, "光の追加攻撃が出る"),
    it("ひっさつのゆみ", Kind::Weapon, 4500, 3600, 66, 0, "必殺の一撃が出やすい"),
    it("炎のゆみ", Kind::Weapon, 4000, 3200, 63, 0, "炎の力を持つ"),
    it("ネコマタスペシャル", Kind::Weapon, 3000, 2400, 53, 0, "毒の矢を放つことがある"),
    it("あんこくのゆびわ", Kind::Ring, 2000, 1600, 0, 4, "光耐性50"),
    it("スリーパー", Kind::Weapon, 2250, 1800, 46, 0, "相手を眠らせる"),
    it("こうてつホルター", Kind::Body, 3250, 2600, 25, 0, "こうてつホルター。"),
    it("ちょうじんのゆびわ", Kind::Ring, 750, 600, 0, 0, "ふらつき効果UP大"),
    it("ホリーメイス", Kind::Weapon, 2250, 1800, 47, 0, "幽霊や骨の魔物に大ダメージ"),
    it("マジックローブ", Kind::Body, 3500, 2800, 23, 0, "炎水光耐性各20"),
    it("じょうぶつのゆびわ", Kind::Ring, 950, 760, 0, 0, "必殺の出る確率UP中"),
    it("ロングアクス", Kind::Weapon, 3000, 2400, 54, 0, "敵全員を攻撃することがある"),
    it("ひっさつのぼうし", Kind::Head, 1250, 1000, 12, 0, "必殺の一撃が出やすくなる"),
    it("ちからのティアラ", Kind::Head, 1750, 1400, 14, 0, "攻撃力+4"),
    it("ハルバード", Kind::Weapon, 4750, 3800, 68, 0, "敵全員を攻撃することがある"),
    it("火のはなのね", Kind::Food, 0, 600, 0, 0, "火のはなのね。"),
    it("ホリーダッガー", Kind::Weapon, 5000, 4000, 69, 0, "幽霊や骨の魔物に大ダメージ"),
    it("あくまのティアラ", Kind::Head, 2500, 2000, 16, 0, "必殺の一撃がでやすくなる"),
    it("まじょのゆびわ", Kind::Ring, 1250, 1000, 0, 0, "炎水光耐性各20"),
    it("ダイヤホルター", Kind::Body, 8250, 6600, 41, 0, "ダイヤホルター。"),
    it("まふうじのゆびわ", Kind::Ring, 1500, 1200, 0, 0, "炎水光耐性各30"),
    it("カオスブリンガー", Kind::Weapon, 7500, 6000, 86, 0, "相手に毒を与えることがある"),
    it("めがみのティアラ", Kind::Head, 3000, 2400, 17, 0, "光耐性40"),
    it("めいおうのつえ", Kind::Weapon, 8500, 6800, 92, 0, "敵のHPをぎりぎりまで落としてくれることがある"),
    it("せいぎのよろい", Kind::Body, 10000, 8000, 45, 0, "せいぎのよろい。"),
    it("ひっさつのゆびわ", Kind::Ring, 1750, 1400, 0, 0, "必殺の出る確率UP大"),
    it("いにしえのゆびわ", Kind::Ring, 2250, 1800, 0, 0, "炎水耐性各40"),
    it("めいおうのしょうこ", Kind::Quest, 0, 0, 0, 0, "デビルロードの証。村長に見せよう。"),
    it("フレイムフラワーのはな", Kind::Quest, 0, 0, 0, 0, "フレイムフラワーの花。村長に見せよう。"),
    it("エレファントムのきば", Kind::Quest, 0, 0, 0, 0, "エレファントムの牙。村長に見せよう。"),
    it("キングヘビリンゴのへた", Kind::Quest, 0, 0, 0, 0, "キングヘビリンゴのへた。村長に見せよう。"),
    it("ガイアのゆびわ", Kind::Quest, 0, 0, 0, 0, "ドロヌマの落とした指輪。村長に見せよう。"),
    it("マサムネ", Kind::Quest, 0, 0, 0, 0, "ショーグンウルフの名刀。村長に見せよう。"),
    it("デビルダッガー", Kind::Quest, 0, 0, 0, 0, "デビルプリンセスの短剣。村長に見せよう。"),
    it("ヒトツメクラブ", Kind::Quest, 0, 0, 0, 0, "ヒトツメオニのこん棒。村長に見せよう。"),
    it("コテツ", Kind::Quest, 0, 0, 0, 0, "カゲの持っていた刀。村長に見せよう。"),
    it("ミツメクラブ", Kind::Quest, 0, 0, 0, 0, "ミツメオニのこん棒。村長に見せよう。"),
    it("わぎりのにく", Kind::Quest, 0, 0, 0, 0, "エレファントムの肉。村長に見せよう。"),
    it("どくリンゴ", Kind::Quest, 0, 0, 0, 0, "キングヘビリンゴの毒リンゴ。村長に見せよう。"),
    it("ザウルスのバラにく", Kind::Food, 0, 2000, 0, 0, "ザウルスのバラにく。"),
    it("ドラゴンのヒレにく", Kind::Food, 0, 2500, 0, 0, "ドラゴンのヒレにく。"),
    it("デブリンコーン", Kind::Head, 3000, 2400, 17, 0, "デブリンコーン。"),
    it("うみのマント", Kind::Body, 10250, 8200, 36, 0, "うみのマント。"),
    it("キングのやり", Kind::Weapon, 9500, 7600, 97, 0, "キングのやり。"),
    it("リオンメイル", Kind::Body, 11000, 8800, 46, 0, "リオンメイル。"),
    it("ドラゴンマフラー", Kind::Body, 7750, 6200, 38, 0, "ドラゴンマフラー。"),
    it("シニスターヘルム", Kind::Head, 1000, 800, 11, 0, "シニスターヘルム。"),
    it("プリンスメイル", Kind::Body, 11250, 9000, 47, 0, "プリンスメイル。"),
    it("プリンスブーツ", Kind::Feet, 3000, 2400, 24, 0, "プリンスブーツ。"),
    it("コロシアム", Kind::Room, 9000, 7200, 0, 0, "3x3の大きな部屋。強い魔物が集まる。4方向から入れる。"),
    it("れいぞうこ", Kind::Room, 6000, 4800, 0, 0, "食べ物系の魔物が集まる。"),
    it("まほうじん", Kind::Room, 8000, 6400, 0, 0, "魔法の装備の魔物が集まる。魔物が最高に強くなる。"),
    it("瘴気発生装置", Kind::Room, 12000, 9600, 0, 0, "魔物は出ない。置いた階の魔物が1.2倍に強くなる(最大5個まで重ねがけ)。落とす装備に +1〜+5(置いた数まで)がつく。"),
];

/// 瘴気発生装置を 置ける 数(1つの階に)と、ふやせる 強さ
pub const MAX_MIASMA: usize = 5;
pub const MIASMA_ROOM: &str = "瘴気発生装置";

/// 「ナイフ+3」を ("ナイフ", 3) に わける。+が なければ (name, 0)
pub fn plus_split(name: &str) -> (&str, i32) {
    if let Some(i) = name.rfind('+') {
        if let Ok(k) = name[i + 1..].parse::<i32>() {
            if (1..=MAX_MIASMA as i32).contains(&k) {
                return (&name[..i], k);
            }
        }
    }
    (name, 0)
}

/// +の ない もとの 名前
pub fn base_name(name: &str) -> &str {
    plus_split(name).0
}

/// +k の 能力: もとの (1 + 0.06k) 倍(切り上げ)。特殊な 効果は ふえない
fn plus_value(base: i32, k: i32) -> i32 {
    (base * (100 + 6 * k) + 99) / 100
}

pub fn item(name: &str) -> Option<&'static ItemDef> {
    if let Some(d) = ITEMS.iter().find(|i| i.name == name) {
        return Some(d);
    }
    let (base, k) = plus_split(name);
    if k == 0 {
        return None;
    }
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<String, &'static ItemDef>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut c = cache.lock().ok()?;
    if let Some(d) = c.get(name) {
        return Some(d);
    }
    let b = ITEMS.iter().find(|i| i.name == base)?;
    if !b.kind.is_equip() {
        return None;
    }
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    let d: &'static ItemDef = Box::leak(Box::new(ItemDef { name: leaked, kind: b.kind, buy: plus_value(b.buy, k), sell: plus_value(b.sell, k), power: plus_value(b.power, k), ring: b.ring, desc: b.desc }));
    c.insert(name.to_string(), d);
    Some(d)
}

/// 初期の武具屋の品揃え(売却すると増える)
pub const ARMOR_STOCK0: &[&str] = &["ナイフ"];
pub const FURNITURE_STOCK: &[&str] = &["エサおけ", "ゴミすてば", "わらのベッド", "てつのたからばこ"];
/// 家具屋に並ぶ部屋と、その階に降りたことがあれば並ぶ(階, 部屋)
pub const FURNITURE_UNLOCK: &[(usize, &str)] = &[
    (2, "さかば"),
    (2, "さいだん"),
    (2, "はたけ"),
    (6, "ぎんのたからばこ"),
    (3, "いろり"),
    (3, "はなぞの"),
    (4, "水のみば"),
    (11, "おうごんチェスト"),
    (5, "りっぱなベッド"),
    (5, "どうじょう"),
    (6, "だんろ"),
    (6, "けっかいのへや"),
    (7, "ふんすい"),
    (8, "ごうかなベッド"),
    (9, "れいはいどう"),
    (10, "じっけんしつ"),
    (14, "コロシアム"),
    (16, "れいぞうこ"),
    (20, "まほうじん"),
    (6, "瘴気発生装置"),
];
pub const MAGIC_SHOP_ITEMS: &[&str] = &["かいふくやく", "まほうのくすり", "光バクダン", "炎バクダン"];
/// 魔法屋に並ぶ品物と、その階に降りたことがあれば並ぶ(階, 品物)
pub const MAGIC_SHOP_UNLOCK: &[(usize, &str)] = &[(2, "ふぶきバクダン")];
pub const IRON_CHEST: &[&str] = &["かわのベスト", "かわのぼうし", "ガラスのゆびわ", "どうのゆびわ", "シルバーコイン"];
pub const IRON_CHEST_2F: &[&str] = &["ショートスピア", "ガラスのティアラ", "デニムジャケット", "どうのゆびわ", "シルバーコイン"];
pub const IRON_CHEST_3F: &[&str] = &["てつのつえ", "ラウンドシールド", "ルビーのゆびわ", "だげきのゆびわ", "ゴールドコイン"];
pub const IRON_CHEST_4F: &[&str] = &["ヌンチャク", "ブロンズブーツ", "ぎんのゆびわ", "サファイアのゆびわ", "スケールメイル"];
pub const IRON_CHEST_5F: &[&str] = &["ロングスピア", "光のティアラ", "ひかりのゆびわ", "ぎんのゆびわ", "ゴールドコイン"];

pub const IRON_CHEST_6F: &[&str] = &["チカラウシのロース", "ルビーホルター", "炎のティアラ", "炎のゆびわ", "くさりのローブ"];
pub const IRON_CHEST_7F: &[&str] = &["バトルアクス", "炎のつえ", "サファイアホルター", "氷のティアラ", "氷のゆびわ"];
pub const IRON_CHEST_8F: &[&str] = &["モーニングスター", "シルバーヘルム", "かいりきのゆびわ", "ゴールドコイン", "ブルのサーロイン"];
pub const IRON_CHEST_9F: &[&str] = &["カイトシールド", "きんのゆびわ", "大だげきのゆびわ", "ゴールドコイン", "プラチナコイン", "タコのあし"];
pub const IRON_CHEST_10F: &[&str] = &["オバケタルト", "ブタイノシシのバラ", "カラスのももにく", "スライムのひもの"];

pub const IRON_CHEST_11F: &[&str] = &["シルフのゆみ", "ゴールドヘルム", "ハマグリ", "ドクライムのひもの", "カラスネコのしっぽ"];
pub const IRON_CHEST_12F: &[&str] = &["スリーパー", "こうてつホルター", "チカラウシのロース", "ブルのサーロイン", "イワシ"];
pub const IRON_CHEST_13F: &[&str] = &["ホリーメイス", "タコのあし", "マジックローブ", "かわざかな", "モグラのももにく"];
pub const IRON_CHEST_14F: &[&str] = &["カラスのももにく", "ロングアクス", "てばさき", "カラスネコのしっぽ", "ひっさつのぼうし"];
pub const IRON_CHEST_15F: &[&str] = &["かぜのローブ", "かぜのつえ", "火ライムのひもの", "チカラウシのロース", "ヒブタのロース"];
pub const IRON_CHEST_16F: &[&str] = &["ソードオブファイア", "炎のゆみ", "ちからのティアラ", "水ライムのひもの", "ブルのサーロイン"];
pub const IRON_CHEST_17F: &[&str] = &["ハルバード", "かがみのたて", "ハマグリ", "キノコ", "火のはなのね"];
pub const IRON_CHEST_18F: &[&str] = &["ホリーダッガー", "あくまのティアラ", "オバケタルト", "タネあぶら", "サツマイモ"];
pub const IRON_CHEST_19F: &[&str] = &["カラスネコのしっぽ", "ヒブタのロース", "ブルのサーロイン", "モグラのももにく", "てばさき"];
pub const IRON_CHEST_20F: &[&str] = &["エルフのゆみ", "ダイヤホルター", "ホタテ"];
/// 銀の宝箱(wiki 各階の宝箱 B10〜B20。10Fから)
pub const SILVER_CHEST: [&[&str]; 11] = [
    &["シルフのゆみ", "ゴールドヘルム", "きりさきのゆびわ", "かがみのころも", "プラチナコイン"],
    &["スリーパー", "こうてつホルター", "プラチナコイン", "ちょうじんのゆびわ", "タコのあし"],
    &["ホリーメイス", "マジックローブ", "じょうぶつのゆびわ", "プラチナコイン", "ドクライムのひもの"],
    &["まもりのゆびわ", "ロングアクス", "ブルのサーロイン", "ひっさつのぼうし", "ダイヤモンド"],
    &["てばさき", "せんしのゆびわ", "かぜのつえ", "かぜのローブ", "ダイヤモンド"],
    &["でたらめなにく", "炎のゆみ", "ソードオブファイア", "ちからのティアラ", "ダイヤモンド"],
    &["ハルバード", "かがみのたて", "タコのあし", "火のはなのね", "ダイヤモンド"],
    &["ホリーダッガー", "あくまのティアラ", "ドクライムのひもの", "ダイヤモンド"],
    &["キラーリング", "まじょのゆびわ", "ブルのサーロイン", "てばさき", "ダイヤモンド"],
    &["エルフのゆみ", "ダイヤホルター", "ダイヤモンド"],
    &["カオスバスター", "カオスブリンガー", "めがみのティアラ", "タコのあし", "ダイヤモンド"],
];
/// 金の宝箱(wiki B16〜B20。16Fから)
pub const GOLD_CHEST: [&[&str]; 5] = [
    &["ホリーダッガー", "あくまのティアラ", "ドクライムのひもの", "ダイヤモンド"],
    &["キラーリング", "まじょのゆびわ", "ブルのサーロイン", "てばさき", "ダイヤモンド"],
    &["ホリーダッガー", "エルフのゆみ", "ダイヤホルター", "あくまのティアラ", "まふうじのゆびわ"],
    &["カオスバスター", "かがみのたて", "めがみのティアラ", "まふうじのゆびわ", "まもりのゆびわ"],
    &["めいおうのつえ", "せいぎのよろい", "ひっさつのゆびわ", "プラチナコイン", "ダイヤモンド"],
];

/// 宝箱の中身の表。鉄=その階、銀=1階ぶん深い階、金=2階ぶん深い階の鉄の表。
/// 食べ物だけの表(9F・10F)は飛ばして次の階の表から。最上階を超えたら 特別な銀・金の表
pub fn chest_items(floor: usize, deeper: usize) -> &'static [&'static str] {
    // wiki に 銀・金の表がある階は それをそのまま使う
    if deeper == 1 && (10..=MAX_FLOOR).contains(&floor) {
        return SILVER_CHEST[floor - 10];
    }
    if deeper >= 2 && (16..=MAX_FLOOR).contains(&floor) {
        return GOLD_CHEST[floor - 16];
    }
    let mut f = (floor + deeper).min(MAX_FLOOR);
    while deeper > 0 && f < MAX_FLOOR && iron_chest(f).iter().all(|n| item(n).map(|d| d.kind == Kind::Food).unwrap_or(false)) {
        f += 1;
    }
    iron_chest(f)
}

pub fn iron_chest(floor: usize) -> &'static [&'static str] {
    match floor {
        1 => IRON_CHEST,
        2 => IRON_CHEST_2F,
        3 => IRON_CHEST_3F,
        4 => IRON_CHEST_4F,
        5 => IRON_CHEST_5F,
        6 => IRON_CHEST_6F,
        7 => IRON_CHEST_7F,
        8 => IRON_CHEST_8F,
        9 => IRON_CHEST_9F,
        10 => IRON_CHEST_10F,
        11 => IRON_CHEST_11F,
        12 => IRON_CHEST_12F,
        13 => IRON_CHEST_13F,
        14 => IRON_CHEST_14F,
        15 => IRON_CHEST_15F,
        16 => IRON_CHEST_16F,
        17 => IRON_CHEST_17F,
        18 => IRON_CHEST_18F,
        19 => IRON_CHEST_19F,
        20 => IRON_CHEST_20F,
        _ => panic!("iron_chest: {}F", floor),
    }
}

// ---------------- モンスター ----------------

pub struct MonDef {
    pub id: &'static str,
    pub name: &'static str,
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub spd: i32,
    pub gold: i32,
    pub drops: &'static [(&'static str, u32)],
    pub boss: bool,
    /// 光に弱い(アンデット)
    pub undead: bool,
    /// 描画用スプライト名
    pub sprite: &'static str,
}

pub static MONS: &[MonDef] = &[
    MonDef { id: "bat", name: "カラスコウモリ", hp: 20, atk: 7, def: 1, spd: 6, gold: 6, sprite: "mon_bat", boss: false, undead: false,
        drops: &[("カラスのももにく", 35)] },
    MonDef { id: "pig", name: "ブタイノシシ", hp: 26, atk: 8, def: 2, spd: 3, gold: 9, sprite: "mon_pig", boss: false, undead: false,
        drops: &[("ブタイノシシのバラ", 30), ("イノシシのけがわ", 20)] },
    MonDef { id: "cat", name: "ワーネコ", hp: 24, atk: 8, def: 2, spd: 7, gold: 10, sprite: "mon_cat", boss: false, undead: false,
        drops: &[("カツオブシ", 25), ("ワーネコホルター", 12), ("ワーネコダッガー", 6), ("ネコみみぼうし", 6), ("けがわのくつ", 8)] },
    MonDef { id: "gob", name: "こぶリン", hp: 27, atk: 9, def: 2, spd: 4, gold: 12, sprite: "mon_gob", boss: false, undead: false,
        drops: &[("こぶりなかぶと", 8), ("バックラー", 8), ("かわのくつ", 8), ("デカすぎこんぼう", 5)] },
    MonDef { id: "needleworm", name: "ニードルワーム", hp: 22, atk: 7, def: 3, spd: 3, gold: 9, sprite: "mon_needleworm", boss: false, undead: false,
        drops: &[("ワームシールド", 12)] },
    MonDef { id: "hinezumi", name: "ヒネズミ", hp: 24, atk: 8, def: 2, spd: 7, gold: 10, sprite: "mon_hinezumi", boss: false, undead: false,
        drops: &[("ヒネズミのにく", 35), ("ヒネズミのけがわ", 12)] },
    MonDef { id: "shieldmaiden", name: "シールドメイデン", hp: 96, atk: 30, def: 14, spd: 4, gold: 60, sprite: "mon_shieldmaiden", boss: false, undead: false,
        drops: &[("メイデンシールド", 10)] },
    MonDef { id: "detaramedice", name: "でたらめサイコロ", hp: 44, atk: 15, def: 4, spd: 7, gold: 22, sprite: "mon_detaramedice", boss: false, undead: false,
        drops: &[("でたらめなにくへん", 30)] },
    MonDef { id: "slime", name: "スライム", hp: 20, atk: 6, def: 1, spd: 2, gold: 5, sprite: "mon_slime", boss: false, undead: false,
        drops: &[("スライムのひもの", 35), ("ガラスのゆびわ", 3), ("どうのゆびわ", 3)] },
    MonDef { id: "wan", name: "ワーワン", hp: 34, atk: 12, def: 3, spd: 6, gold: 15, sprite: "mon_wan", boss: false, undead: false,
        drops: &[("ワーワンソード", 6), ("ウッドシールド", 8), ("キルトのぼうし", 8), ("かわよろい", 6), ("かわのブーツ", 6), ("シルバーコイン", 8)] },
    MonDef { id: "gobl", name: "ゴブリン", hp: 40, atk: 13, def: 4, spd: 4, gold: 18, sprite: "mon_gobl", boss: false, undead: false,
        drops: &[("ゴブリンクラブ", 6), ("ゴブリメット", 8), ("かわのベスト", 8), ("ちからのゆびわ", 3), ("シルバーコイン", 8)] },
    MonDef { id: "ghost", name: "ワライオバケ", hp: 30, atk: 11, def: 2, spd: 8, gold: 14, sprite: "mon_ghost", boss: false, undead: true,
        drops: &[("オバケタルト", 30), ("オバケローブ", 8), ("オバケぼうし", 6)] },
    MonDef { id: "potato", name: "バクダンイモ", hp: 24, atk: 10, def: 1, spd: 5, gold: 12, sprite: "mon_potato", boss: false, undead: false,
        drops: &[("ジャガイモ", 35), ("炎バクダン", 12), ("ふぶきバクダン", 8)] },
    MonDef { id: "lion", name: "リオン", hp: 300, atk: 22, def: 5, spd: 7, gold: 200, sprite: "mon_lion", boss: true, undead: false,
        drops: &[("リオンのたてがみ", 100)] },
    MonDef { id: "hibuta", name: "ヒブタ", hp: 46, atk: 15, def: 4, spd: 4, gold: 20, sprite: "mon_hibuta", boss: false, undead: false,
        drops: &[("ヒブタのけがわ", 25), ("ヒブタのロース", 30)] },
    MonDef { id: "ningen", name: "ニンゲンザウルス", hp: 52, atk: 16, def: 5, spd: 5, gold: 24, sprite: "mon_ningen", boss: false, undead: false,
        drops: &[("ハンドアクス", 5), ("ウッドシールド", 6), ("かわのぼうし", 6), ("むねあて", 6), ("かわのブーツ", 6), ("ゴールドコイン", 8), ("シルバーコイン", 10)] },
    MonDef { id: "firelime", name: "火ライム", hp: 36, atk: 14, def: 3, spd: 3, gold: 14, sprite: "mon_firelime", boss: false, undead: false,
        drops: &[("ルビーのゆびわ", 3), ("火ライムのひもの", 30)] },
    MonDef { id: "hotghost", name: "アツイオバケ", hp: 40, atk: 15, def: 3, spd: 8, gold: 22, sprite: "mon_hotghost", boss: false, undead: true,
        drops: &[("アツイローブ", 6), ("アツイぼうし", 6), ("オバケタルト", 25)] },
    MonDef { id: "nekomata", name: "ネコマタ", hp: 44, atk: 14, def: 4, spd: 8, gold: 24, sprite: "mon_nekomata", boss: false, undead: false,
        drops: &[("ネコマタボウ", 5), ("ガラスのティアラ", 5), ("ネコのティアラ", 5), ("ネコのドレス", 5), ("けがわのくつ", 6), ("カツオブシ", 20), ("まほうのくすり", 8)] },
    MonDef { id: "waterlime", name: "水ライム", hp: 38, atk: 15, def: 3, spd: 3, gold: 16, sprite: "mon_waterlime", boss: false, undead: false,
        drops: &[("サファイアのゆびわ", 3), ("水ライムのひもの", 30)] },
    MonDef { id: "coldghost", name: "サムイオバケ", hp: 42, atk: 16, def: 3, spd: 8, gold: 24, sprite: "mon_coldghost", boss: false, undead: true,
        drops: &[("サムイローブ", 6), ("サムイぼうし", 6), ("オバケタルト", 25)] },
    MonDef { id: "otter", name: "カワウソードマン", hp: 56, atk: 18, def: 7, spd: 6, gold: 28, sprite: "mon_otter", boss: false, undead: false,
        drops: &[("カワウソード", 5), ("かいのヘルメット", 6), ("かいのよろい", 6), ("かいのシールド", 6), ("かいのゆびわ", 10), ("ハマグリ", 25)] },
    MonDef { id: "carrot", name: "ミサイルニンジン", hp: 36, atk: 16, def: 2, spd: 6, gold: 20, sprite: "mon_carrot", boss: false, undead: false,
        drops: &[("ニンジンミサイル", 4), ("ニンジン", 30)] },
    MonDef { id: "crowcat", name: "カラスネコ", hp: 50, atk: 19, def: 5, spd: 9, gold: 30, sprite: "mon_crowcat", boss: false, undead: false,
        drops: &[("カラスネコのけがわ", 25), ("カラスネコのしっぽ", 28)] },
    MonDef { id: "blackcat", name: "ブラックワーネコ", hp: 58, atk: 20, def: 7, spd: 8, gold: 34, sprite: "mon_blackcat", boss: false, undead: false,
        drops: &[("ブラックダッガー", 5), ("くろネコぼうし", 5), ("くろねこホルター", 5), ("けがわのくつ", 5), ("ぎんのゆびわ", 5)] },
    MonDef { id: "wizdrill", name: "ウイザードリル", hp: 50, atk: 18, def: 5, spd: 6, gold: 32, sprite: "mon_wizdrill", boss: false, undead: false,
        drops: &[("光のゆみ", 4), ("キルトのぼうし", 5), ("ドリルローブ", 5), ("光バクダン", 15), ("まほうのくすり", 10)] },
    MonDef { id: "blackwar", name: "くろのウオリアー", hp: 74, atk: 23, def: 10, spd: 5, gold: 44, sprite: "mon_blackwar", boss: false, undead: false,
        drops: &[("くろのロングソード", 5), ("アイアンヘルム", 6), ("スケールメイル", 5), ("アイアンシールド", 5), ("ブロンズブーツ", 5), ("いちげきのゆびわ", 5), ("ゴールドコイン", 10)] },
    MonDef { id: "bone", name: "ホネファイター", hp: 64, atk: 20, def: 8, spd: 4, gold: 30, sprite: "mon_bone", boss: false, undead: true,
        drops: &[("ハデスソード", 5), ("アイアンヘルム", 5), ("スケールメイル", 4), ("ラウンドシールド", 6), ("ブロンズブーツ", 6)] },
    MonDef { id: "dino", name: "ホノオザウルス", hp: 460, atk: 27, def: 8, spd: 6, gold: 500, sprite: "mon_dino", boss: true, undead: false,
        drops: &[("ザウルスはくせい", 100), ("ザウルスのバラにく", 35)] },
    MonDef { id: "sorcerer", name: "カワウソーサラー", hp: 520, atk: 27, def: 10, spd: 8, gold: 600, sprite: "mon_sorcerer", boss: true, undead: false,
        drops: &[("カワウソのはた", 100)] },
    MonDef { id: "tetsu", name: "テツジン", hp: 560, atk: 32, def: 16, spd: 5, gold: 800, sprite: "mon_tetsu", boss: true, undead: false,
        drops: &[("テツジンのかけら", 100)] },
    MonDef { id: "boss", name: "ヘビリンゴ", hp: 150, atk: 13, def: 3, spd: 5, gold: 80, sprite: "mon_boss", boss: true, undead: false,
        drops: &[("ヘビのはっぱ", 100)] },
    // ---- 6F〜10F ----
    MonDef { id: "chikaraushi", name: "チカラウシ", hp: 110, atk: 45, def: 13, spd: 5, gold: 52, sprite: "mon_chikaraushi", boss: false, undead: false,
        drops: &[("ちからのヘルム", 6), ("チカラウシのけがわ", 25), ("チカラウシのロース", 30)] },
    MonDef { id: "powerwan", name: "パワーワン", hp: 101, atk: 41, def: 13, spd: 5, gold: 52, sprite: "mon_powerwan", boss: false, undead: false,
        drops: &[("パワーソード", 5), ("アイアンヘルム", 6), ("プレートメイル", 5), ("パワーシールド", 6), ("ブロンズブーツ", 6), ("ちからのゆびわ", 4)] },
    MonDef { id: "flamewar", name: "炎のウオリアー", hp: 110, atk: 45, def: 13, spd: 5, gold: 52, sprite: "mon_flamewar", boss: false, undead: false,
        drops: &[("フランベルジュ", 5), ("炎のたて", 6), ("ゴールドコイン", 10)] },
    MonDef { id: "icewar", name: "氷のウオリアー", hp: 136, atk: 52, def: 16, spd: 6, gold: 64, sprite: "mon_icewar", boss: false, undead: false,
        drops: &[("アイスソード", 5), ("氷のたて", 6), ("氷のバックラー", 6)] },
    MonDef { id: "masterwar", name: "マスターウオリア", hp: 204, atk: 59, def: 22, spd: 7, gold: 76, sprite: "mon_masterwar", boss: false, undead: false,
        drops: &[("ブラックブレード", 4), ("ブラックヘルム", 5), ("ブラックメイル", 4), ("ブラックシールド", 5), ("ブラックブーツ", 5)] },
    MonDef { id: "hotjigoku", name: "ホノオジゴク", hp: 82, atk: 41, def: 13, spd: 5, gold: 52, sprite: "mon_hotjigoku", boss: false, undead: false,
        drops: &[("ルビーのゆびわ", 5), ("炎のゆびわ", 5), ("じごくのゆびわ", 12)] },
    MonDef { id: "icejigoku", name: "フブキジゴク", hp: 102, atk: 48, def: 16, spd: 6, gold: 64, sprite: "mon_icejigoku", boss: false, undead: false,
        drops: &[("サファイアのゆびわ", 5), ("氷のゆびわ", 5), ("じごくのゆびわ", 12)] },
    MonDef { id: "buffalo", name: "バッファローブル", hp: 148, atk: 52, def: 19, spd: 6, gold: 64, sprite: "mon_buffalo", boss: false, undead: false,
        drops: &[("ブルのけがわ", 25), ("ブルのサーロイン", 28)] },
    MonDef { id: "penguin", name: "ワーペン", hp: 114, atk: 48, def: 16, spd: 9, gold: 64, sprite: "mon_penguin", boss: false, undead: false,
        drops: &[("ペンギンスピア", 5), ("氷のバックラー", 6), ("氷のかざり", 6), ("イワシ", 28)] },
    MonDef { id: "pepper", name: "ノロイピーマン", hp: 122, atk: 55, def: 19, spd: 7, gold: 76, sprite: "mon_pepper", boss: false, undead: false,
        drops: &[("のろいのローブ", 6), ("ピーマン", 30)] },
    MonDef { id: "nekomata2", name: "ぎりぎりネコマタ", hp: 136, atk: 55, def: 19, spd: 10, gold: 76, sprite: "mon_nekomata2", boss: false, undead: false,
        drops: &[("ネコマタミサイル", 5), ("ぎんのティアラ", 5), ("ぎんいろドレス", 5), ("きんのゆびわ", 4), ("タコのあし", 20), ("ばんのうやく", 8)] },
    MonDef { id: "nekomata3", name: "いただきネコマタ", hp: 173, atk: 62, def: 22, spd: 7, gold: 88, sprite: "mon_nekomata3", boss: false, undead: false,
        drops: &[("ネコマタミサイル", 5), ("ぎんのティアラ", 5), ("おうごんのティアラ", 5), ("きんのゆびわ", 5), ("いただきバクダン", 10), ("かわざかな", 20)] },
    MonDef { id: "zombie", name: "マダシンデネー", hp: 149, atk: 55, def: 19, spd: 10, gold: 76, sprite: "mon_zombie", boss: false, undead: true,
        drops: &[("キノコのハット", 6), ("くさったタキシード", 6), ("くさったかわぐつ", 6), ("げいにんのゆびわ", 12)] },
    MonDef { id: "platypus", name: "カモノハシーフ", hp: 158, atk: 62, def: 22, spd: 7, gold: 88, sprite: "mon_platypus", boss: false, undead: false,
        drops: &[("とうぞくのオノ", 5), ("とうぞくのバンダナ", 5), ("シルバンレザー", 5), ("とうぞくのくつ", 6), ("かわざかな", 22)] },
    MonDef { id: "valkyrie", name: "ワルキリー", hp: 189, atk: 66, def: 22, spd: 4, gold: 88, sprite: "mon_valkyrie", boss: false, undead: false,
        drops: &[("メイデンスピア", 5), ("メイデンヘルム", 5), ("メイデンホルター", 5), ("メイデンブーツ", 5), ("みこのゆびわ", 5)] },
    MonDef { id: "catqueen", name: "ワーネコクイーン", hp: 234, atk: 73, def: 28, spd: 5, gold: 100, sprite: "mon_catqueen", boss: false, undead: false,
        drops: &[("クイーンダッガー", 5), ("クイーンホルター", 5), ("メイデンブーツ", 5), ("きんのゆびわ", 5), ("きりさきのゆびわ", 4), ("ばんのうやく", 8)] },
    MonDef { id: "emperorwan", name: "テイオウワーワン", hp: 251, atk: 73, def: 28, spd: 5, gold: 100, sprite: "mon_emperorwan", boss: false, undead: false,
        drops: &[("カイザーブレード", 5), ("テイオウのたて", 5), ("ロイヤルブーツ", 5), ("かいりきのゆびわ", 5)] },
    MonDef { id: "poisonlime", name: "ドクライム", hp: 180, atk: 69, def: 25, spd: 5, gold: 100, sprite: "mon_poisonlime", boss: false, undead: false,
        drops: &[("ドクライムのひもの", 30), ("きんのゆびわ", 4), ("ぎんのゆびわ", 5), ("炎のゆびわ", 4), ("氷のゆびわ", 4)] },
    MonDef { id: "demonarmor", name: "アクマノヨロイ", hp: 288, atk: 73, def: 28, spd: 5, gold: 100, sprite: "mon_demonarmor", boss: false, undead: false,
        drops: &[("シニスタースピア", 5), ("じごくのゆびわ", 12), ("プラチナコイン", 6), ("いのちの石", 14)] },
    MonDef { id: "debrin", name: "デブリン", hp: 800, atk: 46, def: 15, spd: 6, gold: 900, sprite: "mon_debrin", boss: true, undead: false,
        drops: &[("炎のつぼ", 100)] },
    MonDef { id: "octopus", name: "アクトパス", hp: 1000, atk: 52, def: 18, spd: 7, gold: 1100, sprite: "mon_octopus", boss: true, undead: false,
        drops: &[("うみのおまもり", 100)] },
    MonDef { id: "kinglion", name: "キングリオン", hp: 1250, atk: 58, def: 21, spd: 9, gold: 1300, sprite: "mon_kinglion", boss: true, undead: true,
        drops: &[("リオンのもんしょう", 100)] },
    MonDef { id: "dragon", name: "アンクルドラゴン", hp: 1500, atk: 64, def: 24, spd: 7, gold: 1500, sprite: "mon_dragon", boss: true, undead: false,
        drops: &[("ドラゴンのはくせい", 100), ("ドラゴンのヒレにく", 35)] },
    MonDef { id: "devilprince", name: "デビルプリンス", hp: 1900, atk: 72, def: 28, spd: 10, gold: 2500, sprite: "mon_devilprince", boss: true, undead: false,
        drops: &[("あくまのもんしょう", 100)] },
    // ---- 11F〜20F ----
    MonDef { id: "anagerilla", name: "アナゲリラ", hp: 181, atk: 76, def: 28, spd: 4, gold: 112, sprite: "mon_anagerilla", boss: false, undead: false,
        drops: &[("モグラのけがわ", 12), ("モグラのももにく", 28)] },
    MonDef { id: "lumberduck", name: "ランバーダック", hp: 222, atk: 76, def: 28, spd: 6, gold: 112, sprite: "mon_lumberduck", boss: false, undead: false,
        drops: &[("ランバーアックス", 5), ("ランバーキャップ", 6), ("ランバーベスト", 6), ("とうぞくのくつ", 6), ("ハマグリ", 28), ("イワシ", 28), ("かわざかな", 28)] },
    MonDef { id: "daruma", name: "ダルマ", hp: 246, atk: 83, def: 31, spd: 7, gold: 124, sprite: "mon_daruma", boss: false, undead: false,
        drops: &[("みつまたやり", 5), ("くさりのローブ", 6), ("ダルマのゲタ", 6), ("ゴールドコイン", 12)] },
    MonDef { id: "samuraiwolf", name: "サムライウルフ", hp: 268, atk: 87, def: 31, spd: 8, gold: 124, sprite: "mon_samuraiwolf", boss: false, undead: false,
        drops: &[("カタナ", 5), ("サムライえぼし", 6), ("サムライのヨロイ", 6), ("サムライシールド", 6), ("ブタイノシシのバラ", 28), ("カラスのももにく", 28), ("カラスネコのしっぽ", 28)] },
    MonDef { id: "shogun", name: "ショーグンウルフ", hp: 642, atk: 136, def: 55, spd: 9, gold: 208, sprite: "mon_shogunwolf", boss: false, undead: false,
        drops: &[("きくいちもんじ", 5), ("ショーグンのカブト", 6), ("ショーグンのヨロイ", 6), ("ショーグンシールド", 6)] },
    MonDef { id: "killercock", name: "キラーコック", hp: 246, atk: 90, def: 34, spd: 9, gold: 136, sprite: "mon_killercock", boss: false, undead: false,
        drops: &[("羽のかみかざり", 6), ("てばさき", 28)] },
    MonDef { id: "hidaruma", name: "ヒダルマ", hp: 290, atk: 104, def: 40, spd: 6, gold: 160, sprite: "mon_hidaruma", boss: false, undead: false,
        drops: &[("ファイアースピア", 5), ("炎のローブ", 6), ("炎のゲタ", 6), ("シルバーコイン", 12)] },
    MonDef { id: "maidoll", name: "メイドール", hp: 319, atk: 104, def: 40, spd: 6, gold: 160, sprite: "mon_maidoll", boss: false, undead: false,
        drops: &[("メイドのエプロン", 6), ("メイドのパンプス", 6), ("まもりのゆびわ", 4), ("きんのゆびわ", 4)] },
    MonDef { id: "yukidaruma", name: "ユキダルマ", hp: 312, atk: 111, def: 43, spd: 7, gold: 172, sprite: "mon_yukidaruma", boss: false, undead: false,
        drops: &[("ブリザードスピア", 5), ("氷のローブ", 6), ("氷のゲタ", 6), ("ゴールドコイン", 12)] },
    MonDef { id: "mushroomlizard", name: "キノコトカゲ", hp: 280, atk: 111, def: 43, spd: 7, gold: 172, sprite: "mon_mushroomlizard", boss: false, undead: false,
        drops: &[("キノコ", 28), ("トカゲのももにく", 28)] },
    MonDef { id: "warbeetle", name: "ワービートル", hp: 405, atk: 115, def: 46, spd: 5, gold: 172, sprite: "mon_warbeetle", boss: false, undead: false,
        drops: &[("きくいちもんじ", 5), ("カブトムシのカブト", 6), ("ビートルアーマー", 6), ("ビートルブーツ", 6), ("せんしのゆびわ", 4)] },
    MonDef { id: "flamemaster", name: "炎のマスター", hp: 467, atk: 122, def: 49, spd: 4, gold: 184, sprite: "mon_flamemaster", boss: false, undead: false,
        drops: &[("ソードオブファイア", 5)] },
    MonDef { id: "highmaster", name: "ハイマスター", hp: 501, atk: 122, def: 49, spd: 4, gold: 184, sprite: "mon_highmaster", boss: false, undead: false,
        drops: &[("マスターブレード", 5), ("まてつのかぶと", 6), ("まてつのたて", 6), ("まてつのよろい", 6), ("まてつのブーツ", 6)] },
    MonDef { id: "detarame", name: "デタラメ", hp: 268, atk: 97, def: 37, spd: 5, gold: 148, sprite: "mon_detarame", boss: false, undead: false,
        drops: &[("でたらめなけん", 5), ("でたらめなたて", 6), ("でたらめなぼうし", 6), ("でたらめなよろい", 6), ("でたらめなくつ", 6), ("でたらめなゆびわ", 4), ("でたらめなにく", 28)] },
    MonDef { id: "caterpillar", name: "イモムシ", hp: 300, atk: 118, def: 46, spd: 2, gold: 184, sprite: "mon_caterpillar", boss: false, undead: false,
        drops: &[("サツマイモ", 28)] },
    MonDef { id: "killerflower", name: "キラーフラワー", hp: 334, atk: 118, def: 46, spd: 4, gold: 184, sprite: "mon_killerflower", boss: false, undead: false,
        drops: &[("花のかみかざり", 6), ("キラーリング", 4), ("タネあぶら", 28)] },
    MonDef { id: "bonewizard", name: "ホネウイザード", hp: 356, atk: 125, def: 49, spd: 5, gold: 196, sprite: "mon_bonewizard", boss: false, undead: true,
        drops: &[("かぜのつえ", 5), ("ウイズコーン", 6), ("まどうのローブ", 6), ("かわのくつ", 6)] },
    MonDef { id: "magicdonkey", name: "マジックドンキー", hp: 391, atk: 125, def: 49, spd: 5, gold: 196, sprite: "mon_magicdonkey", boss: false, undead: false,
        drops: &[("しゃくねつのつえ", 5), ("まどうしのかめん", 6), ("かぜのローブ", 6), ("とくじょうニンジン", 28)] },
    MonDef { id: "devildice", name: "アクマノサイコロ", hp: 415, atk: 132, def: 52, spd: 7, gold: 208, sprite: "mon_devildice", boss: false, undead: false,
        drops: &[("サイコロにく", 28), ("まほうのくすり", 12), ("ばんのうやく", 12), ("ダイヤモンド", 12)] },
    MonDef { id: "hellscream", name: "ジゴクノサケビ", hp: 480, atk: 143, def: 55, spd: 9, gold: 220, sprite: "mon_hellscream", boss: false, undead: true,
        drops: &[("せんしのゆびわ", 4), ("でたらめなゆびわ", 4), ("キラーリング", 4), ("あんこくのゆびわ", 4)] },
    MonDef { id: "magicshield", name: "マホウノタテ", hp: 480, atk: 143, def: 55, spd: 7, gold: 220, sprite: "mon_magicshield", boss: false, undead: false,
        drops: &[("まほうのたて", 6), ("まてつのたて", 6), ("かがみのたて", 6), ("まけんのたて", 6), ("サムライシールド", 6), ("まほうのくすり", 12)] },
    MonDef { id: "magicbow", name: "マホウノユミ", hp: 440, atk: 139, def: 55, spd: 8, gold: 220, sprite: "mon_magicbow", boss: false, undead: false,
        drops: &[("まほうのゆみ", 5), ("エルフのゆみ", 5), ("ひっさつのゆみ", 5), ("炎のゆみ", 5), ("ネコマタスペシャル", 5), ("シルフのゆみ", 5)] },
    MonDef { id: "magicsword", name: "マホウノケン", hp: 480, atk: 143, def: 55, spd: 9, gold: 220, sprite: "mon_magicsword", boss: false, undead: false,
        drops: &[("まほうのけん", 5), ("カオスバスター", 5), ("マスターブレード", 5), ("ソードオブファイア", 5), ("でたらめなけん", 5), ("まけんのつるぎ", 5)] },
    MonDef { id: "valkyrie2", name: "ワルキリー＋", hp: 491, atk: 136, def: 55, spd: 6, gold: 208, sprite: "mon_valkyrie", boss: false, undead: false,
        drops: &[("ワルキリースピア", 5), ("ワルキリーヘルム", 6), ("ワルキリーホルター", 6), ("ワルキリーブーツ", 6), ("よみのゆびわ", 4), ("まもりのゆびわ", 4)] },
    MonDef { id: "sorcerer_n", name: "カワウソーサラー", hp: 427, atk: 129, def: 49, spd: 5, gold: 196, sprite: "mon_sorcerer", boss: false, undead: false,
        drops: &[("ふぶきのつえ", 5), ("かいふくのつえ", 5), ("ウイズコーン", 6), ("氷のローブ", 6), ("ホタテ", 28)] },
    MonDef { id: "dorunuma", name: "ドロヌマ", hp: 2070, atk: 78, def: 31, spd: 10, gold: 2900, sprite: "mon_dorunuma", boss: true, undead: false,
        drops: &[("ガイアのゆびわ", 100)] },
    MonDef { id: "shogunboss", name: "ショーグンウルフ", hp: 2329, atk: 84, def: 34, spd: 8, gold: 3300, sprite: "mon_shogunboss", boss: true, undead: false,
        drops: &[("マサムネ", 100)] },
    MonDef { id: "devilprincess", name: "デビルプリンセス", hp: 2602, atk: 90, def: 37, spd: 9, gold: 3700, sprite: "mon_devilprincess", boss: true, undead: false,
        drops: &[("デビルダッガー", 100)] },
    MonDef { id: "hitotsuoni", name: "ヒトツメオニ", hp: 2889, atk: 96, def: 40, spd: 10, gold: 4100, sprite: "mon_hitotsuoni", boss: true, undead: false,
        drops: &[("ヒトツメクラブ", 100)] },
    MonDef { id: "flameflower", name: "フレイムフラワー", hp: 3190, atk: 102, def: 43, spd: 8, gold: 4500, sprite: "mon_flameflower", boss: true, undead: false,
        drops: &[("フレイムフラワーのはな", 100), ("火のはなのね", 100)] },
    MonDef { id: "kage", name: "カゲ", hp: 3504, atk: 108, def: 46, spd: 9, gold: 4900, sprite: "mon_kage", boss: true, undead: false,
        drops: &[("コテツ", 100)] },
    MonDef { id: "mitsuoni", name: "ミツメオニ", hp: 3831, atk: 114, def: 49, spd: 10, gold: 5300, sprite: "mon_mitsuoni", boss: true, undead: false,
        drops: &[("ミツメクラブ", 100)] },
    MonDef { id: "elephantom", name: "エレファントム", hp: 4172, atk: 120, def: 52, spd: 8, gold: 5700, sprite: "mon_elephantom", boss: true, undead: false,
        drops: &[("エレファントムのきば", 100), ("わぎりのにく", 100)] },
    MonDef { id: "devillord", name: "デビルロード", hp: 4527, atk: 126, def: 55, spd: 9, gold: 6100, sprite: "mon_devillord", boss: true, undead: false,
        drops: &[("めいおうのしょうこ", 100), ("カオスブリンガー", 100)] },
    MonDef { id: "kingsnake", name: "キングヘビリンゴ", hp: 5040, atk: 132, def: 58, spd: 10, gold: 6500, sprite: "mon_kingsnake", boss: true, undead: false,
        drops: &[("キングヘビリンゴのへた", 100), ("どくリンゴ", 100), ("ザウルスのバラにく", 20), ("ドラゴンのヒレにく", 20), ("火のはなのね", 20), ("わぎりのにく", 20), ("カオスブリンガー", 15), ("炎のローブ", 15), ("まてつのたて", 15), ("まほうのたて", 15), ("ショーグンシールド", 15), ("デブリンコーン", 15), ("うみのマント", 15), ("キングのやり", 15), ("リオンメイル", 15), ("ドラゴンマフラー", 15), ("シニスターヘルム", 15), ("プリンスメイル", 15), ("プリンスブーツ", 15), ("まてつのブーツ", 15)] },
];

pub fn mon(id: &str) -> &'static MonDef {
    MONS.iter().find(|m| m.id == id).expect("monster id")
}

/// 部屋ごとに湧くモンスター(wiki 各階のページ)
/// その階に表が無い部屋は、表のある いちばん近い階の魔物で うめる(同じ近さなら 浅い階)
pub fn room_spawns(floor: usize, room: &str) -> &'static [&'static str] {
    if room == MIASMA_ROOM {
        return &[]; // 瘴気発生装置からは 魔物が 出ない
    }
    let own = room_spawns_exact(floor, room);
    if !own.is_empty() {
        return own;
    }
    let mut best: Option<(usize, usize)> = None;
    for f in 1..=MAX_FLOOR {
        if !room_spawns_exact(f, room).is_empty() {
            let d = f.abs_diff(floor);
            if best.map(|(bd, _)| d < bd).unwrap_or(true) {
                best = Some((d, f));
            }
        }
    }
    best.map(|(_, f)| room_spawns_exact(f, room)).unwrap_or(&[])
}

/// wiki の表そのまま(ない階は空)
pub fn room_spawns_exact(floor: usize, room: &str) -> &'static [&'static str] {
    match (floor, room) {
        // 1F
        (1, "エサおけ") => &["bat", "pig"],
        (1, "ゴミすてば") => &["slime"],
        (1, "わらのベッド") => &["cat", "gob"],
        (1, "さかば") | (1, "さいだん") | (1, "はたけ") => &["needleworm"],
        (1, "いろり") | (2, "いろり") => &["hinezumi"],
        // 5Fまでのれいはいどう
        (1, "れいはいどう") | (2, "れいはいどう") | (3, "れいはいどう") | (4, "れいはいどう") | (5, "れいはいどう") => &["shieldmaiden"],
        // 2F
        (2, "エサおけ") => &["bat", "pig"],
        (2, "ゴミすてば") => &["slime"],
        (2, "わらのベッド") => &["cat", "wan"],
        (2, "さかば") => &["gobl"],
        (2, "さいだん") => &["ghost"],
        (2, "はたけ") => &["bat", "pig", "potato"],
        // 3F
        (3, "エサおけ") => &["hibuta"],
        (3, "ゴミすてば") => &["firelime"],
        (3, "わらのベッド") => &["wan", "ningen"],
        (3, "いろり") => &["hibuta", "ningen"],
        (3, "どうじょう") => &["gob", "blackcat"],
        (3, "さかば") => &["gobl"],
        (3, "さいだん") => &["hotghost"],
        (3, "はたけ") => &["hibuta", "potato"],
        (3, "はなぞの") => &["bat", "cat", "nekomata"],
        // 4F
        (4, "エサおけ") => &["bat", "pig"],
        (4, "ゴミすてば") => &["waterlime"],
        (4, "わらのベッド") => &["wan", "otter"],
        // wikiの4Fは いろり が空欄。3Fと5Fの表から補った(設計値)
        (4, "いろり") => &["hibuta", "ningen"],
        (4, "水のみば") => &["otter"],
        (4, "さかば") => &["gobl", "otter"],
        (4, "さいだん") => &["coldghost"],
        (4, "はたけ") => &["pig", "potato", "carrot"],
        (4, "はなぞの") => &["bat", "cat", "nekomata"],
        // 5F
        (5, "エサおけ") => &["crowcat"],
        (5, "ゴミすてば") => &["slime", "firelime", "waterlime"],
        (5, "わらのベッド") => &["cat", "wan"],
        (5, "りっぱなベッド") => &["blackcat", "wizdrill"],
        (5, "いろり") => &["ningen"],
        (5, "水のみば") => &["waterlime", "otter"],
        (5, "さかば") => &["gobl", "otter"],
        (5, "さいだん") => &["bone"],
        (5, "はたけ") => &["pig", "carrot", "potato", "crowcat"],
        (5, "はなぞの") => &["nekomata", "blackcat"],
        (5, "どうじょう") => &["gobl", "ningen", "blackwar"],
        // 6F〜10F
        (6, "エサおけ") => &["hibuta", "chikaraushi"],
        (6, "ゴミすてば") => &["firelime"],
        (6, "わらのベッド") => &["cat"],
        (6, "りっぱなベッド") => &["blackcat", "powerwan"],
        (6, "いろり") => &["ningen"],
        (6, "だんろ") => &["ningen", "flamewar"],
        (6, "水のみば") => &["bat", "crowcat"],
        (6, "さかば") => &["gobl", "wizdrill"],
        (6, "けっかいのへや") => &["hotjigoku"],
        (6, "さいだん") => &["hotghost"],
        (6, "はたけ") => &["potato", "chikaraushi"],
        (6, "はなぞの") => &["nekomata", "blackcat"],
        (6, "どうじょう") => &["blackcat", "blackwar", "powerwan"],
        (7, "エサおけ") => &["crowcat", "chikaraushi", "buffalo"],
        (7, "ゴミすてば") => &["waterlime"],
        (7, "わらのベッド") => &["cat", "wan"],
        (7, "りっぱなベッド") => &["blackcat", "powerwan"],
        (7, "いろり") => &["bat"],
        (7, "だんろ") => &["ningen", "flamewar"],
        (7, "水のみば") => &["otter", "penguin"],
        (7, "ふんすい") => &["otter", "penguin"],
        (7, "さかば") => &["otter", "wizdrill"],
        (7, "けっかいのへや") => &["icejigoku"],
        (7, "さいだん") => &["coldghost"],
        (7, "はたけ") => &["carrot", "buffalo"],
        (7, "はなぞの") => &["nekomata", "blackcat"],
        (7, "どうじょう") => &["otter", "blackwar", "powerwan", "icewar"],
        (8, "エサおけ") => &["crowcat", "buffalo"],
        (8, "ゴミすてば") => &["slime", "firelime", "waterlime"],
        (8, "わらのベッド") => &["cat", "wan"],
        (8, "りっぱなベッド") => &["blackcat", "powerwan"],
        (8, "ごうかなベッド") => &["wizdrill", "nekomata2", "masterwar"],
        (8, "いろり") => &["hibuta", "ningen"],
        (8, "だんろ") => &["ningen", "flamewar"],
        (8, "水のみば") => &["otter"],
        (8, "ふんすい") => &["otter", "penguin", "icewar"],
        (8, "さかば") => &["gobl", "wizdrill", "otter"],
        (8, "けっかいのへや") => &["hotjigoku", "icejigoku"],
        (8, "さいだん") => &["zombie"],
        (8, "はたけ") => &["potato", "carrot", "pepper"],
        (8, "はなぞの") => &["nekomata", "blackcat", "nekomata2"],
        (8, "どうじょう") => &["masterwar"],
        (9, "エサおけ") => &["bat", "pig", "crowcat", "buffalo"],
        (9, "ゴミすてば") => &["slime", "firelime", "waterlime"],
        (9, "わらのベッド") => &["wan"],
        (9, "りっぱなベッド") => &["platypus"],
        (9, "ごうかなベッド") => &["wizdrill", "nekomata2"],
        (9, "いろり") => &["ningen"],
        (9, "だんろ") => &["ningen", "flamewar"],
        (9, "水のみば") => &["otter", "penguin", "platypus"],
        (9, "ふんすい") => &["penguin", "platypus"],
        (9, "さかば") => &["otter", "wizdrill", "penguin"],
        (9, "けっかいのへや") => &["hotjigoku", "icejigoku"],
        (9, "さいだん") => &["ghost", "hotghost", "coldghost"],
        (9, "れいはいどう") => &["valkyrie"],
        (9, "はたけ") => &["potato", "carrot", "pepper"],
        (9, "はなぞの") => &["nekomata", "blackcat", "nekomata3"],
        (9, "どうじょう") => &["flamewar", "icewar", "masterwar"],
        (10, "エサおけ") => &["pig", "crowcat", "chikaraushi"],
        (10, "ゴミすてば") => &["poisonlime"],
        (10, "わらのベッド") => &["cat"],
        (10, "りっぱなベッド") => &["blackcat", "powerwan"],
        (10, "ごうかなベッド") => &["emperorwan", "catqueen"],
        (10, "いろり") => &["hibuta", "ningen"],
        (10, "だんろ") => &["ningen", "flamewar"],
        (10, "水のみば") => &["otter", "buffalo", "penguin", "platypus"],
        (10, "ふんすい") => &["penguin", "platypus"],
        (10, "さかば") => &["ningen", "wizdrill", "platypus"],
        (10, "けっかいのへや") => &["hotjigoku", "icejigoku"],
        (10, "さいだん") => &["ghost", "hotghost", "coldghost", "zombie"],
        (10, "れいはいどう") => &["valkyrie", "catqueen", "emperorwan"],
        (10, "はたけ") => &["potato", "carrot", "pepper"],
        (10, "はなぞの") => &["nekomata2", "nekomata3", "catqueen"],
        (10, "どうじょう") => &["flamewar", "icewar", "masterwar"],
        (10, "じっけんしつ") => &["wizdrill", "demonarmor"],
        // 11F〜20F
        (11, "エサおけ") => &["buffalo", "anagerilla"],
        (11, "ゴミすてば") => &["slime", "firelime", "waterlime", "poisonlime"],
        (11, "わらのベッド") => &["cat", "wan"],
        (11, "りっぱなベッド") => &["lumberduck"],
        (11, "ごうかなベッド") => &["catqueen", "emperorwan"],
        (11, "いろり") => &["hibuta", "ningen"],
        (11, "だんろ") => &["ningen", "flamewar"],
        (11, "水のみば") => &["otter", "penguin", "buffalo", "platypus"],
        (11, "ふんすい") => &["penguin", "platypus"],
        (11, "さかば") => &["wizdrill", "lumberduck"],
        (11, "けっかいのへや") => &["hotjigoku", "icejigoku"],
        (11, "さいだん") => &["ghost", "hotghost", "coldghost"],
        (11, "れいはいどう") => &["valkyrie", "catqueen", "emperorwan"],
        (11, "はたけ") => &["potato", "carrot", "anagerilla", "lumberduck"],
        (11, "はなぞの") => &["nekomata", "blackcat", "nekomata2"],
        (11, "どうじょう") => &["flamewar", "icewar", "masterwar"],
        (11, "じっけんしつ") => &["wizdrill", "demonarmor"],
        (12, "エサおけ") => &["pig", "chikaraushi", "anagerilla"],
        (12, "ゴミすてば") => &["slime", "firelime", "waterlime", "poisonlime"],
        (12, "わらのベッド") => &["cat", "wan"],
        (12, "りっぱなベッド") => &["daruma"],
        (12, "ごうかなベッド") => &["samuraiwolf"],
        (12, "いろり") => &["hibuta", "ningen"],
        (12, "だんろ") => &["ningen", "flamewar"],
        (12, "水のみば") => &["penguin", "buffalo", "platypus"],
        (12, "ふんすい") => &["penguin", "platypus"],
        (12, "さかば") => &["gobl", "wizdrill", "platypus", "lumberduck"],
        (12, "けっかいのへや") => &["hotjigoku", "icejigoku"],
        (12, "さいだん") => &["ghost", "zombie", "hotghost", "coldghost"],
        (12, "れいはいどう") => &["valkyrie", "catqueen", "emperorwan"],
        (12, "はたけ") => &["potato", "carrot", "pepper", "anagerilla"],
        (12, "はなぞの") => &["nekomata", "blackcat", "nekomata2", "nekomata3"],
        (12, "どうじょう") => &["daruma", "samuraiwolf"],
        (12, "じっけんしつ") => &["wizdrill", "demonarmor"],
        (13, "エサおけ") => &["crowcat", "killercock"],
        (13, "ゴミすてば") => &["slime", "firelime", "waterlime", "poisonlime"],
        (13, "わらのベッド") => &["cat"],
        (13, "りっぱなベッド") => &["blackcat"],
        (13, "ごうかなベッド") => &["catqueen", "emperorwan"],
        (13, "いろり") => &["hibuta", "ningen"],
        (13, "だんろ") => &["ningen", "flamewar"],
        (13, "水のみば") => &["otter", "penguin", "platypus"],
        (13, "ふんすい") => &["penguin", "platypus"],
        (13, "さかば") => &["otter", "platypus"],
        (13, "けっかいのへや") => &["hotjigoku", "icejigoku"],
        (13, "さいだん") => &["ghost", "zombie", "hotghost", "coldghost"],
        (13, "れいはいどう") => &["valkyrie", "emperorwan"],
        (13, "はたけ") => &["potato", "carrot", "pepper", "killercock"],
        (13, "はなぞの") => &["nekomata2", "catqueen"],
        (13, "どうじょう") => &["flamewar", "icewar", "masterwar"],
        (13, "じっけんしつ") => &["wizdrill", "demonarmor"],
        (14, "エサおけ") => &["chikaraushi", "buffalo", "anagerilla"],
        (14, "ゴミすてば") => &["slime", "firelime", "poisonlime"],
        (14, "わらのベッド") => &["cat", "wan"],
        (14, "りっぱなベッド") => &["blackcat", "powerwan"],
        (14, "ごうかなベッド") => &["catqueen", "emperorwan"],
        (14, "いろり") => &["hibuta", "ningen"],
        (14, "だんろ") => &["ningen", "flamewar"],
        (14, "水のみば") => &["otter", "penguin", "buffalo", "platypus"],
        (14, "ふんすい") => &["penguin", "platypus"],
        (14, "さかば") => &["gobl", "wizdrill", "otter", "lumberduck"],
        (14, "けっかいのへや") => &["hotjigoku", "icejigoku"],
        (14, "さいだん") => &["detarame"],
        (14, "れいはいどう") => &["valkyrie", "emperorwan"],
        (14, "はたけ") => &["potato", "carrot", "pepper", "anagerilla"],
        (14, "はなぞの") => &["nekomata2", "catqueen"],
        (14, "どうじょう") => &["flamewar", "icewar", "masterwar"],
        (14, "じっけんしつ") => &["wizdrill", "demonarmor"],
        (14, "コロシアム") => &["wan", "powerwan", "emperorwan", "samuraiwolf"],
        (15, "エサおけ") => &["hibuta"],
        (15, "ゴミすてば") => &["firelime"],
        (15, "わらのベッド") => &["cat", "wan"],
        (15, "りっぱなベッド") => &["ningen"],
        (15, "ごうかなベッド") => &["maidoll"],
        (15, "いろり") => &["hibuta", "ningen"],
        (15, "だんろ") => &["hidaruma"],
        (15, "水のみば") => &["bat"],
        (15, "ふんすい") => &["bat"],
        (15, "さかば") => &["gobl", "wizdrill", "lumberduck"],
        (15, "けっかいのへや") => &["hotjigoku"],
        (15, "さいだん") => &["hotghost"],
        (15, "れいはいどう") => &["valkyrie", "emperorwan"],
        (15, "はたけ") => &["potato"],
        (15, "はなぞの") => &["nekomata2", "catqueen"],
        (15, "どうじょう") => &["flamewar"],
        (15, "じっけんしつ") => &["wizdrill", "demonarmor"],
        (15, "コロシアム") => &["ningen", "lumberduck", "emperorwan", "samuraiwolf"],
        (16, "エサおけ") => &["crowcat", "killercock"],
        (16, "ゴミすてば") => &["waterlime"],
        (16, "わらのベッド") => &["cat", "wan"],
        (16, "りっぱなベッド") => &["blackcat", "powerwan"],
        (16, "ごうかなベッド") => &["warbeetle"],
        (16, "いろり") => &["bat"],
        (16, "だんろ") => &["bat", "killercock"],
        (16, "水のみば") => &["penguin", "buffalo", "platypus"],
        (16, "ふんすい") => &["killercock", "yukidaruma"],
        (16, "さかば") => &["otter", "platypus", "lumberduck"],
        (16, "けっかいのへや") => &["icejigoku"],
        (16, "さいだん") => &["coldghost"],
        (16, "れいはいどう") => &["valkyrie", "catqueen"],
        (16, "はたけ") => &["carrot", "pepper", "killercock"],
        (16, "はなぞの") => &["nekomata2", "nekomata3", "catqueen"],
        (16, "どうじょう") => &["icewar"],
        (16, "じっけんしつ") => &["wizdrill", "demonarmor"],
        (16, "コロシアム") => &["ningen", "emperorwan", "samuraiwolf", "warbeetle"],
        (16, "れいぞうこ") => &["mushroomlizard"],
        (17, "エサおけ") => &["bat", "chikaraushi", "mushroomlizard"],
        (17, "ゴミすてば") => &["firelime", "poisonlime"],
        (17, "わらのベッド") => &["cat", "wan"],
        (17, "りっぱなベッド") => &["blackcat", "powerwan"],
        (17, "ごうかなベッド") => &["maidoll"],
        (17, "いろり") => &["hibuta", "ningen"],
        (17, "だんろ") => &["hidaruma", "flamemaster"],
        (17, "水のみば") => &["otter", "buffalo", "penguin", "platypus"],
        (17, "ふんすい") => &["penguin", "yukidaruma"],
        (17, "さかば") => &["platypus", "maidoll"],
        (17, "けっかいのへや") => &["hotjigoku", "icejigoku"],
        (17, "さいだん") => &["zombie", "detarame"],
        (17, "れいはいどう") => &["valkyrie"],
        (17, "はたけ") => &["carrot", "pepper", "caterpillar", "killerflower"],
        (17, "はなぞの") => &["nekomata", "nekomata2", "nekomata3", "catqueen"],
        (17, "どうじょう") => &["flamemaster", "highmaster"],
        (17, "じっけんしつ") => &["wizdrill", "demonarmor"],
        (17, "コロシアム") => &["ningen", "warbeetle", "flamemaster", "highmaster"],
        (17, "れいぞうこ") => &["buffalo", "coldghost", "mushroomlizard"],
        (18, "エサおけ") => &["pig", "crowcat", "buffalo", "mushroomlizard"],
        (18, "ゴミすてば") => &["firelime", "poisonlime"],
        (18, "わらのベッド") => &["cat", "wan"],
        (18, "りっぱなベッド") => &["lumberduck"],
        (18, "ごうかなベッド") => &["catqueen", "emperorwan"],
        (18, "いろり") => &["hibuta", "ningen", "hidaruma"],
        (18, "だんろ") => &["hidaruma", "flamemaster"],
        (18, "水のみば") => &["otter", "buffalo"],
        (18, "ふんすい") => &["otter", "sorcerer_n"],
        (18, "さかば") => &["gobl", "lumberduck"],
        (18, "けっかいのへや") => &["hotjigoku", "icejigoku"],
        (18, "さいだん") => &["bonewizard"],
        (18, "れいはいどう") => &["valkyrie", "emperorwan"],
        (18, "はたけ") => &["carrot", "pepper", "caterpillar", "killerflower"],
        (18, "はなぞの") => &["nekomata", "nekomata2", "nekomata3", "catqueen"],
        (18, "どうじょう") => &["icewar", "highmaster", "flamemaster"],
        (18, "じっけんしつ") => &["demonarmor", "bonewizard", "magicdonkey"],
        (18, "コロシアム") => &["ningen", "lumberduck", "warbeetle", "highmaster"],
        (18, "れいぞうこ") => &["coldghost", "buffalo", "mushroomlizard"],
        (19, "エサおけ") => &["bat", "pig", "crowcat", "killercock"],
        (19, "ゴミすてば") => &["slime", "poisonlime"],
        (19, "わらのベッド") => &["cat", "wan"],
        (19, "りっぱなベッド") => &["blackcat", "powerwan"],
        (19, "ごうかなベッド") => &["shogun"],
        (19, "いろり") => &["ningen", "hibuta", "hidaruma"],
        (19, "だんろ") => &["hidaruma", "flamemaster"],
        (19, "水のみば") => &["otter", "buffalo", "penguin", "platypus"],
        (19, "ふんすい") => &["otter", "penguin", "lumberduck", "sorcerer_n"],
        (19, "さかば") => &["gobl", "lumberduck", "platypus", "maidoll"],
        (19, "けっかいのへや") => &["devildice"],
        (19, "さいだん") => &["zombie", "detarame", "bonewizard"],
        (19, "れいはいどう") => &["catqueen", "valkyrie2"],
        (19, "はたけ") => &["carrot", "pepper", "caterpillar", "killerflower"],
        (19, "はなぞの") => &["nekomata", "nekomata2", "nekomata3", "catqueen"],
        (19, "どうじょう") => &["icewar", "highmaster", "flamemaster"],
        (19, "じっけんしつ") => &["demonarmor", "bonewizard", "magicdonkey"],
        (19, "コロシアム") => &["lumberduck", "warbeetle", "highmaster", "shogun"],
        (19, "れいぞうこ") => &["buffalo", "mushroomlizard"],
        (20, "エサおけ") => &["pig", "chikaraushi", "killercock", "anagerilla"],
        (20, "ゴミすてば") => &["slime", "firelime", "waterlime", "poisonlime"],
        (20, "わらのベッド") => &["cat", "wan"],
        (20, "りっぱなベッド") => &["blackcat", "powerwan", "lumberduck", "warbeetle"],
        (20, "ごうかなベッド") => &["catqueen", "emperorwan", "maidoll", "shogun"],
        (20, "いろり") => &["hibuta", "ningen", "hidaruma"],
        (20, "だんろ") => &["hidaruma", "flamemaster"],
        (20, "水のみば") => &["otter", "buffalo", "penguin", "platypus"],
        (20, "ふんすい") => &["otter", "penguin", "lumberduck", "sorcerer_n"],
        (20, "さかば") => &["gobl", "platypus", "lumberduck", "maidoll"],
        (20, "けっかいのへや") => &["hotjigoku", "icejigoku", "devildice", "hellscream"],
        (20, "さいだん") => &["zombie", "detarame", "bonewizard"],
        (20, "れいはいどう") => &["catqueen", "emperorwan", "valkyrie2"],
        (20, "はたけ") => &["carrot", "pepper", "caterpillar", "killerflower"],
        (20, "はなぞの") => &["nekomata", "nekomata2", "nekomata3", "catqueen"],
        (20, "どうじょう") => &["icewar", "highmaster", "magicshield"],
        (20, "じっけんしつ") => &["demonarmor", "bonewizard", "magicdonkey", "magicbow"],
        (20, "コロシアム") => &["highmaster", "shogun", "magicsword"],
        (20, "れいぞうこ") => &["coldghost", "buffalo", "mushroomlizard"],
        (20, "まほうじん") => &["hellscream", "magicsword", "magicshield", "magicbow"],
                // 1〜9Fのじっけんしつ: その階の魔物 + でたらめサイコロ(設計値)
        (1, "じっけんしつ") => &["bat", "pig", "slime", "cat", "gob", "needleworm", "hinezumi", "detaramedice", "detaramedice"],
        (2, "じっけんしつ") => &["hinezumi", "bat", "pig", "slime", "cat", "wan", "gobl", "ghost", "potato", "detaramedice", "detaramedice", "detaramedice"],
        (3, "じっけんしつ") => &["hibuta", "firelime", "wan", "ningen", "gob", "blackcat", "gobl", "hotghost", "potato", "bat", "cat", "nekomata", "detaramedice", "detaramedice", "detaramedice", "detaramedice"],
        (4, "じっけんしつ") => &["bat", "pig", "waterlime", "wan", "otter", "hibuta", "ningen", "gobl", "coldghost", "potato", "carrot", "cat", "nekomata", "detaramedice", "detaramedice", "detaramedice", "detaramedice"],
        (5, "じっけんしつ") => &["crowcat", "slime", "firelime", "waterlime", "cat", "wan", "blackcat", "wizdrill", "ningen", "otter", "gobl", "bone", "pig", "carrot", "potato", "nekomata", "blackwar", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice"],
        (6, "じっけんしつ") => &["hibuta", "chikaraushi", "firelime", "cat", "blackcat", "powerwan", "ningen", "flamewar", "bat", "crowcat", "gobl", "wizdrill", "hotjigoku", "hotghost", "potato", "nekomata", "blackwar", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice"],
        (7, "じっけんしつ") => &["crowcat", "chikaraushi", "buffalo", "waterlime", "cat", "wan", "blackcat", "powerwan", "bat", "ningen", "flamewar", "otter", "penguin", "wizdrill", "icejigoku", "coldghost", "carrot", "nekomata", "blackwar", "icewar", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice"],
        (8, "じっけんしつ") => &["crowcat", "buffalo", "slime", "firelime", "waterlime", "cat", "wan", "blackcat", "powerwan", "wizdrill", "nekomata2", "masterwar", "hibuta", "ningen", "flamewar", "otter", "penguin", "icewar", "gobl", "hotjigoku", "icejigoku", "zombie", "potato", "carrot", "pepper", "nekomata", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice"],
        (9, "じっけんしつ") => &["bat", "pig", "crowcat", "buffalo", "slime", "firelime", "waterlime", "wan", "platypus", "wizdrill", "nekomata2", "ningen", "flamewar", "otter", "penguin", "hotjigoku", "icejigoku", "ghost", "hotghost", "coldghost", "valkyrie", "potato", "carrot", "pepper", "nekomata", "blackcat", "nekomata3", "icewar", "masterwar", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice", "detaramedice"],
        // 1〜19Fのまほうじん: その階のボスが出る(設計値)
        (1, "まほうじん") => &["boss"],
        (2, "まほうじん") => &["lion"],
        (3, "まほうじん") => &["dino"],
        (4, "まほうじん") => &["sorcerer"],
        (5, "まほうじん") => &["tetsu"],
        (6, "まほうじん") => &["debrin"],
        (7, "まほうじん") => &["octopus"],
        (8, "まほうじん") => &["kinglion"],
        (9, "まほうじん") => &["dragon"],
        (10, "まほうじん") => &["devilprince"],
        (11, "まほうじん") => &["dorunuma"],
        (12, "まほうじん") => &["shogunboss"],
        (13, "まほうじん") => &["devilprincess"],
        (14, "まほうじん") => &["hitotsuoni"],
        (15, "まほうじん") => &["flameflower"],
        (16, "まほうじん") => &["kage"],
        (17, "まほうじん") => &["mitsuoni"],
        (18, "まほうじん") => &["elephantom"],
        (19, "まほうじん") => &["devillord"],
        _ => &[],
    }
}

pub fn corridor_spawn(floor: usize) -> &'static [&'static str] {
    match floor {
        1 => &["bat"],
        2 => &["bat", "pig"],
        3 => &["bat", "hibuta"],
        4 => &["pig"],
        5 => &["crowcat"],
        6 => &["hibuta", "chikaraushi"],
        7 => &["buffalo"],
        8 => &["bat", "crowcat", "chikaraushi"],
        9 => &["pig"],
        10 => &["ghost", "hotghost", "coldghost", "bone"],
        11 => &["buffalo", "anagerilla"],
        12 => &["chikaraushi", "anagerilla"],
        13 => &["crowcat"],
        14 => &["chikaraushi"],
        15 => &[],
        16 => &["buffalo", "mushroomlizard"],
        17 => &["bat", "chikaraushi", "mushroomlizard"],
        18 => &["pig", "crowcat", "mushroomlizard"],
        19 => &["anagerilla"],
        20 => &["crowcat", "chikaraushi", "anagerilla", "killercock"],
        _ => panic!("corridor_spawn: {}F", floor),
    }
}

// ---------------- 料理 ----------------

pub struct Recipe {
    pub name: &'static str,
    pub needs: &'static [&'static str],
    /// HP, MP, つよさ, すばやさ, じょうぶさ, かしこさ
    pub gain: [i32; 6],
    /// 最初から作れるなら None。魔法屋に渡すと覚える食材
    pub taught_by: Option<&'static str>,
}

pub static RECIPES: &[Recipe] = &[
    Recipe { name: "ホウレンソウサラダ", needs: &["ホウレンソウ"], gain: [1, 0, 0, 0, 0, 0], taught_by: None },
    Recipe { name: "フライドカラス", needs: &["カラスのももにく"], gain: [1, 0, 0, 1, 0, 0], taught_by: Some("カラスのももにく") },
    Recipe { name: "ブタイノシシなべ", needs: &["ブタイノシシのバラ", "ホウレンソウ"], gain: [1, 0, 1, 0, 0, 0], taught_by: Some("ブタイノシシのバラ") },
    Recipe { name: "おひたし", needs: &["カツオブシ", "ホウレンソウ"], gain: [0, 2, 0, 0, 0, 0], taught_by: Some("カツオブシ") },
    Recipe { name: "にっころがし", needs: &["ジャガイモ", "カツオブシ"], gain: [1, 1, 0, 0, 0, 0], taught_by: Some("ジャガイモ") },
    Recipe { name: "オバケのタルト", needs: &["オバケタルト"], gain: [0, 2, 0, 0, 0, 1], taught_by: Some("オバケタルト") },
    Recipe { name: "ヒネズミのあぶりやき", needs: &["ヒネズミのにく", "ホウレンソウ"], gain: [0, 1, 0, 2, 1, 0], taught_by: Some("ヒネズミのにく") },
    Recipe { name: "でたらめソテー", needs: &["でたらめなにくへん", "ホウレンソウ"], gain: [2, 1, 2, 1, 1, 1], taught_by: Some("でたらめなにくへん") },
    Recipe { name: "ヒブタのソテー", needs: &["ヒブタのロース", "ジャガイモ"], gain: [2, 0, 1, 0, 0, 0], taught_by: Some("ヒブタのロース") },
    Recipe { name: "やき火ライム", needs: &["火ライムのひもの", "ジャガイモ"], gain: [2, 0, 0, 0, 2, 0], taught_by: Some("火ライムのひもの") },
    Recipe { name: "ニンジンジュース", needs: &["ニンジン", "オバケタルト"], gain: [0, 2, 0, 0, 0, 2], taught_by: Some("ニンジン") },
    Recipe { name: "水ライムソーメン", needs: &["水ライムのひもの", "ブタイノシシのバラ", "ホウレンソウ"], gain: [0, 2, 1, 0, 2, 0], taught_by: Some("水ライムのひもの") },
    Recipe { name: "やきハマグリ", needs: &["ハマグリ", "ハマグリ"], gain: [0, 2, 0, 0, 0, 3], taught_by: Some("ハマグリ") },
    Recipe { name: "カラスネコシチュー", needs: &["カラスネコのしっぽ", "カラスのももにく", "ジャガイモ", "ニンジン"], gain: [2, 1, 0, 2, 0, 0], taught_by: Some("カラスネコのしっぽ") },
    Recipe { name: "ちからの出るグリル", needs: &["チカラウシのロース", "ブタイノシシのバラ", "ホウレンソウ"], gain: [3, 0, 3, 0, 0, 0], taught_by: Some("チカラウシのロース") },
    Recipe { name: "ブルのステーキ", needs: &["ブルのサーロイン", "ジャガイモ"], gain: [4, 0, 2, 0, 0, 0], taught_by: Some("ブルのサーロイン") },
    Recipe { name: "メザシ", needs: &["イワシ", "イワシ", "イワシ", "イワシ"], gain: [0, 4, 0, 0, 0, 4], taught_by: Some("イワシ") },
    Recipe { name: "シーフードグリル", needs: &["タコのあし", "ハマグリ", "イワシ"], gain: [0, 2, 0, 2, 0, 3], taught_by: Some("タコのあし") },
    Recipe { name: "ヒブタのスブタ", needs: &["ピーマン", "ピーマン", "ヒブタのロース", "ニンジン", "カラスネコのしっぽ"], gain: [3, 0, 2, 2, 0, 0], taught_by: Some("ピーマン") },
    Recipe { name: "やきざかな", needs: &["かわざかな", "かわざかな"], gain: [0, 3, 0, 0, 0, 3], taught_by: Some("かわざかな") },
    Recipe { name: "ドクライムなべ", needs: &["ドクライムのひもの", "チカラウシのロース", "ホウレンソウ"], gain: [3, 0, 1, 0, 5, 0], taught_by: Some("ドクライムのひもの") },
    Recipe { name: "モグラのにくジャガ", needs: &["モグラのももにく", "ジャガイモ", "カツオブシ"], gain: [2, 1, 2, 0, 0, 1], taught_by: Some("モグラのももにく") },
    Recipe { name: "てばさきのフライ", needs: &["てばさき", "てばさき"], gain: [3, 0, 0, 3, 0, 0], taught_by: Some("てばさき") },
    Recipe { name: "でたらめシチュー", needs: &["でたらめなにく", "ホウレンソウ", "ジャガイモ", "ニンジン"], gain: [4, 2, 1, 1, 1, 1], taught_by: Some("でたらめなにく") },
    Recipe { name: "火のはなのサラダ", needs: &["火のはなのね"], gain: [4, 5, 0, 0, 0, 0], taught_by: Some("火のはなのね") },
    Recipe { name: "とくじょうステーキ", needs: &["キノコ", "ブルのサーロイン"], gain: [6, 0, 3, 0, 2, 0], taught_by: Some("キノコ") },
    Recipe { name: "トカゲのくしやき", needs: &["トカゲのももにく", "てばさき", "ハマグリ", "ピーマン"], gain: [3, 0, 0, 3, 0, 2], taught_by: Some("トカゲのももにく") },
    Recipe { name: "ヤキイモ", needs: &["サツマイモ", "サツマイモ", "サツマイモ"], gain: [0, 5, 0, 3, 2, 0], taught_by: Some("サツマイモ") },
    Recipe { name: "山のさちのソテー", needs: &["タネあぶら", "かわざかな", "キノコ"], gain: [4, 2, 0, 0, 3, 2], taught_by: Some("タネあぶら") },
    Recipe { name: "とくじょうサラダ", needs: &["とくじょうニンジン", "モグラのももにく"], gain: [0, 4, 1, 2, 0, 0], taught_by: Some("とくじょうニンジン") },
    Recipe { name: "クラムチャウダー", needs: &["ホタテ", "とくじょうニンジン"], gain: [0, 5, 0, 1, 0, 3], taught_by: Some("ホタテ") },
    Recipe { name: "わぎりの丸やき", needs: &["わぎりのにく"], gain: [9, 0, 5, 0, 0, 0], taught_by: Some("わぎりのにく") },
    Recipe { name: "サイコロステーキ", needs: &["サイコロにく", "サイコロにく", "トカゲのももにく", "ピーマン", "ニンジン"], gain: [5, 2, 0, 1, 1, 1], taught_by: Some("サイコロにく") },
    Recipe { name: "やきリンゴ", needs: &["どくリンゴ"], gain: [3, 5, 0, 4, 0, 0], taught_by: Some("どくリンゴ") },
    Recipe { name: "ザウルスなべ", needs: &["ザウルスのバラにく", "キノコ"], gain: [7, 0, 0, 0, 6, 0], taught_by: Some("ザウルスのバラにく") },
    Recipe { name: "ドラゴンステーキ", needs: &["ドラゴンのヒレにく", "タネあぶら", "ホウレンソウ"], gain: [8, 4, 4, 2, 2, 1], taught_by: Some("ドラゴンのヒレにく") },
    Recipe { name: "スライムにこみ", needs: &["スライムのひもの", "カラスのももにく"], gain: [2, 0, 0, 0, 1, 0], taught_by: Some("スライムのひもの") },
];

pub fn recipe(name: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.name == name)
}

/// 料理の 表示順: 教わる 食材が 手に入る 階の 浅い 順(最初から作れる物が 先頭)。同じ階なら 登録順
pub fn recipe_order() -> Vec<&'static Recipe> {
    let mut v: Vec<(usize, usize, &'static Recipe)> = RECIPES
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let floor = match r.taught_by {
                None => 0,
                Some(n) => food_origin(n).map(|(f, _)| f).unwrap_or(1),
            };
            (floor, i, r)
        })
        .collect();
    v.sort_by_key(|(f, i, _)| (*f, *i));
    v.into_iter().map(|(_, _, r)| r).collect()
}

// ---------------- クエスト ----------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Giver {
    Mayor,
    Magic,
    Armor,
}

pub struct Quest {
    pub id: &'static str,
    pub giver: Giver,
    pub title: &'static str,
    pub text: &'static str,
    pub need: &'static str,
    pub prepay: i32,
    pub reward_gold: i32,
    pub reward_item: Option<&'static str>,
    /// この階に降りたことがあれば受けられる
    pub min_floor: usize,
    /// メインクエスト(自動で受けた状態になる)
    pub main: bool,
    /// 必要な個数
    pub count: i32,
    /// 報酬の魔法(アレンが覚える)
    pub reward_spell: Option<&'static str>,
}

pub static QUESTS: &[Quest] = &[
    Quest { id: "snake", giver: Giver::Mayor, title: "ヘビリンゴたいじ",
        text: "森をうろつくまもの「ヘビリンゴ」を\nダンジョンにおびきよせてたおしてくれ。",
        need: "ヘビのはっぱ", prepay: 0, reward_gold: 2000, reward_item: Some("魔法のオリーブオイル"), min_floor: 1, main: true, count: 1, reward_spell: None },
    Quest { id: "crow", giver: Giver::Mayor, title: "カラスのももにく",
        text: "カラスコウモリのももにくがほしいんじゃ。\n1階で手に入るはずじゃよ。",
        need: "カラスのももにく", prepay: 0, reward_gold: 500, reward_item: None, min_floor: 1, main: false, count: 1, reward_spell: None },
    Quest { id: "slime", giver: Giver::Magic, title: "スライムのひもの",
        text: "スライムのひものが食べたいなあ。\nゴミすてばを置いてみてはどうかな。",
        need: "スライムのひもの", prepay: 200, reward_gold: 0, reward_item: None, min_floor: 1, main: false, count: 1, reward_spell: None },
    Quest { id: "halter", giver: Giver::Armor, title: "ワーネコのホルター",
        text: "ワーネコのホルターがほしいの。\nわらのベッドに出てくるワーネコが持ってるわ。",
        need: "ワーネコホルター", prepay: 0, reward_gold: 400, reward_item: Some("ひかりのゆびわ"), min_floor: 1, main: false, count: 1, reward_spell: None },
    Quest { id: "tart", giver: Giver::Magic, title: "オバケタルト",
        text: "オバケタルトが食べたいのう。\n各階にさいだんを置けば、オバケが出るぞ。",
        need: "オバケタルト", prepay: 400, reward_gold: 0, reward_item: Some("魔法のオリーブオイル"), min_floor: 2, main: false, count: 1, reward_spell: None },
    Quest { id: "gobl", giver: Giver::Mayor, title: "ゴブリンたいじ",
        text: "地下2階にゴブリンたちが集まっておる。\nボスのリオンをたおして、ダンジョンを広げてくれ。",
        need: "リオンのたてがみ", prepay: 0, reward_gold: 3000, reward_item: None, min_floor: 2, main: true, count: 1, reward_spell: None },
    Quest { id: "hibuta", giver: Giver::Magic, title: "ヒブタのロース",
        text: "ヒブタのロースが食べたいのう。\n3階のエサおけやいろりに出るヒブタが持っておるぞ。",
        need: "ヒブタのロース", prepay: 0, reward_gold: 0, reward_item: Some("魔法のオリーブオイル"), min_floor: 3, main: false, count: 1, reward_spell: None },
    Quest { id: "dino", giver: Giver::Mayor, title: "ホノオザウルスのはくせい",
        text: "見せものにして観光客を集めたい。\n地下3階のボス、ホノオザウルスのはくせいを持ってきてくれ。",
        need: "ザウルスはくせい", prepay: 0, reward_gold: 3500, reward_item: None, min_floor: 3, main: true, count: 1, reward_spell: None },
    Quest { id: "otterring", giver: Giver::Armor, title: "かいのゆびわ",
        text: "かいのゆびわがほしいの。\n4階のカワウソードマンが持ってるわ。",
        need: "かいのゆびわ", prepay: 0, reward_gold: 0, reward_item: Some("炎のたんけん"), min_floor: 4, main: false, count: 1, reward_spell: None },
    Quest { id: "otter", giver: Giver::Mayor, title: "カワウソぐんだんのボス",
        text: "カワウソぐんだんが地下4階にあらわれた。\nボスのカワウソーサラーをたおしてくれ。",
        need: "カワウソのはた", prepay: 0, reward_gold: 4000, reward_item: Some("魔法のオリーブオイル"), min_floor: 4, main: true, count: 1, reward_spell: None },
    Quest { id: "crowtail", giver: Giver::Magic, title: "カラスネコのしっぽ",
        text: "カラスネコのしっぽが2本ほしいのう。\n5階のはたけやエサおけにおるぞ。",
        need: "カラスネコのしっぽ", prepay: 700, reward_gold: 0, reward_item: None, min_floor: 5, main: false, count: 2, reward_spell: Some("healall") },
    Quest { id: "iron", giver: Giver::Mayor, title: "テツジンたいじ",
        text: "地下5階にテツジンがおる。ものすごく硬いが、\n魔法や必殺の一撃ならなんとかなるはずじゃ。",
        need: "テツジンのかけら", prepay: 0, reward_gold: 5000, reward_item: None, min_floor: 5, main: true, count: 1, reward_spell: None },
    // ---- 6F〜10F ----
    Quest { id: "debrin", giver: Giver::Mayor, title: "デブリンたいじ",
        text: "地下6階にデブリンがあらわれた。\n炎のつぼを持ち帰ってくれ。",
        need: "炎のつぼ", prepay: 0, reward_gold: 6000, reward_item: None, min_floor: 6, main: true, count: 1, reward_spell: None },
    Quest { id: "jigoku", giver: Giver::Armor, title: "じごくのゆびわ",
        text: "じごくのゆびわがほしいの。\n6階のけっかいのへやにいるジゴクが持ってるわ。",
        need: "じごくのゆびわ", prepay: 0, reward_gold: 0, reward_item: Some("氷のつえ"), min_floor: 6, main: false, count: 1, reward_spell: None },
    Quest { id: "octopus", giver: Giver::Mayor, title: "アクトパスたいじ",
        text: "地下7階の海のぬしアクトパスをたおして、\nうみのおまもりを持ってきてくれ。",
        need: "うみのおまもり", prepay: 0, reward_gold: 6500, reward_item: None, min_floor: 7, main: true, count: 1, reward_spell: None },
    Quest { id: "sirloin", giver: Giver::Magic, title: "ブルのサーロイン",
        text: "ブルのサーロインが食べたいのう。\n7階のエサおけやはたけに出るバッファローブルが持っておるぞ。",
        need: "ブルのサーロイン", prepay: 800, reward_gold: 0, reward_item: Some("魔法のオリーブオイル"), min_floor: 7, main: false, count: 1, reward_spell: None },
    Quest { id: "kinglion", giver: Giver::Mayor, title: "キングリオンたいじ",
        text: "地下8階にキングリオンが君臨しておる。\n光に弱いらしい。もんしょうを持ち帰ってくれ。",
        need: "リオンのもんしょう", prepay: 0, reward_gold: 7000, reward_item: None, min_floor: 8, main: true, count: 1, reward_spell: None },
    Quest { id: "comedian", giver: Giver::Armor, title: "げいにんのゆびわ",
        text: "げいにんのゆびわがほしいの。\n8〜9階のさいだんのマダシンデネーが持ってるわ。",
        need: "げいにんのゆびわ", prepay: 0, reward_gold: 0, reward_item: Some("カタナ"), min_floor: 8, main: false, count: 1, reward_spell: None },
    Quest { id: "dragon", giver: Giver::Mayor, title: "ドラゴンたいじ",
        text: "地下9階にアンクルドラゴンがおる。\nはくせいにして村の名物にしたい!",
        need: "ドラゴンのはくせい", prepay: 0, reward_gold: 7500, reward_item: Some("魔法のオリーブオイル"), min_floor: 9, main: true, count: 1, reward_spell: None },
    Quest { id: "devil", giver: Giver::Mayor, title: "デビルプリンスたいじ",
        text: "地下10階にデビルプリンスが住みついた。\nあくまのもんしょうを持ってきてくれ。",
        need: "あくまのもんしょう", prepay: 0, reward_gold: 10000, reward_item: None, min_floor: 10, main: true, count: 1, reward_spell: None },
    Quest { id: "mirror", giver: Giver::Magic, title: "かがみのころも",
        text: "かがみのころも、というものを探しておる。\n10階にぎんのたからばこを置けば 手に入るかもしれん。",
        need: "かがみのころも", prepay: 0, reward_gold: 0, reward_item: None, min_floor: 10, main: false, count: 1, reward_spell: Some("reflect") },
    Quest { id: "lifestone", giver: Giver::Armor, title: "いのちの石",
        text: "いのちの石がほしいの。\n10階にじっけんしつを置いて、アクマノヨロイを倒せば 持ってるかも。",
        need: "いのちの石", prepay: 0, reward_gold: 0, reward_item: Some("かいふくのつえ"), min_floor: 10, main: false, count: 1, reward_spell: None },
    // ---- 11F〜20F ----
    Quest { id: "dorunuma", giver: Giver::Mayor, title: "ドロヌマたいじ",
        text: "地下11階の沼にドロヌマがおる。\nガイアのゆびわを持ち帰ってくれ。",
        need: "ガイアのゆびわ", prepay: 0, reward_gold: 11000, reward_item: None, min_floor: 11, main: true, count: 1, reward_spell: None },
    Quest { id: "shogunboss", giver: Giver::Mayor, title: "ショーグンウルフたいじ",
        text: "地下12階にウルフのショーグンがくる。\n名刀マサムネを持ち帰ってくれ。",
        need: "マサムネ", prepay: 0, reward_gold: 12000, reward_item: None, min_floor: 12, main: true, count: 1, reward_spell: None },
    Quest { id: "devilprincess", giver: Giver::Mayor, title: "デビルプリンセスたいじ",
        text: "地下13階にあくまの大王のむすめが住みついた。\nデビルダッガーを持ってきてくれ。",
        need: "デビルダッガー", prepay: 0, reward_gold: 13000, reward_item: None, min_floor: 13, main: true, count: 1, reward_spell: None },
    Quest { id: "hitotsuoni", giver: Giver::Mayor, title: "ヒトツメオニたいじ",
        text: "地下14階にヒトツメオニがあらわれた。\nヒトツメクラブを持ってきてくれ。",
        need: "ヒトツメクラブ", prepay: 0, reward_gold: 14000, reward_item: None, min_floor: 14, main: true, count: 1, reward_spell: None },
    Quest { id: "flameflower", giver: Giver::Mayor, title: "フレイムフラワーたいじ",
        text: "地下15階の魔穴に大きな魔物がおる。\n火のはなのねを持ってきてくれ。",
        need: "フレイムフラワーのはな", prepay: 0, reward_gold: 15000, reward_item: None, min_floor: 15, main: true, count: 1, reward_spell: None },
    Quest { id: "kage", giver: Giver::Mayor, title: "カゲたいじ",
        text: "地下16階につめたいカゲがしのびよる。\nコテツを持ってきてくれ。",
        need: "コテツ", prepay: 0, reward_gold: 16000, reward_item: None, min_floor: 16, main: true, count: 1, reward_spell: None },
    Quest { id: "mitsuoni", giver: Giver::Mayor, title: "ミツメオニたいじ",
        text: "地下17階にミツメオニがおる。\nミツメクラブを持ってきてくれ。",
        need: "ミツメクラブ", prepay: 0, reward_gold: 17000, reward_item: None, min_floor: 17, main: true, count: 1, reward_spell: None },
    Quest { id: "elephantom", giver: Giver::Mayor, title: "エレファントムたいじ",
        text: "地下18階にエレファントムがおる。\nわぎりのにくを持ってきてくれ。",
        need: "エレファントムのきば", prepay: 0, reward_gold: 18000, reward_item: None, min_floor: 18, main: true, count: 1, reward_spell: None },
    Quest { id: "devillord", giver: Giver::Mayor, title: "デビルロードたいじ",
        text: "地下19階にあくまの大王デビルロードがおる。\nカオスブリンガーを持ってきてくれ。",
        need: "めいおうのしょうこ", prepay: 0, reward_gold: 19000, reward_item: None, min_floor: 19, main: true, count: 1, reward_spell: None },
    Quest { id: "kingsnake", giver: Giver::Mayor, title: "キングヘビリンゴたいじ",
        text: "地下20階にキングヘビリンゴがおる。\nどくリンゴを持ってきてくれ。",
        need: "キングヘビリンゴのへた", prepay: 0, reward_gold: 25000, reward_item: None, min_floor: 20, main: true, count: 1, reward_spell: None },
];

pub fn quest(id: &str) -> &'static Quest {
    QUESTS.iter().find(|q| q.id == id).expect("quest")
}

// ---------------- 魔法 ----------------

/// 魔法の対象
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tgt {
    /// 敵1体
    Foe,
    /// 敵全員
    FoeAll,
    /// 味方1人
    Ally,
    /// 味方全員
    AllyAll,
}

pub struct Spell {
    pub id: &'static str,
    pub name: &'static str,
    pub mp: i32,
    pub price: i32,
    /// 使用者: 0=アレン 2=メイミ(ジェリーは ものまねで おぼえる)
    pub user: usize,
    pub tgt: Tgt,
    pub desc: &'static str,
}

/// 魔法(wiki 「魔法」。値段が wiki に無い物は 設計値)
pub static SPELLS: &[Spell] = &[
    Spell { id: "heal", name: "ひとりかいふく", mp: 3, price: 200, user: 0, tgt: Tgt::Ally, desc: "1人のHPを回復する。" },
    Spell { id: "healall", name: "ぜんいんかいふく", mp: 10, price: 0, user: 0, tgt: Tgt::AllyAll, desc: "みんなのHPを回復する。" },
    Spell { id: "cure", name: "ひとりなおす", mp: 1, price: 150, user: 0, tgt: Tgt::Ally, desc: "1人の どく・ふらつき・ふういん を なおす。" },
    Spell { id: "sleep", name: "すいみん", mp: 2, price: 300, user: 0, tgt: Tgt::FoeAll, desc: "敵全員を眠らせる。" },
    Spell { id: "strong", name: "つよくなれ", mp: 20, price: 5000, user: 0, tgt: Tgt::Ally, desc: "1人の攻撃力を この戦いのあいだ あげる。" },
    Spell { id: "holy", name: "せいなる光", mp: 2, price: 1500, user: 0, tgt: Tgt::Foe, desc: "邪悪なものに大ダメージ。" },
    Spell { id: "reflect", name: "はねかえせ", mp: 10, price: 0, user: 0, tgt: Tgt::Ally, desc: "魔法・特殊攻撃を 1回 はねかえすバリアを はる。" },
    Spell { id: "fire", name: "もえあがれ", mp: 3, price: 0, user: 2, tgt: Tgt::Foe, desc: "敵1体に炎ダメージ。" },
    Spell { id: "firestorm", name: "炎のあらし", mp: 8, price: 2500, user: 2, tgt: Tgt::FoeAll, desc: "敵全員に炎ダメージ。" },
    Spell { id: "frost", name: "氷のつぶて", mp: 4, price: 500, user: 2, tgt: Tgt::Foe, desc: "敵1体に氷ダメージ。" },
    Spell { id: "blizzard", name: "ふぶき", mp: 10, price: 12000, user: 2, tgt: Tgt::FoeAll, desc: "敵全員に吹雪ダメージ。" },
    Spell { id: "seal", name: "ふういん", mp: 2, price: 800, user: 2, tgt: Tgt::Foe, desc: "敵の魔法や特殊攻撃を ふうじる。" },
    Spell { id: "onehp", name: "いのちぎりぎり", mp: 3, price: 10000, user: 2, tgt: Tgt::Foe, desc: "敵1体のHPを 1にしてしまう。ボスには ききにくい。" },
    Spell { id: "drain", name: "いのちいただき", mp: 7, price: 8000, user: 2, tgt: Tgt::FoeAll, desc: "敵全員のHPを吸い出して、仲間全員のHPを回復する。" },
];

/// 魔法屋に見せると 入荷する魔法(魔法の id, 見せる食材)
pub const SPELL_TAUGHT_BY: &[(&str, &str)] = &[
    ("firestorm", "火ライムのひもの"),
    ("frost", "水ライムのひもの"),
    ("blizzard", "イワシ"),
    ("seal", "ハマグリ"),
    ("onehp", "ドクライムのひもの"),
    ("drain", "かわざかな"),
];

pub fn spell(id: &str) -> &'static Spell {
    SPELLS.iter().find(|s| s.id == id).expect("spell")
}


// ---------------- 部屋の形 ----------------

pub struct RoomDef {
    pub name: &'static str,
    pub w: usize,
    pub h: usize,
    /// 4方向から入れる(通り抜けられる)か
    pub open4: bool,
    pub cost: i32,
}

pub static ROOM_DEFS: &[RoomDef] = &[
    RoomDef { name: "エサおけ", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "ゴミすてば", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "わらのベッド", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "てつのたからばこ", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "ぎんのたからばこ", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "おうごんチェスト", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "さかば", w: 1, h: 1, open4: true, cost: ROOM_COST },
    RoomDef { name: "さいだん", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "はたけ", w: 3, h: 3, open4: true, cost: 3 },
    RoomDef { name: "いろり", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "水のみば", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "りっぱなベッド", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "はなぞの", w: 3, h: 3, open4: true, cost: 3 },
    RoomDef { name: "どうじょう", w: 3, h: 3, open4: false, cost: 3 },
    RoomDef { name: "だんろ", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "けっかいのへや", w: 1, h: 1, open4: true, cost: ROOM_COST },
    RoomDef { name: "ふんすい", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "ごうかなベッド", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "れいはいどう", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "じっけんしつ", w: 3, h: 3, open4: false, cost: 3 },
    RoomDef { name: "コロシアム", w: 3, h: 3, open4: true, cost: 3 },
    RoomDef { name: "れいぞうこ", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "まほうじん", w: 1, h: 1, open4: false, cost: ROOM_COST },
    RoomDef { name: "瘴気発生装置", w: 1, h: 1, open4: false, cost: ROOM_COST },
];

pub fn room_def(name: &str) -> &'static RoomDef {
    ROOM_DEFS.iter().find(|r| r.name == name).expect("room def")
}

// ---------------- 階ごとの設定 ----------------

pub const MAX_FLOOR: usize = 20;

pub struct BossRule {
    pub rating: i32,
    pub kills: u32,
    /// 必要な部屋(種類, 数)
    pub rooms: &'static [(&'static str, usize)],
}

pub fn boss_rule(floor: usize) -> BossRule {
    match floor {
        1 => BossRule { rating: BOSS_RATING, kills: BOSS_KILLS, rooms: &[] },
        // 設計値: wiki「モンスターの数 30-40 / さかばの数 4-5」
        2 => BossRule { rating: 40, kills: 30, rooms: &[("さかば", 4)] },
        // 以下は設計値(wiki: 3F モンスター40-50・いろり12-13 / 4F 50-60・水のみば12-13 / 5F 60-70・りっぱなベッド12-13・どうじょう1-2。評価値の目安は100/130)
        3 => BossRule { rating: 55, kills: 40, rooms: &[("いろり", 6)] },
        4 => BossRule { rating: 65, kills: 50, rooms: &[("水のみば", 6)] },
        5 => BossRule { rating: 70, kills: 60, rooms: &[("りっぱなベッド", 5), ("どうじょう", 1)] },
        // 6F〜10F は設計値(ボットの最密プランの約9割を評価値のめやすに)
        6 => BossRule { rating: 66, kills: 50, rooms: &[("だんろ", 6)] },
        7 => BossRule { rating: 68, kills: 60, rooms: &[("ふんすい", 6)] },
        8 => BossRule { rating: 72, kills: 70, rooms: &[("ごうかなベッド", 5), ("どうじょう", 1), ("はなぞの", 1)] },
        9 => BossRule { rating: 72, kills: 80, rooms: &[("れいはいどう", 5), ("どうじょう", 1)] },
        10 => BossRule { rating: 72, kills: 90, rooms: &[("じっけんしつ", 1), ("れいはいどう", 4), ("ごうかなベッド", 2)] },
        // 11F〜20F: 部屋の数は wiki の魔穴の条件(12F ごうかなベッド18個、15F だんろ20個 など)。wiki に無い階と 20F は設計値。評価値はボットのプランの約9割
        11 => BossRule { rating: 100, kills: 50, rooms: &[("はたけ", 2)] },
        12 => BossRule { rating: 218, kills: 60, rooms: &[("はなぞの", 6), ("れいはいどう", 2), ("ごうかなベッド", 18), ("はたけ", 3)] },
        13 => BossRule { rating: 104, kills: 75, rooms: &[("はなぞの", 2), ("ごうかなベッド", 6)] },
        14 => BossRule { rating: 90, kills: 75, rooms: &[("コロシアム", 1), ("ごうかなベッド", 4)] },
        15 => BossRule { rating: 82, kills: 100, rooms: &[("だんろ", 20)] },
        16 => BossRule { rating: 62, kills: 100, rooms: &[("れいぞうこ", 6), ("ふんすい", 8)] },
        17 => BossRule { rating: 208, kills: 100, rooms: &[("はたけ", 4), ("はなぞの", 6), ("ごうかなベッド", 12)] },
        18 => BossRule { rating: 244, kills: 100, rooms: &[("はたけ", 4), ("はなぞの", 6), ("じっけんしつ", 2)] },
        19 => BossRule { rating: 164, kills: 50, rooms: &[("ごうかなベッド", 23), ("じっけんしつ", 2), ("れいはいどう", 4), ("はたけ", 1)] },
        20 => BossRule { rating: 142, kills: 100, rooms: &[("まほうじん", 4), ("コロシアム", 2), ("じっけんしつ", 2), ("ごうかなベッド", 10), ("れいはいどう", 4)] },
        _ => panic!("boss_rule: {}F", floor),
    }
}

pub fn boss_id(floor: usize) -> &'static str {
    match floor {
        1 => "boss",
        2 => "lion",
        3 => "dino",
        4 => "sorcerer",
        5 => "tetsu",
        6 => "debrin",
        7 => "octopus",
        8 => "kinglion",
        9 => "dragon",
        10 => "devilprince",
        11 => "dorunuma",
        12 => "shogunboss",
        13 => "devilprincess",
        14 => "hitotsuoni",
        15 => "flameflower",
        16 => "kage",
        17 => "mitsuoni",
        18 => "elephantom",
        19 => "devillord",
        20 => "kingsnake",
        _ => panic!("boss_id: {}F", floor),
    }
}

/// 各階のメインクエスト(ボスを倒して持ち帰るもの)
pub const FLOOR_MAIN_QUEST: [&str; MAX_FLOOR] = ["snake", "gobl", "dino", "otter", "iron", "debrin", "octopus", "kinglion", "dragon", "devil", "dorunuma", "shogunboss", "devilprincess", "hitotsuoni", "flameflower", "kage", "mitsuoni", "elephantom", "devillord", "kingsnake"];

/// 隠しアイテム(x, y, 名前 / "G数字"はギル)
pub fn hidden_items(floor: usize) -> Vec<(usize, usize, String)> {
    match floor {
        1 => vec![
            (5, 11, "ガラスのゆびわ".to_string()),
            (11, 11, "ちからのゆびわ".to_string()),
            (5, 5, "どうのゆびわ".to_string()),
            (14, 2, "G2000".to_string()),
            (2, 14, "セーラーふく".to_string()),
        ],
        2 => vec![(4, 4, "G4000".to_string())],
        3 => vec![(11, 5, "炎のティアラ".to_string())],
        4 => vec![(5, 11, "氷のティアラ".to_string())],
        5 => vec![(5, 5, "シニスターリング".to_string())],
        6 => vec![(11, 11, "きりさきのゆびわ".to_string())],
        7 => vec![(5, 11, "G20000".to_string())],
        8 => vec![(11, 5, "魔法のオリーブオイル".to_string())],
        9 => vec![(11, 11, "でたらめなけん".to_string())],
        10 => vec![],
        11 => vec![(5, 11, "魔法のオリーブオイル".to_string())],
        12 => vec![(14, 2, "G30000".to_string())],
        13 => vec![(11, 5, "ちからのティアラ".to_string())],
        14 => vec![(11, 11, "魔法のオリーブオイル".to_string())],
        15 => vec![(5, 5, "G40000".to_string())],
        16 => vec![(7, 12, "火のはなのね".to_string())],
        17 => vec![(5, 5, "魔法のオリーブオイル".to_string())],
        18 => vec![(11, 11, "G50000".to_string())],
        19 => vec![(11, 11, "魔法のオリーブオイル".to_string())],
        20 => vec![(5, 5, "いにしえのゆびわ".to_string()), (11, 5, "G100000".to_string())],
        _ => panic!("hidden_items: {}F", floor),
    }
}

/// 装備の耐性(炎%, 氷%, 光%)。ボスの属性攻撃のダメージを減らす
pub fn resist(name: &str) -> (i32, i32, i32) {
    match base_name(name) {
        "ヒネズミのけがわ" => (20, 0, 0),
        "メイデンシールド" => (0, 0, 20),
        "ルビーのゆびわ" => (10, 0, 0),
        "サファイアのゆびわ" => (0, 10, 0),
        "かいのゆびわ" => (0, 20, 0),
        "アツイローブ" => (30, 0, 0),
        "アツイぼうし" => (20, 0, 0),
        "サムイローブ" => (0, 30, 0),
        "サムイぼうし" => (0, 20, 0),
        "かいのヘルメット" => (0, 10, 0),
        "かいのよろい" => (0, 20, 0),
        "かいのシールド" => (0, 10, 0),
        "炎のティアラ" => (30, 0, 0),
        "氷のティアラ" => (0, 30, 0),
        "光のティアラ" => (0, 0, 30),
        "ひかりのゆびわ" => (0, 0, 50),
        "炎のたて" => (20, 0, 0),
        "氷のたて" => (0, 20, 0),
        "氷のバックラー" => (0, 10, 0),
        "氷のかざり" => (0, 20, 0),
        "炎のゆびわ" => (10, 0, 0),
        "氷のゆびわ" => (0, 10, 0),
        "ルビーホルター" => (20, 0, 0),
        "サファイアホルター" => (0, 20, 0),
        "みこのゆびわ" => (0, 0, 20),
        "炎のローブ" => (30, 0, 0),
        "氷のローブ" => (0, 30, 0),
        "炎のゲタ" => (15, 0, 0),
        "氷のゲタ" => (0, 15, 0),
        "かがみのたて" => (0, 0, 30),
        "めがみのティアラ" => (0, 0, 30),
        "あくまのティアラ" => (20, 20, 0),
        "せいぎのよろい" => (20, 20, 20),
        "いにしえのゆびわ" => (40, 40, 0),
        "まもりのゆびわ" => (10, 10, 10),
        "まじょのゆびわ" => (20, 20, 20),
        "まふうじのゆびわ" => (30, 30, 30),
        "あんこくのゆびわ" => (0, 0, 50),
        _ => (0, 0, 0),
    }
}

// ---------------- 行商人(5の倍数の日に 広場へ来る) ----------------

/// その階で 出る魔物(部屋・通路・ボス。wiki の表そのまま)
fn monsters_on_floor(f: usize) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = vec![];
    for r in ROOM_DEFS {
        if chest_tier(r.name).is_some() {
            continue;
        }
        v.extend(room_spawns_exact(f, r.name).iter().copied());
    }
    v.extend(corridor_spawn(f).iter().copied());
    v.push(boss_id(f));
    v.sort_unstable();
    v.dedup();
    v
}

/// 食材の 出どころ(いちばん浅い階, ボスだけが落とすか)。魔物が落とさない食材は 宝箱から探す
pub fn food_origin(name: &str) -> Option<(usize, bool)> {
    for f in 1..=MAX_FLOOR {
        let drops_it = |id: &&str| mon(id).drops.iter().any(|(n, _)| *n == name);
        let ms: Vec<&str> = monsters_on_floor(f).into_iter().filter(drops_it).collect();
        if !ms.is_empty() {
            return Some((f, ms.iter().all(|id| mon(id).boss)));
        }
    }
    for f in 1..=MAX_FLOOR {
        if (0..=2).any(|d| chest_items(f, d).contains(&name)) {
            return Some((f, false));
        }
    }
    None
}

/// その階で 手に入る 換金アイテムの うち いちばん 高い 売値(その階に なければ 浅い階の もの)
pub fn floor_loot_value(floor: usize) -> i32 {
    (1..=floor)
        .rev()
        .find_map(|f| {
            monsters_on_floor(f)
                .iter()
                .flat_map(|id| mon(id).drops.iter())
                .filter_map(|(n, _)| item(n).filter(|d| d.kind == Kind::Loot).map(|d| d.sell))
                .max()
        })
        .unwrap_or(160)
}

/// 行商人の 値段: その階の 換金アイテムの 売値の 3倍。ボスの食材は さらに 3倍
pub fn peddler_price(name: &str) -> Option<i32> {
    let d = item(name)?;
    if d.kind != Kind::Food {
        return None;
    }
    // 個別に 決めた 値段
    if name == "でたらめなにくへん" {
        return Some(3000);
    }
    let (f, boss) = food_origin(name)?;
    Some(floor_loot_value(f) * 3 * if boss { 3 } else { 1 })
}

// ---------------- 宝箱 ----------------

pub struct ChestTier {
    pub room: &'static str,
    pub key: &'static str,
    /// 中身は (階 + deeper) の鉄の宝箱の表から
    pub deeper: usize,
    /// 開けたあと、補充されるまでの日数(金は最大。魔物が多いと早まる)
    pub refill_days: i32,
    /// 補充が進むのに必要な、その階にいる魔物の数
    pub need_monsters: usize,
    /// 魔物が落とす確率(%)と、落とし始める階
    pub drop_pct: u32,
    pub drop_from_floor: usize,
}

/// 鉄・銀・金の宝箱(設計値: 魔物の数・落とす確率。補充日数は 鉄2・銀4・金7)
pub const CHESTS: [ChestTier; 3] = [
    ChestTier { room: "てつのたからばこ", key: "てつのカギ", deeper: 0, refill_days: 2, need_monsters: 3, drop_pct: 5, drop_from_floor: 1 },
    ChestTier { room: "ぎんのたからばこ", key: "ぎんのカギ", deeper: 1, refill_days: 4, need_monsters: 6, drop_pct: 2, drop_from_floor: 6 },
    ChestTier { room: "おうごんチェスト", key: "おうごんのカギ", deeper: 2, refill_days: 7, need_monsters: 9, drop_pct: 1, drop_from_floor: 11 },
];

pub fn chest_tier(room: &str) -> Option<usize> {
    CHESTS.iter().position(|c| c.room == room)
}


// ---------------- 部屋のならべ方(ボットとデバッグで共通) ----------------

/// 回廊(x=8, y=5..11)の右に1x1を6つ(y=10→5の順)、左に(中心y, 部屋)、つきあたり(8,4)に1つ
pub struct Comb {
    pub right: [&'static str; 6],
    pub left: &'static [(usize, &'static str)],
    pub cap: &'static str,
}

/// 3F〜最上階。各階の boss_rule の部屋数をみたす最密のならべ方(ボットの検証用。デバッグのプリセットも同じ)
pub fn comb_layout(floor: usize) -> Comb {
    const B: &str = "ごうかなベッド";
    const REI: &str = "れいはいどう";
    match floor {
        3 => Comb { right: ["いろり"; 6], left: &[(9, "はなぞの"), (7, "エサおけ"), (6, "さいだん"), (5, "ゴミすてば")], cap: "わらのベッド" },
        4 => Comb { right: ["水のみば"; 6], left: &[(9, "はなぞの"), (7, "エサおけ"), (6, "さいだん"), (5, "ゴミすてば")], cap: "わらのベッド" },
        5 => Comb { right: ["りっぱなベッド", "りっぱなベッド", "りっぱなベッド", "りっぱなベッド", "りっぱなベッド", "さかば"], left: &[(9, "どうじょう"), (6, "はなぞの")], cap: "さいだん" },
        6 => Comb { right: ["だんろ"; 6], left: &[(9, "はなぞの"), (7, "いろり"), (6, "けっかいのへや"), (5, "さいだん")], cap: "りっぱなベッド" },
        7 => Comb { right: ["ふんすい"; 6], left: &[(9, "どうじょう"), (7, "水のみば"), (6, "けっかいのへや"), (5, "さいだん")], cap: "りっぱなベッド" },
        8 => Comb { right: [B, B, B, B, B, "さいだん"], left: &[(9, "どうじょう"), (6, "はなぞの")], cap: "ふんすい" },
        9 => Comb { right: [REI, REI, REI, REI, REI, B], left: &[(9, "どうじょう"), (6, "はなぞの")], cap: "ふんすい" },
        10 => Comb { right: [REI, REI, REI, REI, B, B], left: &[(9, "じっけんしつ"), (6, "はなぞの")], cap: "ふんすい" },
        11 => Comb { right: [B; 6], left: &[(9, "はたけ"), (6, "はたけ")], cap: "りっぱなベッド" },
        12 => Comb { right: [B; 6], left: &[(9, "はなぞの"), (7, REI), (6, REI), (5, "エサおけ")], cap: "ふんすい" },
        13 => Comb { right: [B; 6], left: &[(9, "はなぞの"), (6, "はなぞの")], cap: REI },
        14 => Comb { right: [B, B, B, B, REI, REI], left: &[(9, "コロシアム"), (6, "はなぞの")], cap: "ふんすい" },
        15 => Comb { right: ["だんろ"; 6], left: &[(9, "どうじょう"), (6, "コロシアム")], cap: "りっぱなベッド" },
        16 => Comb { right: ["ふんすい", "ふんすい", "ふんすい", "れいぞうこ", "れいぞうこ", "れいぞうこ"], left: &[(9, "どうじょう"), (6, "コロシアム")], cap: "ふんすい" },
        17 => Comb { right: [B; 6], left: &[(9, "はなぞの"), (6, "はたけ")], cap: "さいだん" },
        18 => Comb { right: [REI, REI, REI, REI, B, B], left: &[(9, "じっけんしつ"), (6, "はなぞの")], cap: "ふんすい" },
        19 => Comb { right: [B; 6], left: &[(9, "じっけんしつ"), (7, REI), (6, REI), (5, "けっかいのへや")], cap: "ふんすい" },
        20 => Comb { right: ["まほうじん", "まほうじん", B, B, B, REI], left: &[(9, "コロシアム"), (6, "じっけんしつ")], cap: "ふんすい" },
        _ => panic!("comb_layout: {}F", floor),
    }
}
